use crate::shared::ffi::{ArgExtension, ArgKind, GlobalPtr, Relocation, Type};
use crate::shared::ir::{ArgInfo, Slot};
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
        global_ptrs: Vec<GlobalPtr>
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

        v8i8,
        v16i8,
        v4i16,
        v8i16,
        v2i32,
        v4i32,
        v2i64,
        v2f32,
        v4f32,
        v2f64,

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
        sat_sadd,
        sat_uadd,
        sat_ssub,
        sat_usub,

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