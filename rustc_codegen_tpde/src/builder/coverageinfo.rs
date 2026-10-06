use rustc_codegen_ssa::traits::CoverageInfoBuilderMethods;
use rustc_middle::mir::coverage::CoverageKind;
use rustc_middle::ty::Instance;
use crate::builder::Builder;

impl<'tcx> CoverageInfoBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn add_coverage(&mut self, instance: Instance<'tcx>, kind: &CoverageKind) {
        todo!()
    }
}