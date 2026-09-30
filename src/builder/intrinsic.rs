use crate::builder::Builder;
use crate::shared::ir::{InstructionKind, Type};
use rustc_codegen_ssa::RetagInfo;
use rustc_codegen_ssa::diagnostics::InvalidMonomorphization;
use rustc_codegen_ssa::mir::IntrinsicResult;
use rustc_codegen_ssa::mir::operand::{OperandRef, OperandValue};
use rustc_codegen_ssa::mir::place::PlaceValue;
use rustc_codegen_ssa::traits::{BuilderMethods, IntrinsicCallBuilderMethods};
use rustc_middle::{bug, span_bug};
use rustc_middle::ty::Instance;
use rustc_middle::ty::layout::TyAndLayout;
use rustc_span::{Span, sym};

impl<'tcx> IntrinsicCallBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn codegen_intrinsic_call(
        &mut self,
        instance: Instance<'tcx>,
        args: &[OperandRef<'tcx, Self::Value>],
        result_layout: TyAndLayout<'tcx>,
        result_place: Option<PlaceValue<Self::Value>>,
        span: Span,
    ) -> IntrinsicResult<'tcx, Self::Value> {
        let name = self.tcx.item_name(instance.def_id());

        match name {
            sym::black_box => {
                let input_operand = args[0];

                IntrinsicResult::Operand(input_operand.val)
            }
            sym::ctpop => {
                let OperandValue::Immediate(input_val) = args[0].val else {
                    todo!()
                };
                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::ctpop,
                    vec![input_val]
                );

                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::saturating_add
            | sym::saturating_sub => {
                let ty = args[0].layout.ty;
                if !ty.is_integral() {
                    let err = self.tcx.dcx().emit_err(InvalidMonomorphization::BasicIntegerType {
                        span,
                        name,
                        ty,
                    });
                    return IntrinsicResult::Err(err);
                }
                let (size, signed) = ty.int_size_and_signed(self.tcx);

                let is_add = name == sym::saturating_add;
                let lhs = args[0].immediate();
                let rhs = args[1].immediate();
                let instruction = match (is_add, signed) {
                    (true, true) => InstructionKind::sat_sadd,
                    (true, false) => InstructionKind::sat_uadd,
                    (false, true) => InstructionKind::sat_ssub,
                    (false, false) => InstructionKind::sat_usub
                };

                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    instruction,
                    vec![lhs, rhs]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::is_val_statically_known => {
                let result = self.cx.module.borrow_mut()
                    .add_const(Type::Bool, false as u128);
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::compare_bytes => {
                let cmp = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::MemCmp,
                    vec![args[0].immediate(), args[1].immediate(), args[2].immediate()],
                    Type::i32
                );
                IntrinsicResult::Operand(OperandValue::Immediate(cmp))
            }
            sym::select_unpredictable => {
                let cond = args[0].immediate();
                assert_eq!(args[1].layout, args[2].layout);

                let val = match (args[1].val, args[2].val) {
                    (OperandValue::Ref(true_val), OperandValue::Ref(false_val)) => {
                        assert!(true_val.llextra.is_none());
                        assert!(false_val.llextra.is_none());
                        assert_eq!(true_val.align, false_val.align);

                        let ptr = self.select(cond, true_val.llval, false_val.llval);
                        OperandValue::Ref(PlaceValue::new_sized(ptr, true_val.align))
                    }
                    (OperandValue::Immediate(true_val), OperandValue::Immediate(false_val)) => {
                        OperandValue::Immediate(self.select(cond, true_val, false_val))
                    }
                    (OperandValue::Pair(true_a, true_b), OperandValue::Pair(false_a, false_b)) => {
                        let a = self.select(cond, true_a, false_a);
                        let b = self.select(cond, true_b, false_b);
                        OperandValue::Pair(a, b)
                    }
                    _ => span_bug!(span, "Incompatible OperandValue for select_unpredictable"),
                };

                IntrinsicResult::Operand(val)
            }
            sym::abort => {
                self.abort();
                IntrinsicResult::Operand(OperandValue::ZeroSized)
            }
            sym::ctlz_nonzero
            | sym::ctlz
            | sym::cttz_nonzero
            | sym::cttz => {
                let instr = match name {
                    sym::ctlz_nonzero => InstructionKind::ctlz_nonzero,
                    sym::ctlz => InstructionKind::ctlz,
                    sym::cttz_nonzero => InstructionKind::cttz_nonzero,
                    sym::cttz => InstructionKind::cttz,
                    _ => bug!(),
                };
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    instr,
                    vec![args[0].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            _ => {
                panic!("Unimplemented intrinsic: {}", name.as_str());
            }
        }
    }

    fn codegen_llvm_intrinsic_call(
        &mut self,
        instance: Instance<'tcx>,
        args: &[OperandRef<'tcx, Self::Value>],
        is_cleanup: bool,
    ) -> Self::Value {
        todo!()
    }

    fn abort(&mut self) {
        self.module.borrow_mut().add_instruction(self.basic_block,
                                                 InstructionKind::Abort,
                                                 vec![])
    }

    fn assume(&mut self, val: Self::Value) {
        // Just some tips for the optimizer
    }

    fn expect(&mut self, cond: Self::Value, expected: bool) -> Self::Value {
        cond
    }

    fn type_checked_load(
        &mut self,
        llvtable: Self::Value,
        vtable_byte_offset: u64,
        typeid: &[u8],
    ) -> Self::Value {
        todo!()
    }

    fn va_start(&mut self, val: Self::Value) {
        todo!()
    }

    fn retag_mem(&mut self, place: Self::Value, info: &RetagInfo<Self::Value>) {
        todo!()
    }

    fn retag_reg(&mut self, ptr: Self::Value, info: &RetagInfo<Self::Value>) -> Self::Value {
        todo!()
    }
}
