pub use super::ffi::ModuleTpde;
use super::{ffi, ir};
use crate::shared::ffi::{CalleeInfo, GlobalPtr};
use crate::shared::optimize::OptConfig;
use core::fmt::{Debug, Formatter};
use rustc_attr_ir::Linkage;
use rustc_codegen_ssa::traits::ReturnSlot;
use rustc_middle::mir::Mutability;
use std::ops::Range;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Function(pub(super) usize);
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Global(pub(super) usize);
#[derive(Debug, Copy, Clone)]
pub struct BasicBlock {
    pub(super) function: Function,
    pub(super) index: usize,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Value {
    Slot(Function, u32),
    Pair(u32),
    ConstVector(u32),
    Const(u32),
    Raw(u32),
    Alloc(u32),
    Func(Function),
    Global(Global),
    GlobalPtr(u32),
    Phi(Function, u32, u32),
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FullType {
    Single(Type),
    Pair(Type, Type, u32),
    Memory { sized: bool, size: u32 },
}

pub struct FunctionSignatureRef(usize);
#[derive(Clone)]
pub struct FunctionSignature {
    pub slots: Vec<Type>,
    pub arg_infos: Vec<ArgInfo>,
    pub ret: Option<FullType>,
}

pub fn size_of_type(ty: Type) -> u32 {
    match ty {
        Type::Void => 0,
        Type::Bool | Type::i8 => 1,
        Type::i16 => 2,
        Type::i32 => 4,
        Type::i64 => 8,
        Type::i128 => 16,
        Type::f32 => 4,
        Type::f64 => 8,
        Type::ptr => 8,
        ty => match vector_info(ty) {
            Some((elem, count)) => size_of_type(elem) * count,
            None => todo!(),
        },
    }
}

/// Decompose a vector type into its element type and element count.
pub fn vector_info(ty: Type) -> Option<(Type, u32)> {
    Some(match ty {
        // Vectors narrower than 64 bit
        Type::v2i8 => (Type::i8, 2),
        Type::v4i8 => (Type::i8, 4),
        Type::v2i16 => (Type::i16, 2),

        // 64 bit vectors
        Type::v8i8 => (Type::i8, 8),
        Type::v4i16 => (Type::i16, 4),
        Type::v2i32 => (Type::i32, 2),
        Type::v2f32 => (Type::f32, 2),

        // 128 bit vectors
        Type::v16i8 => (Type::i8, 16),
        Type::v8i16 => (Type::i16, 8),
        Type::v4i32 => (Type::i32, 4),
        Type::v2i64 => (Type::i64, 2),
        Type::v4f32 => (Type::f32, 4),
        Type::v2f64 => (Type::f64, 2),

        // 256 bit vectors
        Type::v32i8 => (Type::i8, 32),
        Type::v16i16 => (Type::i16, 16),
        Type::v8i32 => (Type::i32, 8),
        Type::v4i64 => (Type::i64, 4),
        Type::v8f32 => (Type::f32, 8),
        Type::v4f64 => (Type::f64, 4),

        // 512 bit vectors
        Type::v64i8 => (Type::i8, 64),
        Type::v32i16 => (Type::i16, 32),
        Type::v16i32 => (Type::i32, 16),
        Type::v8i64 => (Type::i64, 8),
        Type::v16f32 => (Type::f32, 16),
        Type::v8f64 => (Type::f64, 8),

        // 1024 bit vectors
        Type::v128i8 => (Type::i8, 128),
        Type::v64i16 => (Type::i16, 64),
        Type::v32i32 => (Type::i32, 32),
        Type::v16i64 => (Type::i64, 16),
        Type::v32f32 => (Type::f32, 32),
        Type::v16f64 => (Type::f64, 16),

        // 2048 bit vectors
        Type::v64i32 => (Type::i32, 64),
        Type::v32i64 => (Type::i64, 32),

        // 4096 bit vectors
        Type::v64i64 => (Type::i64, 64),
        _ => return None,
    })
}

/// Vector type with the given element type and element count, if it is supported.
pub fn vector_type(elem: Type, count: u64) -> Option<Type> {
    Some(match (elem, count) {
        // Vectors narrower than 64 bit
        (Type::i8, 2) => Type::v2i8,
        (Type::i8, 4) => Type::v4i8,
        (Type::i16, 2) => Type::v2i16,

        // 64 bit vectors
        (Type::i8, 8) => Type::v8i8,
        (Type::i16, 4) => Type::v4i16,
        (Type::i32, 2) => Type::v2i32,
        (Type::f32, 2) => Type::v2f32,

        // 128 bit vectors
        (Type::i8, 16) => Type::v16i8,
        (Type::i16, 8) => Type::v8i16,
        (Type::i32, 4) => Type::v4i32,
        (Type::i64, 2) => Type::v2i64,
        (Type::f32, 4) => Type::v4f32,
        (Type::f64, 2) => Type::v2f64,

        // 256 bit vectors
        (Type::i8, 32) => Type::v32i8,
        (Type::i16, 16) => Type::v16i16,
        (Type::i32, 8) => Type::v8i32,
        (Type::i64, 4) => Type::v4i64,
        (Type::f32, 8) => Type::v8f32,
        (Type::f64, 4) => Type::v4f64,

        // 512 bit vectors
        (Type::i8, 64) => Type::v64i8,
        (Type::i16, 32) => Type::v32i16,
        (Type::i32, 16) => Type::v16i32,
        (Type::i64, 8) => Type::v8i64,
        (Type::f32, 16) => Type::v16f32,
        (Type::f64, 8) => Type::v8f64,

        // 1024 bit vectors
        (Type::i8, 128) => Type::v128i8,
        (Type::i16, 64) => Type::v64i16,
        (Type::i32, 32) => Type::v32i32,
        (Type::i64, 16) => Type::v16i64,
        (Type::f32, 32) => Type::v32f32,
        (Type::f64, 16) => Type::v16f64,

        // 2048 bit vectors
        (Type::i32, 64) => Type::v64i32,
        (Type::i64, 32) => Type::v32i64,

        // 4096 bit vectors
        (Type::i64, 64) => Type::v64i64,
        _ => return None,
    })
}

pub use super::ffi::{
    AccessInfo, AccessInfoState, ArgExtension, ArgInfo, ArgKind, AtomicOrdering, AtomicRmwBinOp,
    InstructionKind, LandingPadKind, Type,
};

pub struct Module {
    tpde: ModuleTpde,

    pairs: Vec<PairRef>,
    opt: OptConfig,

    defs_dso_local: bool,
}

/// We inline smaller constants to avoid overusing the constant pool with small values
pub const INLINE_CONST_BIT: u32 = 1 << 28;
pub const CONST_CONTENT: u32 = u32::MAX >> 4;
pub const INLINE_CONST_TYPE_BITS_COUNT: u32 = u32::BITS - (Type::Last.repr as u32).leading_zeros();
pub const INLINE_CONST_VAL_BITS_COUNT: u32 = u32::BITS - 4 - INLINE_CONST_TYPE_BITS_COUNT;
pub const INLINE_CONST_VAL_MASK: u32 = CONST_CONTENT >> INLINE_CONST_TYPE_BITS_COUNT;
pub const INLINE_CONST_LIMIT: u128 = INLINE_CONST_VAL_MASK as u128;

#[derive(PartialEq)]
pub enum Binding {
    Declaration,
    Definition,
}

#[derive(Copy, Clone)]
pub struct PairRef {
    slot_a: Value,
    slot_b: Value,
    offset_b: u32,
}

impl Module {
    pub fn new(opt: OptConfig, defs_dso_local: bool) -> Self {
        Self {
            tpde: ModuleTpde {
                functions: vec![],
                consts: vec![],

                globals: vec![],
                global_ptrs: vec![],
                const_vectors: vec![],
            },
            pairs: vec![],
            opt,
            defs_dso_local,
        }
    }

    pub fn tpde_mut(&mut self) -> &mut ModuleTpde {
        &mut self.tpde
    }

    /// Runs the optimizations selected by the optimization level
    pub fn optimize(&mut self) {
        self.tpde.inline_functions(self.opt);
        if self.opt.promote {
            self.tpde.promote_allocas();
        }
    }

    /// Whether the access infos needed by the promotion have to be collected
    pub fn tracks_access(&self) -> bool {
        self.opt.promote
    }

    // Approximates the size of the IR
    pub fn byte_size(&self) -> usize {
        fn bytes<T>(v: &Vec<T>) -> usize {
            v.capacity() * size_of::<T>()
        }

        let tpde = &self.tpde;
        let functions: usize = tpde
            .functions
            .iter()
            .map(|f| {
                f.name.capacity()
                    + bytes(&f.args)
                    + bytes(&f.slots)
                    + bytes(&f.phis)
                    + bytes(&f.operands)
                    + bytes(&f.instructions)
                    + bytes(&f.allocas)
                    + bytes(&f.basic_blocks)
                    + bytes(&f.successors)
                    + bytes(&f.callee_infos)
                    + bytes(&f.called_by)
                    + f.callee_infos.iter().map(|c| bytes(&c.info)).sum::<usize>()
            })
            .sum();
        let globals: usize = tpde
            .globals
            .iter()
            .map(|g| g.name.capacity() + g.data.capacity() + bytes(&g.relocations))
            .sum();
        let const_vectors: usize = tpde.const_vectors.iter().map(|c| c.data.capacity()).sum();

        size_of::<Self>()
            + bytes(&tpde.functions)
            + bytes(&tpde.consts)
            + bytes(&tpde.globals)
            + bytes(&tpde.global_ptrs)
            + bytes(&tpde.const_vectors)
            + bytes(&self.pairs)
            + functions
            + globals
            + const_vectors
    }

    pub fn add_function<'tpde, 'tcx>(
        &mut self,
        name: &str,
        fn_sign: &FunctionSignature,
        linkage: Linkage,
        ty: Binding,
    ) -> Function {
        let FunctionSignature {
            slots,
            arg_infos,
            ret,
        } = fn_sign;
        self.tpde.functions.push(ffi::Function {
            name: name.to_string(),
            has_ret: ret.is_some(),
            args: arg_infos.clone(),
            has_personality: false,
            personality: 0,
            slots: slots.iter().map(|ty| ffi::Slot { ty: *ty }).collect(),
            phis: vec![],
            calling: 0,
            called_by: vec![],
            cost: 0,
            flags: Self::create_flags(linkage, ty, self.defs_dso_local),
            operands: vec![],
            instructions: vec![],
            allocas: vec![],
            basic_blocks: vec![],
            successors: vec![],
            callee_infos: vec![],
        });
        Function(self.tpde.functions.len() - 1)
    }

    /// Avoid constant resizing by already reserving the given stimations
    pub fn reserve_function(
        &mut self,
        func: Function,
        blocks: usize,
        insts: usize,
        slots: usize,
        operands: usize,
    ) {
        let function = &mut self.tpde.functions[func.0];
        function.basic_blocks.reserve(blocks);
        function.successors.reserve(blocks);
        function.instructions.reserve(insts);
        function.slots.reserve(slots);
        function.operands.reserve(operands);
    }

    pub fn set_always_inline(&mut self, func: Function) {
        self.tpde.functions[func.0].flags.always_inline = true;
    }

    pub fn set_never_inline(&mut self, func: Function) {
        self.tpde.functions[func.0].flags.never_delete = true;
    }

    pub fn return_value(&mut self, func: Function) -> Option<Value> {
        if self.tpde.functions[func.0].has_ret {
            Some(Value::new_val(func, 0))
        } else {
            None
        }
    }

    pub fn add_personality(&mut self, func: Function, personality: Function) {
        self.tpde.functions[func.0].has_personality = true;
        self.tpde.functions[func.0].personality = personality.0 as u32;
        self.tpde.functions[personality.0].flags.never_delete = true;
    }

    pub fn add_global(
        &mut self,
        name: &str,
        linkage: Linkage,
        mutability: Mutability,
        binding: Binding,
    ) -> Global {
        self.tpde.globals.push(ffi::Global {
            name: name.to_string(),
            size: 0,
            align: 0,
            thread_loc: false,
            read_only: mutability != Mutability::Mut,
            flags: Self::create_flags(linkage, binding, self.defs_dso_local),

            init: false,
            data: vec![],
            relocations: vec![],
        });
        Global(self.tpde.globals.len() - 1)
    }

    pub fn global_set_align(&mut self, global: Global, align: u32) {
        self.tpde.globals[global.0].align = align;
    }

    pub fn global_set_thread_local(&mut self, global: Global) {
        self.tpde.globals[global.0].thread_loc = true;
    }

    pub fn global_add_init_chunk(&mut self, global: Global, chunk: &[u8]) {
        let global = &mut self.tpde.globals[global.0];

        if !global.init {
            global.init = true;
            global.data.resize(global.size as usize, 0);
        }

        global.data.extend_from_slice(chunk);
        global.size = global.data.len() as u32;
    }

    fn global_add_unit_intern(global: &mut ffi::Global, size: usize) {
        global.size += size as u32;

        if global.init {
            global.data.resize(global.size as usize, 0);
        }
    }

    pub fn global_add_unit_chunk(&mut self, global: Global, size: usize) {
        let global = &mut self.tpde.globals[global.0];
        Module::global_add_unit_intern(global, size);
    }

    pub fn global_add_reloc_chunk(
        &mut self,
        global: Global,
        offset: u32,
        slot: Value,
        pointer_size: usize,
    ) {
        let global = &mut self.tpde.globals[global.0];
        if !global.init {
            global.init = true;
            global.data.resize(global.size as usize, 0);
        }
        Module::global_add_unit_intern(global, pointer_size);

        if let Value::Func(func) = slot {
            self.tpde.functions[func.0].flags.never_delete = true;
        }

        global.relocations.push(ffi::Relocation {
            offset,
            slot: slot.to_ffi(),
        });
    }

    pub fn add_global_ptr(&mut self, global: Global, offset: u32) -> Value {
        self.tpde.global_ptrs.push(ffi::GlobalPtr {
            global: global.0 as u32,
            offset,
        });
        Value::GlobalPtr(self.tpde.global_ptrs.len() as u32 - 1)
    }

    pub fn add_to_ptr(&mut self, ptr: Value, off: u32) -> Value {
        match ptr {
            Value::Global(glob) => self.add_global_ptr(glob, off),
            Value::GlobalPtr(i) => {
                let GlobalPtr { global, offset } = self.tpde.global_ptrs[i as usize];
                self.add_global_ptr(Global(global as usize), offset + off)
            }
            _ => todo!(),
        }
    }

    fn create_flags(linkage: Linkage, ty: Binding, defs_dso_local: bool) -> ffi::Flags {
        let extern_link = ty == Binding::Declaration
            || matches!(
                linkage,
                Linkage::AvailableExternally | Linkage::ExternalWeak
            );
        let only_local = linkage == Linkage::Internal;
        ffi::Flags {
            extern_link,
            only_local,
            dso_local: !extern_link && (only_local || defs_dso_local),
            weak_link: linkage == Linkage::WeakODR
                || linkage == Linkage::WeakAny
                || linkage == Linkage::ExternalWeak,
            always_inline: false,
            never_delete: false,
            dead: false,
        }
    }

    pub fn get_slot(&self, func: Function, index: u32) -> Value {
        Value::new_val(func, index)
    }

    pub fn type_of_slot(&self, slot: Value) -> FullType {
        match slot {
            Value::Slot(func, ind) => {
                FullType::Single(self.tpde.functions[func.0].slots[ind as usize].ty)
            }
            Value::Const(c) => FullType::Single(self.const_value(c).0),
            Value::Pair(ind) => {
                let pair = &self.pairs[ind as usize];
                let FullType::Single(slot_a) = self.type_of_slot(pair.slot_a) else {
                    unreachable!()
                };
                let FullType::Single(slot_b) = self.type_of_slot(pair.slot_b) else {
                    unreachable!()
                };
                FullType::Pair(slot_a, slot_b, pair.offset_b)
            }
            Value::ConstVector(ind) => FullType::Single(self.tpde.const_vectors[ind as usize].ty),
            Value::Raw(_) => unreachable!(),
            phi @ Value::Phi(..) => FullType::Single(self.phi(phi).ty),
            Value::Alloc(_) | Value::Func(_) | Value::Global(_) | Value::GlobalPtr(..) => {
                FullType::Single(Type::ptr)
            }
        }
    }

    fn get_function_mut(&mut self, func: &Function) -> &mut ffi::Function {
        self.tpde
            .functions
            .get_mut(func.0)
            .unwrap_or_else(|| panic!("Function not found"))
    }

    fn get_function(&self, func: &Function) -> &ffi::Function {
        self.tpde
            .functions
            .get(func.0)
            .unwrap_or_else(|| panic!("Function not found"))
    }

    pub fn add_basic_block(self: &mut Self, func: &Function) -> BasicBlock {
        let function = self.get_function_mut(func);
        function.basic_blocks.push(ffi::BasicBlock {
            insts_start: 0,
            insts_len: 0,
            phi_start: 0,
            phi_len: 0,
            suc_start: 0,
            suc_len: 0,
            info1: 0,
            info2: 0,
        });
        BasicBlock::new(*func, function.basic_blocks.len() - 1)
    }

    fn get_basic_block_mut_helper(
        func: &mut ffi::Function,
        bb: BasicBlock,
    ) -> &mut ffi::BasicBlock {
        func.basic_blocks
            .get_mut(bb.index())
            .unwrap_or_else(|| panic!("Basic block not found"))
    }

    fn get_basic_block_mut(&mut self, bb: BasicBlock) -> &mut ffi::BasicBlock {
        let function = self.get_function_mut(&bb.function());
        Module::get_basic_block_mut_helper(function, bb)
    }

    fn get_basic_block(&self, bb: BasicBlock) -> &ffi::BasicBlock {
        let function = self.get_function(&bb.function());
        function
            .basic_blocks
            .get(bb.index())
            .unwrap_or_else(|| panic!("Basic block not found"))
    }

    #[inline]
    pub fn add_instruction(&mut self, bb: BasicBlock, instr: InstructionKind, ops: &[Value]) {
        self.add_instruction_raw_internal(bb, instr, ops, None);
    }

    #[inline]
    pub fn add_instruction_ret(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: &[Value],
        ret: Type,
    ) -> Value {
        self.add_instruction_raw_internal(bb, instr, ops, Some(FullType::Single(ret)))
            .unwrap()
    }

    #[inline]
    pub fn add_instruction_ret_x(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: &[Value],
        ret: usize,
    ) -> Value {
        assert!(ops.len() >= 1);

        let func = self.get_function_mut(&bb.function());
        let op = *ops.get(ret).unwrap();

        // create new slot with type of first arg
        let ret = match op {
            Value::Slot(f, v) => {
                assert_eq!(bb.function(), f);
                func.slots.get(v as usize).unwrap().ty
            }
            phi @ Value::Phi(f, ..) => {
                assert_eq!(bb.function(), f);
                self.phi(phi).ty
            }
            Value::Const(v) => self.const_value(v).0,
            _ => panic!("First operand of return instruction must be a value slot"),
        };
        self.add_instruction_raw_internal(bb, instr, ops, Some(FullType::Single(ret)))
            .unwrap()
    }

    #[inline]
    pub fn add_instruction_ret_first(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: &[Value],
    ) -> Value {
        self.add_instruction_ret_x(bb, instr, ops, 0)
    }

    #[inline(always)]
    fn add_instruction_raw_internal(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: &[Value],
        ret: Option<FullType>,
    ) -> Option<Value> {
        enum ReturnType {
            None,
            Single(Value),
            Pair(Value, Value, Value),
        }

        impl ReturnType {
            fn num_ret(&self) -> u8 {
                match self {
                    ReturnType::None => 0,
                    ReturnType::Single(_) => 1,
                    ReturnType::Pair(_, _, _) => 2,
                }
            }

            fn result_a(&self) -> u32 {
                match self {
                    ReturnType::None => 0,
                    ReturnType::Single(slot) => slot.to_ffi(),
                    ReturnType::Pair(_, a, _) => a.to_ffi(),
                }
            }

            fn result_b(&self) -> u32 {
                match self {
                    ReturnType::None => 0,
                    ReturnType::Single(_) => 0,
                    ReturnType::Pair(_, _, b) => b.to_ffi(),
                }
            }

            fn result(&self) -> Option<Value> {
                match self {
                    ReturnType::None => None,
                    ReturnType::Single(slot) => Some(*slot),
                    ReturnType::Pair(slot, _, _) => Some(*slot),
                }
            }
        }

        let ret: ReturnType = match ret {
            None => ReturnType::None,
            Some(FullType::Single(ty)) => ReturnType::Single(self.add_slot(bb.function, ty)),
            Some(FullType::Pair(ty_a, ty_b, offset_b)) => {
                let slot_a = self.add_slot(bb.function, ty_a);
                let slot_b = self.add_slot(bb.function, ty_b);
                let pair = self.add_pair(slot_a, slot_b, offset_b);
                ReturnType::Pair(pair, slot_a, slot_b)
            }
            Some(FullType::Memory { .. }) => {
                ReturnType::Single(self.add_slot(bb.function, Type::ptr))
            }
        };

        // Do not inline a function that's mentioned in any argument other than
        // the call site of a call (not invoke)
        let mut references = ops;
        if instr == InstructionKind::Call
            && let [Value::Func(callee), args @ ..] = ops
        {
            references = args;
            // Extern functions can never be inlined, so they should not block their callers
            if !self.tpde.functions[callee.0].flags.extern_link {
                let caller = &mut self.tpde.functions[bb.function().0];
                caller.calling += 1;
                let instr = caller.basic_blocks[bb.index()].insts_len;
                self.tpde.functions[callee.0]
                    .called_by
                    .push(ffi::InstructionReference {
                        func: bb.function().0 as u32,
                        block: bb.index() as u32,
                        instr,
                    });
            }
        }
        for op in references {
            if let Value::Func(func) = op {
                self.tpde.functions[func.0].flags.never_delete = true;
            }
        }

        let function = &mut self.tpde.functions[bb.function().0];
        function.cost += 1;

        let block = &mut function.basic_blocks[bb.index()];

        // In the default case we write each basic_block in on go
        // Sometimes the ssa generater jumps to other blocks, this happens just rarely
        // In this case we just copy everything of the block written so far to the end
        let end = function.instructions.len() as u32;
        if block.insts_len == 0 {
            block.insts_start = end;
        } else if block.insts_start + block.insts_len != end {
            let start = block.insts_start as usize;
            function
                .instructions
                .extend_from_within(start..start + block.insts_len as usize);
            block.insts_start = end;
        }

        let ops_start = function.operands.len() as u32;
        function.operands.extend(ops.iter().map(|s| s.to_ffi()));

        function.instructions.push(ffi::Instruction {
            kind: instr,
            ops_start,
            ops_len: ops.len() as u32,
            has_result: ret.num_ret() >= 1,
            result: ret.result_a(),
        });
        block.insts_len += 1;

        if ret.num_ret() >= 2 {
            function.instructions.push(ffi::Instruction {
                kind: InstructionKind::AddRet,
                ops_start,
                ops_len: 0,
                has_result: true,
                result: ret.result_b(),
            });
            block.insts_len += 1;
        }

        if self.opt.promote {
            match instr {
                InstructionKind::Load | InstructionKind::Store | InstructionKind::PairGEP => {}
                InstructionKind::Call => {
                    for op in ops {
                        if let Some(info) = access_info_intern(function, op) {
                            info.call_forwards += 1;
                        }
                    }
                }
                _ => {
                    for op in ops {
                        if let Some(info) = access_info_intern(function, op) {
                            info.state = AccessInfoState::Complex;
                        }
                    }
                }
            }
        }

        ret.result()
    }

    #[inline]
    fn set_successors(&mut self, bb: BasicBlock, successors: impl IntoIterator<Item = BasicBlock>) {
        let function = &mut self.tpde.functions[bb.function().0];
        let start = function.successors.len();
        function
            .successors
            .extend(successors.into_iter().map(|s| s.index as u32));
        let block = &mut function.basic_blocks[bb.index()];
        block.suc_start = start as u32;
        block.suc_len = (function.successors.len() - start) as u32;
    }

    pub fn access_info(&mut self, f: Function, ptr: &Value) -> Option<&mut AccessInfo> {
        let func = self.get_function_mut(&f);
        access_info_intern(func, ptr)
    }

    pub fn add_call(
        &mut self,
        bb: BasicBlock,
        func_ref: Value,
        func_sign: &FunctionSignature,
        return_slot: ReturnSlot<Value>,
        args: &[Value],
    ) -> Option<Value> {
        let mut ops = Vec::with_capacity(1 + args.len());
        ops.push(func_ref);
        match func_ref {
            Value::Func(_) => {}
            Value::Slot(..) | Value::Phi(..) => {
                let callee_infos = &mut self.tpde.functions[bb.function.0].callee_infos;
                callee_infos.push(CalleeInfo {
                    info: func_sign.arg_infos.clone(),
                });
                ops.push(Value::new_raw(callee_infos.len() as u32 - 1))
            }
            _ => todo!(),
        }
        if let ReturnSlot::Indirect(sret) = return_slot {
            ops.push(sret);
        }
        for &arg in args {
            match arg {
                Value::Pair(ind) => {
                    let pair = self.pairs[ind as usize];
                    ops.push(pair.slot_a);
                    ops.push(pair.slot_b);
                }
                s => ops.push(s),
            }
        }
        self.add_instruction_raw_internal(bb, InstructionKind::Call, &ops, func_sign.ret)
    }

    pub fn add_invoke(
        &mut self,
        bb: BasicBlock,
        func_ref: Value,
        func_sign: &FunctionSignature,
        then: BasicBlock,
        catch: BasicBlock,
        return_slot: ReturnSlot<Value>,
        args: &[Value],
    ) -> Option<Value> {
        self.set_successors(bb, [then, catch]);

        let mut ops = Vec::with_capacity(2 + args.len());
        ops.push(func_ref);
        match func_ref {
            Value::Func(_) => {}
            Value::Slot(..) | Value::Phi(..) => {
                let callee_infos = &mut self.tpde.functions[bb.function.0].callee_infos;
                callee_infos.push(CalleeInfo {
                    info: func_sign.arg_infos.clone(),
                });
                ops.push(Value::new_raw(callee_infos.len() as u32 - 1))
            }
            _ => todo!(),
        }
        if let ReturnSlot::Indirect(sret) = return_slot {
            ops.push(sret);
        }
        ops.extend_from_slice(args);
        self.add_instruction_raw_internal(bb, InstructionKind::Invoke, &ops, func_sign.ret)
    }

    pub fn add_alloca(&mut self, func: Function, size: usize, align: usize) -> Value {
        let func = self.get_function_mut(&func);

        func.allocas.push(ffi::Alloca {
            size,
            align,
            access_info: AccessInfo::default(),
        });
        Value::new_alloc((func.allocas.len() - 1) as u32)
    }

    pub fn add_const(&mut self, ty: Type, data: u128) -> Value {
        self.tpde.add_const(ty, data)
    }

    /// Wrapper to also convert back inlined constants
    fn const_value(&self, c: u32) -> (Type, u128) {
        self.tpde.const_value(c)
    }

    pub fn const_data(&self, slot: Value) -> Option<u128> {
        self.tpde.const_data(slot)
    }

    pub fn add_const_vector(&mut self, elems: &[Value]) -> Value {
        assert!(!elems.is_empty());
        let FullType::Single(elem_ty) = self.type_of_slot(elems[0]) else {
            unreachable!()
        };
        let Some(ty) = vector_type(elem_ty, elems.len() as u64) else {
            todo!(
                "unsupported constant vector <{} x {:?}>",
                elems.len(),
                elem_ty
            )
        };

        let elem_bytes = size_of_type(elem_ty) as usize;
        let mut data = Vec::with_capacity(elem_bytes * elems.len());
        for elem in elems {
            let value = self
                .const_data(*elem)
                .expect("non-constant element in constant vector");
            data.extend_from_slice(&value.to_le_bytes()[..elem_bytes]);
        }

        self.tpde.const_vectors.push(ffi::ConstVector { ty, data });
        Value::ConstVector((self.tpde.const_vectors.len() - 1) as u32)
    }

    pub fn add_pair(&mut self, slot_a: Value, slot_b: Value, offset_b: u32) -> Value {
        let slot_pairs = &mut self.pairs;
        slot_pairs.push(PairRef {
            slot_a: slot_a,
            slot_b: slot_b,
            offset_b,
        });
        Value::new_pair((slot_pairs.len() - 1) as u32)
    }

    pub fn add_pair_consts(
        &mut self,
        ty_a: Type,
        ty_b: Type,
        offset_b: u32,
        data_a: u128,
        data_b: u128,
    ) -> Value {
        let slot_a = self.add_const(ty_a, data_a);
        let slot_b = self.add_const(ty_b, data_b);
        self.add_pair(slot_a, slot_b, offset_b)
    }

    pub fn extract_vals(&self, pair: Value) -> (Value, Value, u32) {
        let v = match pair {
            Value::Pair(ind) => self.pairs[ind as usize],
            _ => panic!("agg_val has to be a pair"),
        };
        (v.slot_a, v.slot_b, v.offset_b)
    }

    #[inline]
    pub fn add_slot(&mut self, func: Function, ty: Type) -> Value {
        let slots = &mut self.tpde.functions[func.0].slots;
        slots.push(ffi::Slot { ty });
        Value::new_val(func, slots.len() as u32 - 1)
    }

    #[inline(always)]
    fn phi(&self, phi: Value) -> &ffi::Phi {
        let Value::Phi(func, block, ind) = phi else {
            panic!("Not a phi: {:?}", phi)
        };
        let func = &self.tpde.functions[func.0];
        func.phi(block, ind)
    }

    /// Reserve space for all the phis, later call set_phi_incoming to set the incoming values
    /// All phis of a basic block have to be added at once (or we need more movement later)
    pub fn add_phi(&mut self, bb: BasicBlock, ty: Type, incoming_len: u32) -> Value {
        let function = &mut self.tpde.functions[bb.function().0];
        function.add_phi(bb, ty, incoming_len)
    }

    pub fn set_phi_incoming(&mut self, phi: Value, slot: u32, from: BasicBlock, value: Value) {
        let Value::Phi(func, ..) = phi else {
            panic!("Not a phi: {:?}", phi)
        };
        let function = &mut self.tpde.functions[func.0];

        function.set_phi_incoming(phi, slot, from, value.to_ffi());
        if self.opt.promote
            && let Some(info) = access_info_intern(function, &value)
        {
            info.state = AccessInfoState::Complex;
        }
    }

    pub fn add_br(&mut self, bb: BasicBlock, to: BasicBlock) {
        self.set_successors(bb, [to]);
        self.add_instruction(bb, InstructionKind::Br, &[]);
    }

    pub fn add_switch(
        &mut self,
        bb: BasicBlock,
        val: Value,
        else_block: BasicBlock,
        cases: &[(u128, BasicBlock)],
    ) {
        let mut ops = Vec::with_capacity(cases.len() + 2);
        ops.push(val);

        let FullType::Single(ty) = self.type_of_slot(val) else {
            todo!()
        };

        assert!(cases.len() <= 200000, "Too many cases: {}", cases.len());

        self.set_successors(
            bb,
            std::iter::once(else_block).chain(cases.iter().map(|&(_, target)| target)),
        );
        for (val, _) in cases {
            assert!(
                *val <= u64::MAX as u128,
                "switch case does not fit in 64 bit"
            );
            let case_val = self.add_const(ty, *val);
            ops.push(case_val);
        }
        self.add_instruction(bb, InstructionKind::Switch, &ops);
    }

    pub fn add_cond_br(
        &mut self,
        bb: BasicBlock,
        cond: Value,
        thenbb: BasicBlock,
        elsebb: BasicBlock,
    ) {
        self.set_successors(bb, [thenbb, elsebb]);
        self.add_instruction(bb, InstructionKind::CondBr, &[cond]);
    }
}

impl ffi::Function {
    pub(super) fn add_phi(&mut self, bb: BasicBlock, ty: Type, incoming_len: u32) -> Value {
        assert!(
            bb.index() < 1 << PHI_BLOCK_BITS,
            "Too many blocks a single phi"
        );
        let block = &mut self.basic_blocks[bb.index()];
        assert!(
            block.phi_len < 1 << PHI_POS_BITS,
            "Too many phis in a block"
        );

        {
            // Make sure there's enough space for the new phi
            let phi_end = self.phis.len() as u32;
            if block.phi_len == 0 {
                block.phi_start = phi_end;
            } else if block.phi_start + block.phi_len != phi_end {
                // move phis to the end if there's not enough space at the current spot
                let start = block.phi_start as usize;
                block.phi_start = phi_end;
                let len = block.phi_len as usize;
                self.phis.extend_from_within(start..start + len);
            }
        }

        // Insert the new phi
        let index = block.phi_len;
        block.phi_len += 1;

        let incoming_start = self.operands.len() as u32;
        self.operands.extend(std::iter::repeat_n(
            Value::new_raw(0).to_ffi(),
            incoming_len as usize * 2,
        ));
        self.phis.push(ffi::Phi {
            ty,
            incoming_start,
            incoming_len,
        });
        Value::Phi(bb.function(), bb.index() as u32, index)
    }

    fn phi(&self, block: u32, ind: u32) -> &ffi::Phi {
        let start = self.basic_blocks[block as usize].phi_start;
        &self.phis[(start + ind) as usize]
    }

    pub(super) fn set_phi_incoming(&mut self, phi: Value, slot: u32, from: BasicBlock, value: u32) {
        let Value::Phi(func, block, index) = phi else {
            panic!("Not a phi: {:?}", phi)
        };
        let phi = self.phi(block, index);
        assert!(slot < phi.incoming_len);
        let start = (phi.incoming_start + slot * 2) as usize;
        self.operands[start] = Value::new_raw(from.index() as u32).to_ffi();
        self.operands[start + 1] = value;
    }
}

impl BasicBlock {
    pub(super) fn dummy(index: usize) -> Self {
        BasicBlock {
            function: ir::Function(0),
            index,
        }
    }
}

pub(super) fn access_info_intern<'a>(
    func: &'a mut ffi::Function,
    ptr: &Value,
) -> Option<&'a mut AccessInfo> {
    match ptr {
        Value::Alloc(a) => Some(&mut func.allocas[*a as usize].access_info),
        Value::Slot(f, v) if *v < (func.args.len() as u32) => {
            Some(&mut func.args[*v as usize].access_info)
        }
        _ => None,
    }
}

pub(super) fn decode_inline_const(c: u32) -> Option<(Type, u128)> {
    if c & INLINE_CONST_BIT == 0 {
        return None;
    }

    let ty = Type {
        repr: ((c & CONST_CONTENT) >> (u32::BITS - 4 - INLINE_CONST_TYPE_BITS_COUNT)) as _,
    };
    let val = c & INLINE_CONST_VAL_MASK;
    Some((ty, val as u128))
}

pub(super) fn decode_const(consts: &[ffi::Const], c: u32) -> (Type, u128) {
    decode_inline_const(c).unwrap_or_else(|| {
        let value = &consts[c as usize];
        (value.ty, value.data())
    })
}

impl ModuleTpde {
    pub(super) fn const_value(&self, c: u32) -> (Type, u128) {
        decode_const(&self.consts, c)
    }

    pub fn const_data(&self, slot: Value) -> Option<u128> {
        match slot {
            Value::Const(c) => Some(self.const_value(c).1),
            _ => None,
        }
    }

    pub fn add_const(&mut self, ty: Type, data: u128) -> Value {
        if data < INLINE_CONST_LIMIT {
            return Value::new_inline_const(ty, data as u32);
        }
        let consts = &mut self.consts;
        consts.push(ffi::Const::new(ty, data));
        Value::new_pool_const((consts.len() - 1) as u32)
    }
}

type Marker = u32;
pub const MARKER_BLOCK: Marker = 7_u32 << (u32::BITS - 3);

pub const MARKER_VAL: Marker = 0_u32 << (u32::BITS - 3);
pub const MARKER_CONST: Marker = 1_u32 << (u32::BITS - 3);
pub const MARKER_RAW: Marker = 2_u32 << (u32::BITS - 3);
pub const MARKER_ALLOC: Marker = 3_u32 << (u32::BITS - 3);
pub const MARKER_FUNC: Marker = 4_u32 << (u32::BITS - 3);
pub const MARKER_GLOBAL: Marker = 5_u32 << (u32::BITS - 3);
pub const MARKER_CONST_VECTOR: Marker = 6_u32 << (u32::BITS - 3);
pub const MARKER_PHI: Marker = 7_u32 << (u32::BITS - 3);

pub const PHI_POS_BITS: u32 = 9;
pub const PHI_BLOCK_BITS: u32 = u32::BITS - 3 - PHI_POS_BITS;
pub const PHI_POS_MASK: u32 = (1 << PHI_POS_BITS) - 1;

// With the marker set it has an offset saved in global_ptrs,
// otherwise it points directly to the global
pub const GLOBAL_PTR_BIT: u32 = 1_u32 << (u32::BITS - 4);
impl Value {
    fn new_val(func: Function, index: u32) -> Self {
        Self::Slot(func, index)
    }

    fn new_pair(index: u32) -> Self {
        Self::Pair(index)
    }

    pub(super) fn new_inline_const(ty: Type, val: u32) -> Self {
        Self::Const(INLINE_CONST_BIT | (ty.repr as u32) << INLINE_CONST_VAL_BITS_COUNT | val)
    }

    fn new_pool_const(index: u32) -> Self {
        Self::Const(index)
    }

    fn new_alloc(index: u32) -> Self {
        Self::Alloc(index)
    }

    pub fn new_raw(u: u32) -> Self {
        Self::Raw(u)
    }

    pub fn new_phi(block: u32, index: u32) -> Self {
        Self::Phi(Function(0), block, index)
    }

    pub fn new_func(func: Function) -> Self {
        Self::Func(func)
    }

    pub fn new_global(global: Global) -> Self {
        Self::Global(global)
    }

    pub fn to_ffi(&self) -> u32 {
        match self {
            Self::Slot(_, v) => *v,
            Self::Const(i) => *i | MARKER_CONST,
            Self::Alloc(p) => *p | MARKER_ALLOC,
            Self::Raw(r) => *r | MARKER_RAW,
            Self::Func(f) => (f.0 as u32) | MARKER_FUNC,
            Self::Global(g) => g.0 as u32 | MARKER_GLOBAL,
            Self::GlobalPtr(p) => *p | MARKER_GLOBAL | GLOBAL_PTR_BIT,
            Self::Phi(_, block, index) => MARKER_PHI | block << PHI_POS_BITS | index,
            Self::ConstVector(v) => *v | MARKER_CONST_VECTOR,
            Self::Pair(..) => unreachable!("Only used for tracking during generation"),
        }
    }

    #[inline]
    fn is(ffi: u32, marker: Marker) -> Option<u32> {
        if (ffi & MARKER_BLOCK) == marker {
            return Some(ffi & !MARKER_BLOCK);
        }
        None
    }

    pub fn from_ffi(ffi: u32) -> Self {
        if let Some(u) = Self::is(ffi, MARKER_VAL) {
            return Self::Slot(Function(0), u);
        }
        if let Some(u) = Self::is(ffi, MARKER_CONST) {
            return Self::Const(u);
        }
        if let Some(u) = Self::is(ffi, MARKER_ALLOC) {
            return Self::Alloc(u);
        }
        if let Some(u) = Self::is(ffi, MARKER_RAW) {
            return Self::Raw(u);
        }
        if let Some(f) = Self::is(ffi, MARKER_FUNC) {
            return Self::Func(Function(f as usize));
        }
        if let Some(g) = Self::is(ffi, MARKER_GLOBAL) {
            if g & GLOBAL_PTR_BIT != 0 {
                return Self::GlobalPtr(g & !GLOBAL_PTR_BIT);
            }
            return Self::Global(Global(g as usize));
        }
        if let Some(p) = Self::is(ffi, MARKER_PHI) {
            return Self::Phi(Function(0), p >> PHI_POS_BITS, p & PHI_POS_MASK);
        }
        if let Some(v) = Self::is(ffi, MARKER_CONST_VECTOR) {
            return Self::ConstVector(v);
        }
        unreachable!()
        // if let Some(u) = Self::is(ffi, MARKER_CPAIR) {
        //     return Self::CPair(u);
        // }
        // if let Some(u) = Self::is(ffi, MARKER_PAIR) {
        //     return Self::Pair(Function(0), u);
        // }
    }

    pub fn get_func(&self) -> Option<Function> {
        match self {
            Value::Slot(func, _) | Value::Phi(func, ..) => Some(*func),
            _ => None,
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Slot(_, v) => write!(f, "s{v}"),
            Self::Alloc(v) => write!(f, "a{v}"),
            Self::Raw(v) => write!(f, "{v}"),
            Self::ConstVector(v) => write!(f, "[const vector #{v}]"),
            Self::Func(v) => write!(f, "f{}", v.0),
            Self::Global(g) => write!(f, "g{}", g.0),
            Self::Const(c) => match decode_inline_const(*c) {
                Some((ty, val)) => write!(f, "[{ty:?} {val}]"),
                None => write!(f, "[const #{c}]"),
            },
            Self::GlobalPtr(p) => write!(f, "gp{p}"),
            Self::Phi(_, block, ind) => write!(f, "p(bb{block}, {ind})"),
            Self::Pair(..) => unreachable!("Only used for tracking during generation"),
        }
    }
}

impl BasicBlock {
    fn new(function: Function, index: usize) -> BasicBlock {
        BasicBlock { function, index }
    }

    pub fn function(&self) -> Function {
        self.function
    }

    fn index(&self) -> usize {
        self.index
    }
}

impl ffi::BasicBlock {
    pub(super) fn successors(&self) -> Range<usize> {
        self.suc_start as usize..(self.suc_start + self.suc_len) as usize
    }
}

impl Value {
    pub fn pair_slots(&self, ir: &Module) -> Option<(Value, Value)> {
        let pair = match self {
            Value::Pair(ind) => ir.pairs[*ind as usize],
            _ => return None,
        };
        Some((pair.slot_a, pair.slot_b))
    }
}

impl ffi::Const {
    fn new(ty: Type, v: u128) -> Self {
        ffi::Const {
            ty,
            data1: (v >> 64) as u64,
            data2: v as u64,
        }
    }

    pub fn data(&self) -> u128 {
        ((self.data1 as u128) << 64) + (self.data2 as u128)
    }
}

pub fn convert_atomic_op(op: rustc_codegen_ssa::common::AtomicRmwBinOp) -> AtomicRmwBinOp {
    match op {
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicXchg => AtomicRmwBinOp::Xchg,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicAdd => AtomicRmwBinOp::Add,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicSub => AtomicRmwBinOp::Sub,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicAnd => AtomicRmwBinOp::And,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicNand => AtomicRmwBinOp::Nand,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicOr => AtomicRmwBinOp::Or,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicXor => AtomicRmwBinOp::Xor,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicMax => AtomicRmwBinOp::Max,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicMin => AtomicRmwBinOp::Min,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicUMax => AtomicRmwBinOp::UMax,
        rustc_codegen_ssa::common::AtomicRmwBinOp::AtomicUMin => AtomicRmwBinOp::UMin,
    }
}

pub fn convert_atomic_order(order: rustc_middle::ty::AtomicOrdering) -> AtomicOrdering {
    match order {
        rustc_middle::ty::AtomicOrdering::Relaxed => AtomicOrdering::Monotonic,
        rustc_middle::ty::AtomicOrdering::Release => AtomicOrdering::Release,
        rustc_middle::ty::AtomicOrdering::Acquire => AtomicOrdering::Acquire,
        rustc_middle::ty::AtomicOrdering::AcqRel => AtomicOrdering::AcquireRelease,
        rustc_middle::ty::AtomicOrdering::SeqCst => AtomicOrdering::SequentiallyConsistent,
    }
}

pub fn update_access_info(
    access_info: &mut AccessInfo,
    ty: Type,
    size: u32,
    offset: u32,
    written: bool,
) {
    // We only care about small values as we don't promote larger structs/vecs
    if size > u8::MAX as u32 || offset > u8::MAX as u32 {
        access_info.state = AccessInfoState::Complex;
        return;
    }
    let size = size as u8;
    let offset = offset as u8;

    if offset == 0 {
        if size == access_info.size_a && ty == access_info.ty_a {
            return;
        }
        if access_info.size_a == 0 && (access_info.size_b == 0 || size <= access_info.offset_b) {
            access_info.ty_a = ty;
            access_info.size_a = size;
            return;
        }
    } else if offset > 0 {
        if ty == access_info.ty_b && offset == access_info.offset_b && size == access_info.size_b {
            return;
        }
        if access_info.size_b == 0 && offset >= access_info.size_a {
            access_info.ty_b = ty;
            access_info.size_b = size;
            access_info.offset_b = offset;
            return;
        }
    }

    access_info.state = AccessInfoState::Complex;
}

pub fn merge_access_states(a: AccessInfoState, b: AccessInfoState) -> AccessInfoState {
    match (a, b) {
        (AccessInfoState::Complex, _) => AccessInfoState::Complex,
        (_, AccessInfoState::Complex) => AccessInfoState::Complex,
        _ => AccessInfoState::Promotable,
    }
}

pub fn alloc_used_a(info: &AccessInfo) -> bool {
    info.size_a != 0
}

pub fn alloc_used_b(info: &AccessInfo) -> bool {
    info.size_b != 0
}
