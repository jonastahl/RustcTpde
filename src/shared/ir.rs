use super::ffi;
pub use super::ffi::ModuleTpde;
use crate::shared::ffi::{CalleeInfo, GlobalPtr};
use core::fmt::{Debug, Formatter};
use rustc_hir::attrs::Linkage;
use rustc_middle::mir::Mutability;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Function(usize);
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Global(usize);
#[derive(Debug, Copy, Clone)]
pub struct BasicBlock {
    function: Function,
    index: usize,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Slot {
    Value(Function, u32),
    Pair(u32),
    // Constant vector with elements that are tracked only during generation,
    // e.g. shuffle indices that are wider than a single `Value` can hold.
    ConstVector(u32),
    Const(u32),
    Raw(u32),
    Alloc(u32),
    Func(Function),
    Global(Global),
    GlobalPtr(u32)
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FullType {
    Single(Type),
    Pair(Type, Type, u32),
    Memory{sized: bool, size: u32}
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

pub use super::ffi::{ArgInfo, ArgKind, Type, InstructionKind, ArgExtension, AtomicRmwBinOp, AtomicOrdering};

pub struct Module {
    tpde: ModuleTpde,

    pairs: Vec<PairRef>,
    const_vectors: Vec<Vec<Slot>>,
}

#[derive(PartialEq)]
pub enum Binding {
    Declaration,
    Definition
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
                global_ptrs: vec![]
            },
            pairs: vec![],
            const_vectors: vec![],
        }
    }

    pub fn tpde(&self) -> &ModuleTpde {
        &self.tpde
    }

    pub fn tpde_mut(&mut self) -> &mut ModuleTpde {
        &mut self.tpde
    }

    pub fn add_function<'tpde, 'tcx>(
        &mut self,
        name: &str,
        fn_sign: FunctionSignature,
        linkage: Linkage,
        ty: Binding
    ) -> Function {
        let FunctionSignature{slots, arg_infos, ret} = fn_sign;
        self.tpde.functions.push(ffi::Function {
            name: name.to_string(),
            has_ret: fn_sign.ret.is_some(),
            args: arg_infos,
            has_personality: false,
            personality: 0,
            slots: slots.iter().map(|ty| ffi::Slot { ty: *ty }).collect(),
            flags: Self::create_flags(linkage, ty),
            allocas: vec![],
            basic_blocks: vec![],
            callee_infos: vec![],
        });
        Function(self.tpde.functions.len() - 1)
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
    }

    pub fn add_global(
        &mut self,
        name: &str,
        linkage: Linkage,
        mutability: Mutability,
        binding: Binding
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

    pub fn global_set_align(
        &mut self,
        global: Global,
        align: u32
    ) {
        self.tpde.globals[global.0].align = align;
    }

    pub fn global_set_thread_local(
        &mut self,
        global: Global
    ) {
        self.tpde.globals[global.0].thread_loc = true;
    }

    pub fn global_add_init_chunk(
        &mut self,
        global: Global,
        chunk: &[u8]
    ) {
        let global = &mut self.tpde.globals[global.0];

        if !global.init {
            global.init = true;
            global.data.resize(global.size as usize, 0);
        }

        global.data.extend_from_slice(chunk);
        global.size = global.data.len() as u32;
    }

    fn global_add_unit_intern(
        global: &mut ffi::Global,
        size: usize
    ) {
        global.size += size as u32;

        if global.init {
            global.data.resize(global.size as usize, 0);
        }
    }

    pub fn global_add_unit_chunk(
        &mut self,
        global: Global,
        size: usize
    ) {
        let global = &mut self.tpde.globals[global.0];
        Module::global_add_unit_intern(global, size);
    }

    pub fn global_add_reloc_chunk(
        &mut self,
        global: Global,
        offset: u32,
        slot: Slot,
        pointer_size: usize
    ) {
        let global = &mut self.tpde.globals[global.0];
        if !global.init {
            global.init = true;
            global.data.resize(global.size as usize, 0);
        }
        Module::global_add_unit_intern(global, pointer_size);

        global.relocations.push(ffi::Relocation {
            offset,
            slot: slot.to_ffi()
        });
    }

    pub fn add_global_ptr(
        &mut self,
        global: Global,
        offset: u32
    ) -> Slot {
        self.tpde.global_ptrs.push(ffi::GlobalPtr {
            global: global.0 as u32,
            offset
        });
        Slot::GlobalPtr(self.tpde.global_ptrs.len() as u32 - 1)
    }

    pub fn add_to_ptr(
        &mut self,
        ptr: Slot,
        off: u32
    ) -> Slot {
        match ptr {
            Slot::Global(glob) => self.add_global_ptr(glob, off),
            Slot::GlobalPtr(i) => {
                let GlobalPtr { global, offset } = self.tpde.global_ptrs[i as usize];
                self.add_global_ptr(Global(global as usize), offset + off)
            },
            _ => todo!()
        }
    }

    fn create_flags(
        linkage: Linkage,
        ty: Binding
    ) -> ffi::Flags {
        ffi::Flags {
            extern_link: ty == Binding::Declaration
                || matches!(linkage, Linkage::AvailableExternally | Linkage::ExternalWeak),
            only_local: linkage == Linkage::Internal,
            weak_link: linkage == Linkage::WeakODR
                || linkage == Linkage::WeakAny
                || linkage == Linkage::ExternalWeak,
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
            Slot::Const(ind) => FullType::Single(self.tpde.consts[ind as usize].ty),
            Slot::Pair(ind) => {
                let pair = &self.pairs[ind as usize];
                let FullType::Single(slot_a) = self.type_of_slot(pair.slot_a)
                else {
                    unreachable!()
                };
                let FullType::Single(slot_b) = self.type_of_slot(pair.slot_b)
                else {
                    unreachable!()
                };
                FullType::Pair(slot_a, slot_b, pair.offset_b)
            }
            Slot::ConstVector(ind) => {
                let elems = &self.const_vectors[ind as usize];
                let FullType::Single(elem) = self.type_of_slot(elems[0]) else {
                    unreachable!()
                };
                match vector_type(elem, elems.len() as u64) {
                    Some(ty) => FullType::Single(ty),
                    None => todo!("unsupported constant vector <{} x {:?}>", elems.len(), elem),
                }
            }
            Slot::Raw(_) => unreachable!(),
            Slot::Alloc(_) | Slot::Func(_) | Slot::Global(_) | Slot::GlobalPtr(..)
                => FullType::Single(Type::ptr),
        }
    }

    fn get_function_mut(&mut self, func: &Function) -> &mut ffi::Function {
        self.tpde.functions
            .get_mut(func.0)
            .unwrap_or_else(|| panic!("Function not found"))
    }

    fn get_function(&self, func: &Function) -> &ffi::Function {
        self.tpde.functions
            .get(func.0)
            .unwrap_or_else(|| panic!("Function not found"))
    }

    pub fn add_basic_block(self: &mut Self, func: &Function, name: &str) -> BasicBlock {
        let function = self.get_function_mut(func);
        function.basic_blocks.push(ffi::BasicBlock {
            name: name.to_string(),
            instructions: vec![],
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
    pub fn add_instruction(&mut self, bb: BasicBlock, instr: InstructionKind, ops: Vec<Slot>) {
        self.add_instruction_raw_internal(bb, instr, ops.as_slice(), None);
    }

    #[inline]
    pub fn add_instruction_ret(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: Vec<Slot>,
        ret: Type,
    ) -> Slot {
        self.add_instruction_raw_internal(bb, instr, ops.as_slice(), Some(FullType::Single(ret))).unwrap()
    }

    #[inline]
    pub fn add_instruction_ret_x(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: Vec<Slot>,
        ret: usize
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
            Slot::Const(v) => {
                self.tpde.consts.get(v as usize).unwrap().ty
            }
            _ => panic!("First operand of return instruction must be a value slot"),
        };
        self.add_instruction_raw_internal(bb, instr, ops.as_slice(), Some(FullType::Single(ret))).unwrap()
    }

    #[inline]
    pub fn add_instruction_ret_first(
        &mut self,
        bb: BasicBlock,
        instr: InstructionKind,
        ops: Vec<Slot>,
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

        let ret: ReturnType =
            match ret {
                None => ReturnType::None,
                Some(FullType::Single(ty)) => ReturnType::Single(self.add_slot(bb.function, ty)),
                Some(FullType::Pair(ty_a, ty_b, offset_b)) => {
                    let slot_a = self.add_slot(bb.function, ty_a);
                    let slot_b = self.add_slot(bb.function, ty_b);
                    let pair = self.add_pair(slot_a, slot_b, offset_b);
                    ReturnType::Pair(pair, slot_a, slot_b)
                }
                Some(FullType::Memory { .. }) => ReturnType::Single(self.add_slot(bb.function, Type::ptr)),
            };

        // Constant vectors are only tracked symbolically, the backend needs real constants
        let ffi_ops: Vec<u32> = ops.iter()
            .map(|s| self.materialize_const_vector(*s).unwrap_or(*s).to_ffi())
            .collect();

        let basic_block = self.get_basic_block_mut(bb);

        basic_block.instructions.push(ffi::Instruction {
            kind: instr,
            ops: ffi_ops,
            has_result: ret.num_ret() >= 1,
            result: ret.result_a(),
        });
        if ret.num_ret() >= 2 {
            basic_block.instructions.push(ffi::Instruction {
                kind: InstructionKind::AddRet,
                ops: vec![],
                has_result: true,
                result: ret.result_b(),
            });
        }

        ret.result()
    }

    pub fn add_call(
        &mut self,
        bb: BasicBlock,
        func_ref: Slot,
        func_sign: &FunctionSignature,
        args: &[Slot]) -> Option<Slot> {
        let mut ops = Vec::with_capacity(1 + args.len());
        ops.push(func_ref);
        match func_ref {
            Slot::Func(_) => {},
            Slot::Value(..) => {
                let callee_infos = &mut self.tpde.functions[bb.function.0].callee_infos;
                callee_infos.push(CalleeInfo { info: func_sign.arg_infos.clone() });
                ops.push(Slot::new_raw(callee_infos.len() as u32 - 1))
            }
            _ => todo!()
        }
        ops.extend_from_slice(args);
        self.add_instruction_raw_internal(
            bb,
            InstructionKind::Call,
            ops.as_slice(),
            func_sign.ret
        )
    }

    pub fn add_invoke(
        &mut self,
        bb: BasicBlock,
        func_ref: Slot,
        func_sign: &FunctionSignature,
        then: BasicBlock,
        catch: BasicBlock,
        args: &[Slot]) -> Option<Slot> {
        let mut ops = Vec::with_capacity(3 + args.len());
        ops.push(func_ref);
        ops.push(Slot::new_raw(then.index as u32));
        ops.push(Slot::new_raw(catch.index as u32));
        match func_ref {
            Slot::Func(_) => {},
            Slot::Value(..) => {
                let callee_infos = &mut self.tpde.functions[bb.function.0].callee_infos;
                callee_infos.push(CalleeInfo { info: func_sign.arg_infos.clone() });
                ops.push(Slot::new_raw(callee_infos.len() as u32 - 1))
            }
            _ => todo!()
        }
        ops.extend_from_slice(args);
        self.add_instruction_raw_internal(
            bb,
            InstructionKind::Invoke,
            ops.as_slice(),
            func_sign.ret
        )
    }

    pub fn add_alloca(&mut self, func: Function, size: usize, align: usize) -> Slot {
        let func = self.get_function_mut(&func);

        func.allocas.push(ffi::Alloca { size, align });
        Slot::new_alloc((func.allocas.len() - 1) as u32)
    }

    pub fn add_const(&mut self, ty: Type, data: u128) -> Slot {
        let consts = &mut self.tpde.consts;
        consts.push(ffi::Value::new(ty, data));
        Slot::new_const((consts.len() - 1) as u32)
    }

    pub fn const_data(&self, slot: Slot) -> Option<u128> {
        match slot {
            Slot::Const(i) =>
                Some(self.tpde.consts.get(i as usize).unwrap().data()),
            _ => None
        }
    }

    pub fn add_const_vector(&mut self, elems: &[Slot]) -> Slot {
        assert!(!elems.is_empty());
        self.const_vectors.push(elems.to_vec());
        Slot::ConstVector((self.const_vectors.len() - 1) as u32)
    }

    pub fn const_vector_elems(&self, slot: Slot) -> Option<&[Slot]> {
        match slot {
            Slot::ConstVector(i) => Some(&self.const_vectors[i as usize]),
            _ => None,
        }
    }

    pub fn materialize_const_vector(&mut self, slot: Slot) -> Option<Slot> {
        let elems = self.const_vector_elems(slot)?.to_vec();
        let FullType::Single(ty) = self.type_of_slot(slot) else {
            return None;
        };
        let (elem_ty, _) = vector_info(ty)?;
        let elem_bits = size_of_type(elem_ty) * 8;
        if elem_bits * elems.len() as u32 > 128 {
            return None;
        }

        let mut data = 0u128;
        for (i, elem) in elems.iter().enumerate() {
            let mut value = self.const_data(*elem)?;
            if elem_bits < 128 {
                value &= (1u128 << elem_bits) - 1;
            }
            data |= value << (i as u32 * elem_bits);
        }
        Some(self.add_const(ty, data))
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
        (
            v.slot_a,
            v.slot_b,
            v.offset_b,
        )
    }

    #[inline]
    pub fn add_slot(&mut self, func: Function, ty: Type) -> Slot {
        let slots = &mut self.tpde.functions[func.0].slots;
        slots.push(ffi::Slot { ty });
        Slot::new_val(func, slots.len() as u32 - 1)
    }

    pub fn add_br(&mut self, bb: BasicBlock, to: BasicBlock) {
        self.add_instruction(
            bb,
            InstructionKind::Br,
            vec![Slot::new_raw(to.index as u32)],
        );
    }

    pub fn add_switch(&mut self, bb: BasicBlock, val: Slot, else_block: BasicBlock, cases: &[(u128, BasicBlock)]) {
        let mut ops = vec![val, Slot::new_raw(else_block.index as u32)];
        ops.reserve(cases.len() * 2 + 2);

        let FullType::Single(ty) = self.type_of_slot(val) else {
            todo!()
        };

        assert!(cases.len() <= 200000, "Too many cases: {}", cases.len());
        for (val, bb) in cases {
            ops.push(Slot::Raw(*val as u32));
            ops.push(Slot::new_raw(bb.index as u32));
        }
        self.add_instruction(
            bb,
            InstructionKind::Switch,
            ops
        );
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
            vec![
                cond,
                Slot::new_raw(thenbb.index as u32),
                Slot::new_raw(elsebb.index as u32),
            ],
        );
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
pub const MARKER_GLOBAL_PTR: Marker = 6_u32 << (u32::BITS - 3);
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
            Self::GlobalPtr(p) => *p | MARKER_GLOBAL_PTR,
            Self::Pair(..) | Self::ConstVector(..) =>
                unreachable!("Only used for tracking during generation"),
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
            return Self::Global(Global(g as usize));
        }
        if let Some(p) = Self::is(ffi, MARKER_GLOBAL_PTR) {
            return Self::GlobalPtr(p)
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
            Slot::Value(func, _) => Some(*func),
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
            Self::Const(v) => write!(f, "[const: {}]", v),
            Self::GlobalPtr(p) => write!(f, "[globalptr: {}]", p),
            Self::Pair(..) =>
                unreachable!("Only used for tracking during generation"),
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