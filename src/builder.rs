mod coverageinfo;
mod intrinsic;

use core::borrow::Borrow;
use crate::context::{CodegenCx, GenericCx, SCx};
use crate::shared::ir::{convert_atomic_order, size_of_type, BasicBlock, FullType, Function, InstructionKind, Module, Slot, Type, convert_atomic_op};
use rustc_ast::expand::typetree::FncTree;
use rustc_codegen_ssa::MemFlags;
use rustc_codegen_ssa::common::{
    AtomicRmwBinOp, IntPredicate, RealPredicate, SynchronizationScope,
};
use rustc_codegen_ssa::mir::operand::{OperandRef, OperandValue};
use rustc_codegen_ssa::mir::place::PlaceRef;
use rustc_codegen_ssa::traits::{BackendTypes, BaseTypeCodegenMethods, BuilderMethods, ConstCodegenMethods, OverflowOp};
use rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs;
use rustc_middle::ty::layout::TyAndLayout;
use rustc_middle::ty::{AtomicOrdering, Instance, Ty};
use rustc_span::Span;
use std::ops::Deref;
use crate::shared::ir;

pub struct Builder<'a, 'tpde, 'tcx> {
    pub cx: &'a CodegenCx<'tpde, 'tcx>,
    pub basic_block: BasicBlock,
}

impl<'a, 'tpde, 'tcx> BackendTypes for Builder<'a, 'tpde, 'tcx> {
    type Function = <CodegenCx<'tpde, 'tcx> as BackendTypes>::Function;
    type BasicBlock = <CodegenCx<'tpde, 'tcx> as BackendTypes>::BasicBlock;
    type Funclet = <CodegenCx<'tpde, 'tcx> as BackendTypes>::Funclet;

    type Value = <CodegenCx<'tpde, 'tcx> as BackendTypes>::Value;
    type Type = <CodegenCx<'tpde, 'tcx> as BackendTypes>::Type;
    type FunctionSignature = <CodegenCx<'tpde, 'tcx> as BackendTypes>::FunctionSignature;

    type DIScope = <CodegenCx<'tpde, 'tcx> as BackendTypes>::DIScope;
    type DILocation = <CodegenCx<'tpde, 'tcx> as BackendTypes>::DILocation;
    type DIVariable = <CodegenCx<'tpde, 'tcx> as BackendTypes>::DIVariable;
}

impl<'tpde, CX: Borrow<SCx<'tpde>>> BackendTypes for GenericCx<'tpde, CX> {
    type Function = Function;
    type BasicBlock = BasicBlock;
    type Funclet = ();
    type Value = Slot;
    type Type = FullType;
    type FunctionSignature = usize;
    type DIScope = ();
    type DILocation = ();
    type DIVariable = ();
}

impl<'a, 'tpde, 'tcx> Builder<'a, 'tpde, 'tcx> {
    fn with_cx(cx: &'a CodegenCx<'tpde, 'tcx>, basic_block: BasicBlock) -> Self {
        Builder { cx, basic_block }
    }
}

impl<'tpde, 'tcx> Deref for Builder<'_, 'tpde, 'tcx> {
    type Target = CodegenCx<'tpde, 'tcx>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.cx
    }
}

impl<'a, 'tpde, 'tcx> BuilderMethods<'a, 'tcx> for Builder<'a, 'tpde, 'tcx> {
    type CodegenCx = CodegenCx<'tpde, 'tcx>;

    fn build(cx: &'a Self::CodegenCx, bb: Self::BasicBlock) -> Self {
        Builder::with_cx(cx, bb)
    }

    fn cx(&self) -> &Self::CodegenCx {
        self.cx
    }

    fn llbb(&self) -> Self::BasicBlock {
        self.basic_block
    }

    fn set_span(&mut self, span: Span) {}

    fn append_block(
        cx: &'a Self::CodegenCx,
        tpde_fn: Self::Function,
        name: &str,
    ) -> Self::BasicBlock {
        cx.module.borrow_mut().add_basic_block(&tpde_fn, name)
    }

    fn append_sibling_block(&mut self, name: &str) -> Self::BasicBlock {
        Self::append_block(self.cx, self.basic_block.function(), name)
    }

    fn switch_to_block(&mut self, llbb: Self::BasicBlock) {
        self.basic_block = llbb;
    }

    fn ret_void(&mut self) {
        let module = &mut self.module.borrow_mut();
        module.add_instruction(self.basic_block, InstructionKind::Ret, vec![])
    }

    fn ret(&mut self, v: Self::Value) {
        let module = &mut self.module.borrow_mut();

        let ops = match v.pair_slots(module) {
            Some((slot_a, slot_b)) => vec![slot_a, slot_b],
            None => vec![v],
        };
        module.add_instruction(self.basic_block, InstructionKind::Ret, ops)
    }

    fn br(&mut self, dest: Self::BasicBlock) {
        self.module.borrow_mut().add_br(self.basic_block, dest);
    }

    fn cond_br(&mut self, cond: Self::Value, then_bb: Self::BasicBlock, else_bb: Self::BasicBlock) {
        self.module
            .borrow_mut()
            .add_cond_br(self.basic_block, cond, then_bb, else_bb)
    }

    fn switch(
        &mut self,
        v: Self::Value,
        else_llbb: Self::BasicBlock,
        cases: impl ExactSizeIterator<Item = (u128, Self::BasicBlock)>,
    ) {
        let cases = cases.map(|(val, bb)| (val, bb)).collect::<Vec<_>>();
        self.module.borrow_mut().add_switch(self.basic_block, v, else_llbb, &cases);
    }

    fn invoke(
        &mut self,
        func_sig: Self::FunctionSignature,
        fn_attrs: Option<&CodegenFnAttrs>,
        fn_abi: Option<&rustc_target::callconv::FnAbi<'tcx, Ty<'tcx>>>,
        fn_val: Self::Value,
        args: &[Self::Value],
        then: Self::BasicBlock,
        catch: Self::BasicBlock,
        funclet: Option<&Self::Funclet>,
        instance: Option<Instance<'tcx>>,
    ) -> Self::Value {
        let func_sig = &self.function_signatures.borrow()[func_sig];

        self.module
            .borrow_mut()
            .add_invoke(self.basic_block, fn_val, func_sig, then, catch, args)
            .unwrap_or_else(|| Slot::new_raw(0))
    }

    fn unreachable(&mut self) {
        self.module.borrow_mut().add_instruction(self.basic_block, InstructionKind::Unreachable, vec![]);
    }

    fn add(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Add,
            vec![lhs, rhs],
        )
    }

    fn fadd(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::fAdd,
            vec![lhs, rhs],
        )
    }

    fn fadd_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fadd(lhs, rhs)
    }

    fn fadd_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fadd(lhs, rhs)
    }

    fn sub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Sub,
            vec![lhs, rhs],
        )
    }

    fn fsub(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::fSub,
            vec![lhs, rhs],
        )
    }

    fn fsub_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fsub(lhs, rhs)
    }

    fn fsub_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fsub(lhs, rhs)
    }

    fn mul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Mul,
            vec![lhs, rhs],
        )
    }

    fn fmul(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::fMul,
            vec![lhs, rhs],
        )
    }

    fn fmul_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fmul(lhs, rhs)
    }

    fn fmul_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fmul(lhs, rhs)
    }

    fn udiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::uDiv,
            vec![lhs, rhs]
        )
    }

    fn exactudiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.udiv(lhs, rhs)
    }

    fn sdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::sDiv,
            vec![lhs, rhs]
        )
    }

    fn exactsdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.sdiv(lhs, rhs)
    }

    fn fdiv(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::fDiv,
            vec![lhs, rhs]
        )
    }

    fn fdiv_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fdiv(lhs, rhs)
    }

    fn fdiv_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.fdiv(lhs, rhs)
    }

    fn urem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::uRem,
            vec![lhs, rhs]
        )
    }

    fn srem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_x(
            self.basic_block,
            InstructionKind::sRem,
            vec![lhs, rhs],
            1
        )
    }

    fn frem(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_x(
            self.basic_block,
            InstructionKind::fRem,
            vec![lhs, rhs],
            1
        )
    }

    fn frem_fast(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.frem(lhs, rhs)
    }

    fn frem_algebraic(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.frem(lhs, rhs)
    }

    fn shl(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Shl,
            vec![lhs, rhs],
        )
    }

    fn lshr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_x(
            self.basic_block,
            InstructionKind::lShr,
            vec![lhs, rhs],
            1
        )
    }

    fn ashr(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_x(
            self.basic_block,
            InstructionKind::aShr,
            vec![lhs, rhs],
            1
        )
    }

    fn and(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::And,
            vec![lhs, rhs],
        )
    }

    fn or(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Or,
            vec![lhs, rhs],
        )
    }

    fn xor(&mut self, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Xor,
            vec![lhs, rhs],
        )
    }

    fn neg(&mut self, v: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Neg,
            vec![v],
        )
    }

    fn fneg(&mut self, v: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::fNeg,
            vec![v],
        )
    }

    fn not(&mut self, v: Self::Value) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_first(
            self.basic_block,
            InstructionKind::Not,
            vec![v],
        )
    }

    fn checked_binop(
        &mut self,
        oop: OverflowOp,
        ty: Ty<'tcx>,
        lhs: Self::Value,
        rhs: Self::Value,
    ) -> (Self::Value, Self::Value) {
        let (size, signed) = ty.int_size_and_signed(self.tcx);
        let width = size.bits();

        let res = match oop {
            OverflowOp::Sub => {
                self.sub(lhs, rhs)
            }
            OverflowOp::Add => {
                self.add(lhs, rhs)
            }
            OverflowOp::Mul => {
                self.mul(lhs, rhs)
            }
        };

        if !signed {
            match oop {
                OverflowOp::Sub => {
                    let cmp = self.icmp(IntPredicate::IntULT, lhs, rhs);
                    return (res, cmp);
                }
                OverflowOp::Add => {
                    let cmp = self.icmp(IntPredicate::IntULT, res, lhs);
                    return (res, cmp);
                }
                OverflowOp::Mul => {}
            }
        }
        let of = self.module.borrow_mut()
            .add_instruction_ret(self.basic_block,
                                 InstructionKind::OverflowCheck,
                                 vec![Slot::Raw(if signed { 1 } else { 0 })],
                                 Type::Bool);
        (res, of)
    }

    fn from_immediate(&mut self, val: Self::Value) -> Self::Value {
        // TODO maybe extend this?
        val
    }

    fn to_immediate_scalar(&mut self, val: Self::Value, scalar: rustc_abi::Scalar) -> Self::Value {
        // TODO
        val
    }

    fn alloca(&mut self, size: rustc_abi::Size, align: rustc_abi::Align) -> Self::Value {
        self.module.borrow_mut().add_alloca(
            self.basic_block.function(),
            size.bytes_usize(),
            align.bytes_usize(),
        )
    }

    fn alloca_with_ty(&mut self, layout: TyAndLayout<'tcx>) -> Self::Value {
        todo!()
    }

    fn load(&mut self, ty: Self::Type, ptr: Self::Value, align: rustc_abi::Align) -> Self::Value {
        match ty {
            FullType::Single(ret_ty) => {
                self.load_single(
                    &mut self.module.borrow_mut(),
                    ptr,
                    align,
                    ret_ty
                )
            }
            FullType::Pair(ty_a, ty_b, offset) => {
                let module = &mut self.module.borrow_mut();
                let (slot_a, slot_b) = self.load_pair(
                    module,
                    ptr,
                    offset,
                    align,
                    ty_a,
                    ty_b
                );
                module.add_pair(slot_a, slot_b, offset)
            }
            FullType::Memory { sized } => unimplemented!(),
        }
    }

    fn volatile_load(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        align: rustc_abi::Align,
    ) -> Self::Value {
        todo!()
    }

    fn atomic_load(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        order: AtomicOrdering,
        volatile: bool,
        size: rustc_abi::Size,
    ) -> Self::Value {
        let FullType::Single(ty) = ty else { todo!() };

        self.module.borrow_mut().add_instruction_ret(
            self.basic_block,
            InstructionKind::Atomic_load,
            vec![
                ptr,
                Slot::new_raw(convert_atomic_order(order).repr),
                Slot::new_raw(volatile as u32),
            ],
            ty,
        )
    }

    fn load_operand(
        &mut self,
        place: PlaceRef<'tcx, Self::Value>,
    ) -> OperandRef<'tcx, Self::Value> {
        let val = match self.cx.tpde_direct_type(place.layout) {
            FullType::Single(ret_ty) => {
                let slot = self.load_single(
                    &mut self.module.borrow_mut(),
                    place.val.llval,
                    place.val.align,
                    ret_ty
                );
                OperandValue::Immediate(slot)
            }
            FullType::Pair(ty_a, ty_b, offset) => {
                let (slot_a, slot_b) = self.load_pair(
                    &mut self.module.borrow_mut(),
                    place.val.llval,
                    offset,
                    place.val.align,
                    ty_a,
                    ty_b
                );

                OperandValue::Pair(slot_a, slot_b)
            }
            FullType::Memory { sized } => OperandValue::Ref(place.val),
        };
        OperandRef {
            val,
            layout: place.layout,
            move_annotation: None,
        }
    }

    fn write_operand_repeatedly(
        &mut self,
        elem: OperandRef<'tcx, Self::Value>,
        count: u64,
        dest: PlaceRef<'tcx, Self::Value>,
    ) {
        todo!()
    }

    fn range_metadata(&mut self, load: Self::Value, range: rustc_abi::WrappingRange) {
    }

    fn nonnull_metadata(&mut self, load: Self::Value) {
    }

    fn store(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        align: rustc_abi::Align,
    ) -> Self::Value {
        let module = &mut self.module.borrow_mut();
        match module.type_of_slot(val) {
            FullType::Single(ty) => {
                self.store_single(module, val, ptr, align);
            }
            FullType::Pair(..) => {
                let (val_a, val_b, offset) = module.extract_vals(val);
                self.store_pair(module, val_a, val_b, ptr, offset, align);
            }
            _ => todo!()
        };
        val
    }

    fn store_with_flags(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        align: rustc_abi::Align,
        flags: MemFlags,
    ) -> Self::Value {
        self.store(val, ptr, align)
    }

    fn atomic_store(
        &mut self,
        val: Self::Value,
        ptr: Self::Value,
        order: AtomicOrdering,
        volatile: bool,
        size: rustc_abi::Size,
    ) {
        let module = &mut self.module.borrow_mut();
        let FullType::Single(ty) = module.type_of_slot(val) else { todo!() };

        module.add_instruction(
            self.basic_block,
            InstructionKind::Atomic_store,
            vec![
                val,
                ptr,
                Slot::new_raw(convert_atomic_order(order).repr),
                Slot::new_raw(volatile as u32),
            ],
        );
    }

    fn gep(&mut self, ty: Self::Type, ptr: Self::Value, indices: &[Self::Value]) -> Self::Value {
        let offset = match ty {
            FullType::Single(ty) => size_of_type(ty),
            FullType::Pair(_, _, offset) => offset as u32,
            FullType::Memory { sized } => 1,
        };
        assert_eq!(indices.len(), 1);

        self.module.borrow_mut().add_instruction_ret(
            self.basic_block,
            InstructionKind::GEP,
            vec![ptr, Slot::new_raw(offset), indices[0]],
            Type::i64,
        )
    }

    fn inbounds_gep(
        &mut self,
        ty: Self::Type,
        ptr: Self::Value,
        indices: &[Self::Value],
    ) -> Self::Value {
        self.gep(ty, ptr, indices)
    }

    fn trunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::Trunc, val, dest_ty)
    }

    fn zext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        if self.int_width(self.val_ty(val)) == self.int_width(dest_ty) {
            return self.bitcast(val, dest_ty);
        }
        self.unop(InstructionKind::zExt, val, dest_ty)
    }

    fn sext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::sExt, val, dest_ty)
    }

    fn fptoui_sat(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::fTou_sat, val, dest_ty)
    }

    fn fptosi_sat(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::fTos_sat, val, dest_ty)
    }

    fn fptoui(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::fTou, val, dest_ty)
    }

    fn fptosi(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::fTos, val, dest_ty)
    }

    fn uitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::uTof, val, dest_ty)
    }

    fn sitofp(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::sTof, val, dest_ty)
    }

    fn fptrunc(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::fTrunc, val, dest_ty)
    }

    fn fpext(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::fExt, val, dest_ty)
    }

    fn ptrtoint(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.bitcast(val, dest_ty)
    }

    fn inttoptr(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.bitcast(val, dest_ty)
    }

    fn bitcast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.unop(InstructionKind::Cast, val, dest_ty)
    }

    fn intcast(&mut self, val: Self::Value, dest_ty: Self::Type, is_signed: bool) -> Self::Value {
        let src_ty = self.val_ty(val);
        let src_width = self.int_width(src_ty);
        let dest_width = self.int_width(dest_ty);

        if src_ty == dest_ty {
            val
        } else if src_width == dest_width {
            self.bitcast(val, dest_ty)
        } else if src_width > dest_width {
            self.trunc(val, dest_ty)
        } else if is_signed {
            self.sext(val, dest_ty)
        } else {
            self.zext(val, dest_ty)
        }
    }

    fn pointercast(&mut self, val: Self::Value, dest_ty: Self::Type) -> Self::Value {
        self.bitcast(val, dest_ty)
    }

    fn icmp(&mut self, op: IntPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = match op {
            IntPredicate::IntEQ => InstructionKind::CMPeq,
            IntPredicate::IntNE => InstructionKind::CMPne,
            IntPredicate::IntUGT => InstructionKind::CMPugt,
            IntPredicate::IntUGE => InstructionKind::CMPuge,
            IntPredicate::IntULT => InstructionKind::CMPult,
            IntPredicate::IntULE => InstructionKind::CMPule,
            IntPredicate::IntSGT => InstructionKind::CMPsgt,
            IntPredicate::IntSGE => InstructionKind::CMPsge,
            IntPredicate::IntSLT => InstructionKind::CMPslt,
            IntPredicate::IntSLE => InstructionKind::CMPsle,
        };

        self.module.borrow_mut().add_instruction_ret(
            self.basic_block,
            instr,
            vec![lhs, rhs],
            Type::Bool,
        )
    }

    fn fcmp(&mut self, op: RealPredicate, lhs: Self::Value, rhs: Self::Value) -> Self::Value {
        let instr = match op {
            RealPredicate::RealPredicateFalse => return self.const_bool(false),
            RealPredicate::RealOEQ => InstructionKind::RealOEQ,
            RealPredicate::RealOGT => InstructionKind::RealOGT,
            RealPredicate::RealOGE => InstructionKind::RealOGE,
            RealPredicate::RealOLT => InstructionKind::RealOLT,
            RealPredicate::RealOLE => InstructionKind::RealOLE,
            RealPredicate::RealONE => InstructionKind::RealONE,
            RealPredicate::RealORD => InstructionKind::RealORD,
            RealPredicate::RealUNO => InstructionKind::RealUNO,
            RealPredicate::RealUEQ => InstructionKind::RealUEQ,
            RealPredicate::RealUGT => InstructionKind::RealUGT,
            RealPredicate::RealUGE => InstructionKind::RealUGE,
            RealPredicate::RealULT => InstructionKind::RealULT,
            RealPredicate::RealULE => InstructionKind::RealULE,
            RealPredicate::RealUNE => InstructionKind::RealUNE,
            RealPredicate::RealPredicateTrue => return self.const_bool(true),
        };

        self.module.borrow_mut().add_instruction_ret(
            self.basic_block,
            instr,
            vec![lhs, rhs],
            Type::Bool,
        )
    }

    fn memcpy(
        &mut self,
        dst: Self::Value,
        dst_align: rustc_abi::Align,
        src: Self::Value,
        src_align: rustc_abi::Align,
        size: Self::Value,
        flags: MemFlags,
        tt: Option<FncTree>,
    ) {
        self.module.borrow_mut().add_instruction(
            self.basic_block,
            InstructionKind::MemCpy,
            vec![
                dst,
                Slot::new_raw(dst_align.bytes_usize() as u32),
                src,
                Slot::new_raw(src_align.bytes_usize() as u32),
                size,
            ],
        );
    }

    fn memmove(
        &mut self,
        dst: Self::Value,
        dst_align: rustc_abi::Align,
        src: Self::Value,
        src_align: rustc_abi::Align,
        size: Self::Value,
        flags: MemFlags,
    ) {
        self.module.borrow_mut().add_instruction(
            self.basic_block,
            InstructionKind::MemMove,
            vec![
                dst,
                Slot::new_raw(dst_align.bytes_usize() as u32),
                src,
                Slot::new_raw(src_align.bytes_usize() as u32),
                size,
            ],
        );
    }

    fn memset(
        &mut self,
        ptr: Self::Value,
        fill_byte: Self::Value,
        size: Self::Value,
        align: rustc_abi::Align,
        flags: MemFlags,
    ) {
        self.module.borrow_mut().add_instruction(
            self.basic_block,
            InstructionKind::MemSet,
            vec![
                ptr,
                fill_byte,
                size,
                Slot::new_raw(align.bytes_usize() as u32),
            ],
        );
    }

    fn vscale(&mut self, ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn select(
        &mut self,
        cond: Self::Value,
        then_val: Self::Value,
        else_val: Self::Value,
    ) -> Self::Value {
        self.module.borrow_mut().add_instruction_ret_x(
            self.basic_block,
            InstructionKind::Select,
            vec![cond, then_val, else_val],
            1
        )
    }

    fn va_arg(&mut self, list: Self::Value, ty: Self::Type) -> Self::Value {
        todo!()
    }

    fn extract_element(&mut self, vec: Self::Value, idx: Self::Value) -> Self::Value {
        todo!()
    }

    fn vector_splat(&mut self, num_elts: usize, elt: Self::Value) -> Self::Value {
        todo!()
    }

    fn extract_value(&mut self, agg_val: Self::Value, idx: u64) -> Self::Value {
        let (slot_a, slot_b, _) = self.module.borrow_mut().extract_vals(agg_val);
        match idx {
            0 => slot_a,
            1 => slot_b,
            _ => panic!("pairs only support index 0 or 1"),
        }
    }

    fn insert_value(&mut self, agg_val: Self::Value, elt: Self::Value, idx: u64) -> Self::Value {
        let module = &mut self.module.borrow_mut();
        let (slot_a, slot_b, offset_b) = module.extract_vals(agg_val);

        let func = self.basic_block.function();

        match idx {
            0 => module.add_pair(elt, slot_b, offset_b),
            1 => module.add_pair(slot_a, elt, offset_b),
            _ => panic!("pairs only support index 0 or 1"),
        }
    }

    fn set_personality_fn(&mut self, personality: Self::Function) {
        self.module.borrow_mut().add_personality(self.basic_block.function(), personality);
    }

    fn cleanup_landing_pad(&mut self, pers_fn: Self::Function) -> (Self::Value, Self::Value) {
        self.set_personality_fn(pers_fn);
        let module = &mut self.module.borrow_mut();
        let a = module.add_instruction_ret(
            self.basic_block,
            InstructionKind::LandingPad,
            vec![],
            Type::ptr
        );
        let b = module.add_instruction_ret(
            self.basic_block,
            InstructionKind::AddRet,
            vec![],
            Type::i32
        );
        (a, b)
    }

    fn filter_landing_pad(&mut self, pers_fn: Self::Function) {
        self.cleanup_landing_pad(pers_fn);
    }

    fn resume(&mut self, exn0: Self::Value, exn1: Self::Value) {
        self.module.borrow_mut().add_instruction(
            self.basic_block,
            InstructionKind::Resume,
            vec![exn0]
        );
    }

    fn cleanup_pad(&mut self, parent: Option<Self::Value>, args: &[Self::Value]) -> Self::Funclet {
        unimplemented!("Only for windows")
    }

    fn cleanup_ret(&mut self, funclet: &Self::Funclet, unwind: Option<Self::BasicBlock>) {
        unimplemented!("Only for windows")
    }

    fn catch_pad(&mut self, parent: Self::Value, args: &[Self::Value]) -> Self::Funclet {
        unimplemented!("Only for windows")
    }

    fn catch_switch(
        &mut self,
        parent: Option<Self::Value>,
        unwind: Option<Self::BasicBlock>,
        handlers: &[Self::BasicBlock],
    ) -> Self::Value {
        unimplemented!("Windows not supported")
    }

    fn get_funclet_cleanuppad(&self, funclet: &Self::Funclet) -> Self::Value {
        unimplemented!("Windows not supported")
    }

    fn atomic_cmpxchg(
        &mut self,
        dst: Self::Value,
        cmp: Self::Value,
        src: Self::Value,
        order: AtomicOrdering,
        failure_order: AtomicOrdering,
        weak: bool,
    ) -> (Self::Value, Self::Value) {
        use rustc_middle::ty::AtomicOrdering::*;
        let merged_order: ir::AtomicOrdering = match (order, failure_order) {
            (SeqCst, _) | (_, SeqCst) => ir::AtomicOrdering::SequentiallyConsistent,
            (AcqRel, _) | (Release, Acquire) => ir::AtomicOrdering::AcquireRelease,
            (Acquire, _) | (_, Acquire) => ir::AtomicOrdering::Acquire,
            (Release, Relaxed) => ir::AtomicOrdering::Release,
            (Relaxed, Relaxed) => ir::AtomicOrdering::Monotonic,
            _ => unreachable!(),
        };

        let mut module = self.module.borrow_mut();
        let ret_val = module.add_instruction_ret_x(
            self.basic_block,
            InstructionKind::Atomic_cmpxchg,
            vec![dst, cmp, src, Slot::new_raw(merged_order.repr)],
            2
        );
        let res = module.add_instruction_ret(
            self.basic_block,
            InstructionKind::AddRet,
            vec![],
            Type::Bool
        );
        (ret_val, res)
    }

    fn atomic_rmw(
        &mut self,
        op: AtomicRmwBinOp,
        dst: Self::Value,
        src: Self::Value,
        order: AtomicOrdering,
        ret_ptr: bool,
    ) -> Self::Value {
        let op = convert_atomic_op(op);
        let order = convert_atomic_order(order);
        let ret_val = self.module.borrow_mut().add_instruction_ret_x(
            self.basic_block,
            InstructionKind::Atomic_rmw,
            vec![Slot::new_raw(op.repr), dst, src, Slot::new_raw(order.repr)],
            2
        );

        ret_val
    }

    fn atomic_fence(&mut self, order: AtomicOrdering, scope: SynchronizationScope) {
        self.module.borrow_mut().add_instruction(
            self.basic_block,
            InstructionKind::Atomic_fence,
            vec![
                Slot::new_raw(convert_atomic_order(order).repr),
                Slot::new_raw(match scope {
                    SynchronizationScope::SingleThread => 0,
                    SynchronizationScope::CrossThread => 1,
                } as u32)
            ],
        )
    }

    fn set_invariant_load(&mut self, load: Self::Value) {
    }

    fn lifetime_start(&mut self, ptr: Self::Value, size: rustc_abi::Size) {
    }

    fn lifetime_end(&mut self, ptr: Self::Value, size: rustc_abi::Size) {
    }

    fn call(
        &mut self,
        func_sig: Self::FunctionSignature,
        caller_attrs: Option<&CodegenFnAttrs>,
        fn_abi: Option<&rustc_target::callconv::FnAbi<'tcx, Ty<'tcx>>>,
        fn_val: Self::Value,
        args: &[Self::Value],
        funclet: Option<&Self::Funclet>,
        callee_instance: Option<Instance<'tcx>>,
    ) -> Self::Value {
        let func_sig = &self.function_signatures.borrow()[func_sig];

        self.module
            .borrow_mut()
            .add_call(self.basic_block, fn_val, func_sig, args)
            .unwrap_or_else(|| Slot::new_raw(0))
    }

    fn tail_call(
        &mut self,
        llty: Self::FunctionSignature,
        caller_attrs: Option<&CodegenFnAttrs>,
        fn_abi: &rustc_target::callconv::FnAbi<'tcx, Ty<'tcx>>,
        llfn: Self::Value,
        args: &[Self::Value],
        funclet: Option<&Self::Funclet>,
        callee_instance: Option<Instance<'tcx>>,
    ) {
        unimplemented!()
    }

    fn apply_attrs_to_cleanup_callsite(&mut self, llret: Self::Value) {
    }
}

impl<'a, 'tpde, 'tcx> Builder<'a, 'tpde, 'tcx> {
    fn unop(&self, instr: InstructionKind, val: Slot, dest_ty: FullType) -> Slot {
        let FullType::Single(dest_ty) = dest_ty else {
            todo!()
        };
        self.module.borrow_mut().add_instruction_ret(
            self.basic_block,
            instr,
            vec![val],
            dest_ty,
        )
    }

    fn load_single(&self, module: &mut Module, ptr: Slot, align: rustc_abi::Align, ret_ty: Type) -> Slot {
        module.add_instruction_ret(
            self.basic_block,
            InstructionKind::Load,
            vec![
                ptr,
                Slot::new_raw(align.bytes() as u32),
            ],
            ret_ty,
        )
    }

    fn store_single(&self, module: &mut Module, val: Slot, ptr: Slot, align: rustc_abi::Align) {
        module.add_instruction(
            self.basic_block,
            InstructionKind::Store,
            vec![val, ptr, Slot::new_raw(align.bytes_usize() as u32)],
        )
    }

    fn load_pair(&self, module: &mut Module, ptr: Slot, offset: u32, align: rustc_abi::Align, ty_a: Type, ty_b: Type) -> (Slot, Slot) {
        let slot_a = self.load_single(module, ptr, align, ty_a);
        let ind = module.add_const(Type::i64, 1);
        let ptr_b = module.add_instruction_ret(
            self.basic_block,
            InstructionKind::GEP,
            vec![ptr, Slot::new_raw(offset), ind],
            Type::i64,
        );
        let slot_b = self.load_single(module, ptr_b, align, ty_b);
        (slot_a, slot_b)
    }

    fn store_pair(&self, module: &mut Module, val_a: Slot, val_b: Slot, ptr: Slot, offset: u32, align: rustc_abi::Align) {
        self.store_single(module, val_a, ptr, align);
        let ind = module.add_const(Type::i64, 1);
        let ptr_b = module.add_instruction_ret(
            self.basic_block,
            InstructionKind::GEP,
            vec![ptr, Slot::new_raw(offset), ind],
            Type::i64,
        );
        self.store_single(module, val_b, ptr_b, align);
    }
}
