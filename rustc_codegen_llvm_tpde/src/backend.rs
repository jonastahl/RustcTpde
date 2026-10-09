use rustc_codegen_llvm::{LlvmCodegenBackend, ModuleLlvm};
use rustc_codegen_ssa::back::lto::ThinModule;
use rustc_codegen_ssa::back::write::{EmitObj, CodegenContext, FatLtoInput, ModuleConfig, SharedEmitter, TargetMachineFactoryFn, ThinLtoInput};
use rustc_codegen_ssa::traits::{CodegenBackend, ExtraBackendMethods, ModuleBufferMethods, WriteBackendMethods};
use rustc_codegen_ssa::{CompiledModule, CompiledModules, CrateInfo, ModuleCodegen};
use rustc_data_structures::profiling::SelfProfilerRef;
use rustc_middle::dep_graph::WorkProduct;
use rustc_middle::dep_graph::WorkProductMap;
use rustc_middle::ty::TyCtxt;
use rustc_session::IncrCompSession;
use rustc_session::config::OutputType;
use crate::tpde;
use rustc_session::Session;
use rustc_session::config::OptLevel;
use std::any::Any;
use std::path::PathBuf;
use std::sync::Arc;

fn llvm_backend() -> LlvmCodegenBackend {
    unsafe { std::mem::transmute::<(), LlvmCodegenBackend>(()) }
}

#[derive(Clone)]
pub struct LLvmTpdeCodegenBackend();

impl LLvmTpdeCodegenBackend {
    pub fn new() -> LLvmTpdeCodegenBackend {
        LLvmTpdeCodegenBackend()
    }
}

impl CodegenBackend for LLvmTpdeCodegenBackend {
    fn name(&self) -> &'static str {
        "LLVM TPDE"
    }

    fn target_cpu(&self, sess: &rustc_session::Session) -> String {
        sess.opts.cg.target_cpu.as_deref().unwrap_or_else(|| &sess.target.cpu).to_string()
    }

    fn init(&self, sess: &Session) {
        llvm_backend().init(sess);
    }

    fn target_config(&self, sess: &Session) -> rustc_codegen_ssa::TargetConfig {
        llvm_backend().target_config(sess)
    }

    fn codegen_crate<'tcx>(&self, tcx: rustc_middle::ty::TyCtxt<'tcx>) -> Box<dyn Any> {
        Box::new(rustc_codegen_ssa::base::codegen_crate(
            LLvmTpdeCodegenBackend::new(),
            tcx
        ))
    }

    fn join_codegen(&self, ongoing_codegen: Box<dyn Any>, sess: &rustc_session::Session, incr_comp_session: Option<&IncrCompSession>, outputs: &rustc_session::config::OutputFilenames, crate_info: &CrateInfo) -> (CompiledModules, WorkProductMap) {
        ongoing_codegen
            .downcast::<rustc_codegen_ssa::back::write::OngoingCodegen<LLvmTpdeCodegenBackend>>()
            .expect("Expected LlvmTpdeCodegenBackend's OngoingCodegen, found Box<Any>")
            .join(sess, incr_comp_session, crate_info)
    }
}

impl ExtraBackendMethods for LLvmTpdeCodegenBackend {
    type Module = ModuleLlvm;

    fn codegen_allocator<'tcx>(&self, tcx: TyCtxt<'tcx>, module_name: &str, methods: &[rustc_ast::expand::allocator::AllocatorMethod]) -> Self::Module {
        llvm_backend().codegen_allocator(tcx, module_name, methods)
    }

    fn compile_codegen_unit(&self, tcx: TyCtxt<'_>, cgu_name: rustc_span::symbol::Symbol) -> (ModuleCodegen<Self::Module>, u64) {
        llvm_backend().compile_codegen_unit(tcx, cgu_name)
    }
}

impl WriteBackendMethods for LLvmTpdeCodegenBackend {
    type Module = ModuleLlvm;
    type TargetMachine = ();
    type ModuleBuffer = <LlvmCodegenBackend as WriteBackendMethods>::ModuleBuffer;
    type ThinData = ();

    fn target_machine_factory(&self, sess: &Session, opt_level: OptLevel, target_features: &[String]) -> TargetMachineFactoryFn<Self> {
        // By now we only support x64 and no optimization anyway
        Arc::new(|_, _| ())
    }

    fn optimize_and_codegen_fat_lto(sess: &Session, cgcx: &CodegenContext, shared_emitter: &SharedEmitter, tm_factory: TargetMachineFactoryFn<Self>, exported_symbols_for_lto: &[String], each_linked_rlib_for_lto: &[PathBuf], modules: Vec<FatLtoInput<Self>>) -> CompiledModule {
        unimplemented!()
    }

    fn run_thin_lto(cgcx: &CodegenContext, prof: &SelfProfilerRef, dcx: rustc_errors::DiagCtxtHandle<'_>, exported_symbols_for_lto: &[String], each_linked_rlib_for_lto: &[PathBuf], modules: Vec<ThinLtoInput<Self>>) -> (Vec<ThinModule<Self>>, Vec<WorkProduct>) {
        unimplemented!()
    }

    fn optimize(cgcx: &CodegenContext, prof: &SelfProfilerRef, shared_emitter: &SharedEmitter, module: &mut ModuleCodegen<Self::Module>, config: &ModuleConfig) {
        // We don't support optimizations by now
    }

    fn optimize_and_codegen_thin(cgcx: &CodegenContext, prof: &SelfProfilerRef, shared_emitter: &SharedEmitter, tm_factory: TargetMachineFactoryFn<Self>, thin: ThinModule<Self>) -> CompiledModule {
        unimplemented!()
    }

    fn codegen(cgcx: &CodegenContext, prof: &SelfProfilerRef, shared_emitter: &SharedEmitter, module: ModuleCodegen<Self::Module>, config: &ModuleConfig) -> CompiledModule {
        let _timer = prof.generic_activity_with_arg("TPDE_LLVM_module_codegen", &*module.name);
        let bc_out = cgcx.output_filenames.temp_path_for_cgu(OutputType::Bitcode, &module.name);
        let obj_out = cgcx.output_filenames.temp_path_for_cgu(OutputType::Object, &module.name);

        // The LLVM module is private to rustc_codegen_llvm and TPDE links a different LLVM anyway,
        // so the module is handed over as bitcode. This consumes the LLVM module and context.
        let ModuleCodegen { name, module_llvm, kind, thin_lto_buffer } = module;
        let bitcode = <LlvmCodegenBackend as WriteBackendMethods>::serialize_module(module_llvm, false);

        if config.emit_bc {
            std::fs::write(&bc_out, bitcode.data()).expect("write bitcode");
        }
        if config.emit_ir || config.emit_asm {
            todo!("--emit=llvm-ir / asm is not supported by the TPDE LLVM backend")
        }

        match config.emit_obj {
            EmitObj::ObjectCode(_) => {
                let _timer = prof.generic_activity_with_arg("TPDE_LLVM_module_codegen_emit_obj", &*name);
                let obj = tpde::compile_bitcode(bitcode.data())
                    .expect("TPDE failed to compile module, consider falling back to llvm");
                std::fs::write(&obj_out, obj).expect("write object file");
            }
            EmitObj::Bitcode => {
                std::fs::write(&obj_out, bitcode.data()).expect("write bitcode as object");
            }
            EmitObj::None => {}
        }

        ModuleCodegen { name, module_llvm: (), kind, thin_lto_buffer }.into_compiled_module(
            config.emit_obj != EmitObj::None,
            false,
            config.emit_bc,
            false,
            false,
            &cgcx.output_filenames,
        )
    }

    fn serialize_module(module: Self::Module, is_thin: bool) -> Self::ModuleBuffer {
        <LlvmCodegenBackend as WriteBackendMethods>::serialize_module(module, is_thin)
    }
}
