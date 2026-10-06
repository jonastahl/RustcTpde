use crate::context::CodegenCx;
use crate::shared::ir::{Binding, Function, Global};
use rustc_codegen_ssa::traits::PreDefineCodegenMethods;
use rustc_hir::attrs::Linkage;
use rustc_middle::mir::Mutability;
use rustc_middle::mono::Visibility;
use rustc_middle::ty;
use rustc_middle::ty::layout::FnAbiOf;
use rustc_middle::ty::{Instance, Ty};
use rustc_span::def_id::DefId;
use rustc_target::callconv::FnAbi;

impl CodegenCx<'_, '_> {
    pub fn declare_static(
        &self,
        def_id: DefId,
        linkage: Linkage,
        visibility: Visibility,
        binding: Binding,
        symbol_name: &str,
    ) -> Global {
        let global = self.module.borrow_mut()
            .add_global(symbol_name, linkage, Mutability::Mut, binding);
        self.globals.borrow_mut().insert(def_id, global);
        global
    }

    pub fn get_global(&self, def_id: DefId) -> Global {
        let global = self.globals.borrow().get(&def_id).copied();
        match global {
            Some(g) => g,
            None => {
                let instance = Instance::mono(self.tcx, def_id);
                let name = self.tcx.symbol_name(instance).name;
                self.declare_static(
                    def_id,
                    Linkage::ExternalWeak,
                    Visibility::Default,
                    Binding::Declaration,
                    name,
                )
            }
        };

        *self.globals.borrow().get(&def_id).expect("Global was not declared before")
    }
}

impl<'tcx> PreDefineCodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn predefine_static(
        &mut self,
        def_id: DefId,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        self.declare_static(def_id, linkage, visibility, Binding::Definition, symbol_name);
    }

    fn predefine_fn(
        &mut self,
        instance: Instance<'tcx>,
        linkage: Linkage,
        visibility: Visibility,
        symbol_name: &str,
    ) {
        self.declare_fn(instance, symbol_name, linkage, visibility, Binding::Definition);
    }
}

impl<'tcx> CodegenCx<'_, 'tcx> {
    pub fn declare_fn(
        &self,
        instance: Instance<'tcx>,
        symbol_name: &str,
        linkage: Linkage,
        visibility: Visibility,
        binding: Binding,
    ) -> Function {
        let fn_abi: &FnAbi<'tcx, Ty<'tcx>> = self.fn_abi_of_instance(instance, ty::List::empty());

        if self.module.try_borrow_mut().is_err() {
            println!("Module is already borrowed")
        }
        let func = self.module.borrow_mut()
            .add_function(symbol_name, self.create_function_signature(fn_abi), linkage, binding);
        self.functions.borrow_mut().insert(instance, func);
        func
    }
}
