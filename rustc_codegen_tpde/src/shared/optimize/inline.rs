use std::env;
use std::sync::LazyLock;
use crate::shared::ffi;
use crate::shared::ffi::{BasicBlock, InstructionReference};
use crate::shared::ir::{ArgKind, InstructionKind, ModuleTpde, Slot};

// Highest cost a function without `#[inline(always)]` is inlined
static INLINE_COST: LazyLock<u32> = LazyLock::new(|| {
    match env::var("TPDE_INLINE_COST") {
        Ok(val) => val
            .parse::<u32>()
            .expect("TPDE_INLINE_COST has to be a number"),
        Err(_) => 32,
    }
});

impl ModuleTpde {
    pub fn inline_functions(&mut self) {
        // Inline functions bottom up
        // Inlinable functions:
        // - are leave functions -> call no other functions
        // - are marked always inlined or cheaper than the maximum specified cost
        // - has no byval arguments
        // - has maximum of one return value

        // info1 is not used during generation
        // we abuse it to store the follower of a block if it has to split up
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
            .filter(|(_, f)| is_inlinable(f))
            .map(|(i, _)| i)
            .collect();
        let mut return_slots: Vec<u32> = vec![];

        while let Some(f) = leaves.pop() {
            let mut all_inlined = true;

            for site in std::mem::take(&mut self.functions[f].called_by) {
                let [caller, callee] = self
                    .functions
                    .get_disjoint_mut([site.func as usize, f])
                    .expect("Trying to inline function into itself");

                all_inlined &= inline_call(caller, callee, &site, &mut return_slots);

                caller.calling -= 1;
                if is_inlinable(caller) {
                    leaves.push(site.func as usize);
                }
            }

            let func = &mut self.functions[f];
            func.flags.dead = all_inlined && func.flags.only_local && !func.flags.never_delete;
        }
    }
}

// Inline this call, returns if it could be inlined or not
fn inline_call(
    caller: &mut ffi::Function,
    callee: &ffi::Function,
    site: &InstructionReference,
    return_slots: &mut Vec<u32>,
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

    // landing pads of the callee need its personality
    if callee.has_personality && !caller.has_personality {
        caller.has_personality = true;
        caller.personality = callee.personality;
    }
    caller.cost += callee.cost;

    return_slots.clear();

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
        info1: prev_cont,
        info2: 0,
    });
    let results_bb = caller.basic_blocks.len();
    caller.basic_blocks.push(BasicBlock {
        insts_start: 0,
        insts_len: 0,
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
        basic_blocks: caller.basic_blocks.len() as u32,
        instructions: caller.instructions.len() as u32,
        operands: caller.operands.len() as u32,
    };

    // let's start copying the function
    copy_inline_function(caller, callee);

    {
        let block_len = caller.basic_blocks.len();
        map_blocks(
            &mut caller.basic_blocks[offsets.basic_blocks as usize..block_len],
            &offsets,
        )
    }

    // start mapping instructions
    {
        caller
            .operands
            .push(Slot::new_raw(results_bb as u32).to_ffi());
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
            return_slots,
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

    true
}

fn is_inlinable(f: &ffi::Function) -> bool {
    let wanted = f.calling == 0 &&
        (f.flags.always_inline || (cfg!(feature = "inline-cost") && f.cost <= *INLINE_COST));

    wanted
        && !f.basic_blocks.is_empty()
        && !f.args.iter().any(|arg| arg.kind == ArgKind::ByVal)
        && f.instructions
        .iter()
        .filter(|i| matches!(i.kind, InstructionKind::Ret))
        .count()
        <= 1
}

fn can_inline_call(caller: &ffi::Function, callee: &ffi::Function, args: &[u32]) -> bool {
    if callee.flags.extern_link {
        return false;
    }
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
                    Slot::Value(..)
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
