use crate::builder::Builder;
use crate::shared::ir::{FullType, InstructionKind, Slot, Type};
use rustc_codegen_ssa::RetagInfo;
use rustc_codegen_ssa::diagnostics::InvalidMonomorphization;
use rustc_codegen_ssa::mir::IntrinsicResult;
use rustc_codegen_ssa::mir::operand::{OperandRef, OperandValue};
use rustc_codegen_ssa::mir::place::PlaceValue;
use rustc_codegen_ssa::traits::{
    BuilderMethods, ConstCodegenMethods, IntrinsicCallBuilderMethods,
    LayoutTypeCodegenMethods,
};
use rustc_middle::{bug, span_bug};
use rustc_middle::ty::{self, Instance};
use rustc_middle::ty::layout::{LayoutOf, TyAndLayout};
use rustc_span::{Span, sym};

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
            // Emits the instruction directly, the result has the type of `$ret`.
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
                    vec![$args[0].immediate(), $args[1].immediate()],
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
                    vec![$args[0].immediate()],
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
                    vec![input_val]
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
                    vec![lhs, rhs]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::is_val_statically_known => {
                let result = self.cx.module.borrow_mut()
                    .add_const(Type::Bool, false as u128);
                IntrinsicResult::Operand(OperandValue::Immediate(result))
            }
            sym::compare_bytes => {
                let cmp = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::MemCmp,
                    vec![args[0].immediate(), args[1].immediate(), args[2].immediate()],
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
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    instr,
                    vec![args[0].immediate()]
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
                let res = self.cx.module.borrow_mut().add_instruction_ret_first(
                    self.basic_block,
                    InstructionKind::simd_splat,
                    vec![args[0].immediate()]
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
                    vec![args[0].immediate(), args[1].immediate()]
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
                    vec![args[0].immediate(), args[1].immediate()]
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }
            sym::simd_neg => simd_unop!(self, args -> result_layout: Uint, Int => Neg, Float => fNeg),
            sym::simd_fabs => simd_unop!(self, args -> result_layout: Float => fAbs),
            sym::simd_minimum_number_nsz => simd_binop!(self, args -> result_layout: Float => fMin),
            sym::simd_maximum_number_nsz => simd_binop!(self, args -> result_layout: Float => fMax),
            sym::simd_saturating_add => simd_binop!(self, args -> result_layout: Int => sat_sadd, Uint => sat_uadd),
            sym::simd_saturating_sub => simd_binop!(self, args -> result_layout: Int => sat_ssub, Uint => sat_usub),

            // Comparisons produce a mask vector with all lane bits set or cleared.
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
                    vec![args[0].immediate(), args[1].immediate(), args[2].immediate()],
                    ret_ty
                );
                IntrinsicResult::Operand(OperandValue::Immediate(res))
            }

            // (a, b, constant indices): the indices are passed as raw operands,
            // index i >= len(a) selects lane i - len(a) of b.
            sym::simd_shuffle => {
                let mut ops = vec![args[0].immediate(), args[1].immediate()];
                {
                    let module = self.cx.module.borrow();
                    let indices = module.const_vector_elems(args[2].immediate())
                        .expect("shuffle indices must be a constant vector");
                    ops.extend(indices.iter().map(|&i| Slot::new_raw(module.const_data(i).unwrap() as u32)));
                }
                let FullType::Single(ret_ty) = self.cx.backend_type(result_layout) else { bug!() };
                let res = self.cx.module.borrow_mut().add_instruction_ret(
                    self.basic_block,
                    InstructionKind::simd_shuffle,
                    ops,
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
                    vec![args[0].immediate()],
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
                let ops = vec![args[0].immediate(), args[1].immediate(), args[2].immediate(), align];
                let mut module = self.cx.module.borrow_mut();
                match name {
                    sym::simd_gather | sym::simd_masked_load => {
                        let instr = if name == sym::simd_gather {
                            InstructionKind::simd_gather
                        } else {
                            InstructionKind::simd_masked_load
                        };
                        let FullType::Single(ret_ty) = self.cx.backend_type(result_layout) else { bug!() };
                        let res = module.add_instruction_ret(self.basic_block, instr, ops, ret_ty);
                        IntrinsicResult::Operand(OperandValue::Immediate(res))
                    }
                    _ => {
                        let instr = if name == sym::simd_scatter {
                            InstructionKind::simd_scatter
                        } else {
                            InstructionKind::simd_masked_store
                        };
                        module.add_instruction(self.basic_block, instr, ops);
                        IntrinsicResult::Operand(OperandValue::ZeroSized)
                    }
                }
            }
            _ => {
                panic!("Unimplemented intrinsic: {}", name.as_str());
            }
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
                                     vec![],
                                    Type::Void)
            }
            _ => todo!()
        }
    }

    fn abort(&mut self) {
        self.module.borrow_mut().add_instruction(self.basic_block,
                                                 InstructionKind::Abort,
                                                 vec![])
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

