use crate::builder::Builder;
use crate::shared::ir::{FullType, InstructionKind, LandingPadKind, Slot, Type};
use rustc_abi::{Align, Size};
use rustc_codegen_ssa::RetagInfo;
use rustc_codegen_ssa::diagnostics::InvalidMonomorphization;
use rustc_codegen_ssa::mir::IntrinsicResult;
use rustc_codegen_ssa::mir::operand::{OperandRef, OperandValue};
use rustc_codegen_ssa::MemFlags;
use rustc_codegen_ssa::mir::place::{PlaceRef, PlaceValue};
use rustc_codegen_ssa::traits::{
    BaseTypeCodegenMethods, BuilderMethods, ConstCodegenMethods, IntrinsicCallBuilderMethods,
    MiscCodegenMethods,
    LayoutTypeCodegenMethods,
};
use rustc_middle::{bug, span_bug};
use rustc_middle::ty::{self, Instance};
use rustc_middle::ty::layout::{LayoutOf, TyAndLayout};
use rustc_span::{Span, sym};
use crate::context::LibFunc;

impl<'tcx> IntrinsicCallBuilderMethods<'tcx> for Builder<'_, '_, 'tcx> {
    fn codegen_intrinsic_call(
        &mut self,
        instance: Instance<'tcx>,
        args: &[OperandRef<'tcx, Self::Value>],
        result_layout: TyAndLayout<'tcx>,
        result_place: Option<PlaceValue<Self::Value>>,
        span: Span,
    ) -> IntrinsicResult<'tcx, Self::Value> {
        let name = self.tcx.item_name(instance.def_id());

        macro_rules! simd_binop {
            ($self:ident, $args:ident: $($($p:ident),+ => $call:ident),*) => {{
                let (_, elem_ty) = $args[0].layout.ty.simd_size_and_type($self.tcx);
                let arg1 = $args[0].immediate();
                let arg2 = $args[1].immediate();
                let result = match elem_ty.kind() {
                    $(
                        $($crate::rustc_middle::ty::$p(_))|+ => $self.$call(arg1, arg2),
                    )*
                    _ => panic!("unsupported SIMD element type"),
                };
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }};
            ($self:ident, $args:ident -> $ret:ident: $($($p:ident),+ => $instr:ident),*) => {{
                let (_, elem_ty) = $args[0].layout.ty.simd_size_and_type($self.tcx);
                let instr = match elem_ty.kind() {
                    $(
                        $($crate::rustc_middle::ty::$p(_))|+ => InstructionKind::$instr,
                    )*
                    _ => panic!("unsupported SIMD element type"),
                };
                let FullType::Single(ret_ty) = $self.cx.backend_type($ret) else { bug!() };
                let result = $self.module.borrow_mut().add_instruction_ret(
                    $self.basic_block,
                    instr,
                    &[$args[0].immediate(), $args[1].immediate()],
                    ret_ty
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }};
        }

        macro_rules! simd_unop {
            ($self:ident, $args:ident -> $ret:ident: $($($p:ident),+ => $instr:ident),*) => {{
                let (_, elem_ty) = $args[0].layout.ty.simd_size_and_type($self.tcx);
                let instr = match elem_ty.kind() {
                    $(
                        $($crate::rustc_middle::ty::$p(_))|+ => InstructionKind::$instr,
                    )*
                    _ => panic!("unsupported SIMD element type"),
                };
                let FullType::Single(ret_ty) = $self.cx.backend_type($ret) else { bug!() };
                let result = $self.module.borrow_mut().add_instruction_ret(
                    $self.basic_block,
                    instr,
                    &[$args[0].immediate()],
                    ret_ty
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }};
        }

        match name {
            sym::black_box => {
                let input_operand = args[0];

                IntrinsicResult::Operand(input_operand.val)
            }
            sym::ctpop => {
                let OperandValue::Immediate(input_val) = args[0].val else {
                    todo!()
                };
                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::ctpop,
                    &[input_val]
                );

                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::bswap => {
                if args[0].layout.size.bytes() == 1 {
                    return IntrinsicResult::Operand(args[0].val);
                }
                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::bswap,
                    &[args[0].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::saturating_add
            | sym::saturating_sub => {
                let ty = args[0].layout.ty;
                if !ty.is_integral() {
                    let err = self.tcx.dcx().emit_err(InvalidMonomorphization::BasicIntegerType {
                        span,
                        name,
                        ty,
                    });
                    return IntrinsicResult::Err(err);
                }
                let (size, signed) = ty.int_size_and_signed(self.tcx);

                let is_add = name == sym::saturating_add;
                let lhs = args[0].immediate();
                let rhs = args[1].immediate();
                let instruction = match (is_add, signed) {
                    (true, true) => InstructionKind::sat_sadd,
                    (true, false) => InstructionKind::sat_uadd,
                    (false, true) => InstructionKind::sat_ssub,
                    (false, false) => InstructionKind::sat_usub
                };

                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    instruction,
                    &[lhs, rhs]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::is_val_statically_known => {
                let result = self.cx.module.borrow_mut()
                    .add_const(Type::Bool, false as u128);
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::raw_eq => {
                let pointee = args[0].layout.ty.builtin_deref(true).unwrap();
                let size = self.layout_of(pointee).size.bytes();
                let result = if size == 0 {
                    self.cx.module.borrow_mut().add_const(Type::Bool, true as u128)
                } else {
                    let len = self.const_usize(size);
                    let func = self.cx.get_lib_fn(LibFunc::MemCmp);
                    let cmp = self.cx.module.borrow_mut().add_instruction_ret(
                        self.basic_block,
                        InstructionKind::Call,
                        &[func, args[0].immediate(), args[1].immediate(), len],
                        Type::i32
                    );
                    let zero = self.const_i32(0);
                    self.icmp(rustc_codegen_ssa::common::IntPredicate::IntEQ, cmp, zero)
                };
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::compare_bytes => {
                let func = self.cx.get_lib_fn(LibFunc::MemCmp);
                let cmp = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::Call,
                    &[func, args[0].immediate(), args[1].immediate(), args[2].immediate()],
                    Type::i32
                );
                IntrinsicResult::Operand(OperandValue::Immediate(cmp))
            }
            sym::select_unpredictable => {
                let cond = args[0].immediate();
                assert_eq!(args[1].layout, args[2].layout);

                let val = match (args[1].val, args[2].val) {
                    (OperandValue::Ref(true_val), OperandValue::Ref(false_val)) => {
                        assert!(true_val.llextra.is_none());
                        assert!(false_val.llextra.is_none());
                        assert_eq!(true_val.align, false_val.align);

                        let ptr = self.select(cond, true_val.llval, false_val.llval);
                        OperandValue::Ref(PlaceValue::new_sized(ptr, true_val.align))
                    }
                    (OperandValue::Immediate(true_val), OperandValue::Immediate(false_val)) => {
                        OperandValue::Immediate(self.select(cond, true_val, false_val))
                    }
                    (OperandValue::Pair(true_a, true_b), OperandValue::Pair(false_a, false_b)) => {
                        let a = self.select(cond, true_a, false_a);
                        let b = self.select(cond, true_b, false_b);
                        OperandValue::Pair(a, b)
                    }
                    _ => span_bug!(span, "Incompatible OperandValue for select_unpredictable"),
                };

                IntrinsicResult::Operand(val)
            }
            sym::volatile_load | sym::unaligned_volatile_load => {
                let ptr = args[0].immediate();
                let abi_align = result_layout.align.abi;
                let ptr_align = if name == sym::volatile_load { abi_align } else { Align::ONE };
                if result_layout.is_zst() {
                    return IntrinsicResult::Operand(OperandValue::ZeroSized);
                }
                match self.cx.tpde_direct_type(result_layout) {
                    FullType::Memory { .. } => {
                        let temp = match result_place {
                            Some(val) => PlaceRef { val, layout: result_layout },
                            None => PlaceRef::alloca(self, result_layout),
                        };
                        let size = self.const_usize(result_layout.size.bytes());
                        self.memcpy(temp.val.llval, abi_align, ptr, ptr_align, size, MemFlags::VOLATILE, None);
                        if result_place.is_none() {
                            IntrinsicResult::Operand(self.load_operand(temp).val)
                        } else {
                            IntrinsicResult::WroteIntoPlace
                        }
                    }
                    _ => {
                        let src = PlaceRef { val: PlaceValue::new_sized(ptr, ptr_align), layout: result_layout };
                        IntrinsicResult::Operand(self.load_operand(src).val)
                    }
                }
            }
            sym::typed_swap_nonoverlapping => {
                let pointee_layout = self.layout_of(instance.args.type_at(0));
                if !pointee_layout.is_zst() {
                    let align = pointee_layout.align.abi;
                    let x_place = PlaceValue::new_sized(args[0].immediate(), align);
                    let y_place = PlaceValue::new_sized(args[1].immediate(), align);
                    self.typed_place_swap(x_place, y_place, pointee_layout);
                }
                IntrinsicResult::Operand(OperandValue::ZeroSized)
            }
            sym::abort => {
                self.abort();
                IntrinsicResult::Operand(OperandValue::ZeroSized)
            }
            sym::ctlz_nonzero
            | sym::ctlz
            | sym::cttz_nonzero
            | sym::cttz => {
                let instr = match name {
                    sym::ctlz_nonzero => InstructionKind::ctlz_nonzero,
                    sym::ctlz => InstructionKind::ctlz,
                    sym::cttz_nonzero => InstructionKind::cttz_nonzero,
                    sym::cttz => InstructionKind::cttz,
                    _ => bug!(),
                };
                let res = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    instr,
                    &[args[0].immediate()],
                    Type::i32
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::rotate_left => {
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::rotl,
                    &[args[0].immediate(), args[1].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::rotate_right => {
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::rotr,
                    &[args[0].immediate(), args[1].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::unchecked_funnel_shl => {
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::funnel_shl,
                    &[args[0].immediate(), args[1].immediate(), args[2].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::unchecked_funnel_shr => {
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::funnel_shr,
                    &[args[0].immediate(), args[1].immediate(), args[2].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::simd_add => simd_binop!(self, args: Uint, Int => add, Float => fadd),
            sym::simd_sub => simd_binop!(self, args: Uint, Int => sub, Float => fsub),
            sym::simd_mul => simd_binop!(self, args: Uint, Int => mul, Float => fmul),
            sym::simd_div => simd_binop!(self, args: Uint => udiv, Int => sdiv, Float => fdiv),
            sym::simd_rem => simd_binop!(self, args: Uint => urem, Int => srem, Float => frem),
            sym::simd_shl => simd_binop!(self, args: Uint, Int => shl),
            sym::simd_shr => simd_binop!(self, args: Uint => lshr, Int => ashr),
            sym::simd_and => simd_binop!(self, args: Uint, Int => and),
            sym::simd_or => simd_binop!(self, args: Uint, Int => or),
            sym::simd_xor => simd_binop!(self, args: Uint, Int => xor),
            sym::simd_splat => {
                let FullType::Single(ret_ty) = self.cx.backend_type(result_layout) else { bug!() };
                let res = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::simd_splat,
                    &[args[0].immediate()],
                    ret_ty
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::minimum_number_nsz_f16
            | sym::minimum_number_nsz_f32
            | sym::minimum_number_nsz_f64
            | sym::minimum_number_nsz_f128 => {
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::fMin,
                    &[args[0].immediate(), args[1].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            | sym::maximum_number_nsz_f16
            | sym::maximum_number_nsz_f32
            | sym::maximum_number_nsz_f64
            | sym::maximum_number_nsz_f128 => {
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::fMax,
                    &[args[0].immediate(), args[1].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::simd_neg => simd_unop!(self, args -> result_layout: Uint, Int => Neg, Float => fNeg),
            // (a, b, c) -> a * b + c per lane, fused: no vector instruction without FMA3, so call
            // libm fma(f) on each lane.
            sym::simd_fma => {
                let (lanes, elem) = args[0].layout.ty.simd_size_and_type(self.tcx);
                let is_f64 = matches!(elem.kind(), ty::Float(rustc_ast::FloatTy::F64));
                let (lib_name, elem_ty) = if is_f64 { ("fma", Type::f64) } else { ("fmaf", Type::f32) };
                let func = self.cx.get_lib_fn(LibFunc::Math(lib_name, is_f64, 3));
                let FullType::Single(vec_ty) = self.val_ty(args[0].immediate()) else { bug!() };
                let mut res = args[0].immediate();
                for i in 0..lanes {
                    let idx = self.const_usize(i);
                    let a = self.extract_element(args[0].immediate(), idx);
                    let b = self.extract_element(args[1].immediate(), idx);
                    let c = self.extract_element(args[2].immediate(), idx);
                    let lane = self.cx.module.borrow_mut().add_instruction_ret(
                        self.basic_block,
                        InstructionKind::Call,
                        &[func, a, b, c],
                        elem_ty,
                    );
                    res = self.cx.module.borrow_mut().add_instruction_ret(
                        self.basic_block,
                        InstructionKind::simd_insert,
                        &[res, idx, lane],
                        vec_ty,
                    );
                }
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::simd_fabs => simd_unop!(self, args -> result_layout: Float => fAbs),
            sym::simd_fsqrt => simd_unop!(self, args -> result_layout: Float => fSqrt),
            sym::simd_minimum_number_nsz => simd_binop!(self, args -> result_layout: Float => fMin),
            sym::simd_maximum_number_nsz => simd_binop!(self, args -> result_layout: Float => fMax),
            sym::simd_saturating_add => simd_binop!(self, args -> result_layout: Int => sat_sadd, Uint => sat_uadd),
            sym::simd_saturating_sub => simd_binop!(self, args -> result_layout: Int => sat_ssub, Uint => sat_usub),

            sym::simd_eq => simd_binop!(self, args -> result_layout: Uint, Int => CMPeq, Float => RealOEQ),
            sym::simd_ne => simd_binop!(self, args -> result_layout: Uint, Int => CMPne, Float => RealUNE),
            sym::simd_lt => simd_binop!(self, args -> result_layout: Int => CMPslt, Uint => CMPult, Float => RealOLT),
            sym::simd_le => simd_binop!(self, args -> result_layout: Int => CMPsle, Uint => CMPule, Float => RealOLE),
            sym::simd_gt => simd_binop!(self, args -> result_layout: Int => CMPsgt, Uint => CMPugt, Float => RealOGT),
            sym::simd_ge => simd_binop!(self, args -> result_layout: Int => CMPsge, Uint => CMPuge, Float => RealOGE),

            // Bit i of the result is the top bit of lane i.
            sym::simd_bitmask => simd_unop!(self, args -> result_layout: Uint, Int => simd_bitmask),

            // Reductions; the ordered variants take the start value as second operand.
            sym::simd_reduce_add_ordered => simd_binop!(self, args -> result_layout: Uint, Int, Float => simd_reduce_add_ordered),
            sym::simd_reduce_mul_ordered => simd_binop!(self, args -> result_layout: Uint, Int, Float => simd_reduce_mul_ordered),
            sym::simd_reduce_add_unordered => simd_unop!(self, args -> result_layout: Uint, Int, Float => simd_reduce_add_unordered),
            sym::simd_reduce_mul_unordered => simd_unop!(self, args -> result_layout: Uint, Int, Float => simd_reduce_mul_unordered),
            sym::simd_reduce_min => simd_unop!(self, args -> result_layout: Int => simd_reduce_smin, Uint => simd_reduce_umin, Float => simd_reduce_fmin),
            sym::simd_reduce_max => simd_unop!(self, args -> result_layout: Int => simd_reduce_smax, Uint => simd_reduce_umax, Float => simd_reduce_fmax),
            sym::simd_reduce_and => simd_unop!(self, args -> result_layout: Uint, Int => simd_reduce_and),
            sym::simd_reduce_or => simd_unop!(self, args -> result_layout: Uint, Int => simd_reduce_or),
            sym::simd_reduce_xor => simd_unop!(self, args -> result_layout: Uint, Int => simd_reduce_xor),
            sym::simd_reduce_any => simd_unop!(self, args -> result_layout: Uint, Int => simd_reduce_any),
            sym::simd_reduce_all => simd_unop!(self, args -> result_layout: Uint, Int => simd_reduce_all),

            // (vector, index)
            sym::simd_extract | sym::simd_extract_dyn =>
                simd_binop!(self, args -> result_layout: Uint, Int, Float => simd_extract),

            // (mask vector, then, else) / (integer bitmask, then, else) / (vector, index, element)
            sym::simd_select | sym::simd_select_bitmask | sym::simd_insert | sym::simd_insert_dyn => {
                let instr = match name {
                    sym::simd_select => InstructionKind::Select,
                    sym::simd_select_bitmask => InstructionKind::simd_select_bitmask,
                    _ => InstructionKind::simd_insert,
                };
                let FullType::Single(ret_ty) = self.cx.backend_type(result_layout) else { bug!() };
                let res = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    instr,
                    &[args[0].immediate(), args[1].immediate(), args[2].immediate()],
                    ret_ty
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }

            // (a, b, constant index vector): the backend reads the indices from the constant,
            // index i >= len(a) selects lane i - len(a) of b.
            sym::simd_shuffle => {
                let FullType::Single(ret_ty) = self.cx.backend_type(result_layout) else { bug!() };
                let res = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::simd_shuffle,
                    &[args[0].immediate(), args[1].immediate(), args[2].immediate()],
                    ret_ty
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }

            sym::simd_cast | sym::simd_as => {
                let (_, src) = args[0].layout.ty.simd_size_and_type(self.tcx);
                let (_, dst) = result_layout.ty.simd_size_and_type(self.tcx);
                if src == dst {
                    return IntrinsicResult::Operand(args[0].val);
                }
                let src_bits = self.layout_of(src).size.bits();
                let dst_bits = self.layout_of(dst).size.bits();
                let saturating = name == sym::simd_as;
                let instr = match (src.kind(), dst.kind()) {
                    (ty::Float(_), ty::Float(_)) if src_bits < dst_bits => InstructionKind::fExt,
                    (ty::Float(_), ty::Float(_)) => InstructionKind::fTrunc,
                    (ty::Float(_), ty::Int(_)) if saturating => InstructionKind::fTos_sat,
                    (ty::Float(_), ty::Int(_)) => InstructionKind::fTos,
                    (ty::Float(_), _) if saturating => InstructionKind::fTou_sat,
                    (ty::Float(_), _) => InstructionKind::fTou,
                    (ty::Int(_), ty::Float(_)) => InstructionKind::sTof,
                    (_, ty::Float(_)) => InstructionKind::uTof,
                    _ if src_bits > dst_bits => InstructionKind::Trunc,
                    _ if src_bits == dst_bits => InstructionKind::Cast,
                    (ty::Int(_), _) => InstructionKind::sExt,
                    _ => InstructionKind::zExt,
                };
                let FullType::Single(ret_ty) = self.cx.backend_type(result_layout) else { bug!() };
                let res = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    instr,
                    &[args[0].immediate()],
                    ret_ty
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }

            // Vectors of pointers are vectors of i64, so these do not change the bits.
            sym::simd_cast_ptr | sym::simd_expose_provenance | sym::simd_with_exposed_provenance => {
                IntrinsicResult::Operand(args[0].val)
            }
            // (pointers, offsets): pointer + offset * size_of::<pointee>()
            sym::simd_arith_offset => {
                let (lanes, elem) = args[0].layout.ty.simd_size_and_type(self.tcx);
                let ty::RawPtr(pointee, _) = elem.kind() else {
                    bug!("simd_arith_offset on non-pointer vector")
                };
                let size = self.const_u64(self.layout_of(*pointee).size.bytes());
                let size = self.vector_splat(lanes as usize, size);
                let offset = self.mul(args[1].immediate(), size);
                IntrinsicResult::Operand(OperandValue::Immediate(self.add(args[0].immediate(), offset)))
            }

            // gather: (default, pointers, mask), scatter: (values, pointers, mask),
            // masked_load: (mask, pointer, default), masked_store: (mask, pointer, values).
            // The lane alignment is appended as raw operand.
            sym::simd_gather | sym::simd_scatter | sym::simd_masked_load | sym::simd_masked_store => {
                let values = if matches!(name, sym::simd_gather | sym::simd_scatter) { 0 } else { 2 };
                let (_, elem) = args[values].layout.ty.simd_size_and_type(self.tcx);
                let align = Slot::new_raw(self.layout_of(elem).align.bytes() as u32);
                let ops = [args[0].immediate(), args[1].immediate(), args[2].immediate(), align];
                let mut module = self.cx.module.borrow_mut();
                match name {
                    sym::simd_gather | sym::simd_masked_load => {
                        let instr = if name == sym::simd_gather {
                            InstructionKind::simd_gather
                        } else {
                            InstructionKind::simd_masked_load
                        };
                        let FullType::Single(ret_ty) = self.cx.backend_type(result_layout) else { bug!() };
                        let res = module.add_instruction_ret(self.basic_block, instr, &ops, ret_ty);
                        IntrinsicResult::Operand(OperandValue::Immediate(res))
                    }
                    _ => {
                        let instr = if name == sym::simd_scatter {
                            InstructionKind::simd_scatter
                        } else {
                            InstructionKind::simd_masked_store
                        };
                        module.add_instruction(self.basic_block, instr, &ops);
                        IntrinsicResult::Operand(OperandValue::ZeroSized)
                    }
                }
            }
            sym::copysignf32 | sym::copysignf64 => {
                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::copysign,
                    &[args[0].immediate(), args[1].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::catch_unwind => {
                let (try_fn, data, catch_fn) = (args[0].immediate(), args[1].immediate(), args[2].immediate());
                let ptr_ty = FullType::Single(Type::ptr);
                let try_sig = self.function_signature(&[ptr_ty], None);

                let result = if !self.tcx.sess.panic_strategy().unwinds() {
                    // No unwinding, so just try and panic
                    self.module.borrow_mut().add_call(self.basic_block, try_fn, &try_sig, &[data]);
                    self.const_bool(false)
                } else {
                    // cur_block:
                    // panic = false
                    // try_fn(...)
                    // - suc -> continue_block
                    // - exc -> exception_block

                    // exception_block:
                    // catch_fn()
                    // panic = true
                    // jump to continue_block

                    // continue_block:
                    // return panic

                    let catch_sig = self.function_signature(&[ptr_ty, ptr_ty], None);
                    let align = Align::ONE;
                    let res = self.alloca(Size::from_bytes(1), align);
                    let catch_bb = self.append_sibling_block("catch_unwind_catch");
                    let cont_bb = self.append_sibling_block("catch_unwind_cont");

                    // res = false, set to true in the catch block
                    let no_panic = self.const_bool(false);
                    self.store(no_panic, res, align);
                    self.module.borrow_mut().add_invoke(
                        self.basic_block, try_fn, &try_sig, cont_bb, catch_bb, &[data]);

                    self.switch_to_block(catch_bb);
                    let pers = self.eh_personality();
                    let (exn, _) = self.landing_pad(pers, LandingPadKind::CatchAll);
                    self.module.borrow_mut().add_call(self.basic_block, catch_fn, &catch_sig, &[data, exn]);
                    let panicked = self.const_bool(true);
                    self.store(panicked, res, align);
                    self.br(cont_bb);

                    self.switch_to_block(cont_bb);
                    let bool_ty = FullType::Single(Type::Bool);
                    self.load(bool_ty, res, align)
                };
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::fabs => {
                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::fAbs,
                    &[args[0].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::sqrtf32 | sym::sqrtf64 => {
                let result = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::fSqrt,
                    &[args[0].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::sinf32 | sym::sinf64 | sym::cosf32 | sym::cosf64
            | sym::expf32 | sym::expf64 | sym::exp2f32 | sym::exp2f64
            | sym::logf32 | sym::logf64 | sym::log10f32 | sym::log10f64
            | sym::log2f32 | sym::log2f64 | sym::powf32 | sym::powf64
            | sym::ceilf32 | sym::ceilf64 | sym::floorf32 | sym::floorf64
            | sym::truncf32 | sym::truncf64 | sym::roundf32 | sym::roundf64
            | sym::round_ties_even_f32 | sym::round_ties_even_f64
            | sym::fmaf32 | sym::fmaf64 | sym::powif32 | sym::powif64 => {
                let (lib_name, is_f64) = match name {
                    sym::sinf32 => ("sinf", false), sym::sinf64 => ("sin", true),
                    sym::cosf32 => ("cosf", false), sym::cosf64 => ("cos", true),
                    sym::expf32 => ("expf", false), sym::expf64 => ("exp", true),
                    sym::exp2f32 => ("exp2f", false), sym::exp2f64 => ("exp2", true),
                    sym::logf32 => ("logf", false), sym::logf64 => ("log", true),
                    sym::log10f32 => ("log10f", false), sym::log10f64 => ("log10", true),
                    sym::log2f32 => ("log2f", false), sym::log2f64 => ("log2", true),
                    sym::powf32 => ("powf", false), sym::powf64 => ("pow", true),
                    sym::ceilf32 => ("ceilf", false), sym::ceilf64 => ("ceil", true),
                    sym::floorf32 => ("floorf", false), sym::floorf64 => ("floor", true),
                    sym::truncf32 => ("truncf", false), sym::truncf64 => ("trunc", true),
                    sym::roundf32 => ("roundf", false), sym::roundf64 => ("round", true),
                    sym::round_ties_even_f32 => ("rintf", false),
                    sym::round_ties_even_f64 => ("rint", true),
                    sym::fmaf32 => ("fmaf", false), sym::fmaf64 => ("fma", true),
                    sym::powif32 => ("__powisf2", false), sym::powif64 => ("__powidf2", true),
                    _ => unreachable!()
                };
                let func = self.cx.get_lib_fn(LibFunc::Math(lib_name, is_f64, args.len()));
                let mut ops = vec![func];
                ops.extend(args.iter().map(|arg| arg.immediate()));
                let result = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::Call,
                    &ops,
                    if is_f64 { Type::f64 } else { Type::f32 }
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::bitreverse => {
                let ty = self.cx.backend_type(args[0].layout);
                let bits = args[0].layout.size.bits();
                let mut x = args[0].immediate();
                if bits > 8 {
                    x = self.cx.module.borrow_mut().add_instruction_ret_first(
                        self.basic_block,
                        InstructionKind::bswap,
                        &[x]
                    );
                }
                for (shift, pattern) in [(4, 0x0F), (2, 0x33), (1, 0x55)] {
                    let mask = (u128::MAX / 0xFF * pattern) & (u128::MAX >> (128 - bits));
                    let mask = self.const_uint_big(ty, mask);
                    let shift = self.const_uint(ty, shift);
                    let hi = self.lshr(x, shift);
                    let hi = self.and(hi, mask);
                    let lo = self.and(x, mask);
                    let lo = self.shl(lo, shift);
                    x = self.or(hi, lo);
                }
                IntrinsicResult::Operand(OperandValue::Immediate(x))
            }
            _ => IntrinsicResult::Fallback(instance)
        }
    }

    fn codegen_llvm_intrinsic_call(
        &mut self,
        instance: Instance<'tcx>,
        args: &[OperandRef<'tcx, Self::Value>],
        is_cleanup: bool,
    ) -> Self::Value {
        let name = self.tcx.item_name(instance.def_id());

        match name.as_str() {
            "pause" => {
                self.module.borrow_mut()
                    .add_instruction_ret(self.basic_block,
                                     InstructionKind::Pause,
                                     &[],
                                    Type::Void)
            }
            "pshufb" | "pshufb128" => {
                let (a, idx) = (args[0].immediate(), args[1].immediate());
                let FullType::Single(vec_ty) = self.val_ty(a) else { bug!() };
                let (_, nelem) = crate::shared::ir::vector_info(vec_ty)
                    .map(|(e, n)| (e, n))
                    .expect("pshufb on non-vector");
                let zero = self.const_u8(0);
                let i64_ty = self.type_i64();
                let mut res = a;
                for i in 0..nelem as u64 {
                    let i_const = self.const_usize(i);
                    let sel = self.extract_element(idx, i_const);
                    let sel_idx = self.zext(sel, i64_ty);
                    let elem = self.extract_element(a, sel_idx);
                    let neg = self.icmp(rustc_codegen_ssa::common::IntPredicate::IntSLT, sel, zero);
                    let elem = self.select(neg, zero, elem);
                    res = self.module.borrow_mut().add_instruction_ret(
                        self.basic_block,
                        InstructionKind::simd_insert,
                        &[res, i_const, elem],
                        vec_ty,
                    );
                }
                res
            }
            "psadbw" => {
                let (a, b) = (args[0].immediate(), args[1].immediate());
                let FullType::Single(in_ty) = self.val_ty(a) else { bug!() };
                // SSE2 / AVX2 / AVX-512 forms: one 64-bit sum per 8 bytes
                let out_ty = match in_ty {
                    Type::v16i8 => Type::v2i64,
                    Type::v32i8 => Type::v4i64,
                    Type::v64i8 => Type::v8i64,
                    _ => bug!("psadbw on {in_ty:?}"),
                };
                self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::psadbw,
                    &[a, b],
                    out_ty,
                )
            }
            "pmaddwd" => {
                let (a, b) = (args[0].immediate(), args[1].immediate());
                let FullType::Single(in_ty) = self.val_ty(a) else { bug!() };
                let out_ty = match in_ty {
                    Type::v8i16 => Type::v4i32,
                    Type::v16i16 => Type::v8i32,
                    Type::v32i16 => Type::v16i32,
                    _ => bug!("pmaddwd on {in_ty:?}"),
                };
                self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::pmaddwd,
                    &[a, b],
                    out_ty,
                )
            }
            "pmaddubsw" | "pmaddubsw128" => {
                let (a, b) = (args[0].immediate(), args[1].immediate());
                let FullType::Single(in_ty) = self.val_ty(a) else { bug!() };
                let out_ty = match in_ty {
                    Type::v16i8 => Type::v8i16,
                    Type::v32i8 => Type::v16i16,
                    Type::v64i8 => Type::v32i16,
                    _ => bug!("pmaddubsw on {in_ty:?}"),
                };
                self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::pmaddubsw,
                    &[a, b],
                    out_ty,
                )
            }
            "psllw" | "pslld" | "psllq" | "psrlw" | "psrld" | "psrlq" | "psraw" | "psrad" => {
                let op = ["psllw", "pslld", "psllq", "psrlw", "psrld", "psrlq", "psraw", "psrad"]
                    .iter()
                    .position(|n| *n == name.as_str())
                    .unwrap();
                self.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::simd_shift,
                    &[args[0].immediate(), args[1].immediate(), Slot::new_raw(op as u32)],
                )
            }
            "minps" | "maxps" | "minpd" | "maxpd" | "minss" | "maxss" | "minsd" | "maxsd"
            | "vminps" | "vmaxps" | "vminpd" | "vmaxpd" => {
                let op = ["minps", "maxps", "minpd", "maxpd", "minss", "maxss", "minsd", "maxsd"]
                    .iter()
                    .position(|n| *n == name.as_str().trim_start_matches('v'))
                    .unwrap();
                self.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::simd_minmax,
                    &[args[0].immediate(), args[1].immediate(), Slot::new_raw(op as u32)],
                )
            }
            "rcpps" | "rsqrtps" | "rcpss" | "rsqrtss" | "vrcpps" | "vrsqrtps" => {
                let op = ["rcpps", "rsqrtps", "rcpss", "rsqrtss"]
                    .iter()
                    .position(|n| *n == name.as_str().trim_start_matches('v'))
                    .unwrap();
                self.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::simd_approx,
                    &[args[0].immediate(), Slot::new_raw(op as u32)],
                )
            }
            "cmpps" | "cmppd" | "cmpss" | "cmpsd" | "vcmpps" | "vcmppd" => {
                let kind = ["cmpps", "cmppd", "cmpss", "cmpsd"]
                    .iter()
                    .position(|n| *n == name.as_str().trim_start_matches('v'))
                    .unwrap();
                let Some(pred) = self.cx.const_to_opt_uint(args[2].immediate()) else {
                    bug!("{} predicate must be a constant", name);
                };
                if pred >= 8 {
                    bug!("{} predicate {} needs AVX encoding, only 0..8 supported", name, pred);
                }
                self.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::simd_cmp,
                    &[args[0].immediate(), args[1].immediate(),
                         Slot::new_raw((kind as u32) | ((pred as u32) << 2))],
                )
            }
            "cvtps2dq" | "cvttps2dq" | "vcvtps2dq" | "vcvttps2dq" => {
                let a = args[0].immediate();
                let FullType::Single(in_ty) = self.val_ty(a) else { bug!() };
                let out_ty = match in_ty {
                    Type::v4f32 => Type::v4i32,
                    Type::v8f32 => Type::v8i32,
                    Type::v16f32 => Type::v16i32,
                    _ => bug!("{} on {in_ty:?}", name),
                };
                let truncate = name.as_str().contains("cvtt");
                self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::simd_cvt,
                    &[a, Slot::new_raw(truncate as u32)],
                    out_ty,
                )
            }
            "pclmulqdq" => {
                let (a, b) = (args[0].immediate(), args[1].immediate());
                let Some(imm) = self.cx.const_to_opt_uint(args[2].immediate()) else {
                    bug!("pclmulqdq immediate must be a constant");
                };
                let sel = (imm & 1) | (((imm >> 4) & 1) << 1);
                let vec_ty = self.val_ty(a);
                let FullType::Single(vec_ty) = vec_ty else { bug!() };
                self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::clmul,
                    &[a, b, Slot::new_raw(sel as u32)],
                    vec_ty,
                )
            }
            "crc32_32_8" | "crc32_32_16" | "crc32_32_32" | "crc32_64_64" => {
                self.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::crc32,
                    &[args[0].immediate(), args[1].immediate()],
                )
            }
            "lddqu" | "vlddqu" => {
                let ty = if name.as_str() == "vlddqu" { Type::v32i8 } else { Type::v16i8 };
                self.load(FullType::Single(ty), args[0].immediate(), Align::ONE)
            }
            "pmulhrsw" | "pmulhrsw128" => {
                self.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::pmulhrsw,
                    &[args[0].immediate(), args[1].immediate()],
                )
            }
            "llvm_vcvtps2ph_128" | "llvm_vcvtps2ph_256" => {
                let op = if name.as_str().ends_with("256") { 3 } else { 2 };
                let Some(imm) = self.cx.const_to_opt_uint(args[1].immediate()) else {
                    bug!("vcvtps2ph rounding control must be a constant");
                };
                self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::simd_cvt,
                    &[args[0].immediate(), Slot::new_raw(op | ((imm as u32 & 0xFF) << 2))],
                    Type::v8i16,
                )
            }
            "rdtscp" => {
                let aux_align = Align::from_bytes(4).unwrap();
                let aux_ptr = self.alloca(Size::from_bytes(4), aux_align);
                let tsc = self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::rdtscp,
                    &[aux_ptr],
                    Type::i64,
                );
                let aux = self.load(FullType::Single(Type::i32), aux_ptr, aux_align);
                self.module.borrow_mut().add_pair(tsc, aux, 8)
            }
            "llvm_addcarry_u32" | "llvm_addcarry_u64" | "llvm_subborrow_u32" | "llvm_subborrow_u64" => {
                use rustc_codegen_ssa::common::IntPredicate::{IntNE, IntULT};
                let is_64 = name.as_str().ends_with("64");
                let is_add = name.as_str().contains("addcarry");
                let carry_in = args[0].immediate();
                let a = args[1].immediate();
                let b = args[2].immediate();
                let ty = self.cx.backend_type(args[1].layout);
                let i8_ty = FullType::Single(Type::i8);

                let zero = self.const_u8(0);
                let carry_in = self.icmp(IntNE, carry_in, zero);
                let carry_in = self.zext(carry_in, ty);
                let (o1, res, o2) = if is_add {
                    let t = self.add(a, b);
                    let res = self.add(t, carry_in);
                    (self.icmp(IntULT, t, a), res, self.icmp(IntULT, res, t))
                } else {
                    let t = self.sub(a, b);
                    let res = self.sub(t, carry_in);
                    (self.icmp(IntULT, a, b), res, self.icmp(IntULT, t, carry_in))
                };
                let o1 = self.zext(o1, i8_ty);
                let o2 = self.zext(o2, i8_ty);
                let carry_out = self.or(o1, o2);
                self.module.borrow_mut().add_pair(carry_out, res, if is_64 { 8 } else { 4 })
            }
            "rdtsc" => {
                self.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::rdtsc,
                    &[],
                    Type::i64,
                )
            }
            _ => {
                todo!("Unimplemented llvm intrinsic: {}", name.as_str());
            }
        }
    }

    fn abort(&mut self) {
        self.module.borrow_mut().add_instruction(self.basic_block,
                                                 InstructionKind::Abort,
                                                 &[])
    }

    fn assume(&mut self, val: Self::Value) {
        // Just some tips for the optimizer
    }

    fn expect(&mut self, cond: Self::Value, expected: bool) -> Self::Value {
        cond
    }

    fn type_checked_load(
        &mut self,
        llvtable: Self::Value,
        vtable_byte_offset: u64,
        typeid: &[u8],
    ) -> Self::Value {
        todo!()
    }

    fn va_start(&mut self, val: Self::Value) {
        todo!()
    }

    fn retag_mem(&mut self, place: Self::Value, info: &RetagInfo<Self::Value>) {
        todo!()
    }

    fn retag_reg(&mut self, ptr: Self::Value, info: &RetagInfo<Self::Value>) -> Self::Value {
        todo!()
    }
}
