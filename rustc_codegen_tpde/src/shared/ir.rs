use super::ffi;
pub use super::ffi::ModuleTpde;
use crate::shared::ffi::{CalleeInfo, GlobalPtr};
#[cfg(feature = "promote")]
use crate::shared::ffi::AccessInfoState;
use core::fmt::{Debug, Formatter};
use rustc_attr_ir::Linkage;
use rustc_codegen_ssa::traits::ReturnSlot;
use rustc_middle::mir::Mutability;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Function(pub(super) usize);
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Global(usize);
#[derive(Debug, Copy, Clone)]
pub struct BasicBlock {
    pub(super) function: Function,
    pub(super) index: usize,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Slot {
    Value(Function, u32),
    Pair(u32),
    ConstVector(u32),
    Const(u32),
    Raw(u32),
    Alloc(u32),
    Func(Function),
    Global(Global),
    GlobalPtr(u32),
    Phi(Function, u32),
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
    ArgExtension, ArgInfo, ArgKind, AtomicOrdering, AtomicRmwBinOp, InstructionKind,
    LandingPadKind, Type, AccessInfo
};

pub struct Module {
    tpde: ModuleTpde,

    pairs: Vec<PairRef>,
}

/// We inline smaller constants to avoid overusing the constant pool with small values.
pub const INLINE_CONST_BITS: u32 = 6;
pub const INLINE_CONST_LIMIT: u128 = 1 << INLINE_CONST_BITS;
pub const INLINE_CONST_END: u32 = (Type::Last.repr as u32) << INLINE_CONST_BITS;

#[derive(PartialEq)]
pub enum Binding {
    Declaration,
    Definition,
}

#[derive(Copy, Clone)]
pub struct PairRef {
    slot_a: Slot,
    slot_b: Slot,
    offset_b: u32,
}

impl Module {
    pub fn new() -> Self {
        Self {
            tpde: ModuleTpde {
                functions: vec![],
                consts: vec![],

                globals: vec![],
                global_ptrs: vec![],
                const_vectors: vec![],
            },
            pairs: vec![],
        }
    }

    pub fn tpde_mut(&mut self) -> &mut ModuleTpde {
        &mut self.tpde
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
            flags: Self::create_flags(linkage, ty),
            operands: vec![],
            instructions: vec![],
            allocas: vec![],
            basic_blocks: vec![],
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

    pub fn return_value(&mut self, func: Function) -> Option<Slot> {
        if self.tpde.functions[func.0].has_ret {
            Some(Slot::new_val(func, 0))
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
            flags: Self::create_flags(linkage, binding),

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
        slot: Slot,
        pointer_size: usize,
    ) {
        let global = &mut self.tpde.globals[global.0];
        if !global.init {
            global.init = true;
            global.data.resize(global.size as usize, 0);
        }
        Module::global_add_unit_intern(global, pointer_size);

        if let Slot::Func(func) = slot {
            self.tpde.functions[func.0].flags.never_delete = true;
        }

        global.relocations.push(ffi::Relocation {
            offset,
            slot: slot.to_ffi(),
        });
    }

    pub fn add_global_ptr(&mut self, global: Global, offset: u32) -> Slot {
        self.tpde.global_ptrs.push(ffi::GlobalPtr {
            global: global.0 as u32,
            offset,
        });
        Slot::GlobalPtr(self.tpde.global_ptrs.len() as u32 - 1)
    }

    pub fn add_to_ptr(&mut self, ptr: Slot, off: u32) -> Slot {
        match ptr {
            Slot::Global(glob) => self.add_global_ptr(glob, off),
            Slot::GlobalPtr(i) => {
                let GlobalPtr { global, offset } = self.tpde.global_ptrs[i as usize];
                self.add_global_ptr(Global(global as usize), offset + off)
            }
            _ => todo!(),
        }
    }

    fn create_flags(linkage: Linkage, ty: Binding) -> ffi::Flags {
        ffi::Flags {
            extern_link: ty == Binding::Declaration
                || matches!(
                    linkage,
                    Linkage::AvailableExternally | Linkage::ExternalWeak
                ),
            only_local: linkage == Linkage::Internal,
            weak_link: linkage == Linkage::WeakODR
                || linkage == Linkage::WeakAny
                || linkage == Linkage::ExternalWeak,
            always_inline: false,
            never_delete: false,
            dead: false,
        }
    }

    pub fn get_slot(&self, func: Function, index: u32) -> Slot {
        Slot::new_val(func, index)
    }

    pub fn type_of_slot(&self, slot: Slot) -> FullType {
        match slot {
            Slot::Value(func, ind) => {
                FullType::Single(self.tpde.functions[func.0].slots[ind as usize].ty)
            }
            Slot::Const(c) => FullType::Single(self.const_value(c).0),
            Slot::Pair(ind) => {
                let pair = &self.pairs[ind as usize];
                let FullType::Single(slot_a) = self.type_of_slot(pair.slot_a) else {
                    unreachable!()
                };
                let FullType::Single(slot_b) = self.type_of_slot(pair.slot_b) else {
                    unreachable!()
                };
                FullType::Pair(slot_a, slot_b, pair.offset_b)
            }
            Slot::ConstVector(ind) => FullType::Single(self.tpde.const_vectors[ind as usize].ty),
            Slot::Raw(_) => unreachable!(),
            Slot::Phi(func, ind) => {
                FullType::Single(self.tpde.functions[func.0].phis[ind as usize].ty)
            }
            Slot::Alloc(_) | Slot::Func(_) | Slot::Global(_) | Slot::GlobalPtr(..) => {
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
    pub fn add_instruction(&mut self, bb: BasicBlock, instr: InstructionKind, ops: &[Slot]) {
        self.add_instruction_raw_internal(bb, instr, ops, None);
    }

    #[inline]
    pub fn add_instruction_ret(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: &[Slot],
        ret: Type,
    ) -> Slot {
        self.add_instruction_raw_internal(bb, instr, ops, Some(FullType::Single(ret)))
            .unwrap()
    }

    #[inline]
    pub fn add_instruction_ret_x(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: &[Slot],
        ret: usize,
    ) -> Slot {
        assert!(ops.len() >= 1);

        let func = self.get_function_mut(&bb.function());
        let op = *ops.get(ret).unwrap();

        // create new slot with type of first arg
        let ret = match op {
            Slot::Value(f, v) => {
                assert_eq!(bb.function(), f);
                func.slots.get(v as usize).unwrap().ty
            }
            Slot::Phi(f, p) => {
                assert_eq!(bb.function(), f);
                func.phis[p as usize].ty
            }
            Slot::Const(v) => self.const_value(v).0,
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
        ops: &[Slot],
    ) -> Slot {
        self.add_instruction_ret_x(bb, instr, ops, 0)
    }

    #[inline]
    fn add_instruction_raw_internal(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: &[Slot],
        ret: Option<FullType>,
    ) -> Option<Slot> {
        enum ReturnType {
            None,
            Single(Slot),
            Pair(Slot, Slot, Slot),
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

            fn result(&self) -> Option<Slot> {
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
            && let [Slot::Func(callee), args @ ..] = ops
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
            if let Slot::Func(func) = op {
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

        #[cfg(feature = "promote")]
        {
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

    #[cfg(feature = "promote")]
    pub fn access_info(&mut self, f: Function, ptr: &Slot) -> Option<&mut AccessInfo> {
        let func = self.get_function_mut(&f);
        access_info_intern(func, ptr)
    }

    pub fn add_call(
        &mut self,
        bb: BasicBlock,
        func_ref: Slot,
        func_sign: &FunctionSignature,
        return_slot: ReturnSlot<Slot>,
        args: &[Slot],
    ) -> Option<Slot> {
        let mut ops = Vec::with_capacity(1 + args.len());
        ops.push(func_ref);
        match func_ref {
            Slot::Func(_) => {}
            Slot::Value(..) | Slot::Phi(..) => {
                let callee_infos = &mut self.tpde.functions[bb.function.0].callee_infos;
                callee_infos.push(CalleeInfo {
                    info: func_sign.arg_infos.clone(),
                });
                ops.push(Slot::new_raw(callee_infos.len() as u32 - 1))
            }
            _ => todo!(),
        }
        if let ReturnSlot::Indirect(sret) = return_slot {
            ops.push(sret);
        }
        for &arg in args {
            match arg {
                Slot::Pair(ind) => {
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
        func_ref: Slot,
        func_sign: &FunctionSignature,
        then: BasicBlock,
        catch: BasicBlock,
        return_slot: ReturnSlot<Slot>,
        args: &[Slot],
    ) -> Option<Slot> {
        let mut ops = Vec::with_capacity(3 + args.len());
        ops.push(func_ref);
        ops.push(Slot::new_raw(then.index as u32));
        ops.push(Slot::new_raw(catch.index as u32));
        match func_ref {
            Slot::Func(_) => {}
            Slot::Value(..) | Slot::Phi(..) => {
                let callee_infos = &mut self.tpde.functions[bb.function.0].callee_infos;
                callee_infos.push(CalleeInfo {
                    info: func_sign.arg_infos.clone(),
                });
                ops.push(Slot::new_raw(callee_infos.len() as u32 - 1))
            }
            _ => todo!(),
        }
        if let ReturnSlot::Indirect(sret) = return_slot {
            ops.push(sret);
        }
        ops.extend_from_slice(args);
        self.add_instruction_raw_internal(bb, InstructionKind::Invoke, &ops, func_sign.ret)
    }

    pub fn add_alloca(&mut self, func: Function, size: usize, align: usize) -> Slot {
        let func = self.get_function_mut(&func);

        func.allocas.push(ffi::Alloca {
            size,
            align,
            access_info: AccessInfo::default(),
        });
        Slot::new_alloc((func.allocas.len() - 1) as u32)
    }

    pub fn add_const(&mut self, ty: Type, data: u128) -> Slot {
        self.tpde.add_const(ty, data)
    }

    /// Wrapper to also convert back inlined constants
    fn const_value(&self, c: u32) -> (Type, u128) {
        self.tpde.const_value(c)
    }

    pub fn const_data(&self, slot: Slot) -> Option<u128> {
        self.tpde.const_data(slot)
    }

    pub fn add_const_vector(&mut self, elems: &[Slot]) -> Slot {
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
        Slot::ConstVector((self.tpde.const_vectors.len() - 1) as u32)
    }

    pub fn add_pair(&mut self, slot_a: Slot, slot_b: Slot, offset_b: u32) -> Slot {
        let slot_pairs = &mut self.pairs;
        slot_pairs.push(PairRef {
            slot_a: slot_a,
            slot_b: slot_b,
            offset_b,
        });
        Slot::new_pair((slot_pairs.len() - 1) as u32)
    }

    pub fn add_pair_consts(
        &mut self,
        ty_a: Type,
        ty_b: Type,
        offset_b: u32,
        data_a: u128,
        data_b: u128,
    ) -> Slot {
        let slot_a = self.add_const(ty_a, data_a);
        let slot_b = self.add_const(ty_b, data_b);
        self.add_pair(slot_a, slot_b, offset_b)
    }

    pub fn extract_vals(&self, pair: Slot) -> (Slot, Slot, u32) {
        let v = match pair {
            Slot::Pair(ind) => self.pairs[ind as usize],
            _ => panic!("agg_val has to be a pair"),
        };
        (v.slot_a, v.slot_b, v.offset_b)
    }

    #[inline]
    pub fn add_slot(&mut self, func: Function, ty: Type) -> Slot {
        let slots = &mut self.tpde.functions[func.0].slots;
        slots.push(ffi::Slot { ty });
        Slot::new_val(func, slots.len() as u32 - 1)
    }

    /// Reserve space for all the phis, later call set_phi_incoming to set the incoming values
    /// All phis of a basic block have to be added at once (or we need more movement later)
    pub fn add_phi(&mut self, bb: BasicBlock, ty: Type, incoming_len: u32) -> Slot {
        let function = &mut self.tpde.functions[bb.function().0];
        let phi_index = function.phis.len() as u32;

        let block = &mut function.basic_blocks[bb.index()];
        if block.phi_len == 0 {
            block.phi_start = phi_index;
        }
        assert_eq!(block.phi_start + block.phi_len, phi_index);
        block.phi_len += 1;

        let incoming_start = function.operands.len() as u32;
        function.operands.extend(std::iter::repeat_n(
            Slot::new_raw(0).to_ffi(),
            incoming_len as usize * 2,
        ));
        function.phis.push(ffi::Phi {
            ty,
            incoming_start,
            incoming_len,
        });
        Slot::Phi(bb.function(), phi_index)
    }

    pub fn set_phi_incoming(&mut self, phi: Slot, slot: u32, from: BasicBlock, value: Slot) {
        let Slot::Phi(func, index) = phi else {
            panic!("Not a phi: {:?}", phi)
        };
        let function = &mut self.tpde.functions[func.0];
        let phi = function.phis[index as usize];
        assert!(slot < phi.incoming_len);
        let start = (phi.incoming_start + slot * 2) as usize;
        function.operands[start] = Slot::new_raw(from.index() as u32).to_ffi();
        function.operands[start + 1] = value.to_ffi();
    }

    pub fn add_br(&mut self, bb: BasicBlock, to: BasicBlock) {
        self.add_instruction(bb, InstructionKind::Br, &[Slot::new_raw(to.index as u32)]);
    }

    pub fn add_switch(
        &mut self,
        bb: BasicBlock,
        val: Slot,
        else_block: BasicBlock,
        cases: &[(u128, BasicBlock)],
    ) {
        let mut ops = Vec::with_capacity(cases.len() * 2 + 2);
        ops.push(val);
        ops.push(Slot::new_raw(else_block.index as u32));

        let FullType::Single(ty) = self.type_of_slot(val) else {
            todo!()
        };

        assert!(cases.len() <= 200000, "Too many cases: {}", cases.len());

        for (val, target) in cases {
            assert!(
                *val <= u64::MAX as u128,
                "switch case does not fit in 64 bit"
            );
            ops.push(Slot::new_raw(target.index as u32));
        }
        // Case values live in the constant pool, a raw operand only has 29 bits.
        for (val, target) in cases {
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
        cond: Slot,
        thenbb: BasicBlock,
        elsebb: BasicBlock,
    ) {
        self.add_instruction(
            bb,
            InstructionKind::CondBr,
            &[
                cond,
                Slot::new_raw(thenbb.index as u32),
                Slot::new_raw(elsebb.index as u32),
            ],
        );
    }
}

#[cfg(feature = "promote")]
pub(super) fn access_info_intern<'a>(func: &'a mut ffi::Function, ptr: &Slot) -> Option<&'a mut AccessInfo> {
    match ptr {
        Slot::Alloc(a) => Some(&mut func.allocas[*a as usize].access_info),
        Slot::Value(f, v) if *v < (func.args.len() as u32) => {
            Some(&mut func.args[*v as usize].access_info)
        }
        _ => None,
    }
}

impl ModuleTpde {
    pub(super) fn const_value(&self, c: u32) -> (Type, u128) {
        if c < INLINE_CONST_END {
            let ty = Type {
                repr: (c >> INLINE_CONST_BITS) as _,
            };
            (ty, (c & (INLINE_CONST_LIMIT as u32 - 1)) as u128)
        } else {
            let value = &self.consts[(c - INLINE_CONST_END) as usize];
            (value.ty, value.data())
        }
    }

    pub fn const_data(&self, slot: Slot) -> Option<u128> {
        match slot {
            Slot::Const(c) => Some(self.const_value(c).1),
            _ => None,
        }
    }

    pub fn add_const(&mut self, ty: Type, data: u128) -> Slot {
        if data < INLINE_CONST_LIMIT {
            return Slot::new_const((ty.repr as u32) << INLINE_CONST_BITS | data as u32);
        }
        let consts = &mut self.consts;
        consts.push(ffi::Value::new(ty, data));
        Slot::new_const(INLINE_CONST_END + (consts.len() - 1) as u32)
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

// With the marker set it has an offset saved in global_ptrs,
// otherwise it points directly to the global
pub const GLOBAL_PTR_BIT: u32 = 1_u32 << (u32::BITS - 4);
impl Slot {
    fn new_val(func: Function, index: u32) -> Self {
        Self::Value(func, index)
    }

    fn new_pair(index: u32) -> Self {
        Self::Pair(index)
    }

    fn new_const(index: u32) -> Self {
        Self::Const(index)
    }

    fn new_alloc(index: u32) -> Self {
        Self::Alloc(index)
    }

    pub fn new_raw(u: u32) -> Self {
        Self::Raw(u)
    }

    pub fn new_phi(index: u32) -> Self {
        Self::Phi(Function(0), index)
    }

    pub fn new_func(func: Function) -> Self {
        Self::Func(func)
    }

    pub fn new_global(global: Global) -> Self {
        Self::Global(global)
    }

    pub fn to_ffi(&self) -> u32 {
        match self {
            Self::Value(_, v) => *v,
            Self::Const(i) => *i | MARKER_CONST,
            Self::Alloc(p) => *p | MARKER_ALLOC,
            Self::Raw(r) => *r | MARKER_RAW,
            Self::Func(f) => (f.0 as u32) | MARKER_FUNC,
            Self::Global(g) => g.0 as u32 | MARKER_GLOBAL,
            Self::GlobalPtr(p) => *p | MARKER_GLOBAL | GLOBAL_PTR_BIT,
            Self::Phi(_, p) => *p | MARKER_PHI,
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
            return Self::Value(Function(0), u);
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
            return Self::Phi(Function(0), p);
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
            Slot::Value(func, _) | Slot::Phi(func, _) => Some(*func),
            _ => None,
        }
    }
}

impl Debug for Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Value(func, v) => write!(f, "[val: {}]", v),
            Self::Alloc(v) => write!(f, "[alloc: {}]", v),
            Self::Raw(v) => write!(f, "[raw: {}]", v),
            Self::ConstVector(v) => write!(f, "[const vector: {}]", v),
            Self::Func(v) => write!(f, "[func: {}]", v.0),
            Self::Global(g) => write!(f, "[global: {}]", g.0),
            Self::Const(c) if *c < INLINE_CONST_END => {
                let ty = Type {
                    repr: (*c >> INLINE_CONST_BITS) as _,
                };
                write!(
                    f,
                    "[const: {:?} {}]",
                    ty,
                    *c & (INLINE_CONST_LIMIT as u32 - 1)
                )
            }
            Self::Const(c) => write!(f, "[const pool: {}]", c - INLINE_CONST_END),
            Self::GlobalPtr(p) => write!(f, "[globalptr: {}]", p),
            Self::Phi(_, p) => write!(f, "[phi: {}]", p),
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

impl Slot {
    pub fn pair_slots(&self, ir: &Module) -> Option<(Slot, Slot)> {
        let pair = match self {
            Slot::Pair(ind) => ir.pairs[*ind as usize],
            _ => return None,
        };
        Some((pair.slot_a, pair.slot_b))
    }
}

impl ffi::Value {
    fn new(ty: Type, v: u128) -> Self {
        ffi::Value {
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

#[cfg(feature = "promote")]
pub fn update_access_info(access_info: &mut AccessInfo, size: u32, offset: u32) {
    assert_eq!(size, size as u8 as u32);
    assert_eq!(offset, offset as u8 as u32);
    let size = size as u8;
    let offset = offset as u8;

    enum Match {
        A,
        B,
        None,
    }
    let m = if offset == 0 {
        if size == access_info.size_a {
            Match::A
        } else {
            match access_info.state {
                AccessInfoState::Unused => Match::A,
                AccessInfoState::OnceWrittenB | AccessInfoState::WrittenB
                    if size <= access_info.offset_b =>
                {
                    Match::A
                }
                _ => Match::None,
            }
        }
    } else if offset > 0 {
        if offset == access_info.offset_b && size == access_info.size_b {
            Match::B
        } else {
            match access_info.state {
                AccessInfoState::Unused => Match::B,
                AccessInfoState::OnceWrittenA | AccessInfoState::WrittenA
                    if offset >= access_info.size_a =>
                {
                    Match::B
                }
                _ => Match::None,
            }
        }
    } else {
        Match::None
    };
    match m {
        Match::A => {
            access_info.size_a = size;
        }
        Match::B => {
            access_info.size_b = size;
            access_info.offset_b = offset;
        }
        _ => {}
    }

    access_info.state = match (access_info.state, m) {
        (AccessInfoState::Unused, Match::A) => AccessInfoState::OnceWrittenA,
        (AccessInfoState::Unused, Match::B) => AccessInfoState::OnceWrittenB,

        (AccessInfoState::OnceWrittenA, Match::A) => AccessInfoState::WrittenA,
        (AccessInfoState::OnceWrittenA, Match::B) => AccessInfoState::Written,
        (AccessInfoState::OnceWrittenB, Match::A) => AccessInfoState::Written,
        (AccessInfoState::OnceWrittenB, Match::B) => AccessInfoState::WrittenB,

        (AccessInfoState::WrittenA, Match::A) => AccessInfoState::WrittenA,
        (AccessInfoState::WrittenA, Match::B) => AccessInfoState::Written,
        (AccessInfoState::WrittenB, Match::A) => AccessInfoState::Written,
        (AccessInfoState::WrittenB, Match::B) => AccessInfoState::WrittenB,

        (AccessInfoState::Written, Match::A) => AccessInfoState::Written,
        (AccessInfoState::Written, Match::B) => AccessInfoState::Written,

        _ => AccessInfoState::Complex,
    }
}

pub fn merge_access_states(a: AccessInfoState, b: AccessInfoState) -> AccessInfoState {
    match (a, b) {
        (AccessInfoState::Unused, _) => b,
        (_, AccessInfoState::Unused) => a,
        (AccessInfoState::Complex, _) => AccessInfoState::Complex,
        (_, AccessInfoState::Complex) => AccessInfoState::Complex,

        // Unused and Complex no longer possible as result

        (AccessInfoState::OnceWrittenA, AccessInfoState::OnceWrittenB) => AccessInfoState::OnceWritten,
        (AccessInfoState::OnceWrittenB, AccessInfoState::OnceWrittenA) => AccessInfoState::OnceWritten,

        // Once writtens are no longer possible

        (AccessInfoState::OnceWrittenA, AccessInfoState::OnceWrittenA) => AccessInfoState::WrittenA,
        (AccessInfoState::WrittenA, AccessInfoState::OnceWrittenA) => AccessInfoState::WrittenA,
        (AccessInfoState::OnceWrittenA, AccessInfoState::WrittenA) => AccessInfoState::WrittenA,

        (AccessInfoState::OnceWrittenB, AccessInfoState::OnceWrittenB) => AccessInfoState::WrittenB,
        (AccessInfoState::WrittenB, AccessInfoState::OnceWrittenB) => AccessInfoState::WrittenB,
        (AccessInfoState::OnceWrittenB, AccessInfoState::WrittenB) => AccessInfoState::WrittenB,

        // Written A and B no longer possible, only Written left

        _ => AccessInfoState::Written
    }
}

pub fn written_to_a(state: AccessInfoState) -> bool {
    matches!(state, AccessInfoState::OnceWrittenA | AccessInfoState::OnceWritten
        | AccessInfoState::WrittenA | AccessInfoState::Written)
}

pub fn written_to_b(state: AccessInfoState) -> bool {
    matches!(state, AccessInfoState::OnceWrittenB | AccessInfoState::OnceWritten
        | AccessInfoState::WrittenB | AccessInfoState::Written)
}
