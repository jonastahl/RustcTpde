use crate::shared::ffi::BasicBlock;
use crate::shared::ir::{ArgKind, Function, InstructionKind, ModuleTpde, Slot};
use crate::shared::ffi;

/// Upper bound of inlined calls per function, guards against recursive always inline functions
const MAX_INLINES_PER_FUNCTION: u32 = 1024;

impl ModuleTpde {
    pub fn inline_functions(&mut self) {
        let inline_functions: Vec<bool> = self
            .functions
            .iter()
            .map(|f| {
                if !f.flags.always_inline {
                    return false;
                }
                if f.instructions
                    .iter()
                    .filter(|i| matches!(i.kind, InstructionKind::Ret))
                    .count()
                    > 1 {
                    return false;
                };
                if f.args.iter().any(|arg| arg.kind == ArgKind::ByVal) {
                    return false;
                }
                true
            })
            .collect();

        // Fast path if there is no function to inline
        if !inline_functions.contains(&true) {
            return;
        }

        let mut return_slots: Vec<u32> = vec![];

        for f in 0..self.functions.len() {
            let caller = &self.functions[f];
            if caller.flags.always_inline || caller.flags.extern_link {
                continue;
            }

            let mut inline_budget = MAX_INLINES_PER_FUNCTION;

            let mut i = 0;
            while i < self.functions[f].basic_blocks.len() {
                let bb = &self.functions[f].basic_blocks[i];
                let (insts_start, insts_len) = (bb.insts_start as usize, bb.insts_len as usize);
                i += 1;

                for j in 0..insts_len {
                    let caller = &mut self.functions[f];
                    let call = &mut caller.instructions[insts_start + j];

                    // We only support inlining calls, no invokes
                    if call.kind != InstructionKind::Call {
                        continue;
                    }

                    // no inlining for indirect calls
                    let Slot::Func(callee_ref) = Slot::from_ffi(caller.operands[call.ops_start as usize]) else {
                        continue;
                    };

                    let callee = Slot::from_ffi(caller.operands[call.ops_start as usize]);
                    let Slot::Func(callee) = callee else {
                        continue;
                    };
                    if !inline_functions[callee.0] {
                        continue;
                    }
                    let [caller, callee] = self.functions.get_disjoint_mut([f, callee_ref.0])
                        .expect("Trying to inline function into itself");

                    let call = &mut caller.instructions[insts_start + j];
                    let args_start = call.ops_start as usize + 1;
                    let args_end = args_start + call.ops_len as usize - 1;
                    if !can_inline_call(caller, callee, &caller.operands[args_start..args_end])
                        || inline_budget == 0
                    {
                        continue;
                    }
                    inline_budget -= 1;

                    // landing pads of the callee need its personality
                    if callee.has_personality && !caller.has_personality {
                        caller.has_personality = true;
                        caller.personality = callee.personality;
                    }

                    return_slots.clear();

                    // split up the block after the call
                    let cur = &mut caller.basic_blocks[i - 1];
                    cur.insts_len = (insts_start + j + 1 - cur.insts_start as usize) as u32;

                    let cont_bb = Slot::new_raw(caller.basic_blocks.len() as u32);
                    caller.basic_blocks.push(BasicBlock {
                        name: "Inline cont".parse().unwrap(),
                        insts_start: (insts_start + j + 1) as u32,
                        insts_len: (insts_len - j - 1) as u32,
                        info1: 0,
                        info2: 0,
                    });
                    let results_bb = caller.basic_blocks.len();
                    caller.basic_blocks.push(BasicBlock {
                        name: "Inline result".parse().unwrap(),
                        insts_start: 0,
                        insts_len: 0,
                        info1: 0,
                        info2: 0,
                    });

                    // switch to a br instruction jumping to the first basic block of the inlined function
                    let call = &mut caller.instructions[insts_start + j];
                    call.kind = InstructionKind::Br;
                    call.ops_len = 1;
                    caller.operands[call.ops_start as usize] =
                        Slot::new_raw(caller.basic_blocks.len() as u32).to_ffi();

                    let expected_slots: &[u32] = if call.has_result {
                        let result = call.result;
                        call.has_result = false;
                        let next = &mut caller.instructions[insts_start + j + 1];
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
                        basic_blocks: caller.basic_blocks.len() as u32,
                        instructions: caller.instructions.len() as u32,
                        operands: caller.operands.len() as u32,
                    };

                    // let's start copying the function
                    copy_inline_function(caller, callee);

                    {
                        let block_len = caller.basic_blocks.len();
                        map_blocks(&mut caller.basic_blocks[offsets.basic_blocks as usize..block_len], &offsets)
                    }


                    // start mapping instructions
                    {
                        caller.operands.push(Slot::new_raw(results_bb as u32).to_ffi());
                        let results_op = caller.operands.len() - 1;
                        let (ex_ops, new_ops) = caller.operands.split_at_mut(offsets.operands as usize);

                        // mapping table for the arguments of the function
                        let argument_mapping = &ex_ops[args_start..args_end];

                        // start remapping all operands, before the instructions so that the
                        // return values are already mapped
                        map_operands(new_ops, argument_mapping, &offsets);

                        let instr_len = caller.instructions.len();
                        map_instructions(
                            &mut caller.instructions[offsets.instructions as usize..instr_len],
                            new_ops,
                            &callee.operands,
                            &offsets,
                            results_op as u32 - offsets.operands,
                            &mut return_slots,
                        );
                    }

                    let bb = &mut caller.basic_blocks[results_bb];
                    bb.insts_start = caller.instructions.len() as u32;

                    // Map every return slot to the expected slot
                    // TODO replace this with phi nodes for multiple ret per inlined function
                    for (from, to) in return_slots.iter().zip(expected_slots) {
                        caller.instructions.push(ffi::Instruction {
                            kind: InstructionKind::Cast,
                            ops_start: caller.operands.len() as u32,
                            ops_len: 1,
                            has_result: true,
                            result: *to,
                        });
                        caller.operands.push(*from);
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

                    // This block was split up so we can continue with the next one
                    break;
                }
            }
        }

        self.mark_dead_functions(&inline_functions);
    }

    /// Local inlined functions are dead unless they are still referenced by a global (e.g. a vtable) or by any
    /// operand of a live function (a call that could not be inlined, an invoke or a function pointer) or are its
    /// personality
    fn mark_dead_functions(&mut self, inline_functions: &[bool]) {
        for (func, &inlined) in self.functions.iter_mut().zip(inline_functions) {
            func.flags.dead = inlined && func.flags.only_local;
        }

        let mut worklist: Vec<Function> = self
            .functions
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.flags.dead && !f.flags.extern_link)
            .map(|(i, _)| Function(i))
            .collect();
        let mut referenced: Vec<Function> = vec![];

        for global in &self.globals {
            for reloc in &global.relocations {
                if let Slot::Func(callee) = Slot::from_ffi(reloc.slot) {
                    referenced.push(callee);
                }
            }
        }

        loop {
            for callee in referenced.drain(..) {
                let callee_func = &mut self.functions[callee.0];
                if callee_func.flags.dead {
                    callee_func.flags.dead = false;
                    worklist.push(callee);
                }
            }

            let Some(f) = worklist.pop() else {
                break;
            };
            let func = &self.functions[f.0];
            if func.has_personality {
                referenced.push(Function(func.personality as usize));
            }
            for instr in &func.instructions {
                let ops = instr.ops_start as usize..(instr.ops_start + instr.ops_len) as usize;
                for op in &func.operands[ops] {
                    if let Slot::Func(callee) = Slot::from_ffi(*op) {
                        referenced.push(callee);
                    }
                }
            }
        }
    }
}

fn can_inline_call(
    caller: &ffi::Function,
    callee: &ffi::Function,
    args: &[u32],
) -> bool {
    // stop at variadic arguments
    if args.len() != callee.args.len() {
        return false;
    }
    if callee.has_personality && caller.has_personality && callee.personality != caller.personality {
        return false;
    }
    true
}

struct InlineOffsets {
    allocas: u32,
    callee_infos: u32,
    slots: u32,
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
}

fn map_blocks(basic_blocks: &mut [BasicBlock], inline_offsets: &InlineOffsets) {
    for bb in basic_blocks {
        bb.insts_start += inline_offsets.instructions;
    }
}

fn map_operands(operands: &mut [u32], argument_mapping: &[u32], inline_offsets: &InlineOffsets) {
    // map: val, alloca
    for operand in operands {
        match Slot::from_ffi(*operand) {
            Slot::Value(f, v) => {
                *operand = if v < argument_mapping.len() as u32 {
                    argument_mapping[v as usize]
                } else {
                    Slot::Value(f, v + inline_offsets.slots).to_ffi()
                };
            }
            Slot::Alloc(a) => *operand = Slot::Alloc(a + inline_offsets.allocas).to_ffi(),
            _ => {}
        };
    }
}

fn map_instructions(
    instructions: &mut [ffi::Instruction],
    operands: &mut [u32],
    callee_operands: &[u32],
    inline_offsets: &InlineOffsets,
    return_op: u32,
    return_slots: &mut Vec<u32>,
) {
    for instr in instructions {
        // map: br, condbr, switch, funcs (callee info), invoke
        match instr.kind {
            InstructionKind::Br => {
                // map op 0 raw bb
                operands[instr.ops_start as usize] += inline_offsets.basic_blocks;
            }
            InstructionKind::CondBr => {
                // Invoke: map op 1 and 2 (then and catch bb)
                // CondBr: map op 1 and 2 raw bb
                operands[instr.ops_start as usize + 1] += inline_offsets.basic_blocks;
                operands[instr.ops_start as usize + 2] += inline_offsets.basic_blocks;
            }
            InstructionKind::Switch => {
                // [cond, default, targets..., case values...]: map default and targets
                let targets = instr.ops_start as usize + 1..instr.ops_start as usize + 1 + instr.ops_len as usize / 2;
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
                if matches!(Slot::from_ffi(callee_operands[instr.ops_start as usize]), Slot::Value(..)) {
                    if matches!(Slot::from_ffi(operands[instr.ops_start as usize]), Slot::Func(_)) {
                        // The argument made it a direct call, which has no callee info
                        operands.copy_within(info + 1..instr.ops_start as usize + instr.ops_len as usize, info);
                        instr.ops_len -= 1;
                    } else {
                        operands[info] += inline_offsets.callee_infos;
                    }
                }
            }
            InstructionKind::Ret => {
                instr.kind = InstructionKind::Br;
                for i in 0..instr.ops_len {
                    return_slots.push(operands[(instr.ops_start + i) as usize]);
                }
                instr.ops_start = return_op;
                instr.ops_len = 1;
            }
            _ => {}
        }

        // map the start ops of the operands
        instr.ops_start += inline_offsets.operands;
        if instr.has_result {
            instr.result += inline_offsets.slots;
        }
    }
}
