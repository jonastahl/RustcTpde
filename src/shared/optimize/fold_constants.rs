use crate::shared::ffi;
use crate::shared::ffi::{InstructionKind, ModuleTpde};
use crate::shared::ir::{Slot, size_of_type};

fn sign_extend(value: u128, bits: u32) -> i128 {
    ((value << (128 - bits)) as i128) >> (128 - bits)
}

fn all_const(ops: &[u32]) -> bool {
    ops.iter()
        .all(|op| matches!(Slot::from_ffi(*op), Slot::Const(_)))
}

fn mask(bits: u32) -> u128 {
    u128::MAX >> (128 - bits)
}

impl ModuleTpde {
    /// Propagates constants that might have shown up after inlining, replaces if applicable:
    /// - condbr to br
    /// - gets rid of overflow checks
    /// - replaces results of mov
    pub(crate) fn fold_constants(&mut self, function: u32) {
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
                if instr.kind == InstructionKind::Mov {
                    known[instr.result as usize] = Some(func.operands[ops]);
                    continue;
                }
                if instr.kind == InstructionKind::Noop {
                    continue;
                }

                if instr.has_result & all_const(&func.operands[ops..ops_end])
                    && let Some(value) = self.eval_const(instr.kind, &func.operands[ops..ops_end])
                {
                    let ty = func.slots[instr.result as usize].ty;
                    known[instr.result as usize] = Some(self.add_const(ty, value).to_ffi());

                    // An overflow check is only defined by the arithmetic before it, so both have to go
                    let func = &self.functions[function as usize];
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
                    let bb = &mut self.functions[function as usize].basic_blocks[block];
                    if !taken {
                        bb.suc_start += 1;
                    }
                    bb.suc_len = 1;
                    instr.kind = InstructionKind::Br;
                    instr.ops_len = 0;
                }

                let func = &mut self.functions[function as usize];

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
            let bb = &mut func.basic_blocks[block];
            bb.insts_len = (write - start) as u32;

            stack.extend(func.successors[bb.successors()].iter().map(|&s| s as usize));
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

        let (slhs, srhs) = (sign_extend(lhs, bits), sign_extend(rhs, bits));
        Some(match kind {
            InstructionKind::Add => lhs.wrapping_add(rhs) & mask(bits),
            InstructionKind::Sub => lhs.wrapping_sub(rhs) & mask(bits),
            InstructionKind::Mul => lhs.wrapping_mul(rhs) & mask(bits),
            InstructionKind::CMPeq => (lhs == rhs) as u128,
            InstructionKind::CMPne => (lhs != rhs) as u128,
            InstructionKind::CMPugt => (lhs > rhs) as u128,
            InstructionKind::CMPuge => (lhs >= rhs) as u128,
            InstructionKind::CMPult => (lhs < rhs) as u128,
            InstructionKind::CMPule => (lhs <= rhs) as u128,
            InstructionKind::CMPsgt => (slhs > srhs) as u128,
            InstructionKind::CMPsge => (slhs >= srhs) as u128,
            InstructionKind::CMPslt => (slhs < srhs) as u128,
            InstructionKind::CMPsle => (slhs <= srhs) as u128,
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

        if signed {
            let (slhs, shrs) = (sign_extend(lhs, bits), sign_extend(rhs, bits));
            let result = match kind {
                InstructionKind::Add => slhs.checked_add(shrs),
                InstructionKind::Sub => slhs.checked_sub(shrs),
                InstructionKind::Mul => slhs.checked_mul(shrs),
                _ => unreachable!(),
            };
            result.is_none_or(|r| sign_extend(r as u128 & mask(bits), bits) != r)
        } else {
            let result = match kind {
                InstructionKind::Add => lhs.checked_add(rhs),
                InstructionKind::Sub => lhs.checked_sub(rhs),
                InstructionKind::Mul => lhs.checked_mul(rhs),
                _ => unreachable!(),
            };
            result.is_none_or(|r| r > mask(bits))
        }
    }
}
