use super::OptConfig;
use crate::shared::ffi;
use crate::shared::ffi::AccessInfoState;
use crate::shared::ffi::{BasicBlock, Instruction, InstructionReference, Phi};
use crate::shared::ir::{ArgKind, InstructionKind, ModuleTpde, Slot, size_of_type};
use crate::shared::ir::{access_info_intern, alloc_used_a, alloc_used_b, merge_access_states};
use std::collections::BTreeSet;
use std::ops::Range;

impl ModuleTpde {
    pub fn inline_functions(&mut self, config: OptConfig) {
        // Inline functions bottom up
        // Inlinable functions:
        // - are leave functions -> call no other functions
        // - are marked always inlined or cheaper than the maximum specified cost
        // - has no byval arguments

        // info1 and info2 are not used during generation
        // we use info1 to store the follower of a block if it has to split up
        // after the inlining the value has no meaning and could be set back to the default
        // assumes that info1 is set to 0 in the beginning, maybe use this if we use it also in
        // further optimizations
        // for func in &mut self.functions {
        //     for bb in &mut func.basic_blocks {
        //         bb.info1 = 0;
        //     }
        // }

        let mut leaves: Vec<usize> = self
            .functions
            .iter()
            .enumerate()
            .filter(|(_, f)| is_inlinable(f, config.inline_cost))
            .map(|(i, _)| i)
            .collect();
        let mut returns: Vec<ReturnSite> = vec![];

        // Used over hash set for determinism
        let mut to_optimize: BTreeSet<u32> = BTreeSet::new();

        while let Some(f) = leaves.pop() {
            let mut all_inlined = true;

            for site in std::mem::take(&mut self.functions[f].called_by) {
                let [caller, callee] = self
                    .functions
                    .get_disjoint_mut([site.func as usize, f])
                    .expect("Trying to inline function into itself");

                all_inlined &= inline_call(caller, callee, &site, &mut returns, config.promote);

                caller.calling -= 1;
                if is_inlinable(caller, config.inline_cost) {
                    to_optimize.remove(&site.func);
                    self.fold_constants(site.func);

                    leaves.push(site.func as usize);
                } else {
                    to_optimize.insert(site.func);
                }
            }

            let func = &mut self.functions[f];
            func.flags.dead = all_inlined && func.flags.only_local && !func.flags.never_delete;
        }

        for f in to_optimize {
            self.fold_constants(f);
        }
    }

    /// Propagates constants that might have shown up after inlining, replaces if applicable:
    /// - condbr to br
    /// - gets rid of overflow checks
    fn fold_constants(&mut self, function: u32) {
        let mut stack = vec![0];

        let func = &self.functions[function as usize];
        let mut known: Vec<Option<u32>> = vec![None; func.slots.len()];
        let mut seen = vec![false; func.basic_blocks.len()];

        while let Some(block) = stack.pop() {
            let func = &self.functions[function as usize];
            // Phis never become known, so a value is always known before its uses are visited
            if std::mem::replace(&mut seen[block], true) {
                continue;
            }
            let start = func.basic_blocks[block].insts_start as usize;
            let end = start + func.basic_blocks[block].insts_len as usize;
            let (mut read, mut write) = (start, start);

            while read < end {
                let func = &mut self.functions[function as usize];
                let mut instr = func.instructions[read];
                read += 1;
                let ops = instr.ops_start as usize;
                let ops_end = ops + instr.ops_len as usize;

                for op in &mut func.operands[ops..ops_end] {
                    if let Slot::Value(_, v) = Slot::from_ffi(*op)
                        && let Some(c) = known[v as usize]
                    {
                        *op = c;
                    }
                }

                let func = &self.functions[function as usize];
                if instr.has_result & all_const(&func.operands[ops..ops_end])
                    && let Some(value) = self.eval_const(instr.kind, &func.operands[ops..ops_end])
                {
                    let ty = func.slots[instr.result as usize].ty;
                    known[instr.result as usize] = Some(self.add_const(ty, value).to_ffi());

                    // An overflow check is only defined by the arithmetic before it, so both have to go
                    let func = &mut self.functions[function as usize];
                    if read < end && func.instructions[read].kind == InstructionKind::OverflowCheck
                    {
                        let check = func.instructions[read];
                        read += 1;
                        let signed = matches!(
                            Slot::from_ffi(func.operands[check.ops_start as usize]),
                            Slot::Raw(1)
                        );
                        let func = &self.functions[function as usize];
                        let overflow =
                            self.overflows(instr.kind, &func.operands[ops..ops_end], signed)
                                as u128;
                        known[check.result as usize] =
                            Some(self.add_const(ffi::Type::Bool, overflow).to_ffi());
                    }
                    continue; // skip write so delete instruction
                }

                if instr.kind == InstructionKind::CondBr
                    && let Slot::Const(cond) = Slot::from_ffi(func.operands[ops])
                {
                    let taken = self.const_value(cond).1 & 1 != 0;
                    self.functions[function as usize].operands[ops] =
                        func.operands[if taken { ops + 1 } else { ops + 2 }];
                    instr.kind = InstructionKind::Br;
                    instr.ops_len = 1;
                }

                let func = &mut self.functions[function as usize];
                let successors = successor_ops(&instr);
                for op in &func.operands[successors] {
                    if let Slot::Raw(target) = Slot::from_ffi(*op) {
                        stack.push(target as usize);
                    }
                }

                func.instructions[write] = instr;
                write += 1;
            }

            // Mark everything stale as unreachable to avoid confusion of later inlinings
            let func = &mut self.functions[function as usize];
            for stale in &mut func.instructions[write..end] {
                stale.kind = InstructionKind::Unreachable;
                stale.ops_len = 0;
                stale.has_result = false;
            }
            func.basic_blocks[block].insts_len = (write - start) as u32;
        }

        // The incoming values of a phi can be defined after it, so these are mapped last
        let func = &mut self.functions[function as usize];
        for phi in &func.phis {
            let start = phi.incoming_start as usize;
            let incoming = &mut func.operands[start..start + phi.incoming_len as usize * 2];
            for value in incoming.iter_mut().skip(1).step_by(2) {
                if let Slot::Value(_, v) = Slot::from_ffi(*value)
                    && let Some(c) = known[v as usize]
                {
                    *value = c;
                }
            }
        }
    }

    fn eval_const(&self, kind: InstructionKind, ops: &[u32]) -> Option<u128> {
        let &[lhs, rhs] = ops else { return None };
        let (Slot::Const(lhs), Slot::Const(rhs)) = (Slot::from_ffi(lhs), Slot::from_ffi(rhs))
        else {
            unreachable!()
        };
        let (tya, lhs) = self.const_value(lhs);
        let (_, rhs) = self.const_value(rhs);
        let bits = size_of_type(tya) * 8;
        let (lhs, rhs) = (lhs & mask(bits), rhs & mask(bits));

        use InstructionKind as I;
        let (slhs, srhs) = (sign_extend(lhs, bits), sign_extend(rhs, bits));
        Some(match kind {
            I::Add => lhs.wrapping_add(rhs) & mask(bits),
            I::Sub => lhs.wrapping_sub(rhs) & mask(bits),
            I::Mul => lhs.wrapping_mul(rhs) & mask(bits),
            I::CMPeq => (lhs == rhs) as u128,
            I::CMPne => (lhs != rhs) as u128,
            I::CMPugt => (lhs > rhs) as u128,
            I::CMPuge => (lhs >= rhs) as u128,
            I::CMPult => (lhs < rhs) as u128,
            I::CMPule => (lhs <= rhs) as u128,
            I::CMPsgt => (slhs > srhs) as u128,
            I::CMPsge => (slhs >= srhs) as u128,
            I::CMPslt => (slhs < srhs) as u128,
            I::CMPsle => (slhs <= srhs) as u128,
            _ => return None,
        })
    }

    fn overflows(&self, kind: InstructionKind, ops: &[u32], signed: bool) -> bool {
        let &[lhs, rhs] = ops else { unreachable!() };
        let (Slot::Const(lhs), Slot::Const(rhs)) = (Slot::from_ffi(lhs), Slot::from_ffi(rhs))
        else {
            unreachable!()
        };
        let (tya, lhs) = self.const_value(lhs);
        let (_, rhs) = self.const_value(rhs);
        let bits = size_of_type(tya) * 8;
        let (lhs, rhs) = (lhs & mask(bits), rhs & mask(bits));

        use InstructionKind as I;
        if signed {
            let (slhs, shrs) = (sign_extend(lhs, bits), sign_extend(rhs, bits));
            let result = match kind {
                I::Add => slhs.checked_add(shrs),
                I::Sub => slhs.checked_sub(shrs),
                I::Mul => slhs.checked_mul(shrs),
                _ => unreachable!(),
            };
            result.is_none_or(|r| sign_extend(r as u128 & mask(bits), bits) != r)
        } else {
            let result = match kind {
                I::Add => lhs.checked_add(rhs),
                I::Sub => lhs.checked_sub(rhs),
                I::Mul => lhs.checked_mul(rhs),
                _ => unreachable!(),
            };
            result.is_none_or(|r| r > mask(bits))
        }
    }
}

fn replace_phi_predecessor(func: &mut ffi::Function, from: u32, to: u32) {
    if func.phis.is_empty() {
        return;
    }
    let new_block = &func.basic_blocks[to as usize];
    if new_block.insts_len == 0 {
        return;
    }
    let terminator = func.instructions[(new_block.insts_start + new_block.insts_len - 1) as usize];
    let (from, to) = (Slot::new_raw(from).to_ffi(), Slot::new_raw(to).to_ffi());

    for op in successor_ops(&terminator) {
        let Slot::Raw(target) = Slot::from_ffi(func.operands[op]) else {
            continue;
        };
        let target = &func.basic_blocks[target as usize];
        for phi in
            &func.phis[target.phi_start as usize..(target.phi_start + target.phi_len) as usize]
        {
            let incoming = &mut func.operands
                [phi.incoming_start as usize..(phi.incoming_start + phi.incoming_len) as usize * 2];
            for block in incoming.iter_mut().step_by(2) {
                if *block == from {
                    *block = to;
                }
            }
        }
    }
}

fn successor_ops(instr: &Instruction) -> Range<usize> {
    let ops = instr.ops_start as usize;
    match instr.kind {
        InstructionKind::Br => ops..ops + 1,
        InstructionKind::CondBr | InstructionKind::Invoke => ops + 1..ops + 3,
        InstructionKind::Switch => ops + 1..ops + 1 + instr.ops_len as usize / 2,
        _ => 0..0,
    }
}

fn all_const(ops: &[u32]) -> bool {
    ops.iter()
        .all(|op| matches!(Slot::from_ffi(*op), Slot::Const(_)))
}

fn mask(bits: u32) -> u128 {
    u128::MAX >> (128 - bits)
}

fn sign_extend(value: u128, bits: u32) -> i128 {
    ((value << (128 - bits)) as i128) >> (128 - bits)
}

// Inline this call, returns if it could be inlined or not
fn inline_call(
    caller: &mut ffi::Function,
    callee: &ffi::Function,
    site: &InstructionReference,
    returns: &mut Vec<ReturnSite>,
    track_access: bool,
) -> bool {
    let (block, offset) = {
        let (mut block, mut offset) = (site.block as usize, site.instr);
        while offset >= caller.basic_blocks[block].insts_len {
            offset -= caller.basic_blocks[block].insts_len;
            block = caller.basic_blocks[block].info1 as usize;
        }
        (block, offset)
    };
    let (insts_start, insts_len) = (
        caller.basic_blocks[block].insts_start,
        caller.basic_blocks[block].insts_len,
    );
    let call_idx = (insts_start + offset) as usize;

    let call = &caller.instructions[call_idx];
    let args_start = call.ops_start as usize + 1;
    let args_end = args_start + call.ops_len as usize - 1;
    if !can_inline_call(caller, callee, &caller.operands[args_start..args_end]) {
        return false;
    }

    // decrement forwarding for all arguments
    if track_access {
        for i in 0..call.ops_len as usize - 1 {
            let arg = Slot::from_ffi(caller.operands[args_start + i]);
            if let Some(call_info) = access_info_intern(caller, &arg) {
                let callee_info = &callee.args[i].access_info;

                call_info.call_forwards -= 1;
                call_info.call_forwards += callee_info.call_forwards;

                if alloc_used_a(callee_info) {
                    if !alloc_used_a(call_info) {
                        call_info.size_a = callee_info.size_a;
                    } else if call_info.size_a != callee_info.size_a {
                        call_info.state = AccessInfoState::Complex
                    }
                }
                if alloc_used_b(callee_info) {
                    if !alloc_used_b(call_info) {
                        call_info.size_b = callee_info.size_b;
                        call_info.offset_b = callee_info.offset_b;
                    } else if call_info.size_b != callee_info.size_b
                        || call_info.offset_b != callee_info.offset_b
                    {
                        call_info.state = AccessInfoState::Complex
                    }
                }
                call_info.state = merge_access_states(call_info.state, callee_info.state)
            }
        }
    }

    // landing pads of the callee need its personality
    if callee.has_personality && !caller.has_personality {
        caller.has_personality = true;
        caller.personality = callee.personality;
    }
    caller.cost += callee.cost;

    // split up the block after the call
    let cont_idx = caller.basic_blocks.len() as u32;
    let cur = &mut caller.basic_blocks[block];
    cur.insts_len = offset + 1;
    let prev_cont = cur.info1;
    cur.info1 = cont_idx;

    let cont_bb = Slot::new_raw(cont_idx);
    caller.basic_blocks.push(BasicBlock {
        insts_start: call_idx as u32 + 1,
        insts_len: insts_len - offset - 1,
        phi_start: 0,
        phi_len: 0,
        info1: prev_cont,
        info2: 0,
    });
    replace_phi_predecessor(caller, block as u32, cont_idx);

    let results_bb = caller.basic_blocks.len();
    caller.basic_blocks.push(BasicBlock {
        insts_start: 0,
        insts_len: 0,
        phi_start: 0,
        phi_len: 0,
        info1: 0,
        info2: 0,
    });

    // switch to a br instruction jumping to the first basic block of the inlined function
    let call = &mut caller.instructions[call_idx];
    call.kind = InstructionKind::Br;
    call.ops_len = 1;
    caller.operands[call.ops_start as usize] =
        Slot::new_raw(caller.basic_blocks.len() as u32).to_ffi();

    let expected_slots: &[u32] = if call.has_result {
        let result = call.result;
        call.has_result = false;
        let next = &mut caller.instructions[call_idx + 1];
        if next.kind == InstructionKind::AddRet {
            next.has_result = false;
            &[result, next.result]
        } else {
            &[result]
        }
    } else {
        &[]
    };

    let offsets = InlineOffsets {
        allocas: caller.allocas.len() as u32,
        callee_infos: caller.callee_infos.len() as u32,
        slots: caller.slots.len() as u32,
        phis: caller.phis.len() as u32,
        basic_blocks: caller.basic_blocks.len() as u32,
        instructions: caller.instructions.len() as u32,
        operands: caller.operands.len() as u32,
    };

    returns.clear();

    // let's start copying the function
    copy_inline_function(caller, callee);

    let ops_len = caller.operands.len() as u32;
    let mut additional_ops: Vec<u32> = vec![];
    let (ex_ops, new_ops) = caller.operands.split_at_mut(offsets.operands as usize);
    let argument_mapping = &ex_ops[args_start..args_end];
    for i in offsets.basic_blocks as usize..caller.basic_blocks.len() {
        map_block(
            &mut caller.basic_blocks[i],
            i as u32,
            &mut caller.instructions,
            new_ops,
            &mut caller.phis,
            &callee.operands,
            argument_mapping,
            &mut additional_ops,
            ops_len,
            &offsets,
            returns,
            results_bb as u32,
        );
    }
    caller.operands.extend_from_slice(additional_ops.as_slice());

    let bb = &mut caller.basic_blocks[results_bb];
    bb.insts_start = caller.instructions.len() as u32;
    bb.phi_start = caller.phis.len() as u32;

    for (i, to) in expected_slots.iter().enumerate() {
        let from = if let [ret] = returns.as_slice() {
            caller.operands[ret.ops_start as usize + i]
        } else {
            let incoming_start = caller.operands.len() as u32;
            for ret in returns.iter() {
                let value = caller.operands[ret.ops_start as usize + i];
                caller.operands.push(Slot::new_raw(ret.block).to_ffi());
                caller.operands.push(value);
            }
            caller.phis.push(ffi::Phi {
                ty: caller.slots[*to as usize].ty,
                incoming_start,
                incoming_len: returns.len() as u32,
            });
            bb.phi_len += 1;
            Slot::new_phi(caller.phis.len() as u32 - 1).to_ffi()
        };
        caller.instructions.push(ffi::Instruction {
            kind: InstructionKind::Cast,
            ops_start: caller.operands.len() as u32,
            ops_len: 1,
            has_result: true,
            result: *to,
        });
        caller.operands.push(from);
    }
    caller.instructions.push(ffi::Instruction {
        kind: InstructionKind::Br,
        ops_start: caller.operands.len() as u32,
        ops_len: 1,
        has_result: false,
        result: 0,
    });
    caller.operands.push(cont_bb.to_ffi());

    bb.insts_len = caller.instructions.len() as u32 - bb.insts_start;

    true
}

fn is_inlinable(f: &ffi::Function, max_cost: Option<u32>) -> bool {
    let cheap = max_cost.is_some_and(|max| f.cost <= max);
    let wanted = f.calling == 0 && (f.flags.always_inline || cheap);

    wanted && !f.basic_blocks.is_empty() && !f.args.iter().any(|arg| arg.kind == ArgKind::ByVal)
}

fn can_inline_call(caller: &ffi::Function, callee: &ffi::Function, args: &[u32]) -> bool {
    // stop at variadic arguments
    if args.len() != callee.args.len() {
        return false;
    }
    if callee.has_personality && caller.has_personality && callee.personality != caller.personality
    {
        return false;
    }
    true
}

struct ReturnSite {
    block: u32,
    ops_start: u32,
    ops_len: u32,
}

struct InlineOffsets {
    allocas: u32,
    callee_infos: u32,
    slots: u32,
    phis: u32,
    basic_blocks: u32,
    instructions: u32,
    operands: u32,
}

/// Transfer all the necessary information to the new caller functions
fn copy_inline_function(caller: &mut ffi::Function, callee: &ffi::Function) {
    caller.allocas.extend_from_slice(&callee.allocas);
    caller.callee_infos.extend_from_slice(&callee.callee_infos);

    caller.basic_blocks.extend_from_slice(&callee.basic_blocks);
    caller.instructions.extend_from_slice(&callee.instructions);
    caller.operands.extend_from_slice(&callee.operands);
    caller.slots.extend_from_slice(&callee.slots);
    caller.phis.extend_from_slice(&callee.phis);
}

// TODO reduce the amount of arguments
fn map_block(
    bb: &mut BasicBlock,
    block_index: u32,
    instructions: &mut [Instruction],
    operands: &mut [u32],
    phis: &mut [Phi],
    callee_operands: &[u32],
    argument_mapping: &[u32],
    adding_ops: &mut Vec<u32>,
    ops_len: u32,
    inline_offsets: &InlineOffsets,
    returns: &mut Vec<ReturnSite>,
    return_bb: u32,
) {
    bb.insts_start += inline_offsets.instructions;
    bb.phi_start += inline_offsets.phis;

    {
        let start = bb.insts_start as usize;
        let end = start + bb.insts_len as usize;
        for instr in &mut instructions[start..end] {
            map_instruction(
                instr,
                block_index,
                operands,
                callee_operands,
                argument_mapping,
                adding_ops,
                ops_len,
                inline_offsets,
                returns,
                return_bb,
            )
        }
    }

    {
        let start = bb.phi_start as usize;
        let end = start + bb.phi_len as usize;
        for phi in &mut phis[start..end] {
            map_phi(phi, operands, argument_mapping, inline_offsets)
        }
    }
}

fn map_instruction(
    instr: &mut Instruction,
    block_index: u32,
    operands: &mut [u32],
    callee_operands: &[u32],
    argument_mapping: &[u32],
    adding_ops: &mut Vec<u32>,
    ops_len: u32,
    inline_offsets: &InlineOffsets,
    returns: &mut Vec<ReturnSite>,
    return_bb: u32,
) {
    {
        let start = instr.ops_start as usize;
        let end = start + instr.ops_len as usize;
        for op in &mut operands[start..end] {
            map_operands(op, argument_mapping, inline_offsets);
        }
    }

    // map: br, condbr, switch, funcs (callee info), invoke
    match instr.kind {
        InstructionKind::Br => {
            // map op 0 raw bb
            operands[instr.ops_start as usize] += inline_offsets.basic_blocks;
        }
        InstructionKind::CondBr => {
            // Invoke: map op 1 and 2 (then and catch bb)
            // CondBr: map op 1 and 2 raw bb
            let ops = instr.ops_start as usize;
            operands[ops + 1] += inline_offsets.basic_blocks;
            operands[ops + 2] += inline_offsets.basic_blocks;
        }
        InstructionKind::Switch => {
            // [cond, default, targets..., case values...]: map default and targets
            let targets = instr.ops_start as usize + 1
                ..instr.ops_start as usize + 1 + instr.ops_len as usize / 2;
            for op in &mut operands[targets] {
                *op += inline_offsets.basic_blocks;
            }
        }
        InstructionKind::Call | InstructionKind::Invoke => {
            let info = if instr.kind == InstructionKind::Invoke {
                // map op 1 and 2 (then and catch bb)
                operands[instr.ops_start as usize + 1] += inline_offsets.basic_blocks;
                operands[instr.ops_start as usize + 2] += inline_offsets.basic_blocks;
                instr.ops_start as usize + 3
            } else {
                instr.ops_start as usize + 1
            };

            // map the raw callee_info if the original callee is a val
            if matches!(
                Slot::from_ffi(callee_operands[instr.ops_start as usize]),
                Slot::Value(..) | Slot::Phi(..)
            ) {
                if matches!(
                    Slot::from_ffi(operands[instr.ops_start as usize]),
                    Slot::Func(_)
                ) {
                    // The argument made it a direct call, which has no callee info
                    operands.copy_within(
                        info + 1..instr.ops_start as usize + instr.ops_len as usize,
                        info,
                    );
                    instr.ops_len -= 1;
                } else {
                    operands[info] += inline_offsets.callee_infos;
                }
            }
        }
        InstructionKind::Ret => {
            returns.push(ReturnSite {
                block: block_index,
                ops_start: instr.ops_start + inline_offsets.operands,
                ops_len: instr.ops_len,
            });

            instr.kind = InstructionKind::Br;
            instr.ops_start = ops_len + adding_ops.len() as u32;
            instr.ops_len = 1;
            adding_ops.push(Slot::new_raw(return_bb).to_ffi());
            return;
        }
        _ => {}
    }

    // map the start ops of the operands
    instr.ops_start += inline_offsets.operands;
    if instr.has_result {
        instr.result += inline_offsets.slots;
    }
}

fn map_operands(operand: &mut u32, argument_mapping: &[u32], inline_offsets: &InlineOffsets) {
    // map: val, alloca
    match Slot::from_ffi(*operand) {
        Slot::Value(f, v) => {
            *operand = if v < argument_mapping.len() as u32 {
                argument_mapping[v as usize]
            } else {
                Slot::Value(f, v + inline_offsets.slots).to_ffi()
            };
        }
        Slot::Alloc(a) => *operand = Slot::Alloc(a + inline_offsets.allocas).to_ffi(),
        Slot::Phi(f, p) => *operand = Slot::Phi(f, p + inline_offsets.phis).to_ffi(),
        _ => {}
    };
}

fn map_phi(
    phi: &mut Phi,
    operands: &mut [u32],
    argument_mapping: &[u32],
    inline_offsets: &InlineOffsets,
) {
    let start = phi.incoming_start as usize;
    let incoming = &mut operands[start..start + phi.incoming_len as usize * 2];
    for block in incoming.iter_mut().step_by(2) {
        *block += inline_offsets.basic_blocks;
    }
    for val in incoming {
        map_operands(val, argument_mapping, inline_offsets);
    }
    phi.incoming_start += inline_offsets.operands;
}
