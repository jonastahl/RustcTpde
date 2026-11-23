#[allow(unused_imports)]
pub use ffi::compile_to_file;

mod display;
pub mod ir;
pub mod optimize;

#[cxx::bridge]
mod ffi {

    pub struct ModuleTpde {
        functions: Vec<Function>,
        consts: Vec<Const>,

        globals: Vec<Global>,
        global_ptrs: Vec<GlobalPtr>,
        const_vectors: Vec<ConstVector>,
    }

    pub struct Function {
        name: String,
        has_ret: bool,
        args: Vec<ArgInfo>,
        slots: Vec<Slot>,
        phis: Vec<Phi>,

        // tracks how many other functions this functions calls to determine leave functions
        // ignores invokes and extern functions as they cannot be inlined anyway
        calling: u32,
        // points to the call instruction where this function is called for fast inlining
        called_by: Vec<InstructionReference>,
        // tracks const of the function to evaluate inlining of not always inline functions
        cost: u32,

        flags: Flags,

        has_personality: bool,
        personality: u32,

        operands: Vec<u32>,
        instructions: Vec<Instruction>,
        allocas: Vec<Alloca>,
        basic_blocks: Vec<BasicBlock>,

        successors: Vec<u32>,

        callee_infos: Vec<CalleeInfo>,
    }

    pub struct InstructionReference {
        func: u32,
        block: u32,
        instr: u32,
    }

    #[derive(Debug, Default)]
    pub enum ArgKind {
        #[default]
        Direct,
        ByVal,
        sRet,
    }

    #[derive(Debug, Copy, Clone, Default)]
    pub enum ArgExtension {
        #[default]
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

    #[derive(Debug, Copy, Clone, Default)]
    pub struct ArgInfo {
        kind: ArgKind,
        extension: ArgExtension,
        size: u32,
        align: u32,
        access_info: AccessInfo,
    }

    #[derive(Debug, Clone)]
    pub struct CalleeInfo {
        info: Vec<ArgInfo>,
    }

    #[derive(Debug)]
    pub struct Flags {
        extern_link: bool,
        only_local: bool,
        dso_local: bool,
        weak_link: bool,
        always_inline: bool,
        // tracks when a function may not be deleted after inlining it
        // triggered by #[inline(never)] or referencing it ouside the call argument of call
        // e.g. in a global, invoke or other arguments
        never_delete: bool,
        dead: bool,
    }

    #[derive(Clone)]
    pub struct BasicBlock {
        insts_start: u32,
        insts_len: u32,
        suc_start: u32,
        suc_len: u32,
        phi_start: u32,
        phi_len: u32,
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

    #[derive(Copy, Clone)]
    pub struct Phi {
        ty: Type,
        incoming_start: u32,
        incoming_len: u32,
    }

    #[derive(Debug, Copy, Clone, Default)]
    pub enum Type {
        #[default]
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

    #[derive(Debug, Copy, Clone)]
    pub enum LandingPadKind {
        Cleanup,
        Filter,
        CatchAll,
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
        PairGEP,

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
        bswap,
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
        rdtsc,
        rdtscp,
        xgetbv,

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

        psadbw,
        pmaddwd,
        pmaddubsw,
        pmulhrsw,
        simd_shift,
        simd_minmax,
        simd_approx,
        simd_cmp,
        simd_cvt,

        copysign,

        fAbs,
        fSqrt,
        fMin,
        fMax,

        clmul,
        crc32,
        sha,

        Mov,
        Noop,

        Last,
    }

    #[derive(Copy, Clone)]
    pub struct Instruction {
        kind: InstructionKind,
        ops_start: u32,
        ops_len: u32,
        has_result: bool,
        result: u32,
    }

    #[derive(Clone)]
    pub struct Alloca {
        size: usize,
        align: usize,
        access_info: AccessInfo,
    }

    #[derive(Default, Debug, Clone, Copy)]
    pub struct AccessInfo {
        state: AccessInfoState,
        /// Tracks to how many calls this value was passed, is decremented by inlining
        call_forwards: u16,
        size_a: u8,
        ty_a: Type,
        size_b: u8,
        ty_b: Type,
        offset_b: u8,
    }

    #[derive(Default, Debug, Clone, Copy)]
    pub enum AccessInfoState {
        #[default]
        Promotable,
        Complex,
    }

    pub struct ConstVector {
        ty: Type,
        data: Vec<u8>,
    }

    pub struct Const {
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

struct BlockDebug<'a> {
    index: u32,
    block: &'a ffi::BasicBlock,
    function: &'a ffi::Function,
}

struct InstructionDebug<'a> {
    inst: &'a ffi::Instruction,
    operands: &'a [u32],
}

struct PhiDebug<'a> {
    block: u32,
    index: u32,
    phi: &'a ffi::Phi,
    operands: &'a [u32],
}
