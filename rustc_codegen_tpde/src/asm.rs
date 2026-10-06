use rustc_ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_codegen_ssa::traits::{AsmBuilderMethods, AsmCodegenMethods, GlobalAsmOperandRef, InlineAsmOperandRef};
use rustc_middle::ty::Instance;
use rustc_span::Span;
use crate::builder::Builder;
use crate::context::CodegenCx;

impl<'tcx> AsmBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn codegen_inline_asm(&mut self, template: &[InlineAsmTemplatePiece], operands: &[InlineAsmOperandRef<'tcx, Self>], options: InlineAsmOptions, line_spans: &[Span], instance: Instance<'_>, dest: Option<Self::BasicBlock>, catch_funclet: Option<(Self::BasicBlock, Option<&Self::Funclet>)>) {
        todo!()
    }
}

impl<'tcx> AsmCodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn codegen_global_asm(&mut self, template: &[InlineAsmTemplatePiece], operands: &[GlobalAsmOperandRef<'tcx>], options: InlineAsmOptions, line_spans: &[Span]) {
        todo!()
    }

    fn mangled_name(&self, instance: Instance<'tcx>) -> String {
        todo!()
    }
}