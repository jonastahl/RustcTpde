use crate::builder::Builder;
use crate::context::CodegenCx;
use rustc_ast::{InlineAsmOptions, InlineAsmTemplatePiece};
use rustc_codegen_ssa::traits::{
    AsmBuilderMethods, AsmCodegenMethods, BuilderMethods, GlobalAsmOperandRef, InlineAsmOperandRef,
    IntrinsicCallBuilderMethods,
};
use rustc_middle::ty::Instance;
use rustc_middle::ty::print::with_no_trimmed_paths;
use rustc_span::Span;

impl<'tcx> AsmBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn codegen_inline_asm(
        &mut self,
        template: &[InlineAsmTemplatePiece],
        operands: &[InlineAsmOperandRef<'tcx, Self>],
        options: InlineAsmOptions,
        line_spans: &[Span],
        instance: Instance<'_>,
        dest: Option<Self::BasicBlock>,
        catch_funclet: Option<(Self::BasicBlock, Option<&Self::Funclet>)>,
    ) {
        let tcx = self.tcx;
        let span = line_spans
            .first()
            .copied()
            .unwrap_or_else(|| tcx.def_span(instance.def_id()));
        tcx.sess
            .dcx()
            .struct_span_warn(
                span,
                format!(
                    "TPDE: inline assembly is not supported, emitting a trap instead (in `{}`)",
                    with_no_trimmed_paths!(tcx.def_path_str(instance.def_id()))
                ),
            )
            .with_note(format!(
                "asm template: `{}`",
                InlineAsmTemplatePiece::to_string(template).replace('\n', "; ")
            ))
            .emit();

        self.abort_immediate();
        if let Some(dest) = dest {
            self.br(dest);
        }
    }
}

impl<'tcx> AsmCodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn codegen_global_asm(
        &mut self,
        template: &[InlineAsmTemplatePiece],
        operands: &[GlobalAsmOperandRef<'tcx>],
        options: InlineAsmOptions,
        line_spans: &[Span],
        extra_rust_target_features: &[String],
    ) {
        todo!("global asm: {template:?}")
    }

    fn mangled_name(&self, instance: Instance<'tcx>) -> String {
        todo!()
    }
}
