use crate::shared::ffi::{ArgExtension, ArgKind, GlobalPtr, Relocation, Type};
use crate::shared::ir::{vector_info, ArgInfo, Slot, size_of_type};
use core::fmt::{Debug, Formatter};
#[allow(unused_imports)]
pub use ffi::compile_to_file;

pub mod ir;

#[cxx::bridge]
mod ffi {

    #[derive(Debug)]
    pub struct ModuleTpde {
        functions: Vec<Function>,
        consts: Vec<Value>,

        globals: Vec<Global>,
        global_ptrs: Vec<GlobalPtr>,
        const_vectors: Vec<ConstVector>,
    }

    enum Linkage {
        External,
    }

    #[derive(Debug)]
    pub struct Function {
        name: String,
        has_ret: bool,
        args: Vec<ArgInfo>,
        slots: Vec<Slot>,

        flags: Flags,

        has_personality: bool,
        personality: u32,

        allocas: Vec<Alloca>,
        basic_blocks: Vec<BasicBlock>,

        callee_infos: Vec<CalleeInfo>,
    }

    #[derive(Debug)]
    pub enum ArgKind {
        Direct,
        ByVal,
        sRet
    }

    #[derive(Debug, Copy, Clone)]
    pub enum ArgExtension {
        None,
        zExt,
        sExt,
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone)]
    pub enum AtomicRmwBinOp {
        Xchg = 0,
        Add = 1,
        Sub = 2,
        And = 3,
        Nand = 4,
        Or = 5,
        Xor = 6,
        Max = 7,
        Min = 8,
        UMax = 9,
        UMin = 10,
    }

    #[repr(u32)]
    #[derive(Debug, Copy, Clone)]
    pub enum AtomicOrdering {
        Monotonic = 0,
        Release = 1,
        Acquire = 2,
        AcquireRelease = 3,
        SequentiallyConsistent = 4,
        LAST = 5,
    }

    #[derive(Debug, Copy, Clone)]
    pub struct ArgInfo {
        kind: ArgKind,
        extension: ArgExtension,
        size: u32,
        align: u32,
    }

    #[derive(Debug, Clone)]
    pub struct CalleeInfo {
        info: Vec<ArgInfo>
    }

    #[derive(Debug)]
    pub struct Flags {
        extern_link: bool,
        only_local: bool,
        weak_link: bool,
    }

    pub struct BasicBlock {
        name: String,
        instructions: Vec<Instruction>,
        // TODO add phis and similar
        info1: u32,
        info2: u32,
    }

    unsafe extern "C++" {
        include!("tpde_cpp/tpde.h");

        pub fn compile_to_file(module: &mut ModuleTpde, path: &str) -> bool;
    }

    #[derive(Copy, Clone)]
    pub struct Slot {
        ty: Type,
    }

    #[derive(Debug, Copy, Clone)]
    pub enum Type {
        Void,
        Bool,
        i8,
        i16,
        i32,
        i64,
        i128,
        f32,
        f64,
        ptr,

        // Vectors narrower than 64 bit
        v2i8,
        v4i8,
        v2i16,

        // 64 bit vectors
        v8i8,
        v4i16,
        v2i32,
        v2f32,

        // 128 bit vectors
        v16i8,
        v8i16,
        v4i32,
        v2i64,
        v4f32,
        v2f64,

        // 256 bit vectors
        v32i8,
        v16i16,
        v8i32,
        v4i64,
        v8f32,
        v4f64,

        // 512 bit vectors
        v64i8,
        v32i16,
        v16i32,
        v8i64,
        v16f32,
        v8f64,

        // 1024 bit vectors
        v128i8,
        v64i16,
        v32i32,
        v16i64,
        v32f32,
        v16f64,

        // 2048 bit vectors
        v64i32,
        v32i64,

        // 4096 bit vectors
        v64i64,

        Last,
    }

    #[derive(Debug)]
    pub enum InstructionKind {
        // Bitwise
        And,
        Or,
        Xor,
        Shl,
        lShr,
        aShr,
        Not,

        // Math
        Add,
        Sub,
        Mul,
        uDiv,
        sDiv,
        uRem,
        sRem,
        Neg,
        fAdd,
        fSub,
        fMul,
        fDiv,
        fRem,
        fNeg,

        // Comparators
        CMPeq,
        CMPne,
        CMPsgt,
        CMPsge,
        CMPslt,
        CMPsle,
        CMPugt,
        CMPuge,
        CMPult,
        CMPule,
        RealOEQ,
        RealOGT,
        RealOGE,
        RealOLT,
        RealOLE,
        RealONE,
        RealORD,
        RealUNO,
        RealUEQ,
        RealUGT,
        RealUGE,
        RealULT,
        RealULE,
        RealUNE,

        OverflowCheck,

        // Storage operations
        Alloca,
        Store,
        Load,
        GEP,
        MemCpy,
        MemMove,
        MemSet,
        MemCmp,

        Select,

        // Branching operations
        Ret,
        Br,
        CondBr,
        Switch,

        // Calls
        Call,
        Invoke,
        AddRet, // Pair return type, represents additional argument after call

        // Casts
        zExt,
        sExt,
        Trunc,
        fExt,
        fTrunc,
        sTof,
        uTof,
        fTos,
        fTou,
        fTos_sat,
        fTou_sat,

        Cast,

        Unreachable,
        Abort,
        LandingPad,
        Resume,

        ctpop,
        ctlz,
        ctlz_nonzero,
        cttz,
        cttz_nonzero,
        rotl,
        rotr,
        funnel_shl,
        funnel_shr,
        sat_sadd,
        sat_uadd,
        sat_ssub,
        sat_usub,

        Atomic_cmpxchg,
        Atomic_rmw,
        Atomic_load,
        Atomic_store,
        Atomic_fence,
        Pause,

        TlsAddr,

        simd_splat,
        simd_extract,
        simd_insert,
        simd_shuffle,
        simd_select_bitmask,
        simd_bitmask,

        simd_reduce_add_ordered,
        simd_reduce_mul_ordered,
        simd_reduce_add_unordered,
        simd_reduce_mul_unordered,
        simd_reduce_smin,
        simd_reduce_umin,
        simd_reduce_fmin,
        simd_reduce_smax,
        simd_reduce_umax,
        simd_reduce_fmax,
        simd_reduce_and,
        simd_reduce_or,
        simd_reduce_xor,
        simd_reduce_any,
        simd_reduce_all,

        simd_gather,
        simd_scatter,
        simd_masked_load,
        simd_masked_store,

        fAbs,
        fMin,
        fMax,

        Last,
    }

    pub struct Instruction {
        kind: InstructionKind,
        ops: Vec<u32>,
        has_result: bool,
        result: u32,
    }

    pub struct Alloca {
        size: usize,
        align: usize,
    }

    pub struct ConstVector {
        ty: Type,
        data: Vec<u8>,
    }

    pub struct Value {
        ty: Type,
        data1: u64,
        data2: u64,
    }

    pub struct Global {
        name: String,

        size: u32,
        align: u32,

        read_only: bool,
        thread_loc: bool,

        flags: Flags,

        init: bool,
        data: Vec<u8>,

        relocations: Vec<Relocation>,
    }

    pub struct Relocation {
        offset: u32,
        slot: u32,
    }

    pub struct GlobalPtr {
        global: u32,
        offset: u32,
    }
}

impl Debug for ffi::BasicBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: ", self.name)?;
        f.debug_list().entries(&self.instructions).finish()
    }
}

impl Debug for ffi::Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "<{:?}> (", self.kind)?;
        let mut first = true;
        for &op in &self.ops {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            write!(f, "{:?}", Slot::from_ffi(op))?;
        }
        write!(f, ")")?;

        if self.has_result {
            write!(f, " -> {:?}", Slot::from_ffi(self.result))?;
        }

        Ok(())
    }
}

impl Debug for ffi::Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let val = match self.ty {
            Type::Bool => format!("{}", self.data2 != 0),
            Type::i8 => format!("{}", self.data2 as i8),
            Type::i16 => format!("{}", self.data2 as i16),
            Type::i32 => format!("{}", self.data2 as i32),
            Type::i64 => format!("{}", self.data2 as i64),
            Type::i128 => format!("{}", (self.data1 as i128) << 64 | (self.data1 as i128)),
            Type::ptr => format!("[ptr: {}]", self.data2),
            Type::f32 => format!("{}", f32::from_bits(self.data2 as u32)),
            Type::f64 => format!("{}", f64::from_bits(self.data2)),
            ty if ir::vector_info(ty).is_some() => format!("<{:#018x} {:#018x}>", self.data1, self.data2),
            _ => todo!(),
        };
        write!(
            f,
            "[{:#?}] ({} {}) -> {}",
            self.ty, self.data1, self.data2, val
        )
    }
}

impl Debug for ffi::Slot {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.ty)
    }
}

impl Debug for ffi::Alloca {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "size: {:?}, align: {:?}", self.size, self.align)
    }
}

impl Debug for ffi::Global {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        struct HexDump<'a>(&'a [u8]);

        impl<'a> std::fmt::Debug for HexDump<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({} bytes)", self.0.len())?;

                for (i, chunk) in self.0.chunks(16).enumerate() {
                    // Print offset
                    write!(f, "\n  {:04x}  ", i * 16)?;

                    // Print hex values with padding for incomplete lines
                    for b in chunk { write!(f, "{:02x} ", b)?; }
                    for _ in 0..(16 - chunk.len()) { write!(f, "   ")?; }

                    // Print ASCII representation
                    write!(f, " |")?;
                    for &b in chunk {
                        let c = if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' };
                        write!(f, "{}", c)?;
                    }
                    write!(f, "|")?;
                }
                Ok(())
            }
        }

        f.debug_struct(&self.name)
            .field("size", &self.size)
            .field("align", &self.align)
            .field("read_only", &self.read_only)
            .field("thread_loc", &self.thread_loc)
            .field("flags", &self.flags)
            .field("init", &self.init)
            .field("data", &HexDump(&self.data))
            .field("relocations", &self.relocations)
            .finish()
    }
}

impl Debug for ffi::ConstVector {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let Some((elem_ty, _)) = vector_info(self.ty) else {
            return write!(f, "ConstVector {{ ty: {:?}, data: {:?} }}", self.ty, self.data);
        };
        let elem_bytes = size_of_type(elem_ty) as usize;

        write!(f, "ConstVector {{ ty: {:?}, lanes: [", self.ty)?;
        for (i, lane) in self.data.chunks(elem_bytes).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            let mut bytes = [0u8; 8];
            bytes[..lane.len()].copy_from_slice(lane);
            let bits = u64::from_le_bytes(bytes);
            match elem_ty {
                Type::f32 => write!(f, "{}", f32::from_bits(bits as u32))?,
                Type::f64 => write!(f, "{}", f64::from_bits(bits))?,
                _ => write!(f, "{}", bits)?,
            }
        }
        write!(f, "] }}")
    }
}

impl Debug for GlobalPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{:?}] + {:?}", self.global, self.offset)
    }
}

impl Default for ArgInfo {
    fn default() -> Self {
        Self {
            kind: ArgKind::Direct,
            extension: ArgExtension::None,
            size: 0,
            align: 0,
        }
    }
}

impl Debug for Relocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "offset {:#x} insert {:?}", self.offset, Slot::from_ffi(self.slot))
    }
}