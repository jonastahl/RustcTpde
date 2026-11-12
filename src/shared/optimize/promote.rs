use crate::shared::ffi;
use crate::shared::ir::{
    Function, InstructionKind, ModuleTpde, Slot, Type, decode_const, written_once_a, written_once_b,
};
use rustc_span::bug;

type PromotedParts = (Option<u32>, Option<u32>);

impl ModuleTpde {
    /// We start by only promoting allocas
    /// that have been used only once either single or as a pair.
    /// We rely on that Rustc never generates loads before writing to the alloca first
    /// as it would be undefined anyway.
    pub fn promote_allocas(&mut self) {
        for func in &mut self.functions {
            let mut promoted: Vec<PromotedParts> = Vec::with_capacity(func.allocas.len());
            for alloc in &func.allocas {
                let info = alloc.access_info;
                // A call could read or write the alloca behind our back
                let promotable = info.call_forwards == 0;
                let slot_a =
                    (promotable && written_once_a(info.state)).then(|| new_slot(&mut func.slots));
                let slot_b =
                    (promotable && written_once_b(info.state)).then(|| new_slot(&mut func.slots));
                promoted.push((slot_a, slot_b));
            }
            if promoted.iter().all(|parts| *parts == (None, None)) {
                continue;
            }

            for bb in &func.basic_blocks {
                let insts_start = bb.insts_start as usize;
                for instr_index in insts_start..insts_start + bb.insts_len as usize {
                    let instr = &mut func.instructions[instr_index];
                    let start = instr.ops_start as usize;
                    let ops = &mut func.operands[start..start + instr.ops_len as usize];

                    match instr.kind {
                        InstructionKind::Load
                            if let Slot::Alloc(a) = Slot::from_ffi(ops[0])
                                && let Some(slot_a) = promoted[a as usize].0 =>
                        {
                            instr.kind = InstructionKind::Cast;
                            instr.ops_len = 1;
                            ops[0] = Slot::Value(Function(0), slot_a).to_ffi();
                        }
                        InstructionKind::Store
                            if let Slot::Alloc(a) = Slot::from_ffi(ops[1])
                                && let Some(slot_a) = promoted[a as usize].0 =>
                        {
                            instr.kind = InstructionKind::Cast;
                            instr.ops_len = 1;
                            instr.has_result = true;

                            // Save the type of the stored value
                            let ty = value_type(
                                &func.slots,
                                &func.phis,
                                &self.consts,
                                &self.const_vectors,
                                ops[0],
                            );
                            func.slots[slot_a as usize].ty = ty;

                            instr.result = Slot::Value(Function(0), slot_a).to_ffi();
                        }
                        InstructionKind::PairGEP
                            if let Slot::Alloc(a) = Slot::from_ffi(ops[0])
                                && let Some(slot_b) = promoted[a as usize].1 =>
                        {
                            let instr = &mut func.instructions[instr_index + 1];
                            let start = instr.ops_start as usize;
                            let ops = &mut func.operands[start..start + instr.ops_len as usize];

                            match instr.kind {
                                InstructionKind::Load => {
                                    instr.kind = InstructionKind::Cast;
                                    instr.ops_len = 1;
                                    ops[0] = Slot::Value(Function(0), slot_b).to_ffi();
                                }
                                InstructionKind::Store => {
                                    instr.kind = InstructionKind::Cast;
                                    instr.ops_len = 1;
                                    instr.has_result = true;

                                    // Save the type of the stored value
                                    let ty = value_type(
                                        &func.slots,
                                        &func.phis,
                                        &self.consts,
                                        &self.const_vectors,
                                        ops[0],
                                    );
                                    func.slots[slot_b as usize].ty = ty;

                                    instr.result = Slot::Value(Function(0), slot_b).to_ffi();
                                }
                                _ => bug!(),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn new_slot(slots: &mut Vec<ffi::Slot>) -> u32 {
    slots.push(ffi::Slot { ty: Type::Void });
    slots.len() as u32 - 1
}

fn value_type(
    slots: &[ffi::Slot],
    phis: &[ffi::Phi],
    consts: &[ffi::Value],
    const_vectors: &[ffi::ConstVector],
    value: u32,
) -> Type {
    match Slot::from_ffi(value) {
        Slot::Value(_, v) => slots[v as usize].ty,
        Slot::Phi(_, p) => phis[p as usize].ty,
        Slot::Const(c) => decode_const(consts, c).0,
        Slot::ConstVector(v) => const_vectors[v as usize].ty,
        Slot::Alloc(_) | Slot::Func(_) | Slot::Global(_) | Slot::GlobalPtr(_) => Type::ptr,
        slot => bug!("not a value: {slot:?}"),
    }
}
