use std::ops::Range;
use rustc_abi::Size;
use rustc_codegen_ssa::mir::debuginfo::VariableKind;
use rustc_codegen_ssa::traits::{DebugInfoBuilderMethods, DebugInfoCodegenMethods};
use rustc_middle::ty::{ExistentialTraitRef, Instance, Ty};
use rustc_span::{BytePos, SourceFile, Span, Symbol};
use rustc_target::callconv::FnAbi;
use crate::builder::Builder;
use crate::context::CodegenCx;

impl<'tcx> DebugInfoBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn dbg_scope_fn(&mut self, instance: Instance<'tcx>, fn_abi: &FnAbi<'tcx, Ty<'tcx>>, maybe_definition_llfn: Option<Self::Function>) -> Self::DIScope {
        todo!()
    }

    fn dbg_create_lexical_block(&mut self, pos: BytePos, parent_scope: Self::DIScope) -> Self::DIScope {
        todo!()
    }

    fn dbg_location_clone_with_discriminator(&mut self, loc: Self::DILocation, discriminator: u32) -> Option<Self::DILocation> {
        todo!()
    }

    fn dbg_loc(&mut self, scope: Self::DIScope, inlined_at: Option<Self::DILocation>, span: Span) -> Self::DILocation {
        todo!()
    }

    fn extend_scope_to_file(&mut self, scope_metadata: Self::DIScope, file: &SourceFile) -> Self::DIScope {
        todo!()
    }

    fn create_dbg_var(&mut self, variable_name: Symbol, variable_type: Ty<'tcx>, scope_metadata: Self::DIScope, variable_kind: VariableKind, span: Span) -> Self::DIVariable {
        todo!()
    }

    fn dbg_var_addr(&mut self, dbg_var: Self::DIVariable, dbg_loc: Self::DILocation, variable_alloca: Self::Value, direct_offset: Size, indirect_offsets: &[Size], fragment: &Option<Range<Size>>) {
        todo!()
    }

    fn dbg_var_value(&mut self, dbg_var: Self::DIVariable, dbg_loc: Self::DILocation, value: Self::Value, direct_offset: Size, indirect_offsets: &[Size], fragment: &Option<Range<Size>>) {
        todo!()
    }

    fn set_dbg_loc(&mut self, dbg_loc: Self::DILocation) {
        todo!()
    }

    fn clear_dbg_loc(&mut self) {
        todo!()
    }

    fn insert_reference_to_gdb_debug_scripts_section_global(&mut self) {
        // todo!()
    }

    fn set_var_name(&mut self, value: Self::Value, name: &str) {
        // TODO
        // todo!()
    }
}

impl<'tcx> DebugInfoCodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn create_vtable_debuginfo(&self, ty: Ty<'tcx>, trait_ref: Option<ExistentialTraitRef<'tcx>>, vtable: Self::Value) {
        // TODO
        // todo!()
    }
}