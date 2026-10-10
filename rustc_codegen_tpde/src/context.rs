use crate::builder::Builder;
use crate::shared::ir::{ArgInfo, Binding, FullType, Function, FunctionSignature, Global, Module, Slot, Type};
use core::borrow::Borrow;
use rustc_abi::TargetDataLayout;
use rustc_codegen_ssa::traits::MiscCodegenMethods;
use rustc_data_structures::base_n::{ALPHANUMERIC_ONLY, ToBaseN};
use rustc_data_structures::fx::FxHashMap;
use rustc_hir::attrs::Linkage;
use rustc_middle::mono::{CodegenUnit, Visibility};
use rustc_middle::ty;
use rustc_middle::ty::layout::HasTyCtxt;
use rustc_middle::ty::{ExistentialTraitRef, Instance, Ty, TyCtxt};
use rustc_session::{PointerAuthSchema, Session};
use rustc_span::def_id::DefId;
use rustc_span::{Symbol, sym};
use std::cell::{Cell, OnceCell, RefCell};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct GenericCx<'tpde, T: Borrow<SCx<'tpde>>>(pub T, PhantomData<SCx<'tpde>>);
pub struct SCx<'tpde> {
    pub module: &'tpde RefCell<Module>,
    pub function_signatures: RefCell<Vec<FunctionSignature>>,
}

pub type CodegenCx<'tpde, 'tcx> = GenericCx<'tpde, FullCx<'tpde, 'tcx>>;
pub type SimpleCx<'tpde> = GenericCx<'tpde, SCx<'tpde>>;

pub struct FullCx<'tpde, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub scx: SCx<'tpde>,
    pub codegen_unit: &'tcx CodegenUnit<'tcx>,

    pub functions: RefCell<FxHashMap<Instance<'tcx>, Function>>,
    pub libfuncs: RefCell<FxHashMap<LibFunc, Function>>,

    pub globals: RefCell<FxHashMap<DefId, Global>>,
    pub vtables: RefCell<FxHashMap<(Ty<'tcx>, Option<ty::ExistentialTraitRef<'tcx>>), Slot>>,
    pub fallback_personality: OnceCell<Function>,

    pub data_layout: TargetDataLayout,

    pub local_gen_sym_counter: Cell<usize>,
}

impl<'tpde, T: Borrow<SCx<'tpde>>> Deref for GenericCx<'tpde, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'tpde, T: Borrow<SCx<'tpde>>> DerefMut for GenericCx<'tpde, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'tpde> Borrow<SCx<'tpde>> for FullCx<'tpde, '_> {
    fn borrow(&self) -> &SCx<'tpde> {
        &self.scx
    }
}

impl<'tpde, 'tcx> Deref for FullCx<'tpde, 'tcx> {
    type Target = SCx<'tpde>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.scx
    }
}

impl<'tpde, 'tcx> SCx<'tpde> {
    pub fn new(module: &'tpde RefCell<Module>) -> Self {
        Self {
            module,
            function_signatures: RefCell::new(vec![]),
        }
    }
}

impl<'tpde> SimpleCx<'tpde> {
    pub fn new(
        module: &'tpde RefCell<Module>,
    ) -> Self {
        GenericCx(SCx::new(module), PhantomData)
    }
}

impl<'tpde, 'tcx> CodegenCx<'tpde, 'tcx> {
    pub fn new(
        tcx: TyCtxt<'tcx>,
        cgu: &'tcx CodegenUnit<'tcx>,
        module: &'tpde RefCell<Module>,
    ) -> Self {
        let sess = tcx.sess;

        let data_layout = sess.target.parse_data_layout().unwrap_or_else(|err| {
            sess.dcx().emit_fatal(err);
        });

        GenericCx(
            FullCx {
                tcx,
                scx: SCx::new(module),
                codegen_unit: cgu,
                functions: RefCell::new(FxHashMap::default()),
                libfuncs: RefCell::new(FxHashMap::default()),
                globals: RefCell::new(FxHashMap::default()),
                data_layout,
                local_gen_sym_counter: Cell::new(0),
                vtables: RefCell::new(FxHashMap::default()),
                fallback_personality: OnceCell::new(),
            },
            PhantomData,
        )
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum LibFunc {
    MemCpy,
    MemMove,
    MemSet,
    MemCmp,
    Log2F32,
}

impl<'tcx> CodegenCx<'_, 'tcx> {
    pub fn get_lib_fn(&self, libfunc: LibFunc) -> Slot {
        if let Some(&i) = self.libfuncs.borrow().get(&libfunc) {
            return Slot::new_func(i);
        };

        let (name, arg_infos, ret) = match libfunc {
            LibFunc::MemCpy => ("memcpy", vec![ArgInfo::default(); 3], None),
            LibFunc::MemMove => ("memmove", vec![ArgInfo::default(); 3], None),
            LibFunc::MemSet => ("memset", vec![ArgInfo::default(); 3], None),
            LibFunc::MemCmp => ("memcmp", vec![ArgInfo::default(); 3], Some(FullType::Single(Type::i32))),
            LibFunc::Log2F32 => ("log2f", vec![ArgInfo::default(); 1], Some(FullType::Single(Type::f32))),
        };

        let func = self.module.borrow_mut()
            .add_function(
                name,
                FunctionSignature {
                    slots: vec![],
                    arg_infos,
                    ret,
                },
                Linkage::External,
                Binding::Declaration
            );
        self.libfuncs.borrow_mut().insert(libfunc, func);
        Slot::new_func(func)
    }
}

impl<'tcx> MiscCodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn vtables(
        &self,
    ) -> &RefCell<FxHashMap<(Ty<'tcx>, Option<ExistentialTraitRef<'tcx>>), Self::Value>> {
        &self.vtables
    }

    fn get_fn(&self, instance: Instance<'tcx>) -> Self::Function {
        if let Some(&i) = self.functions.borrow().get(&instance) {
            return i;
        };

        let name = self.tcx.symbol_name(instance).name;
        self.declare_fn(
            instance,
            name,
            Linkage::External,
            Visibility::Hidden, // TODO find exact visibility
            Binding::Declaration,
        )
    }

    fn get_fn_addr(
        &self,
        instance: Instance<'tcx>,
        pointer_auth_schema: Option<&PointerAuthSchema>,
    ) -> Self::Value {
        Slot::new_func(self.get_fn(instance))
    }

    fn eh_personality(&self) -> Self::Function {
        let def_id = match self.tcx.lang_items().eh_personality() {
            Some(id) => id,
            None => {
                // If the function wasnt declared we just insert
                // a placeholder for it (same as LLVM)

                let mut module = self.module.borrow_mut();
                if let Some(&f) = self.fallback_personality.get() {
                    return f;
                }
                let f = module.add_function(
                    "rust_eh_personality",
                    FunctionSignature { slots: vec![], arg_infos: vec![], ret: None },
                    Linkage::External,
                    Binding::Declaration,
                );
                let _ = self.fallback_personality.set(f);
                return f;
            }
        };

        let instance = Instance::mono(self.tcx, def_id);
        let name = self.tcx.symbol_name(instance).name;

        self.declare_fn(
            instance,
            name,
            Linkage::External,
            Visibility::Hidden,
            Binding::Declaration,
        )
    }

    fn sess(&self) -> &Session {
        self.tcx.sess
    }

    fn set_frame_pointer_type(&self, llfn: Self::Function) {
        // We just don't use it
    }

    fn apply_target_cpu_attr(&self, llfn: Self::Function) {
        // We are not that specialized
    }

    fn declare_c_main(&self, fn_type: Self::FunctionSignature) -> Option<Self::Function> {
        let entry_name = self.sess().target.entry_name.as_ref();

        let sign = self.function_signatures.borrow()[fn_type].clone();
        let func = self.module.borrow_mut().add_function(
            entry_name,
            sign,
            Linkage::External,
            Binding::Definition,
        );
        Some(func)
    }

    fn intrinsic_call_expects_place_always(&self, name: Symbol) -> bool {
        matches!(name, sym::black_box)
    }
}

impl<'tcx> HasTyCtxt<'tcx> for Builder<'_, '_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}

impl<'tcx> HasTyCtxt<'tcx> for CodegenCx<'_, 'tcx> {
    fn tcx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
}

impl CodegenCx<'_, '_> {
    /// Generates a new symbol name with the given prefix. This symbol name must
    /// only be used for definitions with `internal` or `private` linkage.
    pub(crate) fn generate_local_symbol_name(&self, prefix: &str) -> String {
        let idx = self.local_gen_sym_counter.get();
        self.local_gen_sym_counter.set(idx + 1);
        // Include a '.' character, so there can be no accidental conflicts with
        // user defined names
        let mut name = String::with_capacity(prefix.len() + 6);
        name.push_str(prefix);
        name.push('.');
        name.push_str(&(idx as u64).to_base(ALPHANUMERIC_ONLY));
        name
    }
}
