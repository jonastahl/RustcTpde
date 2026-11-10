use crate::shared::ffi;
use crate::shared::ir::{
    Function, InstructionKind, ModuleTpde, Slot, Type, alloc_used_a, alloc_used_b, written_once,
};
use rustc_span::bug;

impl ModuleTpde {
    /// We start by only promoting allocas
    /// that have been used only once either single or as a pair.
    /// We rely on that Rustc never generates loads before writing to the alloca first
    /// as it would be undefined anyway.
    pub fn promote_allocas(&mut self) {
        for func in &mut self.functions {
            for (i, alloc) in func.allocas.iter().enumerate() {
                if !written_once(alloc.access_info.state) {
                    continue;
                }

                let slot_a = if alloc_used_a(&alloc.access_info) {
                    func.slots.push(ffi::Slot { ty: Type::Void });
                    Some(Slot::Value(Function(0), func.slots.len() as u32 - 1))
                } else {
                    None
                };
                let slot_b = if alloc_used_b(&alloc.access_info) {
                    func.slots.push(ffi::Slot { ty: Type::Void });
                    Some(Slot::Value(Function(0), func.slots.len() as u32 - 1))
                } else {
                    None
                };

                for bb in &func.basic_blocks {
                    let insts_start = bb.insts_start as usize;
                    for instr_index in insts_start..insts_start + bb.insts_len as usize {
                        let instr = &mut func.instructions[instr_index];
                        let start = instr.ops_start as usize;
                        let ops = &mut func.operands[start..start + instr.ops_len as usize];

                        match instr.kind {
                            InstructionKind::Load
                                if let Slot::Alloc(a) = Slot::from_ffi(ops[0])
                                    && a == i as u32 =>
                            {
                                instr.kind = InstructionKind::Cast;
                                instr.ops_len = 1;
                                let Some(slot_a) = slot_a else { bug!() };
                                ops[0] = slot_a.to_ffi();
                            }
                            InstructionKind::Store
                                if let Slot::Alloc(a) = Slot::from_ffi(ops[1])
                                    && a == i as u32 =>
                            {
                                instr.kind = InstructionKind::Cast;
                                instr.ops_len = 1;
                                let Some(slot_a) = slot_a else { bug!() };

                                // Save the type of the stored value
                                let Slot::Value(_, slot_from) = Slot::from_ffi(ops[0]) else {
                                    bug!()
                                };
                                let Slot::Value(_, slot_to) = slot_a else {
                                    bug!()
                                };
                                func.slots[slot_to as usize].ty = func.slots[slot_from as usize].ty;

                                instr.result = slot_a.to_ffi();
                            }
                            InstructionKind::GEP
                                if let Slot::Alloc(a) = Slot::from_ffi(ops[0])
                                    && a == i as u32 =>
                            {
                                let instr = &mut func.instructions[instr_index + 1];
                                let start = instr.ops_start as usize;
                                let ops = &mut func.operands[start..start + instr.ops_len as usize];

                                match instr.kind {
                                    InstructionKind::Load => {
                                        instr.kind = InstructionKind::Cast;
                                        instr.ops_len = 1;
                                        let Some(slot_b) = slot_b else { bug!() };
                                        ops[0] = slot_b.to_ffi();
                                    }
                                    InstructionKind::Store => {
                                        instr.kind = InstructionKind::Cast;
                                        instr.ops_len = 1;
                                        let Some(slot_b) = slot_b else { bug!() };

                                        // Save the type of the stored value
                                        let Slot::Value(_, slot_from) = Slot::from_ffi(ops[0])
                                        else {
                                            bug!()
                                        };
                                        let Slot::Value(_, slot_to) = slot_b else {
                                            bug!()
                                        };
                                        func.slots[slot_to as usize].ty =
                                            func.slots[slot_from as usize].ty;

                                        instr.result = slot_b.to_ffi();
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
}
