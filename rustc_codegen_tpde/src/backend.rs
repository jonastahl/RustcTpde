use crate::context::SimpleCx;
use crate::shared::ir::*;
use crate::{allocator, back, base};
use rustc_codegen_ssa::back::lto::ThinModule;
use rustc_codegen_ssa::back::write::{CodegenContext, FatLtoInput, ModuleConfig, SharedEmitter, TargetMachineFactoryFn, ThinLtoInput};
use rustc_codegen_ssa::target_features::internal_target_features;
use rustc_codegen_ssa::traits::{
    CodegenBackend, ExtraBackendMethods, ModuleBufferMethods, WriteBackendMethods,
};
use rustc_codegen_ssa::{CompiledModule, CompiledModules, CrateInfo, ModuleCodegen, TargetConfig};
use rustc_data_structures::profiling::SelfProfilerRef;
use rustc_data_structures::smallvec::SmallVec;
use rustc_errors::DiagCtxt;
use rustc_middle::dep_graph::{WorkProduct, WorkProductMap};
use rustc_middle::ty::TyCtxt;
use rustc_middle::util::Providers;
use rustc_session::config::{OptLevel, OutputFilenames, PrintRequest};
use rustc_session::{IncrCompSession, Session};
use rustc_span::Symbol;
use std::any::Any;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct TpdeCodegenBackend();

pub struct ModuleBuffer();
pub struct OwnedTargetMachine();
pub struct ThinData();

impl TpdeCodegenBackend {
    pub fn new() -> TpdeCodegenBackend {
        TpdeCodegenBackend()
    }
}

unsafe impl Send for Module {}
unsafe impl Sync for Module {}

impl ExtraBackendMethods for TpdeCodegenBackend {
    type Module = Module;

    fn codegen_allocator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        module_name: &str,
        methods: &[rustc_ast::expand::allocator::AllocatorMethod],
    ) -> Self::Module {
        let mut module = RefCell::new(Module::new());
        let scx = SimpleCx::new(&mut module);
        allocator::codegen(tcx, scx, module_name, methods);
        module.into_inner()
    }

    fn compile_codegen_unit(
        &self,
        tcx: TyCtxt<'_>,
        cgu_name: Symbol,
    ) -> (ModuleCodegen<Self::Module>, u64) {
        base::compile_codegen_unit(tcx, cgu_name)
    }
}

impl WriteBackendMethods for TpdeCodegenBackend {
    // implementation similar to gcc for less convoluted solution

    type Module = Module;
    type TargetMachine = ();
    type ModuleBuffer = ModuleBuffer;
    type ThinData = ();

    fn target_machine_factory(
        &self,
        _sess: &Session,
        _opt_level: OptLevel,
        _target_features: &[String],
    ) -> TargetMachineFactoryFn<Self> {
        Arc::new(|_, _| ())
    }

    fn optimize_and_codegen_fat_lto(
        sess: &Session,
        cgcx: &CodegenContext,
        shared_emitter: &SharedEmitter,
        tm_factory: TargetMachineFactoryFn<Self>,
        exported_symbols_for_lto: &[String],
        each_linked_rlib_for_lto: &[PathBuf],
        modules: Vec<FatLtoInput<Self>>,
    ) -> CompiledModule {
        todo!()
    }

    fn run_thin_lto(
        cgcx: &CodegenContext,
        prof: &SelfProfilerRef,
        dcx: rustc_errors::DiagCtxtHandle<'_>,
        exported_symbols_for_lto: &[String],
        each_linked_rlib_for_lto: &[PathBuf],
        modules: Vec<ThinLtoInput<Self>>,
    ) -> (Vec<ThinModule<Self>>, Vec<WorkProduct>) {
        unreachable!()
    }

    fn optimize(
        cgcx: &CodegenContext,
        prof: &SelfProfilerRef,
        shared_emitter: &SharedEmitter,
        module: &mut ModuleCodegen<Self::Module>,
        config: &ModuleConfig,
    ) {
        // for setting the optimization level, probably not needed
    }

    fn optimize_and_codegen_thin(
        cgcx: &CodegenContext,
        prof: &SelfProfilerRef,
        shared_emitter: &SharedEmitter,
        tm_factory: TargetMachineFactoryFn<Self>,
        thin: ThinModule<Self>,
    ) -> CompiledModule {
        unreachable!()
    }

    fn codegen(
        cgcx: &CodegenContext,
        prof: &SelfProfilerRef,
        shared_emitter: &SharedEmitter,
        module: ModuleCodegen<Self::Module>,
        config: &ModuleConfig,
    ) -> CompiledModule {
        let dcx = DiagCtxt::new(Box::new(shared_emitter.clone()));
        let dcx = dcx.handle();
        back::write::codegen(cgcx, prof, dcx, module, config)
    }

    fn serialize_module(module: Self::Module, is_thin: bool) -> Self::ModuleBuffer {
        unimplemented!()
    }
}

impl ModuleBufferMethods for ModuleBuffer {
    // from gcc
    fn data(&self) -> &[u8] {
        &[]
    }
}

impl CodegenBackend for TpdeCodegenBackend {
    fn name(&self) -> &'static str {
        "TPDE"
    }

    fn init(&self, _sess: &Session) {
        // TODO init tpde
    }

    fn codegen_crate<'tcx>(&self, tcx: TyCtxt<'tcx>) -> Box<dyn Any> {
        Box::new(rustc_codegen_ssa::base::codegen_crate(
            TpdeCodegenBackend::new(),
            tcx
        ))
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        sess: &Session,
        incr_comp_session: Option<&IncrCompSession>,
        outputs: &OutputFilenames,
        crate_info: &CrateInfo,
    ) -> (CompiledModules, WorkProductMap) {
        ongoing_codegen
            .downcast::<rustc_codegen_ssa::back::write::OngoingCodegen<TpdeCodegenBackend>>()
            .expect("Expected TpdeCodegenBackend's OngoingCodegen, found Box<Any>")
            .join(sess, incr_comp_session, crate_info)
    }

    fn print(&self, _req: &PrintRequest, _out: &mut String, _sess: &Session) {
        todo!()
    }

    fn target_config(&self, sess: &Session) -> TargetConfig {
        let internal_target_features = internal_target_features(
            sess,
            |_| SmallVec::<[_; 0]>::new(),
            |feature| ["x87", "sse2"].contains(&feature),
        );
        TargetConfig {
            internal_target_features,
            has_reliable_f16: false,
            has_reliable_f16_math: false,
            has_reliable_f128: false,
            has_reliable_f128_math: false,
        }
    }

    fn print_passes(&self) {
        todo!()
    }

    fn print_version(&self) {
        println!("rustc_codegen_tpde version 0.0.1");
    }

    fn replaced_intrinsics(&self) -> Vec<Symbol> {
        // let's first use the fallback for everything
        vec![]
    }

    fn fallback_intrinsics(&self) -> Vec<Symbol> {
        // place all not used intrinsics here that we do not replace
        vec![]
    }

    fn thin_lto_supported(&self) -> bool {
        false
    }

    fn has_zstd(&self) -> bool {
        false
    }

    fn has_mnemonic(&self, _sess: &Session, _mnemonic: &str) -> bool {
        false
    }

    fn provide(&self, providers: &mut Providers) {
        // Can parse features provided by the user
        // Maybe use this later
    }

    fn target_cpu(&self, sess: &Session) -> String {
        sess.opts.cg.target_cpu.as_deref().unwrap_or_else(|| &sess.target.cpu).to_string()
    }

    fn print_pass_timings(&self) {
        todo!()
    }

    fn print_statistics(&self) {
        todo!()
    }

    fn print_statistics_json(&self) -> String {
        todo!()
    }
}
