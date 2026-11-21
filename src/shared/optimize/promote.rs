use crate::shared::ffi;
use crate::shared::ffi::{AccessInfoState, Function};
use crate::shared::ir::{BasicBlock, InstructionKind, ModuleTpde, Slot, Type};

type PromotedParts = (Option<(u32, Type)>, Option<(u32, Type)>);

struct Defs(Vec<u32>);
impl Defs {
    fn new() -> Self {
        Defs(vec![])
    }
    fn add_val(&mut self, count: usize) -> u32 {
        let i = (self.0.len() / count) as u32;
        self.0.resize(self.0.len() + count, u32::MAX);
        i
    }
    fn def(&self, var: usize, len: usize, bb: usize) -> Option<&u32> {
        let val = &self.0[var * len + bb];
        if val == &u32::MAX { None } else { Some(val) }
    }
    fn def_mut(&mut self, var: usize, len: usize, bb: usize) -> &mut u32 {
        &mut self.0[var * len + bb]
    }
}

impl ModuleTpde {
    /// Implements alloca promotion based on the algorithm described in
    /// Braun et al. Simple and Efficient Construction of Static Single
    /// Assignment Form
    /// for purely sealed blocks and only for promotable allocas
    /// We iterate twice, once to find all the load and stores and again to promote them
    pub fn promote_allocas(&mut self) {
        for func_id in 0..self.functions.len() {
            let func = &mut self.functions[func_id];
            let mut promoted: Vec<PromotedParts> = Vec::with_capacity(func.allocas.len());
            let mut current_def = Defs::new();
            let mut found_promotable = false;
            for alloc in &func.allocas {
                let info = alloc.access_info;
                // A call could read or write the alloca behind our back
                let promotable = info.call_forwards == 0;
                let (slot_a, slot_b) = if promotable && info.state != AccessInfoState::Complex {
                    let slot_a = if info.size_a > 0 {
                        found_promotable = true;
                        Some((current_def.add_val(func.basic_blocks.len()), info.ty_a))
                    } else {
                        None
                    };
                    let slot_b = if info.size_b > 0 {
                        found_promotable = true;
                        Some((current_def.add_val(func.basic_blocks.len()), info.ty_b))
                    } else {
                        None
                    };
                    (slot_a, slot_b)
                } else {
                    (None, None)
                };
                promoted.push((slot_a, slot_b));
            }
            if !found_promotable {
                continue;
            }

            let preds = func.build_predecessor();

            fill_current_def(func, &promoted, &mut current_def);
            self.promote(func_id, &promoted, preds, current_def);
            self.fold_constants(func_id as u32);
        }
    }

    fn promote(
        &mut self,
        func_id: usize,
        promoted: &Vec<PromotedParts>,
        preds: Preds,
        mut current_def: Defs,
    ) {
        let func = &mut self.functions[func_id];
        let bb_len = func.basic_blocks.len();

        let mut stack = vec![0];
        let mut seen = vec![false; func.basic_blocks.len()];

        let mut local_vars = vec![u32::MAX; current_def.0.len() / bb_len];
        let mut touched: Vec<usize> = vec![];
        while let Some(bb_ind) = stack.pop() {
            let bb = &func.basic_blocks[bb_ind];
            if std::mem::replace(&mut seen[bb_ind], true) {
                continue;
            }
            stack.extend(func.successors[bb.successors()].iter().map(|&s| s as usize));

            local_vars.fill(u32::MAX);

            let insts_start = bb.insts_start as usize;
            for instr_index in insts_start..insts_start + bb.insts_len as usize {
                let instr = &mut func.instructions[instr_index];
                let start = instr.ops_start as usize;
                let ops = &mut func.operands[start..start + instr.ops_len as usize];
                let instr_kind = instr.kind;

                match instr_kind {
                    InstructionKind::Store
                        if let Slot::Alloc(a) = Slot::from_ffi(ops[1])
                            && let Some(prom_var) = promoted[a as usize].0 =>
                    {
                        instr.kind = InstructionKind::Noop;
                        instr.ops_len = 0;
                        instr.has_result = false;

                        let var = prom_var.0 as usize;
                        if local_vars[var] == u32::MAX {
                            touched.push(var);
                        }
                        local_vars[var] = ops[0];
                    }
                    InstructionKind::Load
                        if let Slot::Alloc(a) = Slot::from_ffi(ops[0])
                            && let Some(prom_var) = promoted[a as usize].0 =>
                    {
                        instr.kind = InstructionKind::Mov;
                        instr.ops_len = 1;
                        let local_var = local_vars[prom_var.0 as usize];
                        func.operands[start] = if local_var != u32::MAX {
                            local_var
                        } else {
                            let mut context = Context {
                                func,
                                preds: &preds,
                                current_def: &mut current_def,
                                var: prom_var.0 as usize,
                                bb_len,
                                ty: prom_var.1,
                            };
                            read_variable_recursive(&mut context, bb_ind)
                        };
                    }
                    InstructionKind::PairGEP
                        if let Slot::Alloc(a) = Slot::from_ffi(ops[0])
                            && let Some(prom_var) = promoted[a as usize].1 =>
                    {
                        let parts = func.instructions.split_at_mut(instr_index + 1);
                        let instr = &mut parts.0[instr_index];
                        let mem_instr = &mut parts.1[0];
                        let start = mem_instr.ops_start as usize;
                        let ops = &mut func.operands[start..start + mem_instr.ops_len as usize];

                        if mem_instr.kind == InstructionKind::Store {
                            instr.kind = InstructionKind::Noop;
                            instr.has_result = false;
                            instr.ops_len = 0;

                            mem_instr.kind = InstructionKind::Noop;
                            mem_instr.ops_len = 0;
                            mem_instr.has_result = false;

                            let var = prom_var.0 as usize;
                            if local_vars[var] == u32::MAX {
                                touched.push(var);
                            }
                            local_vars[var] = ops[0];
                        }
                        if mem_instr.kind == InstructionKind::Load {
                            instr.kind = InstructionKind::Noop;
                            instr.has_result = false;
                            instr.ops_len = 0;

                            mem_instr.kind = InstructionKind::Mov;
                            mem_instr.ops_len = 1;
                            let local_var = local_vars[prom_var.0 as usize];
                            func.operands[start] = if local_var != u32::MAX {
                                local_var
                            } else {
                                let mut context = Context {
                                    func,
                                    preds: &preds,
                                    current_def: &mut current_def,
                                    var: prom_var.0 as usize,
                                    bb_len,
                                    ty: prom_var.1,
                                };
                                read_variable_recursive(&mut context, bb_ind)
                            };
                        }
                    }
                    _ => {}
                }
            }

            for var in touched.drain(..) {
                local_vars[var] = u32::MAX;
            }
        }
    }
}

struct Context<'a> {
    func: &'a mut Function,
    preds: &'a Preds,
    current_def: &'a mut Defs,
    var: usize,
    bb_len: usize,
    ty: Type,
}

fn read_variable(c: &mut Context, bb: usize) -> u32 {
    if let Some(&def) = c.current_def.def(c.var, c.bb_len, bb) {
        return def;
    }

    read_variable_recursive(c, bb)
}

fn write_variable(c: &mut Context, bb: usize, value: u32) {
    if let None = c.current_def.def(c.var, c.bb_len, bb) {
        *c.current_def.def_mut(c.var, c.bb_len, bb) = value;
    }
}

fn read_variable_recursive(c: &mut Context, bb: usize) -> u32 {
    // we're always sealed
    let cur_preds = c.preds.of(bb);

    let val = if cur_preds.is_empty() {
        return Slot::new_inline_const(c.ty, 0).to_ffi();
    } else if cur_preds.len() == 1 {
        read_variable(c, cur_preds[0] as usize)
    } else {
        let phi = c
            .func
            .add_phi(BasicBlock::dummy(bb), c.ty, cur_preds.len() as u32);
        write_variable(c, bb, phi.to_ffi());
        add_phi_operands(c, bb, phi)
    };

    write_variable(c, bb, val);
    val
}

fn add_phi_operands(c: &mut Context, bb: usize, phi: Slot) -> u32 {
    let cur_preds = c.preds.of(bb);
    for (pred_ind, &pred) in cur_preds.iter().enumerate() {
        let var = read_variable(c, pred as usize);
        c.func
            .set_phi_incoming(phi, pred_ind as u32, BasicBlock::dummy(pred as usize), var);
    }
    // we don't remove trivial phis for now
    phi.to_ffi()
}

fn fill_current_def(
    func: &mut ffi::Function,
    promoted: &Vec<PromotedParts>,
    current_def: &mut Defs,
) {
    let bb_len = func.basic_blocks.len();
    for (bb_ind, bb) in func.basic_blocks.iter().enumerate() {
        let insts_start = bb.insts_start as usize;
        for instr_index in insts_start..insts_start + bb.insts_len as usize {
            let instr = &mut func.instructions[instr_index];
            let start = instr.ops_start as usize;
            let ops = &mut func.operands[start..start + instr.ops_len as usize];

            if let Some((instr, prom_var, op)) = match instr.kind {
                InstructionKind::Store
                    if let Slot::Alloc(a) = Slot::from_ffi(ops[1])
                        && let Some(prom_var) = promoted[a as usize].0 =>
                {
                    Some((instr, prom_var, ops[0]))
                }
                InstructionKind::PairGEP
                    if let Slot::Alloc(a) = Slot::from_ffi(ops[0])
                        && let Some(prom_var) = promoted[a as usize].1 =>
                {
                    let parts = func.instructions.split_at_mut(instr_index + 1);
                    let instr = &mut parts.0[instr_index];
                    let mem_instr = &mut parts.1[0];
                    let start = mem_instr.ops_start as usize;
                    let ops = &mut func.operands[start..start + mem_instr.ops_len as usize];

                    if mem_instr.kind == InstructionKind::Store {
                        Some((mem_instr, prom_var, ops[0]))
                    } else {
                        None
                    }
                }
                _ => None,
            } {
                *current_def.def_mut(prom_var.0 as usize, bb_len, bb_ind) = op;
            }
        }
    }
}

struct Preds {
    preds_starts: Vec<u32>,
    preds_count: Vec<u32>,
    preds: Vec<u32>,
}
impl Preds {
    fn from(preds_starts: Vec<u32>, preds_count: Vec<u32>, preds: Vec<u32>) -> Self {
        Preds {
            preds_starts,
            preds_count,
            preds,
        }
    }
    #[inline(always)]
    fn of(&self, bb: usize) -> &[u32] {
        let start = self.preds_starts[bb] as usize;
        &self.preds[start..start + self.preds_count[bb] as usize]
    }
}
impl ffi::Function {
    fn build_predecessor(&self) -> Preds {
        let block_len = self.basic_blocks.len();

        let mut preds_count: Vec<u32> = vec![0; block_len];
        let mut all_preds_count: usize = 0;
        for bb in &self.basic_blocks {
            for &suc in &self.successors[bb.successors()] {
                preds_count[suc as usize] += 1;
            }
            all_preds_count += bb.suc_len as usize;
        }
        let mut preds_starts: Vec<u32> = vec![0; block_len];
        {
            let mut sum = 0;
            for (i, count) in preds_count.iter().enumerate() {
                preds_starts[i] = sum;
                sum += count;
            }
        }

        let mut preds: Vec<u32> = vec![0; all_preds_count];
        {
            let mut cur = preds_starts.clone();
            for (i, bb) in self.basic_blocks.iter().enumerate() {
                for &suc in &self.successors[bb.successors()] {
                    preds[cur[suc as usize] as usize] = i as u32;
                    cur[suc as usize] += 1;
                }
            }
        }

        Preds::from(preds_starts, preds_count, preds)
    }
}
