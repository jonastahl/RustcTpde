use super::OptConfig;
use crate::shared::ffi;
use crate::shared::ffi::AccessInfoState;
use crate::shared::ffi::{BasicBlock, Instruction, InstructionReference, Phi};
use crate::shared::ir::{ArgKind, InstructionKind, ModuleTpde, Slot};
use crate::shared::ir::{access_info_intern, alloc_used_a, alloc_used_b, merge_access_states};
use std::collections::BTreeSet;

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
            .filter(|(_, f)| f.calling == 0)
            .map(|(i, _)| i)
            .collect();
        let mut returns: Vec<ReturnSite> = vec![];

        // Used over hash set for determinism
        let mut to_optimize: BTreeSet<u32> = BTreeSet::new();

        while let Some(f) = leaves.pop() {
            if to_optimize.remove(&(f as u32)) {
                self.fold_constants(f as u32);
            }

            let inline = is_inlinable(&self.functions[f], config.inline_cost);
            let mut all_inlined = true;

            for site in std::mem::take(&mut self.functions[f].called_by) {
                let [caller, callee] = self
                    .functions
                    .get_disjoint_mut([site.func as usize, f])
                    .expect("Trying to inline function into itself");

                if inline {
                    all_inlined &= inline_call(caller, callee, &site, &mut returns, config.promote);
                    to_optimize.insert(site.func);
                }

                caller.calling -= 1;
                if caller.calling == 0 {
                    leaves.push(site.func as usize);
                }
            }

            if inline {
                let func = &mut self.functions[f];
                func.flags.dead = all_inlined && func.flags.only_local && !func.flags.never_delete;
            }
        }

        for f in to_optimize {
            self.fold_constants(f);
        }
    }
}

fn replace_phi_predecessor(func: &mut ffi::Function, from: u32, to: u32) {
    if func.phis.is_empty() {
        return;
    }
    let new_block = &func.basic_blocks[to as usize];
    let (from, to) = (Slot::new_raw(from).to_ffi(), Slot::new_raw(to).to_ffi());

    for &target in &func.successors[new_block.successors()] {
        let target = &func.basic_blocks[target as usize];
        for phi in
            &func.phis[target.phi_start as usize..(target.phi_start + target.phi_len) as usize]
        {
            let incoming = &mut func.operands
                [phi.incoming_start as usize..(phi.incoming_start + phi.incoming_len * 2) as usize];
            for block in incoming.iter_mut().step_by(2) {
                if *block == from {
                    *block = to;
                }
            }
        }
    }
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

    // split up the block after the call, continuation gets all the successors
    let cont_idx = caller.basic_blocks.len() as u32;
    let cur = &mut caller.basic_blocks[block];
    cur.insts_len = offset + 1;
    let prev_cont = cur.info1;
    cur.info1 = cont_idx;
    let (suc_start, suc_len) = (cur.suc_start, cur.suc_len);

    caller.basic_blocks.push(BasicBlock {
        insts_start: call_idx as u32 + 1,
        insts_len: insts_len - offset - 1,
        phi_start: 0,
        phi_len: 0,
        suc_start: suc_start,
        suc_len: suc_len,
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
        suc_start: caller.successors.len() as u32,
        suc_len: 1,
        info1: 0,
        info2: 0,
    });
    caller.successors.push(cont_idx);

    // switch to a br instruction jumping to the first basic block of the inlined function
    let cur = &mut caller.basic_blocks[block];
    cur.suc_start = caller.successors.len() as u32;
    cur.suc_len = 1;
    caller.successors.push(caller.basic_blocks.len() as u32);
    let call = &mut caller.instructions[call_idx];
    call.kind = InstructionKind::Br;
    call.ops_len = 0;

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
        successors: caller.successors.len() as u32,
        instructions: caller.instructions.len() as u32,
        operands: caller.operands.len() as u32,
    };

    returns.clear();

    // let's start copying the function
    copy_inline_function(caller, callee);

    let (ex_ops, new_ops) = caller.operands.split_at_mut(offsets.operands as usize);
    let argument_mapping = &ex_ops[args_start..args_end];
    for i in offsets.basic_blocks as usize..caller.basic_blocks.len() {
        map_block(
            &mut caller.basic_blocks[i],
            i as u32,
            &mut caller.instructions,
            new_ops,
            &mut caller.phis,
            &mut caller.successors,
            &callee.operands,
            argument_mapping,
            &offsets,
            returns,
            results_bb as u32,
        );
    }

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
            Slot::new_phi(results_bb as u32, bb.phi_len - 1).to_ffi()
        };
        caller.instructions.push(ffi::Instruction {
            kind: InstructionKind::Mov,
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
        ops_len: 0,
        has_result: false,
        result: 0,
    });

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
    successors: u32,
    instructions: u32,
    operands: u32,
}

/// Transfer all the necessary information to the new caller functions
fn copy_inline_function(caller: &mut ffi::Function, callee: &ffi::Function) {
    caller.allocas.extend_from_slice(&callee.allocas);
    caller.callee_infos.extend_from_slice(&callee.callee_infos);

    caller.basic_blocks.extend_from_slice(&callee.basic_blocks);
    caller.successors.extend_from_slice(&callee.successors);
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
    successors: &mut Vec<u32>,
    callee_operands: &[u32],
    argument_mapping: &[u32],
    inline_offsets: &InlineOffsets,
    returns: &mut Vec<ReturnSite>,
    return_bb: u32,
) {
    bb.insts_start += inline_offsets.instructions;
    bb.phi_start += inline_offsets.phis;
    bb.suc_start += inline_offsets.successors;

    for suc in &mut successors[bb.successors()] {
        *suc += inline_offsets.basic_blocks;
    }

    {
        let start = bb.insts_start as usize;
        let end = start + bb.insts_len as usize;
        for instr in &mut instructions[start..end] {
            let returned = map_instruction(
                instr,
                block_index,
                operands,
                callee_operands,
                argument_mapping,
                inline_offsets,
                returns,
                return_bb,
            );
            if returned {
                bb.suc_start = successors.len() as u32;
                bb.suc_len = 1;
                successors.push(return_bb);
            }
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
    inline_offsets: &InlineOffsets,
    returns: &mut Vec<ReturnSite>,
    return_bb: u32,
) -> bool {
    {
        let start = instr.ops_start as usize;
        let end = start + instr.ops_len as usize;
        for op in &mut operands[start..end] {
            map_operands(op, argument_mapping, inline_offsets);
        }
    }

    // map: call
    match instr.kind {
        InstructionKind::Call | InstructionKind::Invoke => {
            let info = instr.ops_start as usize + 1;

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
            instr.ops_len = 0;
            return true;
        }
        _ => {}
    }

    // map the start ops of the operands
    instr.ops_start += inline_offsets.operands;
    if instr.has_result {
        instr.result += inline_offsets.slots;
    }
    false
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
        Slot::Phi(f, block, ind) => {
            *operand = Slot::Phi(f, block + inline_offsets.basic_blocks, ind).to_ffi()
        }
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
