use rustc_codegen_ssa::diagnostics::InvalidMonomorphization;
use crate::builder::Builder;
use rustc_codegen_ssa::RetagInfo;
use rustc_codegen_ssa::mir::IntrinsicResult;
use rustc_codegen_ssa::mir::operand::{OperandRef, OperandValue};
use rustc_codegen_ssa::mir::place::PlaceValue;
use rustc_codegen_ssa::traits::IntrinsicCallBuilderMethods;
use rustc_middle::ty::Instance;
use rustc_middle::ty::layout::TyAndLayout;
use rustc_span::{Span, sym};
use crate::shared::ir::InstructionKind;

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
        todo!()
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
