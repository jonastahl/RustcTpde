use crate::builder::Builder;
use crate::context::CodegenCx;
use crate::shared::ir::Module;
use rustc_codegen_ssa::ModuleCodegen;
use rustc_codegen_ssa::base::maybe_create_entry_wrapper;
use rustc_codegen_ssa::mono_item::MonoItemExt;
use rustc_middle::dep_graph;
use rustc_middle::ty::TyCtxt;
use rustc_span::Symbol;
use std::cell::RefCell;
use std::time::Instant;

pub fn compile_codegen_unit(tcx: TyCtxt<'_>, cgu_name: Symbol) -> (ModuleCodegen<Module>, u64) {
    let start_time = Instant::now();

    let dep_node = tcx.codegen_unit(cgu_name).codegen_dep_node(tcx);
    let (ir, _) = tcx.dep_graph.with_task(
        dep_node,
        tcx,
        || module_codegen(tcx, cgu_name),
        Some(dep_graph::hash_result),
    );
    let time_to_codegen = start_time.elapsed();

    // We assume that the cost to run TPDE on a CGU is proportional to
    // the time we needed for codegenning it.
    let cost = time_to_codegen.as_nanos() as u64;

    fn module_codegen(tcx: TyCtxt<'_>, cgu_name: Symbol) -> ModuleCodegen<Module> {
        let cgu = tcx.codegen_unit(cgu_name);
        let _prof_timer =
            tcx.prof.generic_activity_with_arg_recorder("codegen_module", |recorder| {
                recorder.record_arg(cgu_name.to_string());
                recorder.record_arg(cgu.size_estimate().to_string());
            });

        let ir = RefCell::new(Module::new());
        {
            let mut cx = CodegenCx::new(tcx, cgu, &ir);

            let mono_items = cgu.items_in_deterministic_order(tcx);
            for &(mono_item, data) in &mono_items {
                mono_item.predefine::<Builder<'_, '_, '_>>(
                    &mut cx,
                    cgu_name.as_str(),
                    data.linkage,
                    data.visibility,
                );
            }

            for &(mono_item, item_data) in &mono_items {
                mono_item.define::<Builder<'_, '_, '_>>(&mut cx, cgu_name.as_str(), item_data);
            }

            if let Some(entry) =
                maybe_create_entry_wrapper::<Builder<'_, '_, '_>>(&cx, cx.codegen_unit)
            {}


            // LLVM includes sanitization here

            // ignore llvm create Objective-C module info

            // we don't need pointer authentication

            // we don't have coverage

            // statics?

            // also ignore debug infoo
        }

        ModuleCodegen::new_regular(cgu_name.to_string(), ir.into_inner())
    }

    (ir, cost)
}
