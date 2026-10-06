use crate::context::SimpleCx;
use crate::shared::ir::{Binding, FullType, InstructionKind, Slot};
use rustc_ast::expand::allocator::{AllocatorMethod, AllocatorTy, NO_ALLOC_SHIM_IS_UNSTABLE, SpecialAllocatorMethod, default_fn_name, global_fn_name};
use rustc_codegen_ssa::traits::BaseTypeCodegenMethods;
use rustc_hir::attrs::Linkage;
use rustc_middle::bug;
use rustc_middle::middle::codegen_fn_attrs::{CodegenFnAttrFlags, CodegenFnAttrs};
use rustc_middle::ty::TyCtxt;
use rustc_session::config::DebugInfo;
use rustc_symbol_mangling::mangle_internal_symbol;

pub fn codegen(
    tcx: TyCtxt<'_>,
    scx: SimpleCx<'_>,
    module_name: &str,
    methods: &[AllocatorMethod],
) {
    let usize = match tcx.sess.target.pointer_width {
        16 => scx.type_i16(),
        32 => scx.type_i32(),
        64 => scx.type_i64(),
        tws => bug!("Unsupported target word size for int: {}", tws),
    };
    let i8p = scx.type_ptr();

    for method in methods {
        let mut args = Vec::with_capacity(method.inputs.len());
        for input in method.inputs.iter() {
            match input.ty {
                AllocatorTy::Layout => {
                    args.push(usize); // size
                    args.push(usize); // align
                }
                AllocatorTy::Ptr => args.push(i8p),
                AllocatorTy::Usize => args.push(usize),

                AllocatorTy::Never | AllocatorTy::ResultPtr | AllocatorTy::Unit => {
                    panic!("invalid allocator arg")
                }
            }
        }

        let mut no_return = false;
        let output = match method.output {
            AllocatorTy::ResultPtr => Some(i8p),
            AllocatorTy::Unit => None,
            AllocatorTy::Never => {
                no_return = true;
                None
            }

            AllocatorTy::Layout | AllocatorTy::Usize | AllocatorTy::Ptr => {
                panic!("invalid allocator output")
            }
        };

        let from_name = mangle_internal_symbol(tcx, &global_fn_name(method.name));
        let to_name = mangle_internal_symbol(tcx, &default_fn_name(method.name));

        let alloc_attr_flag = match method.special {
            Some(SpecialAllocatorMethod::Alloc) => CodegenFnAttrFlags::ALLOCATOR,
            Some(SpecialAllocatorMethod::Dealloc) => CodegenFnAttrFlags::DEALLOCATOR,
            Some(SpecialAllocatorMethod::Realloc) => CodegenFnAttrFlags::REALLOCATOR,
            Some(SpecialAllocatorMethod::AllocZeroed) => CodegenFnAttrFlags::ALLOCATOR_ZEROED,
            None => CodegenFnAttrFlags::empty(),
        };

        let mut attrs = CodegenFnAttrs::new();
        attrs.flags |= alloc_attr_flag;
        create_wrapper_function(
            tcx,
            &scx,
            &from_name,
            Some(&to_name),
            &args,
            output,
            no_return,
            &attrs,
        );
    }

    // __rust_no_alloc_shim_is_unstable_v2
    create_wrapper_function(
        tcx,
        &scx,
        &mangle_internal_symbol(tcx, NO_ALLOC_SHIM_IS_UNSTABLE),
        None,
        &[],
        None,
        false,
        &CodegenFnAttrs::new(),
    );

    if tcx.sess.opts.debuginfo != DebugInfo::None {
        // Ignore debuginfo for now

        // let dbg_cx = debuginfo::CodegenUnitDebugContext::new(cx.llmod, tcx.sess);
        // debuginfo::metadata::build_compile_unit_di_node(tcx, module_name, &dbg_cx);
        // dbg_cx.finalize();
    }
}

fn create_wrapper_function(
    tcx: TyCtxt<'_>,
    scx: &SimpleCx<'_>,
    from_name: &str,
    to_name: Option<&str>,
    args: &[FullType],
    output: Option<FullType>,
    no_return: bool,
    attrs: &CodegenFnAttrs,
) {
    let module = &mut scx.module.borrow_mut();

    let sign = scx.function_signature(args, output);
    let caller = module.add_function(
        from_name,
        sign.clone(),
        Linkage::Common,
        Binding::Definition,
    );

    let bb = module.add_basic_block(&caller, "entry");

    let ret = if let Some(to_name) = to_name {
        let callee = module.add_function(
            to_name,
            sign.clone(),
            Linkage::ExternalWeak,
            Binding::Declaration,
        );

        let args = args.iter().enumerate()
            .map(|(i, _)| module.get_slot(caller, i as u32)).collect::<Vec<_>>();
        let output = module.add_call(bb, Slot::new_func(callee), &sign, &args);

        match output {
            Some(slot) => vec![slot],
            None => vec![],
        }
    } else {
        assert!(output.is_none());
        vec![]
    };

    module.add_instruction(bb, InstructionKind::Ret, ret);
}