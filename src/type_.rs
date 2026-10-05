use crate::context::{CodegenCx, GenericCx, SCx};
use crate::shared::ir::{ArgExtension, ArgInfo, ArgKind, FullType, FunctionSignature, Type, vector_info, vector_type};
use core::borrow::Borrow;
use rustc_abi::{AddressSpace, BackendRepr, Primitive, Reg, RegKind, Scalar};
use rustc_codegen_ssa::common::TypeKind;
use rustc_codegen_ssa::traits::{
    BaseTypeCodegenMethods, DerivedTypeCodegenMethods, LayoutTypeCodegenMethods,
    TypeMembershipCodegenMethods,
};
use rustc_middle::bug;
use rustc_middle::ty::Ty;
use rustc_middle::ty::layout::TyAndLayout;
use rustc_target::callconv::{CastTarget, FnAbi, PassMode};

impl<'tpde, 'tcx> CodegenCx<'tpde, 'tcx> {
    pub fn tpde_direct_type(&self, ty: TyAndLayout<'tcx>) -> FullType {
        match ty.backend_repr {
            BackendRepr::Scalar(scalar) => self.tpde_scalar_type(scalar),
            BackendRepr::ScalarPair { a, b, b_offset } => {
                let FullType::Single(a) = self.tpde_scalar_type(a) else {
                    unreachable!()
                };
                let FullType::Single(b) = self.tpde_scalar_type(b) else {
                    unreachable!()
                };

                FullType::Pair(a, b, b_offset.bytes_usize() as u32)
            }
            BackendRepr::Memory { sized } => FullType::Memory { sized, size: ty.size.bytes() as u32 },
            BackendRepr::SimdVector { element, count } => {
                let FullType::Single(elem) = self.tpde_scalar_type(element) else {
                    unreachable!()
                };
                self.type_vector(FullType::Single(elem), count.as_u64())
            }
            BackendRepr::SimdScalableVector { .. } => todo!(),
        }
    }

    fn tpde_scalar_type(&self, scalar: Scalar) -> FullType {
        if scalar.is_bool() {
            return FullType::Single(Type::Bool);
        }
        match scalar.primitive() {
            Primitive::Int(i, _) => self.type_from_integer(i),
            Primitive::Float(f) => self.type_from_float(f),
            Primitive::Pointer(address_space) => self.type_ptr_ext(address_space),
        }
    }
}

impl<'tpde, CX: Borrow<SCx<'tpde>>> GenericCx<'tpde, CX> {
    pub fn type_void(&self) -> FullType {
        FullType::Single(Type::Void)
    }

    pub fn type_vector(&self, elem: FullType, count: u64) -> FullType {
        let FullType::Single(elem) = elem else {
            bug!("vector element must be a scalar type: {:?}", elem)
        };
        match vector_type(elem, count) {
            Some(ty) => FullType::Single(ty),
            None => todo!("unsupported vector type <{} x {:?}>", count, elem),
        }
    }

    pub fn function_signature(&self, args: &[FullType], ret: Option<FullType>) -> FunctionSignature {
        let mut slots: Vec<Type> = vec![];
        let mut arg_infos: Vec<ArgInfo> = vec![];

        let add_ty = |ty: Type| {
            slots.push(ty);
            arg_infos.push(ArgInfo::default());
        };

        for ty in args {
            match ty {
                FullType::Single(ty) => {
                    slots.push(*ty);
                    arg_infos.push(ArgInfo::default());
                }
                FullType::Pair(a, b, _) => {
                    slots.push(*a);
                    arg_infos.push(ArgInfo::default());
                    slots.push(*b);
                    arg_infos.push(ArgInfo::default());
                }
                FullType::Memory { .. } => { todo!() }
            }
        }

        FunctionSignature { slots, arg_infos, ret }
    }
}

impl<'tpde, CX: Borrow<SCx<'tpde>>> BaseTypeCodegenMethods for GenericCx<'tpde, CX> {
    fn type_i8(&self) -> Self::Type {
        FullType::Single(Type::i8)
    }

    fn type_i16(&self) -> Self::Type {
        FullType::Single(Type::i16)
    }

    fn type_i32(&self) -> Self::Type {
        FullType::Single(Type::i32)
    }

    fn type_i64(&self) -> Self::Type {
        FullType::Single(Type::i64)
    }

    fn type_i128(&self) -> Self::Type {
        FullType::Single(Type::i128)
    }

    fn type_isize(&self) -> Self::Type {
        // TODO so far only support x64 backends
        self.type_i64()
    }

    fn type_f16(&self) -> Self::Type {
        todo!()
    }

    fn type_f32(&self) -> Self::Type {
        FullType::Single(Type::f32)
    }

    fn type_f64(&self) -> Self::Type {
        FullType::Single(Type::f64)
    }

    fn type_f128(&self) -> Self::Type {
        todo!()
    }

    fn type_array(&self, ty: Self::Type, len: u64) -> Self::Type {
        todo!()
    }

    fn type_func(&self, args: &[Self::Type], ret: Self::Type) -> Self::FunctionSignature {
        let sig = self.function_signature(args, Some(ret));
        let signs = &mut self.0.borrow().function_signatures.borrow_mut();
        signs.push(sig);
        signs.len() - 1
    }

    fn type_kind(&self, ty: Self::Type) -> TypeKind {
        match ty {
            FullType::Single(ty) => match ty {
                Type::Void => TypeKind::Void,
                Type::Bool | Type::i8 | Type::i16 | Type::i32 | Type::i64 | Type::i128 => {
                    TypeKind::Integer
                }
                Type::f32 | Type::f64 => TypeKind::Float,
                Type::ptr => TypeKind::Pointer,
                ty if vector_info(ty).is_some() => TypeKind::Vector,
                _ => todo!(),
            },
            FullType::Pair(ty1, ty2, _) => TypeKind::Struct,
            FullType::Memory { .. } => todo!(),
        }
    }

    fn type_ptr(&self) -> Self::Type {
        FullType::Single(Type::ptr)
    }

    fn type_ptr_ext(&self, address_space: AddressSpace) -> Self::Type {
        self.type_ptr()
    }

    fn element_type(&self, ty: Self::Type) -> Self::Type {
        match ty {
            FullType::Single(ty) if let Some((elem, _)) = vector_info(ty) => FullType::Single(elem),
            _ => bug!("element_type called on non-vector type {:?}", ty),
        }
    }

    fn vector_length(&self, ty: Self::Type) -> usize {
        match ty {
            FullType::Single(ty) if let Some((_, count)) = vector_info(ty) => count as usize,
            _ => bug!("vector_length called on non-vector type {:?}", ty),
        }
    }

    fn float_width(&self, ty: Self::Type) -> usize {
        match ty {
            FullType::Single(ty) => match ty {
                Type::f32 => 32,
                Type::f64 => 64,
                _ => todo!(),
            },
            _ => todo!(),
        }
    }

    fn int_width(&self, ty: Self::Type) -> u64 {
        match ty {
            FullType::Single(ty) => match ty {
                Type::Bool | Type::i8 => 8,
                Type::i16 => 16,
                Type::i32 => 32,
                Type::i64 | Type::ptr => 64,
                Type::i128 => 128,
                _ => todo!("Type {:?} not implemented", ty),
            },
            _ => todo!(),
        }
    }

    fn val_ty(&self, v: Self::Value) -> Self::Type {
        self.0.borrow().module.borrow().type_of_slot(v)
    }
}

impl<'tcx> TypeMembershipCodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {}

fn map_arg_extension(ext: rustc_target::callconv::ArgExtension) -> ArgExtension {
    match ext {
        rustc_target::callconv::ArgExtension::None => ArgExtension::None,
        rustc_target::callconv::ArgExtension::Sext => ArgExtension::sExt,
        rustc_target::callconv::ArgExtension::Zext => ArgExtension::zExt,
    }
}

impl<'tcx> CodegenCx<'_, 'tcx> {
    pub fn create_function_signature(&self, fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> FunctionSignature {
        let mut slots: Vec<Type> = vec![];
        let mut arg_infos: Vec<ArgInfo> = vec![];

        match fn_abi.ret.mode {
            PassMode::Indirect {
                attrs,
                meta_attrs,
                on_stack,
            } => {
                slots.push(Type::ptr);
                arg_infos.push(ArgInfo {
                    kind: ArgKind::sRet,
                    extension: ArgExtension::None,
                    size: attrs.pointee_size.bytes() as u32,
                    align: attrs.pointee_align.map_or_else(|| 1, |a| a.bytes() as u32),
                })
            }
            _ => (),
        }
        {
            // we can ignore variadic arguments
            if fn_abi.c_variadic {
                &fn_abi.args[..fn_abi.fixed_count as usize]
            } else {
                &fn_abi.args
            }.iter()
                .for_each(|arg| match &arg.mode {
                    PassMode::Ignore => { },
                    PassMode::Direct(attrs) => {
                        let FullType::Single(ty) = self.tpde_direct_type(arg.layout) else {
                            unreachable!()
                        };
                        slots.push(ty);
                        arg_infos.push(ArgInfo {
                            kind: ArgKind::Direct,
                            extension: map_arg_extension(attrs.arg_ext),
                            size: 0,
                            align: 0,
                        })
                    }
                    PassMode::Pair(attrs_a, attrs_b) => {
                        let FullType::Pair(a, b, _) = self.tpde_direct_type(arg.layout) else {
                            unreachable!()
                        };
                        slots.push(a);
                        arg_infos.push(ArgInfo {
                            kind: ArgKind::Direct,
                            extension: map_arg_extension(attrs_a.arg_ext),
                            size: 0,
                            align: 0,
                        });
                        slots.push(b);
                        arg_infos.push(ArgInfo {
                            kind: ArgKind::Direct,
                            extension: map_arg_extension(attrs_b.arg_ext),
                            size: 0,
                            align: 0,
                        });
                    }
                    PassMode::Indirect {
                        attrs,
                        meta_attrs,
                        on_stack: true,
                    } => {
                        // They cannot be true at once
                        assert!(meta_attrs.is_none());
                        slots.push(Type::ptr);
                        arg_infos.push(ArgInfo {
                            kind: ArgKind::ByVal,
                            extension: ArgExtension::None,
                            size: attrs.pointee_size.bytes() as u32,
                            align: attrs.pointee_align.map_or_else(|| 1, |a| a.bytes() as u32),
                        });
                    }
                    PassMode::Indirect {
                        attrs,
                        meta_attrs,
                        on_stack,
                    } => {
                        slots.push(Type::ptr);
                        arg_infos.push(ArgInfo::default());
                    },
                    PassMode::Cast { cast, pad_i32: _ } => {
                        match self.cast_backend_type(cast) {
                            FullType::Single(ty) => {
                                slots.push(ty);
                                arg_infos.push(ArgInfo {
                                    kind: ArgKind::Direct,
                                    extension: ArgExtension::None,
                                    size: 0,
                                    align: 0,
                                })
                            }
                            FullType::Pair(a, b, offset_b) => {
                                slots.push(a);
                                arg_infos.push(ArgInfo::default());
                                slots.push(b);
                                arg_infos.push(ArgInfo::default());
                            }
                            _ => unreachable!()
                        }
                    },
                });
        }

        let ret = if fn_abi.ret.is_ignore() {
            None
        } else {
            Some(self.tpde_direct_type(fn_abi.ret.layout))
        };

        FunctionSignature { slots, arg_infos, ret }
    }
}

impl CodegenCx<'_, '_> {
    fn reg_to_ir_type(&self, reg: &Reg) -> Type {
        match reg.kind {
            RegKind::Integer => match reg.size.bytes() {
                1 => Type::i8,
                2 => Type::i16,
                3 | 4 => Type::i32,
                5 | 6 | 7 | 8 => Type::i64,
                9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 => Type::i128,
                _ => panic!("Unsupported integer register size"),
            },
            RegKind::Float => match reg.size.bytes() {
                4 => Type::f32,
                8 => Type::f64,
                _ => panic!("Unsupported float register size"),
            },
            RegKind::Vector { hint_vector_elem: _ } => match reg.size.bytes() {
                8 => Type::v8i8,
                16 => Type::v16i8,
                _ => panic!("Unsupported vector register size"),
            },
        }
    }
}

impl<'tcx> LayoutTypeCodegenMethods<'tcx> for CodegenCx<'_, 'tcx> {
    fn backend_type(&self, layout: TyAndLayout<'tcx>) -> Self::Type {
        self.tpde_direct_type(layout)
    }

    fn cast_backend_type(&self, ty: &CastTarget) -> Self::Type {
        let mut registers = Vec::new();
        let mut offsets = Vec::new();
        let mut current_offset = 0;

        for reg in ty.prefix.iter() {
            registers.push(self.reg_to_ir_type(reg));
            offsets.push(current_offset);
            current_offset += reg.size.bytes();
        }

        let unit_type = self.reg_to_ir_type(&ty.rest.unit);
        let repeats = ty.rest.total.bytes() / ty.rest.unit.size.bytes();

        for _ in 0..repeats {
            registers.push(unit_type.clone());
            offsets.push(current_offset);
            current_offset += ty.rest.unit.size.bytes();
        }

        if registers.len() == 1 {
            FullType::Single(registers[0].clone())
        } else if registers.len() == 2 {
            FullType::Pair(registers[0].clone(), registers[1].clone(), offsets[1] as u32)
        } else {
            unimplemented!()
        }
    }

    fn fn_decl_backend_type(&self, fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> Self::FunctionSignature {
        let signs = &mut self.function_signatures.borrow_mut();
        signs.push(self.create_function_signature(fn_abi));
        signs.len() - 1
    }

    fn fn_ptr_backend_type(&self, fn_abi: &FnAbi<'tcx, Ty<'tcx>>) -> Self::Type {
        FullType::Single(Type::ptr)
    }

    fn reg_backend_type(&self, ty: &Reg) -> Self::Type {
        todo!()
    }

    fn immediate_backend_type(&self, layout: TyAndLayout<'tcx>) -> Self::Type {
        // TODO adapt for i1
        self.tpde_direct_type(layout)
    }

    fn scalar_pair_element_backend_type(
        &self,
        layout: TyAndLayout<'tcx>,
        index: usize,
        immediate: bool,
    ) -> Self::Type {
        let BackendRepr::ScalarPair { a, b, b_offset: _ } = layout.backend_repr else {
            bug!("Has to be a pair")
        };
        let scalar = [a, b][index];

        self.tpde_scalar_type(scalar)
    }
}
