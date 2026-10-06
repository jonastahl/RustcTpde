use crate::builder::Builder;
use crate::context::CodegenCx;
use crate::shared::ir::{FullType, Slot};
use rustc_abi::{HasDataLayout, TargetDataLayout};
use rustc_codegen_ssa::mir::operand::OperandValue;
use rustc_codegen_ssa::mir::place::PlaceRef;
use rustc_codegen_ssa::traits::{AbiBuilderMethods, ArgAbiBuilderMethods, BaseTypeCodegenMethods, BuilderMethods, ConstCodegenMethods, LayoutTypeCodegenMethods};
use rustc_middle::bug;
use rustc_middle::ty::layout::{FnAbiError, FnAbiOfHelpers, FnAbiRequest, HasTypingEnv, LayoutError, LayoutOfHelpers, MaybeResult, TyAndLayout};
use rustc_middle::ty::{Ty, TypingEnv};
use rustc_span::Span;
use rustc_target::callconv::{ArgAbi, FnAbi, PassMode};

impl<'tpde, 'tcx> AbiBuilderMethods for Builder<'_, 'tpde, 'tcx> {
    fn get_param(&mut self, index: usize) -> Self::Value {
        (&*self).get_param(index)
    }
}

impl Builder<'_, '_, '_> {
    fn get_param(&self, index: usize) -> Slot {
        self.module.borrow().get_slot(self.basic_block.function(), index as u32)
    }
}

impl<'tpde, 'tcx> ArgAbiBuilderMethods<'tcx> for Builder<'_, 'tpde, 'tcx> {
    fn store_fn_arg(&mut self, arg_abi: &ArgAbi<'tcx, Ty<'tcx>>, idx: &mut usize, dst: PlaceRef<'tcx, Self::Value>) {
        let mut next = || {
            let val = *&(&*self).get_param(*idx);
            *idx += 1;
            val
        };
        match &arg_abi.mode {
            PassMode::Ignore => {}
            PassMode::Indirect { attrs: _, meta_attrs: Some(_), on_stack: _ } => {
                bug!("unsized `ArgAbi` cannot be stored");
            }
            PassMode::Direct(_)
            | PassMode::Indirect { attrs: _, meta_attrs: None, on_stack: _ } => {
                let val = next();
                self.store_arg(arg_abi, val, dst);
            }
            PassMode::Pair(..) => {
                OperandValue::Pair(next(), next()).store(self, dst);
            }
            PassMode::Cast { cast, .. } => {
                match self.cast_backend_type(&cast) {
                    FullType::Single(_) => {
                        let val = next();
                        self.store_arg(arg_abi, val, dst);
                    }
                    FullType::Pair(_, _, offset_b) => {
                        let val_a = next();
                        let val_b = next();

                        self.store_arg(arg_abi, val_a, dst);

                        let offset = self.const_usize(offset_b as u64);
                        let ptr = self.inbounds_gep(self.type_i8(), dst.val.llval, &[offset]);
                        self.store(val_b, ptr, dst.val.align);
                    }
                    _ => todo!("Handle larger casts"),
                }
            }
        }
    }

    fn store_arg(&mut self, arg_abi: &ArgAbi<'tcx, Ty<'tcx>>, val: Self::Value, dst: PlaceRef<'tcx, Self::Value>) {
        self.store(
            val,
            dst.val.llval,
            dst.val.align
        );
    }
}

impl<'tcx> HasDataLayout for Builder<'_, '_, 'tcx> {
    fn data_layout(&self) -> &TargetDataLayout {
        self.tcx.data_layout()
    }
}

impl<'tcx> HasDataLayout for CodegenCx<'_, 'tcx> {
    fn data_layout(&self) -> &TargetDataLayout {
        &self.data_layout
    }
}

impl<'tcx> HasTypingEnv<'tcx> for Builder<'_, '_, 'tcx> {
    fn typing_env(&self) -> TypingEnv<'tcx> {
        self.cx.typing_env()
    }
}

impl<'tcx> HasTypingEnv<'tcx> for CodegenCx<'_, 'tcx> {
    fn typing_env(&self) -> TypingEnv<'tcx> {
        TypingEnv::fully_monomorphized()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
    fn handle_layout_err(&self, err: LayoutError<'tcx>, span: Span, ty: Ty<'tcx>) -> <Self::LayoutOfResult as MaybeResult<TyAndLayout<'tcx>>>::Error {
        todo!()
    }
}

impl<'tcx> LayoutOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    fn handle_layout_err(&self, err: LayoutError<'tcx>, span: Span, ty: Ty<'tcx>) -> <Self::LayoutOfResult as MaybeResult<TyAndLayout<'tcx>>>::Error {
        todo!()
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for Builder<'_, '_, 'tcx> {
    fn handle_fn_abi_err(&self, err: FnAbiError<'tcx>, span: Span, fn_abi_request: FnAbiRequest<'tcx>) -> <Self::FnAbiOfResult as MaybeResult<&'tcx FnAbi<'tcx, Ty<'tcx>>>>::Error {
        todo!()
    }
}

impl<'tcx> FnAbiOfHelpers<'tcx> for CodegenCx<'_, 'tcx> {
    fn handle_fn_abi_err(&self, err: FnAbiError<'tcx>, span: Span, fn_abi_request: FnAbiRequest<'tcx>) -> <Self::FnAbiOfResult as MaybeResult<&'tcx FnAbi<'tcx, Ty<'tcx>>>>::Error {
        todo!()
    }
}