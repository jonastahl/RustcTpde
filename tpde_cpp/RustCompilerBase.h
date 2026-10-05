#pragma once
#include "RustAdaptor.h"
#include "RustCompiler.h"
#include "../deps/tpde/tpde/include/tpde/CompilerBase.hpp"

namespace tpde_rust {
  enum class LibFunc {
    divti3,
    udivti3,
    modti3,
    umodti3,
    fmod,
    fmodf,
    fmodf16,
    floorf,
    floor,
    ceilf,
    ceil,
    roundf,
    round,
    nearbyintf,
    nearbyint,
    rintf,
    rint,
    lround,
    lroundf,
    memcpy,
    memset,
    memmove,
    memcmp,
    resume,
    powisf2,
    powidf2,
    trunc,
    truncf,
    fma,
    fmaf,
    pow,
    powf,
    sin,
    sinf,
    cos,
    cosf,
    tan,
    tanf,
    asin,
    asinf,
    acos,
    acosf,
    atan,
    atanf,
    atan2,
    atan2f,
    sinh,
    sinhf,
    cosh,
    coshf,
    tanh,
    tanhf,
    log,
    logf,
    logl,
    logf128,
    log2,
    log2f,
    log10,
    log10f,
    exp,
    expf,
    exp2,
    exp2f,
    modf,
    modff,
    frexp,
    frexpf,
    trunctfsf2,
    trunctfdf2,
    extendsftf2,
    extenddftf2,
    eqtf2,
    netf2,
    gttf2,
    getf2,
    lttf2,
    letf2,
    unordtf2,
    floatsitf,
    floatditf,
    floatunditf,
    floatunsitf,
    fixtfdi,
    fixunstfdi,
    addtf3,
    subtf3,
    multf3,
    divtf3,
    MAX
  };

  template<typename Adaptor, typename Derived, typename Config>
  struct RustCompilerBase : RustCompiler, tpde::CompilerBase<Adaptor, Derived, Config> {
    using Base = tpde::CompilerBase<Adaptor, Derived, Config>;

    using IRValueRef = Base::IRValueRef;
    using IRBlockRef = Base::IRBlockRef;
    using IRFuncRef = Base::IRFuncRef;
    using ScratchReg = Base::ScratchReg;
    using ValuePartRef = Base::ValuePartRef;
    using ValuePart = Base::ValuePart;
    using ValueRef = Base::ValueRef;
    using GenericValuePart = Base::GenericValuePart;
    using InstRange = Base::InstRange;

    using SecRef = tpde::SecRef;
    using SymRef = tpde::SymRef;

    using AsmReg = Base::AsmReg;

    std::vector<SymRef> global_symbols;

    using ValInfo = Adaptor::ValInfo;

    struct ValRefSpecial {
      enum MODE : uint8_t {
        CONST = 4,
      };

      uint8_t mode;
      IRValueRef data;

    private:
      ValRefSpecial(uint8_t mode, IRValueRef ref) : mode(mode), data{ref} {
      }

    public:
      static ValRefSpecial make_const(IRValueRef data) {
        return {CONST, data};
      }
    };

    tpde::util::BumpAllocator<> const_allocator;

    explicit RustCompilerBase(RustAdaptor *adaptor) : Base{adaptor} {
      static_assert(tpde::Compiler<Derived, Config>);
      static_assert(std::is_same_v<Adaptor, RustAdaptor>);
    }

    Derived *derived() { return static_cast<Derived *>(this); }

    const Derived *derived() const { return static_cast<Derived *>(this); }

    bool compile_to_elf(ModuleTpde &mod, std::vector<uint8_t> &buf) override;

    static bool cur_func_may_emit_calls() { return true; }

    SymRef cur_personality_func() const;

    static bool try_force_fixed_assignment(IRValueRef) { return false; }

    void setup_var_ref_assignments() {
    }

    RustAdaptor::ValueParts val_parts(IRValueRef val) const {
      return this->adaptor->val_parts(val);
    }

    std::optional<ValRefSpecial> val_ref_special(IRValueRef value) {
      if (operands::is_const(value) || operands::is_global(value)
          || operands::is_global_ptr(value) || operands::is_func(value)) {
        return ValRefSpecial::make_const(value);
      }
      return std::nullopt;
    }

    ValuePart val_part_ref_special(ValRefSpecial &vrs, u32 part) {
      if (operands::is_const(vrs.data)) {
        Value &imm = this->adaptor->mod->consts[operands::content(vrs.data)];

        switch (imm.ty) {
          using enum Type;
          case Bool:
          case i8:
            return ValuePart(imm.data2, 1, tpde::RegBank{0});
          case i16:
            return ValuePart(imm.data2, 2, tpde::RegBank{0});
          case i32:
            return ValuePart(imm.data2, 4, tpde::RegBank{0});
          case ptr:
          case i64:
            return ValuePart(imm.data2, 8, tpde::RegBank{0});
          case i128:
            switch (part) {
              case 0:
                return ValuePart(imm.data2, 8, tpde::RegBank{0});
              case 1:
                return ValuePart(imm.data1, 8, tpde::RegBank{0});
              default:
                throw std::runtime_error("invalid part");
            }
          case f32:
              return ValuePart(imm.data2, 4, tpde::RegBank{1});
          case f64:
          case v8i8:
          case v4i16:
          case v2i32:
          case v2f32:
            return ValuePart(imm.data2, 8, tpde::RegBank{1});
          case v16i8:
          case v8i16:
          case v4i32:
          case v2i64:
          case v4f32:
          case v2f64: {
            // Little-endian: low half first
            u64 *data = new (const_allocator) u64[2];
            data[0] = imm.data2;
            data[1] = imm.data1;
            return ValuePart(data, 16, tpde::RegBank{1});
          }

          default:
            throw std::runtime_error("not implemented");
        }
      } {
        uint32_t glob_start = this->adaptor->cur_func->allocas.size()
                              + this->adaptor->cur_func->slots.size();
        uint32_t glob_ptr_start = glob_start
                                  + this->adaptor->mod->globals.size();
        uint32_t func_start = glob_ptr_start
                              + this->adaptor->mod->global_ptrs.size();

        u32 gv_id = operands::content(vrs.data);
        u32 loc_id;
        if (operands::is_global(vrs.data)) {
          loc_id = glob_start + gv_id;
        } else if (operands::is_global_ptr(vrs.data)) {
          loc_id = glob_ptr_start + gv_id;
        } else if (operands::is_func(vrs.data)) {
          // A function used as a value: materialize the address of its symbol.
          assert(part == 0);
          loc_id = func_start + gv_id;
        } else {
          throw std::runtime_error("unknown special mode");
        }
        tpde::ValLocalIdx local_idx{loc_id};

        auto *assignment = this->val_assignment(local_idx);
        if (!assignment) {
          this->init_variable_ref(local_idx, loc_id - glob_start);
          assignment = this->val_assignment(local_idx);
        }
        return ValuePart{local_idx, assignment, 0, /*owned=*/false};
      }
    }

    void prologue_assign_arg(tpde::CCAssigner *cc_assigner,
                             u32 arg_idx,
                             IRValueRef arg) {
      u32 align = size_of_type(Base::adaptor->type_of_ref(arg)) / 8;
      bool allow_split = derived()->arg_allow_split_reg_stack_passing(arg_idx);
      Base::prologue_assign_arg(cc_assigner, arg_idx, arg, align, allow_split);
    }

    void define_func_idx(IRFuncRef func, const u32 idx) {
      // As they are worked through in the same order as in the IR they should be equal
      assert(func - this->adaptor->mod->functions.data() == idx);
    }

    struct IntBinaryOp {
    private:
      static constexpr u32 index_mask = (1 << 4) - 1;
      static constexpr u32 bit_symm = 1 << 4;
      static constexpr u32 bit_signed = 1 << 5;
      static constexpr u32 bit_ext_lhs = 1 << 6;
      static constexpr u32 bit_ext_rhs = 1 << 7;
      static constexpr u32 bit_div = 1 << 8;
      static constexpr u32 bit_rem = 1 << 9;
      static constexpr u32 bit_shift = 1 << 10;

    public:
      enum Value : u32 {
        add = 0 | bit_symm,
        sub = 1,
        mul = 2 | bit_symm,
        udiv = 3 | bit_ext_lhs | bit_ext_rhs | bit_div,
        sdiv = 4 | bit_signed | bit_ext_lhs | bit_ext_rhs | bit_div,
        urem = 5 | bit_ext_lhs | bit_ext_rhs | bit_rem,
        srem = 6 | bit_signed | bit_ext_lhs | bit_ext_rhs | bit_rem,
        land = 7 | bit_symm,
        lor = 8 | bit_symm,
        lxor = 9 | bit_symm,
        shl = 10 | bit_shift,
        shr = 11 | bit_ext_lhs | bit_shift,
        ashr = 12 | bit_signed | bit_ext_lhs | bit_shift,
        num_ops = 13
      };

      Value op;

      constexpr IntBinaryOp(Value op) : op(op) {
      }

      /// Whether the operation is symmetric.
      constexpr bool is_symmetric() const { return op & bit_symm; }
      /// Whether the operation is signed and therefore needs sign-extension.
      constexpr bool is_signed() const { return op & bit_signed; }
      /// Whether the operation needs the first operand extended.
      constexpr bool needs_lhs_ext() const { return op & bit_ext_lhs; }
      /// Whether the operation needs the second operand extended.
      constexpr bool needs_rhs_ext() const { return op & bit_ext_rhs; }
      /// Whether the operation is a div
      constexpr bool is_div() const { return op & bit_div; }
      /// Whether the operation is a rem
      constexpr bool is_rem() const { return op & bit_rem; }
      /// Whether the operation is a shift
      constexpr bool is_shift() const { return op & bit_shift; }

      constexpr unsigned index() const { return op & index_mask; }

      bool operator==(const IntBinaryOp &o) const { return op == o.op; }
    };

    std::array<SymRef, static_cast<size_t>(LibFunc::MAX)> libfunc_syms;

    bool compile(ModuleTpde &mod);

    bool compile_inst(RustAdaptor::IRInstRef, InstRange);

    bool compile_unknown(RustAdaptor::IRInstRef inst, const ValInfo &, u64) {
      Instruction& instr = this->adaptor->get_instruction(inst);
      assert(false);
    }

    bool compile_int_binary_op(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_int_binary_op_i128(RustAdaptor::IRInstRef, const ValInfo &, IntBinaryOp);

    bool compile_float_binary_op(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_overflowable(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_overflow(Instruction &, Instruction &);

    bool compile_saturating_intrin(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_ret(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_br(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_switch(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_select(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_unreachable(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_gep(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_store(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_store_generic(Instruction &, GenericValuePart &&);
    bool compile_store_atomic(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_load(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_load_generic(Instruction &, GenericValuePart &&);
    bool compile_load_atomic(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_memcpy(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_memmove(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_memset(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_memcmp(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_call(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_invoke(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_landing_pad(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_resume(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_cast(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_int_ext(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_int_trunc(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_float_ext_trunc(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_float_to_int(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_int_to_float(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_neg(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_fneg(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_not(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_fcmp(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_ctpop(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_ct_lz_tz(RustAdaptor::IRInstRef, const ValInfo &, u64);

    u64 const_vector_elem(IRValueRef vec, unsigned idx);
    void extract_element(ValueRef &vec_vr, unsigned idx, Type ty, ValuePart &out);
    void insert_element(ValueRef &vec_vr, unsigned idx, Type ty, GenericValuePart &&el);
    bool compile_extract_element(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_insert_element(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_shuffle_vector(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_icmp_vector(Instruction &);
    bool compile_fmuladd(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_vector_reduce(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_abort(RustAdaptor::IRInstRef, const ValInfo &, u64);

    bool compile_cmpxchg(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_atomicrmw(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_fence(RustAdaptor::IRInstRef, const ValInfo &, u64);

    SymRef get_libfunc_sym(LibFunc func);

    bool hook_post_func_sym_init();
  };

  #define DEFINE_U64_ENUM(Name, ...) \
    struct Name { \
        enum Value : u64 { __VA_ARGS__ }; \
        Value v = static_cast<Value>(0); \
        constexpr Name() = default; \
        constexpr Name(Value val) : v(val) {} \
        constexpr operator Value() const { return v; } \
    };

  DEFINE_U64_ENUM(FloatBinaryOp, add, sub, mul, div, rem)
  DEFINE_U64_ENUM(FloatCmpOp, OEQ, OGT, OGE, OLT, OLE, ONE, ORD, UNO, UEQ, UGT, UGE, ULT, ULE, UNE)
  DEFINE_U64_ENUM(OverflowOp, uadd, sadd, usub, ssub, umul, smul)
  DEFINE_U64_ENUM(ReduceOp, add, mul, land, lor, lxor, fadd, fmul)

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_to_elf(
    ModuleTpde &mod, std::vector<uint8_t> &buf) {
    if (this->adaptor->mod) {
      Base::derived()->reset();
    }
    if (!compile(mod)) {
      return false;
    }

    buf = this->assembler.build_object_file();
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  RustCompilerBase<Adaptor, Derived, Config>::SymRef
  RustCompilerBase<Adaptor, Derived, Config>::cur_personality_func() const {
    // TODO
    return {};
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile(ModuleTpde &mod) {
    if (!this->adaptor->switch_module(mod)) {
      return false;
    }

    if (!Base::compile()) {
      return false;
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_inst(RustAdaptor::IRInstRef inst_ref, InstRange) {
    TPDE_LOG_TRACE("Compiling inst {}", this->adaptor->inst_fmt_ref(inst_ref));
    static constexpr auto fns = []() constexpr {
      using CompileFn =
          bool (Derived::*)(RustAdaptor::IRInstRef, const ValInfo &, u64);
      std::array<std::pair<CompileFn, u64>, static_cast<size_t>(InstructionKind::Last)> res{};
      res.fill({&Derived::compile_unknown, 0});

      auto set_fn = [&](InstructionKind kind, CompileFn fn, u64 val = 0) {
        res[static_cast<std::size_t>(kind)] = {fn, val};
      };

      // Int math
      set_fn(InstructionKind::Add, &Derived::compile_overflowable, IntBinaryOp::add);
      set_fn(InstructionKind::Sub, &Derived::compile_overflowable, IntBinaryOp::sub);
      set_fn(InstructionKind::Mul, &Derived::compile_overflowable, IntBinaryOp::mul);
      set_fn(InstructionKind::uDiv, &Derived::compile_int_binary_op, IntBinaryOp::udiv);
      set_fn(InstructionKind::sDiv, &Derived::compile_int_binary_op, IntBinaryOp::sdiv);
      set_fn(InstructionKind::uRem, &Derived::compile_int_binary_op, IntBinaryOp::urem);
      set_fn(InstructionKind::sRem, &Derived::compile_int_binary_op, IntBinaryOp::srem);
      set_fn(InstructionKind::Neg, &Derived::compile_neg);

      // Float math
      set_fn(InstructionKind::fAdd, &Derived::compile_float_binary_op, FloatBinaryOp::add);
      set_fn(InstructionKind::fSub, &Derived::compile_float_binary_op, FloatBinaryOp::sub);
      set_fn(InstructionKind::fMul, &Derived::compile_float_binary_op, FloatBinaryOp::mul);
      set_fn(InstructionKind::fDiv, &Derived::compile_float_binary_op, FloatBinaryOp::div);
      set_fn(InstructionKind::fRem, &Derived::compile_float_binary_op, FloatBinaryOp::rem);
      set_fn(InstructionKind::fNeg, &Derived::compile_fneg);

      // Logic
      set_fn(InstructionKind::And, &Derived::compile_int_binary_op, IntBinaryOp::land);
      set_fn(InstructionKind::Or, &Derived::compile_int_binary_op, IntBinaryOp::lor);
      set_fn(InstructionKind::Shl, &Derived::compile_int_binary_op, IntBinaryOp::shl);
      set_fn(InstructionKind::lShr, &Derived::compile_int_binary_op, IntBinaryOp::shr);
      set_fn(InstructionKind::aShr, &Derived::compile_int_binary_op, IntBinaryOp::ashr);
      set_fn(InstructionKind::Xor, &Derived::compile_int_binary_op, IntBinaryOp::lxor);
      set_fn(InstructionKind::Not, &Derived::compile_not);

      set_fn(InstructionKind::Ret, &Derived::compile_ret);

      // Int cmp
      set_fn(InstructionKind::CMPeq, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPne, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPult, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPule, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPugt, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPuge, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPslt, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPsle, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPsgt, &Derived::compile_cmp);
      set_fn(InstructionKind::CMPsge, &Derived::compile_cmp);

      // Float cmp
      set_fn(InstructionKind::RealOEQ, &Derived::compile_fcmp, FloatCmpOp::OEQ);
      set_fn(InstructionKind::RealOGT, &Derived::compile_fcmp, FloatCmpOp::OGT);
      set_fn(InstructionKind::RealOGE, &Derived::compile_fcmp, FloatCmpOp::OGE);
      set_fn(InstructionKind::RealOLT, &Derived::compile_fcmp, FloatCmpOp::OLT);
      set_fn(InstructionKind::RealOLE, &Derived::compile_fcmp, FloatCmpOp::OLE);
      set_fn(InstructionKind::RealONE, &Derived::compile_fcmp, FloatCmpOp::ONE);
      set_fn(InstructionKind::RealORD, &Derived::compile_fcmp, FloatCmpOp::ORD);
      set_fn(InstructionKind::RealUNO, &Derived::compile_fcmp, FloatCmpOp::UNO);
      set_fn(InstructionKind::RealUEQ, &Derived::compile_fcmp, FloatCmpOp::UEQ);
      set_fn(InstructionKind::RealUGT, &Derived::compile_fcmp, FloatCmpOp::UGT);
      set_fn(InstructionKind::RealUGE, &Derived::compile_fcmp, FloatCmpOp::UGE);
      set_fn(InstructionKind::RealULT, &Derived::compile_fcmp, FloatCmpOp::ULT);
      set_fn(InstructionKind::RealULE, &Derived::compile_fcmp, FloatCmpOp::ULE);
      set_fn(InstructionKind::RealUNE, &Derived::compile_fcmp, FloatCmpOp::UNE);

      set_fn(InstructionKind::GEP, &Derived::compile_gep);
      set_fn(InstructionKind::Store, &Derived::compile_store);
      set_fn(InstructionKind::Load, &Derived::compile_load);
      set_fn(InstructionKind::MemCpy, &Derived::compile_memcpy);
      set_fn(InstructionKind::MemMove, &Derived::compile_memmove);
      set_fn(InstructionKind::MemSet, &Derived::compile_memset);
      set_fn(InstructionKind::MemCmp, &Derived::compile_memcmp);
      set_fn(InstructionKind::Call, &Derived::compile_call);
      set_fn(InstructionKind::Invoke, &Derived::compile_invoke);
      set_fn(InstructionKind::LandingPad, &Derived::compile_landing_pad);
      set_fn(InstructionKind::Resume, &Derived::compile_resume);

      set_fn(InstructionKind::CondBr, &Derived::compile_condbr);
      set_fn(InstructionKind::Br, &Derived::compile_br);

      set_fn(InstructionKind::Switch, &Derived::compile_switch);
      set_fn(InstructionKind::Select, &Derived::compile_select);

      set_fn(InstructionKind::Cast, &Derived::compile_cast);
      set_fn(InstructionKind::Trunc, &Derived::compile_int_trunc);
      set_fn(InstructionKind::zExt, &Derived::compile_int_ext, /*sign=*/false);
      set_fn(InstructionKind::sExt, &Derived::compile_int_ext, /*sign=*/true);
      set_fn(InstructionKind::fTrunc, &Derived::compile_float_ext_trunc);
      set_fn(InstructionKind::fExt, &Derived::compile_float_ext_trunc);
      set_fn(InstructionKind::fTou, &Derived::compile_float_to_int, /*flags=sign,!sat*/0);
      set_fn(InstructionKind::fTos, &Derived::compile_float_to_int, /*flags=!sign,!sat*/1);
      set_fn(InstructionKind::fTou_sat, &Derived::compile_float_to_int, /*flags=!sign|sat*/0b10);
      set_fn(InstructionKind::fTos_sat, &Derived::compile_float_to_int, /*flags=sign|sat*/0b11);
      set_fn(InstructionKind::uTof, &Derived::compile_int_to_float, /*sign=*/false);
      set_fn(InstructionKind::sTof, &Derived::compile_int_to_float, /*sign=*/true);

      set_fn(InstructionKind::ctpop, &Derived::compile_ctpop);
      set_fn(InstructionKind::ctlz, &Derived::compile_ct_lz_tz, /*flags=leading*/0b00);
      set_fn(InstructionKind::ctlz_nonzero, &Derived::compile_ct_lz_tz, /*flags=leading,nonzero*/0b01);
      set_fn(InstructionKind::cttz, &Derived::compile_ct_lz_tz, /*flags=trailing*/0b10);
      set_fn(InstructionKind::cttz_nonzero, &Derived::compile_ct_lz_tz, /*flags=trailing,nonzero*/0b11);
      set_fn(InstructionKind::sat_sadd, &Derived::compile_saturating_intrin, OverflowOp::sadd);
      set_fn(InstructionKind::sat_uadd, &Derived::compile_saturating_intrin, OverflowOp::uadd);
      set_fn(InstructionKind::sat_ssub, &Derived::compile_saturating_intrin, OverflowOp::ssub);
      set_fn(InstructionKind::sat_usub, &Derived::compile_saturating_intrin, OverflowOp::usub);

      set_fn(InstructionKind::Atomic_load, &Derived::compile_load_atomic);
      set_fn(InstructionKind::Atomic_store, &Derived::compile_store_atomic);
      set_fn(InstructionKind::Atomic_cmpxchg, &Derived::compile_cmpxchg);
      set_fn(InstructionKind::Atomic_rmw, &Derived::compile_atomicrmw);
      set_fn(InstructionKind::Atomic_fence, &Derived::compile_fence);

      set_fn(InstructionKind::Abort, &Derived::compile_abort);
      set_fn(InstructionKind::Pause, &Derived::compile_pause);

      return res;
    }();

    Instruction *instr = &this->adaptor->get_instruction(inst_ref);
    const ValInfo val_info = this->adaptor->val_info(instr);
    assert(static_cast<size_t>(instr->kind) < fns.size());
    const auto [compile_fn, arg] = fns[static_cast<std::size_t>(instr->kind)];
    if (! (derived()->*compile_fn)(inst_ref, val_info, arg) ) {
      return false;
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_ret(
    RustAdaptor::IRInstRef instr_ref, const ValInfo &, u64) {
    Instruction *instr = &this->adaptor->get_instruction(instr_ref);

    typename Base::RetBuilder rb{*derived(), *derived()->cur_cc_assigner()};
    if (!instr->ops.empty()) {
      for (auto op: instr->ops) {
        rb.add(op);
      }
    }
    rb.ret();
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_int_binary_op(
    RustAdaptor::IRInstRef instr_ref, const ValInfo &info, u64 op_val) {
    Instruction *instr = &this->adaptor->get_instruction(instr_ref);

    IntBinaryOp op = typename IntBinaryOp::Value(op_val);
    auto parts = this->adaptor->val_parts(info);

    if (info.type == Type::i128) [[unlikely]] {
      return compile_int_binary_op_i128(instr_ref, info, op);
    }

    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&, ValuePart &);
    static constexpr auto fns = []() constexpr {
      std::array<EncodeFnTy[12], IntBinaryOp::num_ops> res{};
      auto entry = [&res](IntBinaryOp op) { return res[op.index()]; };

#define FN_ENTRY_INT(op, fn)                                                   \
    entry(op)[1] = &Derived::encode_##fn##i##32;                                 \
    entry(op)[2] = &Derived::encode_##fn##i##64;
#define FN_ENTRY_VEC(op, fn, sign)                                             \
    entry(op)[3] = &Derived::encode_##fn##v8##sign##8;                           \
    entry(op)[4] = &Derived::encode_##fn##v4##sign##16;                          \
    entry(op)[5] = &Derived::encode_##fn##v2##sign##32;                          \
    entry(op)[6] = &Derived::encode_##fn##v16##sign##8;                          \
    entry(op)[7] = &Derived::encode_##fn##v8##sign##16;                          \
    entry(op)[8] = &Derived::encode_##fn##v4##sign##32;                          \
    entry(op)[9] = &Derived::encode_##fn##v2##sign##64;
#define FN_ENTRY(op, fn, sign) FN_ENTRY_INT(op, fn) FN_ENTRY_VEC(op, fn, sign)

      FN_ENTRY(IntBinaryOp::add, add, u)
      FN_ENTRY(IntBinaryOp::sub, sub, u)
      FN_ENTRY(IntBinaryOp::mul, mul, u)
      FN_ENTRY_INT(IntBinaryOp::udiv, udiv)
      FN_ENTRY_INT(IntBinaryOp::sdiv, sdiv)
      FN_ENTRY_INT(IntBinaryOp::urem, urem)
      FN_ENTRY_INT(IntBinaryOp::srem, srem)
      FN_ENTRY(IntBinaryOp::land, land, u)
      FN_ENTRY(IntBinaryOp::lxor, lxor, u)
      FN_ENTRY(IntBinaryOp::lor, lor, u)
      FN_ENTRY(IntBinaryOp::shl, shl, u)
      FN_ENTRY(IntBinaryOp::shr, shr, u)
      FN_ENTRY(IntBinaryOp::ashr, ashr, i)
#undef FN_ENTRY
#undef FN_ENTRY_VEC
#undef FN_ENTRY_INT

      // i1 is special.
      entry(IntBinaryOp::add)[10] = &Derived::encode_lxori32;
      entry(IntBinaryOp::add)[11] = &Derived::encode_lxori64;
      entry(IntBinaryOp::sub)[10] = &Derived::encode_lxori32;
      entry(IntBinaryOp::sub)[11] = &Derived::encode_lxori64;
      entry(IntBinaryOp::mul)[10] = &Derived::encode_landi32;
      entry(IntBinaryOp::mul)[11] = &Derived::encode_landi64;
      // udiv: x/1 = x; x/0 = UB => and is equivalent
      entry(IntBinaryOp::udiv)[10] = &Derived::encode_landi32;
      entry(IntBinaryOp::udiv)[11] = &Derived::encode_landi64;
      // sdiv: 0/-1 = 0; -1/-1 = UB; x/0 = UB => and is equivalent
      entry(IntBinaryOp::sdiv)[10] = &Derived::encode_landi32;
      entry(IntBinaryOp::sdiv)[11] = &Derived::encode_landi64;
      // urem/srem are always zero, but we have no encode function to return zero.
      // For now, keep them unassigned.
      entry(IntBinaryOp::land)[10] = &Derived::encode_landi32;
      entry(IntBinaryOp::land)[11] = &Derived::encode_landi64;
      entry(IntBinaryOp::lxor)[10] = &Derived::encode_lxori32;
      entry(IntBinaryOp::lxor)[11] = &Derived::encode_lxori64;
      entry(IntBinaryOp::lor)[10] = &Derived::encode_lori32;
      entry(IntBinaryOp::lor)[11] = &Derived::encode_lori64;
      // shl/lshr/ashr are always poison, so we could use any operation... for
      // now, keep them unassigned.

      return res;
    }();
    auto get_encode_fn =
        [op](Type bvt) -> std::pair<EncodeFnTy, bool> {
      static constexpr auto bvt_lut = []() consteval {
        using enum Type;
        std::array<u8, static_cast<unsigned>(Last)> res{};
        res[unsigned(Bool)] = 1;
        res[unsigned(i8)] = 1;
        res[unsigned(i16)] = 1;
        res[unsigned(i32)] = 1;
        res[unsigned(i64)] = 2;
        res[unsigned(v8i8)] = 3;
        res[unsigned(v4i16)] = 4;
        res[unsigned(v2i32)] = 5;
        res[unsigned(v16i8)] = 6;
        res[unsigned(v8i16)] = 7;
        res[unsigned(v4i32)] = 8;
        res[unsigned(v2i64)] = 9;
        return res;
      }();
      unsigned ty_idx = bvt_lut[unsigned(bvt)];
      return {fns[op.index()][ty_idx], ty_idx < 3};
    };

    const Type res_ty = Base::adaptor->type_of_ref(instr->result);

    unsigned int_width = size_of_type(is_vector(res_ty) ? vector_info(res_ty).second : res_ty);
    const auto &operands = instr->ops;
    ValueRef lhs = this->val_ref(operands[0]);
    ValueRef rhs = this->val_ref(operands[1]);
    ValueRef res = this->result_ref(instr->result);

    auto handle_part = [this, int_width, op](EncodeFnTy encode_fn,
                                             bool is_scalar,
                                             ValuePartRef &&lhs_op,
                                             ValuePartRef &&rhs_op,
                                             ValuePartRef &res_op) {
      if (is_scalar) {
        if (op.is_symmetric() && lhs_op.is_const() && !rhs_op.is_const()) {
          // TODO(ts): this is a hack since the encoder can currently not do
          // commutable operations so we reorder immediates manually here
          std::swap(lhs_op, rhs_op);
        }

        // TODO(ts): optimize div/rem by constant to a shift?
        unsigned ext_width = tpde::util::align_up(int_width, 32);
        if (ext_width != int_width) {
          bool sext = op.is_signed();
          if (op.needs_lhs_ext()) {
            lhs_op = std::move(lhs_op).into_extended(sext, int_width, ext_width);
          }
          if (op.needs_rhs_ext()) {
            rhs_op = std::move(rhs_op).into_extended(sext, int_width, ext_width);
          }
        }
      }

      (derived()->*encode_fn)(std::move(lhs_op), std::move(rhs_op), res_op);
    };

    for (u32 i = 0, n = parts.count(); i != n; ++i) {
      const Type ty = parts.type(i);
      ValuePartRef res_part = res.part(i);
      if (auto [encode_fn, is_scalar] = get_encode_fn(ty); encode_fn) [[likely]] {
        handle_part(encode_fn, is_scalar, lhs.part(i), rhs.part(i), res_part);
        continue;
      }

      // This is a legal vector type for which we don't have an encode function.
      // Extract elements individually and use scalar functions.
      if (!is_vector(ty)) {
        return false;
      }

      auto [elem_cnt, elem_ty] = vector_info(ty);
      auto [elem_encode_fn, is_scalar] = get_encode_fn(elem_ty);
      assert(is_scalar && "vector element must be a scalar type");
      if (!elem_encode_fn) {
        return false;
      }

      tpde::RegBank bank = reg_bank_of_type(elem_ty);
      for (u32 j = 0; j != elem_cnt; ++j) {
        u32 elem_idx = i * elem_cnt + j;
        ValuePartRef e_res{this, bank};
        ValuePartRef e_lhs{this, bank};
        ValuePartRef e_rhs{this, bank};
        // TODO: we might pass the last element as owned. But this code is
        // fallback only, so don't bother optimizing.
        ValueRef lhs_unowned = lhs.disowned();
        ValueRef rhs_unowned = rhs.disowned();
        derived()->extract_element(lhs_unowned, elem_idx, elem_ty, e_lhs);
        derived()->extract_element(rhs_unowned, elem_idx, elem_ty, e_rhs);
        handle_part(elem_encode_fn, true, std::move(e_lhs), std::move(e_rhs), e_res);
        // insert_element always treats res as unowned.
        derived()->insert_element(res, elem_idx, elem_ty, std::move(e_res));
      }
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_int_binary_op_i128(RustAdaptor::IRInstRef ref, const ValInfo &,
    IntBinaryOp op) {
    const Instruction& inst = this->adaptor->get_instruction(ref);

    const unsigned int_width = size_of_type(this->adaptor->type_of_ref(inst.result));
    assert(int_width > 64 && int_width <= 128);
    auto lhs_op = inst.ops[0];
    auto rhs_op = inst.ops[1];

    auto res = this->result_ref(inst.result);

    if (op.is_div() || op.is_rem()) {
      LibFunc lf;
      if (op.is_div()) {
        lf = op.is_signed() ? LibFunc::divti3 : LibFunc::udivti3;
      } else {
        lf = op.is_signed() ? LibFunc::modti3 : LibFunc::umodti3;
      }

      if (int_width != 128) {
        return false; // TODO: extend parameters
      }

      std::array<IRValueRef, 2> args{lhs_op, rhs_op};
      derived()->create_helper_call(args, &res, get_libfunc_sym(lf));
      return true;
    }

    auto lhs = this->val_ref(lhs_op);
    auto rhs = this->val_ref(rhs_op);

    // Use has_assignment as proxy for not being a constant.
    if (op.is_symmetric() && !lhs.has_assignment() && rhs.has_assignment()) {
      // TODO(ts): this is a hack since the encoder can currently not do
      // commutable operations so we reorder immediates manually here
      std::swap(lhs, rhs);
    }

    if (op.is_shift()) {
      ValuePartRef lhs_hi = lhs.part(1);
      if (int_width != 128 && op.needs_lhs_ext()) {
        bool sext = op.is_signed(); // Essentially just ashr.
        lhs_hi = std::move(lhs_hi).into_extended(sext, int_width - 64, 64);
      }

      ValuePartRef shift_amt = rhs.part(0);
      if (shift_amt.is_const()) {
        u64 imm1 = shift_amt.const_data()[0] & 0b111'1111; // amt
        if (imm1 < 64) {
          u64 imm2 = (64 - imm1) & 0b11'1111; // iamt
          if (op == IntBinaryOp::shl) {
            derived()->encode_shli128_lt64(
                lhs.part(0),
                std::move(lhs_hi),
                ValuePartRef(this, imm1, 1, Config::GP_BANK),
                ValuePartRef(this, imm2, 1, Config::GP_BANK),
                res.part(0),
                res.part(1));
          } else if (op == IntBinaryOp::shr) {
            derived()->encode_shri128_lt64(
                lhs.part(0),
                std::move(lhs_hi),
                ValuePartRef(this, imm1, 1, Config::GP_BANK),
                ValuePartRef(this, imm2, 1, Config::GP_BANK),
                res.part(0),
                res.part(1));
          } else {
            assert(op == IntBinaryOp::ashr);
            derived()->encode_ashri128_lt64(
                lhs.part(0),
                std::move(lhs_hi),
                ValuePartRef(this, imm1, 1, Config::GP_BANK),
                ValuePartRef(this, imm2, 1, Config::GP_BANK),
                res.part(0),
                res.part(1));
          }
        } else if (imm1 == 64) {
          // For shifts by 64, we just need to move one part to another.
          if (op == IntBinaryOp::shl) {
            res.part(0).set_value(ValuePartRef(this, 0, 8, Config::GP_BANK));
            res.part(1).set_value(lhs.part(0));
          } else if (op == IntBinaryOp::shr) {
            res.part(0).set_value(std::move(lhs_hi));
            res.part(1).set_value(ValuePartRef(this, 0, 8, Config::GP_BANK));
          } else {
            assert(op == IntBinaryOp::ashr);
            (void)lhs_hi.cur_reg_or_load(); // Force into reg for get_unowned_ref.
            derived()->encode_fill_with_sign64(lhs_hi.get_unowned_ref(),
                                               res.part(1));
            res.part(0).set_value(std::move(lhs_hi));
          }
        } else {
          imm1 -= 64;
          if (op == IntBinaryOp::shl) {
            derived()->encode_shli128_ge64(
                lhs.part(0),
                ValuePartRef(this, imm1, 1, Config::GP_BANK),
                res.part(0),
                res.part(1));
          } else if (op == IntBinaryOp::shr) {
            derived()->encode_shri128_ge64(
                std::move(lhs_hi),
                ValuePartRef(this, imm1, 1, Config::GP_BANK),
                res.part(0),
                res.part(1));
          } else {
            assert(op == IntBinaryOp::ashr);
            derived()->encode_ashri128_ge64(
                std::move(lhs_hi),
                ValuePartRef(this, imm1, 1, Config::GP_BANK),
                res.part(0),
                res.part(1));
          }
        }
      } else {
        if (op == IntBinaryOp::shl) {
          derived()->encode_shli128(lhs.part(0),
                                    std::move(lhs_hi),
                                    std::move(shift_amt),
                                    res.part(0),
                                    res.part(1));
        } else if (op == IntBinaryOp::shr) {
          derived()->encode_shri128(lhs.part(0),
                                    std::move(lhs_hi),
                                    std::move(shift_amt),
                                    res.part(0),
                                    res.part(1));
        } else {
          assert(op == IntBinaryOp::ashr);
          derived()->encode_ashri128(lhs.part(0),
                                     std::move(lhs_hi),
                                     std::move(shift_amt),
                                     res.part(0),
                                     res.part(1));
        }
      }
    } else {
      using EncodeFnTy = bool (Derived::*)(GenericValuePart &&,
                                           GenericValuePart &&,
                                           GenericValuePart &&,
                                           GenericValuePart &&,
                                           ValuePart &&,
                                           ValuePart &&);
      static const std::array<EncodeFnTy, 10> encode_ptrs = {
          {
           &Derived::encode_addi128,
           &Derived::encode_subi128,
           &Derived::encode_muli128,
           nullptr, // division/remainder is a libcall
              nullptr, // division/remainder is a libcall
              nullptr, // division/remainder is a libcall
              nullptr, // division/remainder is a libcall
              &Derived::encode_landi128,
           &Derived::encode_lori128,
           &Derived::encode_lxori128,
           }
      };

      (derived()->*(encode_ptrs[op.index()]))(lhs.part(0),
                                              lhs.part(1),
                                              rhs.part(0),
                                              rhs.part(1),
                                              res.part(0),
                                              res.part(1));
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_float_binary_op(
    RustAdaptor::IRInstRef inst, const ValInfo &val_info, u64 op) {
    Instruction &finstr = this->adaptor->get_instruction(inst);

    auto lhs = this->val_ref(finstr.ops[0]);
    auto rhs = this->val_ref(finstr.ops[1]);
    ValueRef res = this->result_ref(finstr.result);

    if (op == FloatBinaryOp::rem) {
      LibFunc lf;
      switch (val_info.type) {
        using enum Type;
        case f32: lf = LibFunc::fmodf;
          break;
        case f64: lf = LibFunc::fmod;
          break;
        default: return false;
      }

      auto cb = derived()->create_call_builder();
      cb->add_arg(lhs.part(0), tpde::CCAssignment{});
      cb->add_arg(rhs.part(0), tpde::CCAssignment{});
      cb->call(get_libfunc_sym(lf));
      cb->add_ret(res);
      return true;
    }

    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&, ValuePart &&);
    EncodeFnTy encode_fn = nullptr;

    switch (val_info.type) {
      using enum Type;
      case f32:
        switch (op) {
          using enum FloatBinaryOp::Value;
          case add: encode_fn = &Derived::encode_addf32;
            break;
          case sub: encode_fn = &Derived::encode_subf32;
            break;
          case mul: encode_fn = &Derived::encode_mulf32;
            break;
          case div: encode_fn = &Derived::encode_divf32;
            break;
          default: TPDE_UNREACHABLE("invalid FloatBinaryOp");
        }
        break;
      case f64:
        switch (op) {
          using enum FloatBinaryOp::Value;
          case add: encode_fn = &Derived::encode_addf64;
            break;
          case sub: encode_fn = &Derived::encode_subf64;
            break;
          case mul: encode_fn = &Derived::encode_mulf64;
            break;
          case div: encode_fn = &Derived::encode_divf64;
            break;
          default: TPDE_UNREACHABLE("invalid FloatBinaryOp");
        }
        break;
      case v2f32:
        switch (op) {
          using enum FloatBinaryOp::Value;
          case add: encode_fn = &Derived::encode_addv2f32;
          break;
          case sub: encode_fn = &Derived::encode_subv2f32;
          break;
          case mul: encode_fn = &Derived::encode_mulv2f32;
          break;
          case div: encode_fn = &Derived::encode_divv2f32;
          break;
          default: TPDE_UNREACHABLE("invalid FloatBinaryOp");
        }
        break;
      case v4f32:
        switch (op) {
          using enum FloatBinaryOp::Value;
          case add: encode_fn = &Derived::encode_addv4f32;
          break;
          case sub: encode_fn = &Derived::encode_subv4f32;
          break;
          case mul: encode_fn = &Derived::encode_mulv4f32;
          break;
          case div: encode_fn = &Derived::encode_divv4f32;
          break;
          default: TPDE_UNREACHABLE("invalid FloatBinaryOp");
        }
        break;
      case v2f64:
        switch (op) {
          using enum FloatBinaryOp::Value;
          case add: encode_fn = &Derived::encode_addv2f64;
          break;
          case sub: encode_fn = &Derived::encode_subv2f64;
          break;
          case mul: encode_fn = &Derived::encode_mulv2f64;
          break;
          case div: encode_fn = &Derived::encode_divv2f64;
          break;
          default: TPDE_UNREACHABLE("invalid FloatBinaryOp");
          break;
        }
        break;
      default: return false;
    }

    return (derived()->*encode_fn)(lhs.part(0), rhs.part(0), res.part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_overflowable(RustAdaptor::IRInstRef instr,
                                                                        const ValInfo &info, u64 op) {
    auto instr_size = this->adaptor->get_basic_block(instr.block).instructions.size();
    if (instr_size > instr.inst) {
      Instruction &pot_overflow = this->adaptor->get_instruction(instr.next());
      if (pot_overflow.kind == InstructionKind::OverflowCheck) {
        Instruction &inst = this->adaptor->get_instruction(instr);
        if (!compile_overflow(inst, pot_overflow)) {
          return false;
        }

        if (instr_size > instr.next().inst) {
          Instruction &pot_condbr = this->adaptor->get_instruction(instr.next().next());
          if (pot_condbr.kind == InstructionKind::CondBr && pot_overflow.result == pot_condbr.ops[0]) {
            assert(this->analyzer.liveness_info(this->adaptor->val_local_idx(pot_overflow.result)).ref_count == 2);
            this->adaptor->next_fused = true;
            // We can drop the register used for the overflow check
            this->val_ref(pot_overflow.result).reset();

            bool is_signed = operands::content(pot_overflow.ops[0]);
            return derived()->compile_overflow_jump(pot_condbr, inst.kind, is_signed);
          }
        }
        return true;
      }
    }

    return this->compile_int_binary_op(instr, info, op);
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_overflow(Instruction &op_instr, Instruction &of_instr) {
    ValueRef lhs = this->val_ref(op_instr.ops[0]);
    ValueRef rhs = this->val_ref(op_instr.ops[1]);
    ValueRef res = this->result_ref(op_instr.result);
    ValueRef of_res = this->result_ref(of_instr.result);

    OverflowOp op;
    bool is_signed = operands::content(of_instr.ops[0]);
    switch (op_instr.kind) {
      case InstructionKind::Add:
        op = is_signed ? OverflowOp::sadd : OverflowOp::uadd;
        break;
      case InstructionKind::Sub:
        op = is_signed ? OverflowOp::ssub : OverflowOp::usub;
        break;
      case InstructionKind::Mul:
        op = is_signed ? OverflowOp::smul : OverflowOp::umul;
        break;
      default:
        assert(false && "Only support integer types");
    }

    const Type ty = this->adaptor->type_of_ref(op_instr.ops[0]);
    switch (ty) {
      using enum Type;
      case i8:
      case i16:
      case i32:
      case i64:
      case i128:
        break;
      default:
        assert(false && "Only support integer types");
    }
    const auto width = size_of_type(ty);

    if (width == 128) {
      if (!derived()->handle_overflow_intrin_128(op,
                                                       lhs.part(0),
                                                       lhs.part(1),
                                                       rhs.part(0),
                                                       rhs.part(1),
                                                       res.part(0),
                                                       res.part(1),
                                                       of_res.part(0))) {
        return false;
      }
      return true;
    }

    u32 width_idx = 0;
    switch (width) {
      case 8: width_idx = 0;
        break;
      case 16: width_idx = 1;
        break;
      case 32: width_idx = 2;
        break;
      case 64: width_idx = 3;
        break;
      default: return false;
    }

    using EncodeFnTy = bool (Derived::*)(
      GenericValuePart &&, GenericValuePart &&, ValuePart &&, ValuePart &&);
    std::array<std::array<EncodeFnTy, 4>, 6> encode_fns = {
      {
        {
          &Derived::encode_of_add_u8,
          &Derived::encode_of_add_u16,
          &Derived::encode_of_add_u32,
          &Derived::encode_of_add_u64
        },
        {
          &Derived::encode_of_add_i8,
          &Derived::encode_of_add_i16,
          &Derived::encode_of_add_i32,
          &Derived::encode_of_add_i64
        },
        {
          &Derived::encode_of_sub_u8,
          &Derived::encode_of_sub_u16,
          &Derived::encode_of_sub_u32,
          &Derived::encode_of_sub_u64
        },
        {
          &Derived::encode_of_sub_i8,
          &Derived::encode_of_sub_i16,
          &Derived::encode_of_sub_i32,
          &Derived::encode_of_sub_i64
        },
        {
          &Derived::encode_of_mul_u8,
          &Derived::encode_of_mul_u16,
          &Derived::encode_of_mul_u32,
          &Derived::encode_of_mul_u64
        },
        {
          &Derived::encode_of_mul_i8,
          &Derived::encode_of_mul_i16,
          &Derived::encode_of_mul_i32,
          &Derived::encode_of_mul_i64
        },
      }
    };

    EncodeFnTy encode_fn = encode_fns[op][width_idx];
    (derived()->*encode_fn)(lhs.part(0), rhs.part(0), res.part(0), of_res.part(0));
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_saturating_intrin(RustAdaptor::IRInstRef inst_ref, const ValInfo &,
    u64 op) {
    Instruction& inst = this->adaptor->get_instruction(inst_ref);

    Type ty = this->adaptor->type_of_ref(inst.result);

    const auto width = size_of_type(ty);
    u32 width_idx = 0;
    switch (width) {
      case 8: width_idx = 0; break;
      case 16: width_idx = 1; break;
      case 32: width_idx = 2; break;
      case 64: width_idx = 3; break;
      default: return false;
    }

    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&, ValuePart &&);
    std::array<std::array<EncodeFnTy, 4>, 4> encode_fns{
      {
        {&Derived::encode_sat_add_u8,
         &Derived::encode_sat_add_u16,
         &Derived::encode_sat_add_u32,
         &Derived::encode_sat_add_u64},
        {&Derived::encode_sat_add_i8,
         &Derived::encode_sat_add_i16,
         &Derived::encode_sat_add_i32,
         &Derived::encode_sat_add_i64},
        {&Derived::encode_sat_sub_u8,
         &Derived::encode_sat_sub_u16,
         &Derived::encode_sat_sub_u32,
         &Derived::encode_sat_sub_u64},
        {&Derived::encode_sat_sub_i8,
         &Derived::encode_sat_sub_i16,
         &Derived::encode_sat_sub_i32,
         &Derived::encode_sat_sub_i64},
    }};

    EncodeFnTy encode_fn = encode_fns[static_cast<u32>(op)][width_idx];

    ValueRef lhs = this->val_ref(inst.ops[0]);
    ValueRef rhs = this->val_ref(inst.ops[1]);
    ValueRef res = this->result_ref(inst.result);
    return (derived()->*encode_fn)(lhs.part(0), rhs.part(0), res.part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_gep(
    RustAdaptor::IRInstRef inst, const ValInfo &, u64) {
    // operands:
    // base - ptr
    // scale - raw
    // index - val or const

    RustAdaptor::IRInstRef gep_ref = inst;
    Instruction *gep = &this->adaptor->get_instruction(inst);

    GenericValuePart addr = typename GenericValuePart::Expr{};
    {
      ValueRef index_vr{this};
      ValuePartRef index_vp{this};
      auto &expr = std::get<typename GenericValuePart::Expr>(addr.state);

      // Kept separate from expr.disp, we don't want to fold the displacement
      // whenever we add an index. Indices are sign-extended, but we must use an
      // unsigned integer here to correctly handle overflows.
      u64 displacement = 0;
      // If set, the base is actually a stack variable reference and expr.base is
      // still uninitialized.
      bool base_is_stack_var = false;

      auto [ptr_ref, base] = this->val_ref_single(gep->ops[0]);
      if (base.has_assignment() && base.assignment().is_stack_variable()) {
        base_is_stack_var = true;
      } else {
        expr.base = base.load_to_reg();
        if (base.can_salvage()) {
          expr.base = ScratchReg{this};
          std::get<ScratchReg>(expr.base).alloc_specific(base.salvage());
        }
      }

      // The instruction following the last fused GEP, if it might be fusable.
      {
        const u64 scale = operands::content(gep->ops[1]);
        assert(scale == 0 || scale == 1 || scale == 2 || scale == 4 || scale == 8);

        const IRValueRef idx = gep->ops[2];
        if (operands::is_const(idx)) {
          // Constant index: fold into the displacement.
          const Value &imm = this->adaptor->mod->consts[operands::content(idx)];
          displacement += static_cast<u64>(scale) * imm.data2;
        } else if (scale == 0) {
          // The index doesn't contribute anything, but we still have to
          // reference it for the reference counting to stay correct.
          (void) this->val_ref(idx);
        } else {
          if (base_is_stack_var) {
            addr = derived()->create_addr_for_alloca(base.assignment());
            assert(addr.is_expr());
            displacement += expr.disp;
            expr.disp = 0;
            base_is_stack_var = false;
          }

          if (expr.scale) {
            // We already have an index; materialize the current address
            // expression into a register and use it as the new base.
            derived()->gval_expr_as_reg(addr);
            index_vp.reset();
            index_vr.reset();
            base.reset();
            ptr_ref.reset();

            ScratchReg new_base = std::move(std::get<ScratchReg>(addr.state));
            addr = typename GenericValuePart::Expr{};
            expr.base = std::move(new_base);
          }

          const unsigned idx_width = size_of_type(this->adaptor->type_of_ref(idx));
          index_vr = this->val_ref(idx);
          if (idx_width != 64) {
            index_vp = index_vr.part(0).into_extended(true, idx_width, 64);
          } else {
            index_vp = index_vr.part(0);
          }
          if (index_vp.can_salvage()) {
            expr.index = ScratchReg{this};
            std::get<ScratchReg>(expr.index).alloc_specific(index_vp.salvage());
          } else {
            expr.index = index_vp.load_to_reg();
          }

          expr.scale = scale;
        }
      }

      Instruction* next_val = nullptr;
      while (true) {
        // The definition itself counts as one reference.
        const auto local_idx = this->adaptor->val_local_idx(gep->result);
        if (this->analyzer.liveness_info(local_idx).ref_count > 2)
          break;

        auto next_ref = gep_ref.next();
        const auto &insts = this->adaptor->get_basic_block(gep_ref.block).instructions;
        if (next_ref.inst >= insts.size())
          break;

        next_val = &this->adaptor->get_instruction(next_ref);
        break;
      }

      if (base_is_stack_var) {
        if (!next_val) {
          // Create a new stack variable reference to avoid materializing this
          // simple addition.
          (void) this->result_ref_stack_slot(
            gep->result, base.assignment(), displacement);
          return true;
        }

        addr = derived()->create_addr_for_alloca(base.assignment());
        expr.disp += displacement;
      } else {
        expr.disp = displacement;
      }

      if (next_val) {
        if (next_val->kind == InstructionKind::Store &&
            next_val->ops[1] == gep->result) {
          this->adaptor->next_fused = true;
          return compile_store_generic(*next_val, std::move(addr));
        }
        if (next_val->kind == InstructionKind::Load &&
            next_val->ops[0] == gep->result) {
          this->adaptor->next_fused = true;
          return compile_load_generic(*next_val, std::move(addr));
        }
      }
    }

    auto [res_vr, res_ref] = this->result_ref_single(gep->result);

    AsmReg res_reg = derived()->gval_expr_as_reg(addr);
    if (auto *op_reg = std::get_if<ScratchReg>(&addr.state)) {
      res_ref.set_value(std::move(*op_reg));
    } else {
      derived()->mov(res_ref.alloc_reg(), res_reg, 8);
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_store(
    RustAdaptor::IRInstRef inst, const ValInfo &, u64) {
    Instruction &storei = this->adaptor->get_instruction(inst);

    auto [_, ptr_ref] = this->val_ref_single(storei.ops[1]);
    if (ptr_ref.has_assignment() && ptr_ref.assignment().is_stack_variable()) {
      GenericValuePart addr =
          derived()->create_addr_for_alloca(ptr_ref.assignment());

      return compile_store_generic(storei, std::move(addr));
    }
    return compile_store_generic(storei, std::move(ptr_ref));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_store_generic(
    Instruction &storei, GenericValuePart &&ptr_op) {
    const auto op_val = storei.ops[0];
    auto op_ref = this->val_ref(op_val);

    Type ty = this->adaptor->type_of_ref(op_val);

    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&);
    static constexpr auto int_fns = []() consteval {
      std::array<EncodeFnTy, 8> res{};
      res[0] = &Derived::encode_storei8;
      res[1] = &Derived::encode_storei16;
      res[2] = &Derived::encode_storei24;
      res[3] = &Derived::encode_storei32;
      res[4] = &Derived::encode_storei40;
      res[5] = &Derived::encode_storei48;
      res[6] = &Derived::encode_storei56;
      res[7] = &Derived::encode_storei64;
      return res;
    }();

    switch (ty) {
      using enum Type;
      case Bool:
      case i8:
      case i16:
      case i32:
      case i64:
      case ptr: {
        const auto num_bytes = size_of_type(ty) / 8;
        EncodeFnTy fn = int_fns[num_bytes - 1];
        (derived()->*fn)(std::move(ptr_op), op_ref.part(0));
        return true;
      }
      case i128: {
        derived()->encode_storei128(std::move(ptr_op), op_ref.part(0), op_ref.part(1));
        return true;
      }
      case f32: {
        derived()->encode_storef32(std::move(ptr_op), op_ref.part(0));
        return true;
      }
      case f64: {
        derived()->encode_storef64(std::move(ptr_op), op_ref.part(0));
        return true;
      }

      default: return false;
    }
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_store_atomic(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction& instr = this->adaptor->get_instruction(inst_ref);

    auto [_, ptr_ref] = this->val_ref_single(instr.ops[1]);
    GenericValuePart addr;
    if (ptr_ref.has_assignment() && ptr_ref.assignment().is_stack_variable()) {
      addr = derived()->create_addr_for_alloca(ptr_ref.assignment());
    } else {
      addr = std::move(ptr_ref);
    }

    const Type ty = this->adaptor->type_of_ref(instr.ops[0]);
    u32 width = size_of_type(ty);
    assert(width == 8 || width == 16 || width == 32 || width == 64);

    const auto order = static_cast<AtomicOrdering>(operands::content(instr.ops[2]));
    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&);
    EncodeFnTy encode_fn = nullptr;
    if (order == AtomicOrdering::Monotonic) {
      switch (width) {
      case 8: encode_fn = &Derived::encode_atomic_store_u8_mono; break;
      case 16: encode_fn = &Derived::encode_atomic_store_u16_mono; break;
      case 32: encode_fn = &Derived::encode_atomic_store_u32_mono; break;
      case 64: encode_fn = &Derived::encode_atomic_store_u64_mono; break;
      default: TPDE_UNREACHABLE("invalid size");
      }
    } else if (order == AtomicOrdering::Release) {
      switch (width) {
      case 8: encode_fn = &Derived::encode_atomic_store_u8_rel; break;
      case 16: encode_fn = &Derived::encode_atomic_store_u16_rel; break;
      case 32: encode_fn = &Derived::encode_atomic_store_u32_rel; break;
      case 64: encode_fn = &Derived::encode_atomic_store_u64_rel; break;
      default: TPDE_UNREACHABLE("invalid size");
      }
    } else {
      assert(order == AtomicOrdering::SequentiallyConsistent);
      switch (width) {
      case 8: encode_fn = &Derived::encode_atomic_store_u8_seqcst; break;
      case 16: encode_fn = &Derived::encode_atomic_store_u16_seqcst; break;
      case 32: encode_fn = &Derived::encode_atomic_store_u32_seqcst; break;
      case 64: encode_fn = &Derived::encode_atomic_store_u64_seqcst; break;
      default: TPDE_UNREACHABLE("invalid size");
      }
    }

    auto op_ref = this->val_ref(instr.ops[0]);
    if (!(derived()->*encode_fn)(std::move(addr), op_ref.part(0))) {
      TPDE_LOG_ERR("fooooo");
      return false;
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_load(
    RustAdaptor::IRInstRef inst, const ValInfo &, u64) {
    Instruction &loadi = this->adaptor->get_instruction(inst);

    auto [_, ptr_ref] = this->val_ref_single(loadi.ops[0]);
    if (ptr_ref.has_assignment() && ptr_ref.assignment().is_stack_variable()) {
      GenericValuePart addr =
          derived()->create_addr_for_alloca(ptr_ref.assignment());

      return compile_load_generic(loadi, std::move(addr));
    }
    return compile_load_generic(loadi, std::move(ptr_ref));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_load_generic(
    Instruction &loadi, GenericValuePart &&ptr_op) {
    Type ty = this->adaptor->type_of_ref(loadi.result);

    using EncodeFnTy = bool (Derived::*)(GenericValuePart &&, ValuePart &&);
    static constexpr auto int_fns = []() consteval {
      std::array<EncodeFnTy[2], 8> res{};
      res[0][0] = &Derived::encode_loadi8_zext;
      res[0][1] = &Derived::encode_loadi8_sext;
      res[1][0] = &Derived::encode_loadi16_zext;
      res[1][1] = &Derived::encode_loadi16_sext;
      res[2][0] = &Derived::encode_loadi24;
      res[3][0] = &Derived::encode_loadi32_zext;
      res[3][1] = &Derived::encode_loadi32_sext;
      res[4][0] = &Derived::encode_loadi40;
      res[5][0] = &Derived::encode_loadi48;
      res[6][0] = &Derived::encode_loadi56;
      res[7][0] = &Derived::encode_loadi64;
      return res;
    }();

    bool sext = false;
    switch (ty) {
      using enum Type;
      case Bool:
      case i8:
      case i16:
      case i32:
      case i64:
      case ptr: {
        const auto num_bytes = size_of_type(ty) / 8;
        EncodeFnTy fn = int_fns[num_bytes - 1][sext];

        (derived()->*fn)(std::move(ptr_op), this->result_ref(loadi.result).part(0));
        return true;
      }
      case i128: {
        ValueRef res = this->result_ref(loadi.result);
        derived()->encode_loadi128(std::move(ptr_op), res.part(0), res.part(1));
        return true;
      }
      case f32: {
        derived()->encode_loadf32(std::move(ptr_op),
                                  this->result_ref(loadi.result).part(0));
        return true;
      }
      case v8i8:
      case v4i16:
      case v2i32:
      case v2f32:
      case f64: {
        derived()->encode_loadf64(std::move(ptr_op),
                                  this->result_ref(loadi.result).part(0));
        return true;
      }
      case v16i8:
      case v8i16:
      case v4i32:
      case v2i64:
      case v4f32:
      case v2f64: {
        derived()->encode_loadv128(std::move(ptr_op),
                                   this->result_ref(loadi.result).part(0));
        return true;
      }

      default: throw std::runtime_error("Unsupported type for loadi");
    }
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_load_atomic(RustAdaptor::IRInstRef instr_ref, const ValInfo &, u64) {
    Instruction& instr = this->adaptor->get_instruction(instr_ref);

    auto [_, ptr_ref] = this->val_ref_single(instr.ops[0]);
    GenericValuePart addr;
    if (ptr_ref.has_assignment() && ptr_ref.assignment().is_stack_variable()) {
      addr = derived()->create_addr_for_alloca(ptr_ref.assignment());
    } else {
      addr = std::move(ptr_ref);
    }

    const Type ty = this->adaptor->type_of_ref(instr.result);
    u32 width = size_of_type(ty);
    assert(width == 8 || width == 16 || width == 32 || width == 64);
    u32 needed_align = 1;
    switch (width) {
    case 16: needed_align = 2; break;
    case 32: needed_align = 4; break;
    case 64: needed_align = 8; break;
    }

    const auto order = static_cast<AtomicOrdering>(operands::content(instr.ops[1]));
    using EncodeFnTy = bool (Derived::*)(GenericValuePart &&, ValuePart &&);
    EncodeFnTy encode_fn = nullptr;
    if (order == AtomicOrdering::Monotonic) {
      switch (width) {
      case 8: encode_fn = &Derived::encode_atomic_load_u8_mono; break;
      case 16: encode_fn = &Derived::encode_atomic_load_u16_mono; break;
      case 32: encode_fn = &Derived::encode_atomic_load_u32_mono; break;
      case 64: encode_fn = &Derived::encode_atomic_load_u64_mono; break;
      default: TPDE_UNREACHABLE("invalid size");
      }
    } else if (order == AtomicOrdering::Acquire) {
      switch (width) {
      case 8: encode_fn = &Derived::encode_atomic_load_u8_acq; break;
      case 16: encode_fn = &Derived::encode_atomic_load_u16_acq; break;
      case 32: encode_fn = &Derived::encode_atomic_load_u32_acq; break;
      case 64: encode_fn = &Derived::encode_atomic_load_u64_acq; break;
      default: TPDE_UNREACHABLE("invalid size");
      }
    } else {
      assert(order == AtomicOrdering::SequentiallyConsistent);
      switch (width) {
      case 8: encode_fn = &Derived::encode_atomic_load_u8_seqcst; break;
      case 16: encode_fn = &Derived::encode_atomic_load_u16_seqcst; break;
      case 32: encode_fn = &Derived::encode_atomic_load_u32_seqcst; break;
      case 64: encode_fn = &Derived::encode_atomic_load_u64_seqcst; break;
      default: TPDE_UNREACHABLE("invalid size");
      }
    }

    ValueRef res = this->result_ref(instr.result);
    (derived()->*encode_fn)(std::move(addr), res.part(0));
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_memcpy(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction &inst = this->adaptor->get_instruction(inst_ref);

    const auto dst = inst.ops[0];
    const auto src = inst.ops[2];
    const auto len = inst.ops[4];

    std::array<IRValueRef, 3> args{dst, src, len};

    derived()->create_helper_call(args, nullptr, get_libfunc_sym(LibFunc::memcpy));
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_memmove(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction &inst = this->adaptor->get_instruction(inst_ref);

    const auto dst = inst.ops[0];
    const auto src = inst.ops[2];
    const auto len = inst.ops[4];

    std::array<IRValueRef, 3> args{dst, src, len};

    derived()->create_helper_call(args, nullptr, get_libfunc_sym(LibFunc::memmove));
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_memset(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction &inst = this->adaptor->get_instruction(inst_ref);

    const auto dst = inst.ops[0];
    const auto val = inst.ops[1];
    const auto len = inst.ops[2];

    std::array<IRValueRef, 3> args{dst, val, len};

    const auto sym = get_libfunc_sym(LibFunc::memset);
    derived()->create_helper_call(args, nullptr, sym);
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_memcmp(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction &inst = this->adaptor->get_instruction(inst_ref);

    const auto lhs = inst.ops[0];
    const auto rhs = inst.ops[1];
    const auto len = inst.ops[2];

    std::array<IRValueRef, 3> args{lhs, rhs, len};

    const auto sym = get_libfunc_sym(LibFunc::memcmp);
    auto res = this->result_ref(inst.result);
    derived()->create_helper_call(args, &res, sym);
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_call(RustAdaptor::IRInstRef instr, const ValInfo &, u64) {
    auto cb = derived()->create_call_builder();
    if (!cb) {
      return false;
    }

    Instruction &calli = this->adaptor->get_instruction(instr);
    const auto func = calli.ops[0];
    size_t arg_start = calli.kind == InstructionKind::Call ? 1 : 3;
    bool is_dynamic_function;
    if (operands::is_func(func)) {
      is_dynamic_function = false;
    } else if (operands::is_val(func)) {
      is_dynamic_function = true;
    } else {
      assert(false);
    }
    if (is_dynamic_function) {
      arg_start += 1;
    }

    {
      auto& infos = is_dynamic_function ?
        this->adaptor->cur_func->callee_infos[operands::content(calli.ops[1])].info
        : this->adaptor->mod->functions[operands::content(func)].args;
      assert(infos.size() == calli.ops.size() - arg_start);
      for (size_t i = 0; i < infos.size(); ++i) {
        auto& op = calli.ops[arg_start + i];
        ArgInfo& info = infos[i];

        using CallArg = typename Derived::CallArg;
        CallArg arg{op};

        Type ty = this->adaptor->type_of_ref(op);

        switch (ty) {
          case Type::Bool:
          case Type::i8:
          case Type::i16:
          case Type::i32:
          case Type::i64:
            if (info.extension == ArgExtension::zExt) {
              arg.flag = CallArg::Flag::zext;
              arg.ext_bits = size_of_type(ty);
            } else if (info.extension == ArgExtension::sExt) {
              arg.flag = CallArg::Flag::sext;
              arg.ext_bits = size_of_type(ty);
            }
            break;
          case Type::i128:
            arg.byval_align = 16;
            break;
          case Type::ptr: {
            if (info.kind == ArgKind::ByVal) {
              arg.flag = CallArg::Flag::byval;
              arg.byval_size = info.size;
              arg.byval_align = info.align;
            } else if (info.kind == ArgKind::sRet) {
              arg.flag = CallArg::Flag::sret;
            }
            break;
          }
          default:
            break;
        }

        cb->add_arg(arg);
      }
    }
    {
      if (!is_dynamic_function) {
        SymRef sym = this->func_syms[operands::content(func)];
        cb->call(sym);
      } else if (operands::is_val(func)) {
        auto [_, tgt_vp] = this->val_ref_single(func);
        cb->call(std::move(tgt_vp));
      }
    }

    if (calli.has_result) {
      tpde::CCAssignment cca;

      const auto res_to_ret = [this, &cb, &cca](uint32_t res) {
        assert(operands::is_val(res));

        size_t count;
        switch (this->adaptor->type_of_ref(res)) {
          case Type::i128:
            count = 2;
            break;
          default:
            count = 1;
            break;
        }
        ValueRef ref = this->result_ref(res);
        for (size_t i = 0; i < count; ++i) {
          ValuePart part = ref.part(i);
          cb->add_ret(part, cca);
        }
      };

      res_to_ret(calli.result);

      if (this->adaptor->get_basic_block(instr.block).instructions.size() > instr.next().inst) {
        const Instruction &pot_addret = this->adaptor->get_instruction(instr.next());
        if (pot_addret.kind == InstructionKind::AddRet) {
          res_to_ret(pot_addret.result);
        }
      }
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_invoke(RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    Instruction &inst = this->adaptor->get_instruction(inst_ref);


  // we need to spill here since the call might branch off
  // TODO: this will also spill the call arguments even if the call kills them
  // however, spillBeforeCall already does this anyways so probably something
  // for later
  auto spilled = this->spill_before_branch();

  const auto off_before_call = this->text_writer.offset();
  // compile the call
  // TODO: in the case of an exception we need to invalidate the result
  // registers
  // TODO: if the call needs stack space, this must be undone in the unwind
  // block! LLVM emits .cfi_escape 0x2e, <off>, we should do the same?
  // (Current workaround by treating invoke as dynamic alloca.)
  if (!this->compile_call(inst_ref, val_info, 0)) {
    return false;
  }
  const auto off_after_call = this->text_writer.offset();

  // build the eh table
    IRBlockRef normal_block_ref = operands::content(inst.ops[1]);
    IRBlockRef unwind_block_ref = operands::content(inst.ops[2]);
    auto unwind_block_has_phi = false; // TODO

  const BasicBlock& unwind_block = this->adaptor->get_basic_block(unwind_block_ref);
  const BasicBlock& normal_block = this->adaptor->get_basic_block(normal_block_ref);
  auto unwind_label =
      this->block_labels[(u32)this->analyzer.block_idx(unwind_block_ref)];

  // We always spill the call result. Also, generate_call might move values
  // again into registers, which we need to release again.
  // TODO: evaluate when exactly this is required.
  spilled |= this->spill_before_branch(/*force_spill=*/true);

  // if the unwind block has phi-nodes, we need more code to propagate values
  // to it so do the propagation logic
  if (unwind_block_has_phi) {
    // generate the jump to the normal successor but don't allow
    // fall-through
    derived()->generate_branch_to_block(Derived::Jump::jmp,
                                        normal_block_ref,
                                        /* split */ false,
                                        /* last_inst */ false);

    this->release_spilled_regs(spilled);

    unwind_label = this->text_writer.label_create();
    this->label_place(unwind_label);

    // allocate the special registers that are set by the unwinding logic
    // so the phi-propagation does not use them as temporaries
    ScratchReg scratch1{derived()}, scratch2{derived()};
    assert(!this->register_file.is_used(Derived::LANDING_PAD_RES_REGS[0]));
    assert(!this->register_file.is_used(Derived::LANDING_PAD_RES_REGS[1]));
    scratch1.alloc_specific(Derived::LANDING_PAD_RES_REGS[0]);
    scratch2.alloc_specific(Derived::LANDING_PAD_RES_REGS[1]);

    derived()->generate_branch_to_block(Derived::Jump::jmp,
                                        unwind_block_ref,
                                        /* split */ false,
                                        /* last_inst */ false);
  } else {
    // allow fall-through
    derived()->generate_branch_to_block(Derived::Jump::jmp,
                                        normal_block_ref,
                                        /* split */ false,
                                        /* last_inst */ true);

    this->release_spilled_regs(spilled);
  }

  const auto is_cleanup = false;  // TODO
  const auto num_clauses = unwind_block.instructions.size();
  const auto only_cleanup = is_cleanup && num_clauses == 0;

  this->text_writer.except_add_call_site(off_before_call,
                                         off_after_call - off_before_call,
                                         unwind_label,
                                         only_cleanup);

  if (only_cleanup) {
    // no clause so we are done
    return true;
  }

  // Only filters are used, no need for catch
  this->text_writer.except_add_empty_spec_action(true);

  if (is_cleanup) {
    assert(num_clauses != 0);
    this->text_writer.except_add_cleanup_action();
  }

  return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_landing_pad(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction &lp = this->adaptor->get_instruction(inst_ref);
    Instruction &lp_next = this->adaptor->get_instruction(inst_ref.next());
    this->result_ref(lp.result).part(0).set_value_reg(Derived::LANDING_PAD_RES_REGS[0]);
    this->result_ref(lp_next.result).part(0).set_value_reg(Derived::LANDING_PAD_RES_REGS[1]);

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_resume(RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    Instruction &inst = this->adaptor->get_instruction(inst_ref);
    IRValueRef arg = inst.ops[0];

    const auto sym = get_libfunc_sym(LibFunc::resume);

    derived()->create_helper_call({&arg, 1}, nullptr, sym);
    return derived()->compile_unreachable(RustAdaptor::IRInstRef{.inst = 0, .block = 0}, val_info, 0);
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_cast(RustAdaptor::IRInstRef instr, const ValInfo &val_info,
                                                                u64) {
    Instruction &casti = this->adaptor->get_instruction(instr);
    assert(operands::is_val(casti.result));

    IRValueRef src_ref = casti.ops[0];
    IRValueRef res_ref = casti.result;

    const Type src_ty = this->adaptor->type_of_ref(src_ref);
    const Type res_ty = this->adaptor->type_of_ref(res_ref);
    assert(size_of_type(src_ty) == size_of_type(res_ty));

    ValueRef src = this->val_ref(src_ref);
    ValueRef res = this->result_ref(res_ref);

    auto part_count = this->adaptor->val_parts(val_info).count();
    for (u32 i = 0; i != part_count; ++i) {
      res.part(i).set_value(src.part(i));
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_int_ext(RustAdaptor::IRInstRef instr, const ValInfo &,
                                                                   u64 sign) {
    Instruction &exti = this->adaptor->get_instruction(instr);
    const Type dst_ty = this->adaptor->type_of_ref(exti.result);

    if (!is_integer(dst_ty))
      return false;

    auto src_val = exti.ops[0];
    const Type src_ty = this->adaptor->type_of_ref(src_val);

    unsigned src_width = size_of_type(src_ty);
    unsigned dst_width = size_of_type(dst_ty);
    assert(dst_width > src_width);

    auto src_ref = this->val_ref(src_val);
    auto res = this->result_ref(exti.result);

    if (src_width <= 64) {
      ValuePartRef low = src_ref.part(0);
      if (src_width < 64) {
        unsigned ext_width = dst_width <= 64 ? dst_width : 64;
        low = std::move(low).into_extended(sign, src_width, ext_width);
      }
      if (dst_width > 64) {
        auto res_ref_high = res.part(1);

        if (sign) {
          if (!low.has_reg()) {
            low.load_to_reg();
          }
          derived()->encode_fill_with_sign64(low.get_unowned_ref(), res_ref_high);
        } else {
          res_ref_high.set_value(ValuePart{u64{0}, 8, res_ref_high.bank()});
        }
      }

      res.part(0).set_value(std::move(low));
      return true;
    }

    if (src_width < 128 && dst_width <= 128) {
      res.part(0).set_value(src_ref.part(0));
      res.part(1).set_value(
        src_ref.part(1).into_extended(sign, src_width - 64, 64));
      return true;
    }

    return false;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_int_trunc(RustAdaptor::IRInstRef inst, const ValInfo &val_info, u64) {
    const Instruction& trunci = this->adaptor->get_instruction(inst);
    auto val = trunci.ops[0];

    ValueRef src_vr = this->val_ref(val);
    ValueRef res_vr = this->result_ref(trunci.result);

    switch (val_info.type) {
      using enum Type;
      case Bool:
      case i8:
      case i16:
      case i32:
      case i64:
        // no-op, users will extend anyways. When truncating an i128, the first part
        // contains the lowest bits.
        res_vr.part(0).set_value(src_vr.part(0));
        return true;
      case i128:
        res_vr.part(0).set_value(src_vr.part(0));
        res_vr.part(1).set_value(src_vr.part(1));
        return true;
      default:
        throw std::runtime_error("Invalid type for trunc");
    }
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::
  compile_float_ext_trunc(RustAdaptor::IRInstRef instref, const ValInfo &, u64) {
    const Instruction& inst = this->adaptor->get_instruction(instref);

    auto src_val = inst.ops[0];
    const Type src_ty = this->adaptor->type_of_ref(src_val);
    const Type dst_ty = this->adaptor->type_of_ref(inst.result);

    auto res_vr = this->result_ref(inst.result);

    if (src_ty == Type::f64 && dst_ty == Type::f32) {
      auto src_ref = this->val_ref(src_val);
      derived()->encode_f64tof32(src_ref.part(0), res_vr.part(0));
    } else if (src_ty == Type::f32 && dst_ty == Type::f64) {
      auto src_ref = this->val_ref(src_val);
      derived()->encode_f32tof64(src_ref.part(0), res_vr.part(0));
    } else {
      return false;
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_float_to_int(RustAdaptor::IRInstRef instref, const ValInfo &, u64 flags) {
    const Instruction& inst = this->adaptor->get_instruction(instref);

    bool sign = flags & 0b01;
    bool saturate = flags & 0b10;

    auto src_val = inst.ops[0];
    const Type src_ty = this->adaptor->type_of_ref(src_val);

    const auto bit_width = size_of_type(this->adaptor->type_of_ref(inst.result));

    if (bit_width > 64) {
      return false;
    }

    unsigned ty_idx;
    switch (src_ty) {
      using enum Type;
      case f32: ty_idx = 0; break;
      case f64: ty_idx = 1; break;
      default: return false;
    }

    using EncodeFnTy = bool (Derived::*)(GenericValuePart &&, ValuePart &&);
    static constexpr auto fns = []() {
      // fns[is_double][dst64][sign][sat]
      std::array<EncodeFnTy[2][2][2], 2> fns{};
      fns[0][0][0][0] = &Derived::encode_f32tou32;
      fns[0][0][0][1] = &Derived::encode_f32tou32_sat;
      fns[0][0][1][0] = &Derived::encode_f32toi32;
      fns[0][0][1][1] = &Derived::encode_f32toi32_sat;
      fns[0][1][0][0] = &Derived::encode_f32tou64;
      fns[0][1][0][1] = &Derived::encode_f32tou64_sat;
      fns[0][1][1][0] = &Derived::encode_f32toi64;
      fns[0][1][1][1] = &Derived::encode_f32toi64_sat;
      fns[1][0][0][0] = &Derived::encode_f64tou32;
      fns[1][0][0][1] = &Derived::encode_f64tou32_sat;
      fns[1][0][1][0] = &Derived::encode_f64toi32;
      fns[1][0][1][1] = &Derived::encode_f64toi32_sat;
      fns[1][1][0][0] = &Derived::encode_f64tou64;
      fns[1][1][0][1] = &Derived::encode_f64tou64_sat;
      fns[1][1][1][0] = &Derived::encode_f64toi64;
      fns[1][1][1][1] = &Derived::encode_f64toi64_sat;
      return fns;
    }();
    EncodeFnTy fn = fns[ty_idx][bit_width > 32][sign][saturate];

    if (saturate && bit_width % 32 != 0) {
      // TODO: clamp result to smaller integer bounds
      return false;
    }

    auto src_ref = this->val_ref(src_val);
    auto res_ref = this->result_ref(inst.result);
    return (derived()->*fn)(src_ref.part(0), res_ref.part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_int_to_float(RustAdaptor::IRInstRef instref, const ValInfo &val_info, u64 sign) {
    const Instruction& inst = this->adaptor->get_instruction(instref);
    const auto src_val = inst.ops[0];
    const Type dst_ty = this->adaptor->type_of_ref(inst.result);

    auto bit_width = size_of_type(this->adaptor->type_of_ref(src_val));
    if (bit_width > 64) {
      return false;
    }

    ValueRef src_ref = this->val_ref(src_val);
    ValuePartRef src_op = src_ref.part(0);
    ValueRef res = this->result_ref(inst.result);

    if (bit_width != 32 && bit_width != 64) {
      unsigned ext = tpde::util::align_up(bit_width, 32);
      src_op = std::move(src_op).into_extended(sign, bit_width, ext);
    }

    unsigned ty_idx;
    switch (val_info.type) {
      using enum Type;
      case f32: ty_idx = 0; break;
      case f64: ty_idx = 1; break;
      default: return false;
    }

    using EncodeFnTy = bool (Derived::*)(GenericValuePart &&, ValuePart &&);
    static constexpr auto encode_fns = []() consteval {
      std::array<EncodeFnTy[2][2], 2> res;
      res[0][0][0] = &Derived::encode_i32tof32;
      res[0][0][1] = &Derived::encode_i32tof64;
      res[0][1][0] = &Derived::encode_i64tof32;
      res[0][1][1] = &Derived::encode_i64tof64;
      res[1][0][0] = &Derived::encode_u32tof32;
      res[1][0][1] = &Derived::encode_u32tof64;
      res[1][1][0] = &Derived::encode_u64tof32;
      res[1][1][1] = &Derived::encode_u64tof64;
      return res;
    }();
    EncodeFnTy fn = encode_fns[!sign][bit_width > 32][ty_idx];
    (derived()->*fn)(std::move(src_op), res.part(0));
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_neg(RustAdaptor::IRInstRef instr, const ValInfo &, u64) {
    Instruction &negi = this->adaptor->get_instruction(instr);

    ValueRef src = this->val_ref(negi.ops[0]);
    ValueRef res = this->result_ref(negi.result);

    const Type type = this->adaptor->type_of_ref(negi.ops[0]);
    switch (type) {
      using enum Type;
      case Bool:
      case i8:
      case i16:
      case i32: derived()->encode_negi32(src.part(0), res.part(0));
        break;
      case i64: derived()->encode_negi64(src.part(0), res.part(0));
        break;
      case i128: derived()->encode_negi128(src.part(0), src.part(1), res.part(0), res.part(1));
        break;
      default: return false;
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_fneg(RustAdaptor::IRInstRef inst, const ValInfo & val_info, u64) {
    Instruction& fnegi = this->adaptor->get_instruction(inst);
    ValueRef src = this->val_ref(fnegi.ops[0]);
    ValueRef res = this->result_ref(fnegi.result);
    switch (val_info.type) {
      using enum Type;
      case f32: derived()->encode_fnegf32(src.part(0), res.part(0)); break;
      case f64: derived()->encode_fnegf64(src.part(0), res.part(0)); break;
      default: return false;
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_not(RustAdaptor::IRInstRef instr, const ValInfo &, u64) {
    Instruction &noti = this->adaptor->get_instruction(instr);

    ValueRef src = this->val_ref(noti.ops[0]);
    ValueRef res = this->result_ref(noti.result);

    const Type type = this->adaptor->type_of_ref(noti.ops[0]);
    switch (type) {
      using enum Type;
      case Bool: derived()->encode_notbool(src.part(0), res.part(0));
        break;
      case i8:
      case i16:
      case i32: derived()->encode_not32(src.part(0), res.part(0));
        break;
      case i64: derived()->encode_not64(src.part(0), res.part(0));
        break;
      case i128: derived()->encode_not128(src.part(0), src.part(1), res.part(0), res.part(1));
        break;
      default: return false;
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_fcmp(RustAdaptor::IRInstRef instr, const ValInfo &, u64 op) {
    Instruction& fcmpi = this->adaptor->get_instruction(instr);
    Type type = this->adaptor->type_of_ref(fcmpi.ops[0]);

    ValueRef lhs = this->val_ref(fcmpi.ops[0]);
    ValueRef rhs = this->val_ref(fcmpi.ops[1]);
    ValueRef res = this->result_ref(fcmpi.result);

    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&, ValuePart &&);
    EncodeFnTy fn = nullptr;

    switch (type) {
      using enum Type;
      using enum FloatCmpOp::Value;
      case f32:
        switch (op) {
          case OEQ: fn = &Derived::encode_fcmp_oeq_float; break;
          case OGT: fn = &Derived::encode_fcmp_ogt_float; break;
          case OGE: fn = &Derived::encode_fcmp_oge_float; break;
          case OLT: fn = &Derived::encode_fcmp_olt_float; break;
          case OLE: fn = &Derived::encode_fcmp_ole_float; break;
          case ONE: fn = &Derived::encode_fcmp_one_float; break;
          case ORD: fn = &Derived::encode_fcmp_ord_float; break;
          case UEQ: fn = &Derived::encode_fcmp_ueq_float; break;
          case UGT: fn = &Derived::encode_fcmp_ugt_float; break;
          case UGE: fn = &Derived::encode_fcmp_uge_float; break;
          case ULT: fn = &Derived::encode_fcmp_ult_float; break;
          case ULE: fn = &Derived::encode_fcmp_ule_float; break;
          case UNE: fn = &Derived::encode_fcmp_une_float; break;
          case UNO: fn = &Derived::encode_fcmp_uno_float; break;
          default: TPDE_UNREACHABLE("invalid fcmp predicate");
        }
        break;
      case f64:
        switch (op) {
          case OEQ: fn = &Derived::encode_fcmp_oeq_double; break;
          case OGT: fn = &Derived::encode_fcmp_ogt_double; break;
          case OGE: fn = &Derived::encode_fcmp_oge_double; break;
          case OLT: fn = &Derived::encode_fcmp_olt_double; break;
          case OLE: fn = &Derived::encode_fcmp_ole_double; break;
          case ONE: fn = &Derived::encode_fcmp_one_double; break;
          case ORD: fn = &Derived::encode_fcmp_ord_double; break;
          case UEQ: fn = &Derived::encode_fcmp_ueq_double; break;
          case UGT: fn = &Derived::encode_fcmp_ugt_double; break;
          case UGE: fn = &Derived::encode_fcmp_uge_double; break;
          case ULT: fn = &Derived::encode_fcmp_ult_double; break;
          case ULE: fn = &Derived::encode_fcmp_ule_double; break;
          case UNE: fn = &Derived::encode_fcmp_une_double; break;
          case UNO: fn = &Derived::encode_fcmp_uno_double; break;
          default: TPDE_UNREACHABLE("invalid fcmp predicate");
        }
        break;
      default: TPDE_UNREACHABLE("invalid fcmp type");
    }

    return (derived()->*fn)(lhs.part(0), rhs.part(0), res.part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_ctpop(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction& inst = this->adaptor->get_instruction(inst_ref);

    const auto width = size_of_type(this->adaptor->type_of_ref(inst.ops[0]));
    if (width > 64) {
      return false;
    }

    ValueRef val_ref = this->val_ref(inst.ops[0]);
    ValuePartRef op = val_ref.part(0);
    if (width % 32) {
      unsigned tgt_width = tpde::util::align_up(width, 32);
      op = std::move(op).into_extended(/*sign=*/false, width, tgt_width);
    }

    auto [res_vr, res_ref] = this->result_ref_single(inst.result);
    if (width <= 32) {
      derived()->encode_ctpopi32(std::move(op), res_ref);
    } else {
      derived()->encode_ctpopi64(std::move(op), res_ref);
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_ct_lz_tz(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64 op) {
    Instruction& inst = this->adaptor->get_instruction(inst_ref);
    auto val = inst.ops[0];

    u32 width_idx = 0;
    switch (size_of_type(this->adaptor->type_of_ref(val))) {
      case 8: width_idx = 0; break;
      case 16: width_idx = 1; break;
      case 32: width_idx = 2; break;
      case 64: width_idx = 3; break;
      default: return false;
    }

    using EncodeFnTy = bool (Derived::*)(GenericValuePart &&, ValuePart &&);
    static constexpr EncodeFnTy encode_fns[4][2][2] = {
#define F(n, op, suffix) &Derived::encode_##op##i##n##suffix
      {{F(8, ctlz, ), F(8, ctlz, _zp)}, {F(8, cttz, ), F(32, cttz, _zp)}},
      {{F(16, ctlz, ), F(16, ctlz, _zp)}, {F(16, cttz, ), F(32, cttz, _zp)}},
      {{F(32, ctlz, ), F(32, ctlz, _zp)}, {F(32, cttz, ), F(32, cttz, _zp)}},
      {{F(64, ctlz, ), F(64, ctlz, _zp)}, {F(64, cttz, ), F(64, cttz, _zp)}},
#undef F
  };
    bool zero_is_poison = (op & 1) == 1;
    bool is_cttz = (op & 2) == 2;
    EncodeFnTy fn = encode_fns[width_idx][is_cttz][zero_is_poison];
    return (derived()->*fn)(this->val_ref(val).part(0),
                            this->result_ref(inst.result).part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  u64 RustCompilerBase<Adaptor, Derived, Config>::const_vector_elem(IRValueRef vec, unsigned idx) {
    assert(operands::is_const(vec));
    const Value &imm = this->adaptor->mod->consts[operands::content(vec)];
    const auto [nelem, elem_ty] = vector_info(imm.ty);
    assert(idx < nelem);
    const unsigned bits = size_of_type(elem_ty);
    const unsigned bit_off = idx * bits;
    // data2 holds the low 64 bits, data1 the high 64 bits
    u64 word = bit_off < 64 ? imm.data2 : imm.data1;
    word >>= bit_off % 64;
    return bits == 64 ? word : word & ((u64{1} << bits) - 1);
  }

  template<typename Adaptor, typename Derived, typename Config>
  void RustCompilerBase<Adaptor, Derived, Config>::extract_element(
    ValueRef &vec_vr, unsigned idx, Type ty, ValuePart &out) {
    if (!vec_vr.has_assignment()) {
      // Constant.
      const u64 elem = const_vector_elem(vec_vr.state.s.data, idx);
      const u32 size = size_of_type(ty) / 8;
      out.set_value(this, ValuePart{elem, size, reg_bank_of_type(ty)});
      return;
    }

    tpde::ValueAssignment *va = vec_vr.assignment();
    u32 elem_sz = size_of_type(ty) / 8;

    if (elem_sz == va->max_part_size) {
      // Scalarized vector: simply take part idx.
      out.set_value(this, vec_vr.part(idx));
      return;
    }

    // Offset inside whole vector
    u32 vector_off = idx * elem_sz;
    // A vector can consist of multiple, equally sized parts.
    u32 part = vector_off / va->max_part_size;
    u32 off_in_part = vector_off % va->max_part_size;
    assert(part < va->part_count);

    this->spill({va, part});
    GenericValuePart addr = derived()->val_spill_slot({va, part});
    auto &expr = std::get<typename GenericValuePart::Expr>(addr.state);
    expr.disp += off_in_part;

    switch (ty) {
      using enum Type;
      case i8: derived()->encode_loadi8_zext(std::move(addr), out); break;
      case i16: derived()->encode_loadi16_zext(std::move(addr), out); break;
      case i32: derived()->encode_loadi32_zext(std::move(addr), out); break;
      case i64:
      case ptr: derived()->encode_loadi64(std::move(addr), out); break;
      case f32: derived()->encode_loadf32(std::move(addr), out); break;
      case f64: derived()->encode_loadf64(std::move(addr), out); break;
      default: TPDE_UNREACHABLE("unexpected vector element type");
    }
  }

  template<typename Adaptor, typename Derived, typename Config>
  void RustCompilerBase<Adaptor, Derived, Config>::insert_element(
    ValueRef &vec_vr, unsigned idx, Type ty, GenericValuePart &&el) {
    tpde::ValueAssignment *va = vec_vr.assignment();
    u32 elem_sz = size_of_type(ty) / 8;

    // Offset inside whole vector
    u32 vector_off = idx * elem_sz;
    // A vector can consist of multiple, equally sized parts.
    u32 part = vector_off / va->max_part_size;
    u32 off_in_part = vector_off % va->max_part_size;
    assert(part < va->part_count);

    tpde::AssignmentPartRef ap{va, part};
    if (ap.register_valid()) {
      this->evict(ap);
    } else if (!ap.stack_valid()) {
      // Value part is uninitialized
      this->allocate_spill_slot(ap);
      ap.set_stack_valid();
    }

    GenericValuePart addr = derived()->val_spill_slot(ap);
    auto &expr = std::get<typename GenericValuePart::Expr>(addr.state);
    expr.disp += off_in_part;

    switch (ty) {
      using enum Type;
      case i8: derived()->encode_storei8(std::move(addr), std::move(el)); break;
      case i16: derived()->encode_storei16(std::move(addr), std::move(el)); break;
      case i32: derived()->encode_storei32(std::move(addr), std::move(el)); break;
      case i64:
      case ptr: derived()->encode_storei64(std::move(addr), std::move(el)); break;
      case f32: derived()->encode_storef32(std::move(addr), std::move(el)); break;
      case f64: derived()->encode_storef64(std::move(addr), std::move(el)); break;
      default: TPDE_UNREACHABLE("unexpected vector element type");
    }
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_extract_element(
    RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    // operands: vec, index
    Instruction &inst = this->adaptor->get_instruction(inst_ref);
    const IRValueRef src = inst.ops[0];
    const IRValueRef index = inst.ops[1];

    ValueRef vec_vr = this->val_ref(src);
    auto [res_vr, result] = this->result_ref_single(inst.result);

    const unsigned nelem = vector_info(this->adaptor->type_of_ref(src)).first;
    const Type bvt = val_info.type;

    if (operands::is_const(index)) {
      unsigned cidx = this->adaptor->mod->consts[operands::content(index)].data2;
      cidx = cidx < nelem ? cidx : 0;
      derived()->extract_element(vec_vr, cidx, bvt, result);
      return true;
    }

    if (!vec_vr.has_assignment()) {
      // TODO: support dynamic extractelement from constant vectors.
      return false;
    }

    // First, copy value into the spill slot.
    for (unsigned i = 0; i < vec_vr.assignment()->part_count; ++i) {
      this->spill(tpde::AssignmentPartRef{vec_vr.assignment(), i});
    }

    // Second, create address. Mask index, out-of-bounds access are just poison.
    ValuePartRef idx_scratch{this, Config::GP_BANK};
    GenericValuePart addr = derived()->val_spill_slot({vec_vr.assignment(), 0});
    auto &expr = std::get<typename GenericValuePart::Expr>(addr.state);
    derived()->encode_landi64(this->val_ref(index).part(0),
                              ValuePartRef{this, u64{nelem - 1}, 8, Config::GP_BANK},
                              idx_scratch);
    assert(expr.scale == 0);
    expr.scale = size_of_type(bvt) / 8;
    expr.index = std::move(idx_scratch).into_scratch();

    // Third, do the load.
    switch (bvt) {
      using enum Type;
      case i8: derived()->encode_loadi8_zext(std::move(addr), result); break;
      case i16: derived()->encode_loadi16_zext(std::move(addr), result); break;
      case i32: derived()->encode_loadi32_zext(std::move(addr), result); break;
      case i64:
      case ptr: derived()->encode_loadi64(std::move(addr), result); break;
      case f32: derived()->encode_loadf32(std::move(addr), result); break;
      case f64: derived()->encode_loadf64(std::move(addr), result); break;
      default: TPDE_UNREACHABLE("unexpected vector element type");
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_insert_element(
    RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    // operands: vec, elem, index
    Instruction &inst = this->adaptor->get_instruction(inst_ref);
    const IRValueRef index = inst.ops[2];

    const auto [nelem, bvt] = vector_info(val_info.type);

    auto [val_ref, val] = this->val_ref_single(inst.ops[1]);
    ValueRef res_vr{derived()};

    // We do the dynamic insert in the spill slot of result.
    // First, copy value into the result. We must also do this for constant
    // indices, because the value reference must always be initialized.
    {
      ValueRef src_vr = this->val_ref(inst.ops[0]);
      if (src_vr.is_owned()) {
        res_vr = this->result_ref_alias(inst.result, std::move(src_vr));
      } else {
        res_vr = this->result_ref(inst.result);
        for (u32 i = 0; i < res_vr.assignment()->part_count; ++i) {
          res_vr.part(i).set_value(src_vr.part(i));
        }
      }
    }

    if (operands::is_const(index)) {
      unsigned cidx = this->adaptor->mod->consts[operands::content(index)].data2;
      cidx = cidx < nelem ? cidx : 0;
      derived()->insert_element(res_vr, cidx, bvt, std::move(val));
      // No need for ref counting: all operands and results were ValuePartRefs.
      return true;
    }

    // Evict, because we will overwrite the value in the stack slot.
    for (unsigned i = 0; i < res_vr.assignment()->part_count; ++i) {
      tpde::AssignmentPartRef ap{res_vr.assignment(), i};
      if (ap.register_valid()) {
        this->evict(ap);
      }
    }

    // Second, create address. Mask index, out-of-bounds access are just poison.
    ValuePartRef idx_scratch{this, Config::GP_BANK};
    GenericValuePart addr = derived()->val_spill_slot({res_vr.assignment(), 0});
    auto &expr = std::get<typename GenericValuePart::Expr>(addr.state);
    derived()->encode_landi64(this->val_ref(index).part(0),
                              ValuePartRef{this, u64{nelem - 1}, 8, Config::GP_BANK},
                              idx_scratch);
    assert(expr.scale == 0);
    expr.scale = val.part_size();
    expr.index = std::move(idx_scratch).into_scratch();

    // Third, do the store.
    switch (bvt) {
      using enum Type;
      case i8: derived()->encode_storei8(std::move(addr), std::move(val)); break;
      case i16: derived()->encode_storei16(std::move(addr), std::move(val)); break;
      case i32: derived()->encode_storei32(std::move(addr), std::move(val)); break;
      case i64:
      case ptr: derived()->encode_storei64(std::move(addr), std::move(val)); break;
      case f32: derived()->encode_storef32(std::move(addr), std::move(val)); break;
      case f64: derived()->encode_storef64(std::move(addr), std::move(val)); break;
      default: TPDE_UNREACHABLE("unexpected vector element type");
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_shuffle_vector(
    RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    // operands: lhs, rhs, mask indices as raw values
    Instruction &inst = this->adaptor->get_instruction(inst_ref);
    const IRValueRef lhs = inst.ops[0];
    const IRValueRef rhs = inst.ops[1];

    const auto [dst_nelem, bvt] = vector_info(val_info.type);
    const unsigned src_nelem = vector_info(this->adaptor->type_of_ref(lhs)).first;
    assert(inst.ops.size() == 2 + dst_nelem);

    auto bank = reg_bank_of_type(bvt);
    auto size = size_of_type(bvt) / 8;

    ValueRef lhs_vr = this->val_ref(lhs);
    ValueRef rhs_vr = this->val_ref(rhs);
    ValueRef res_vr = this->result_ref(inst.result);

    ValuePartRef tmp{this, bank};
    for (unsigned i = 0; i < dst_nelem; i++) {
      const unsigned mask = operands::content(inst.ops[2 + i]);
      const bool src_is_lhs = mask < src_nelem;
      const IRValueRef src = src_is_lhs ? lhs : rhs;
      if (operands::is_const(src)) {
        const u64 const_elem = const_vector_elem(src, mask % src_nelem);
        ValuePartRef const_ref{this, const_elem, size, bank};
        derived()->insert_element(res_vr, i, bvt, std::move(const_ref));
      } else {
        ValueRef src_vr = (src_is_lhs ? lhs_vr : rhs_vr).disowned();
        derived()->extract_element(src_vr, mask % src_nelem, bvt, tmp);
        derived()->insert_element(res_vr, i, bvt, std::move(tmp));
      }
    }

    // Make sure that all parts are initialized.
    for (u32 i = 0, n = res_vr.assignment()->part_count; i != n; ++i) {
      tpde::AssignmentPartRef ap{res_vr.assignment(), i};
      if (!ap.register_valid() && !ap.stack_valid()) {
        // Value part is uninitialized
        this->allocate_spill_slot(ap);
        ap.set_stack_valid();
      }
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_icmp_vector(Instruction &inst) {
    // Unlike LLVM, the result is not an i1 vector but a mask vector with the
    // same type as the operands (all bits of a lane set if true), as required
    // by the Rust simd comparison intrinsics.
    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&, ValuePart &&);
    // fns[pred][type]
    static constexpr auto fns = []() constexpr {
      std::array<EncodeFnTy[7], 10> res{};

#define FN_ENTRY(pred, predname, sign)                                          \
    res[pred][0] = &Derived::encode_icmpmask_##predname##v8##sign##8;             \
    res[pred][1] = &Derived::encode_icmpmask_##predname##v4##sign##16;            \
    res[pred][2] = &Derived::encode_icmpmask_##predname##v2##sign##32;            \
    res[pred][3] = &Derived::encode_icmpmask_##predname##v16##sign##8;            \
    res[pred][4] = &Derived::encode_icmpmask_##predname##v8##sign##16;            \
    res[pred][5] = &Derived::encode_icmpmask_##predname##v4##sign##32;            \
    res[pred][6] = &Derived::encode_icmpmask_##predname##v2##sign##64;

      FN_ENTRY(0, eq, u)
      FN_ENTRY(1, ne, u)
      FN_ENTRY(2, ugt, u)
      FN_ENTRY(3, uge, u)
      FN_ENTRY(4, ult, u)
      FN_ENTRY(5, ule, u)
      FN_ENTRY(6, sgt, i)
      FN_ENTRY(7, sge, i)
      FN_ENTRY(8, slt, i)
      FN_ENTRY(9, sle, i)
#undef FN_ENTRY

      return res;
    }();

    unsigned pred_idx;
    switch (inst.kind) {
      case InstructionKind::CMPeq: pred_idx = 0; break;
      case InstructionKind::CMPne: pred_idx = 1; break;
      case InstructionKind::CMPugt: pred_idx = 2; break;
      case InstructionKind::CMPuge: pred_idx = 3; break;
      case InstructionKind::CMPult: pred_idx = 4; break;
      case InstructionKind::CMPule: pred_idx = 5; break;
      case InstructionKind::CMPsgt: pred_idx = 6; break;
      case InstructionKind::CMPsge: pred_idx = 7; break;
      case InstructionKind::CMPslt: pred_idx = 8; break;
      case InstructionKind::CMPsle: pred_idx = 9; break;
      default: TPDE_UNREACHABLE("invalid icmp predicate");
    }

    unsigned ty_idx;
    switch (this->adaptor->type_of_ref(inst.ops[0])) {
      using enum Type;
      case v8i8: ty_idx = 0; break;
      case v4i16: ty_idx = 1; break;
      case v2i32: ty_idx = 2; break;
      case v16i8: ty_idx = 3; break;
      case v8i16: ty_idx = 4; break;
      case v4i32: ty_idx = 5; break;
      case v2i64: ty_idx = 6; break;
      default: return false;
    }

    EncodeFnTy encode_fn = fns[pred_idx][ty_idx];
    auto lhs_vr = this->val_ref(inst.ops[0]);
    auto rhs_vr = this->val_ref(inst.ops[1]);
    auto res = this->result_ref(inst.result);
    return (derived()->*encode_fn)(lhs_vr.part(0), rhs_vr.part(0), res.part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_fmuladd(
    RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    Instruction &inst = this->adaptor->get_instruction(inst_ref);
    ValueRef op1 = this->val_ref(inst.ops[0]);
    ValueRef op2 = this->val_ref(inst.ops[1]);
    ValueRef op3 = this->val_ref(inst.ops[2]);
    ValueRef res = this->result_ref(inst.result);

    using EncodeFnTy = bool (Derived::*)(GenericValuePart &&,
                                         GenericValuePart &&,
                                         GenericValuePart &&,
                                         ValuePart &&);
    EncodeFnTy fn = nullptr;
    switch (val_info.type) {
      using enum Type;
      case f32: fn = &Derived::encode_fmuladdf32; break;
      case f64: fn = &Derived::encode_fmuladdf64; break;
      case v2f32: fn = &Derived::encode_fmuladdv2f32; break;
      case v4f32: fn = &Derived::encode_fmuladdv4f32; break;
      case v2f64: fn = &Derived::encode_fmuladdv2f64; break;
      default: return false;
    }
    return (derived()->*fn)(op1.part(0), op2.part(0), op3.part(0), res.part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_vector_reduce(
    RustAdaptor::IRInstRef inst_ref, const ValInfo &info, u64 op) {
    // operands: vec, or acc, vec for the ordered float reductions
    Instruction &inst = this->adaptor->get_instruction(inst_ref);
    const Type elem_ty = info.type;
    const bool is_64 = elem_ty == Type::i64 || elem_ty == Type::f64;

    using EncodeFnTy =
        bool (Derived::*)(GenericValuePart &&, GenericValuePart &&, ValuePart &);
    EncodeFnTy fn;
    bool has_start_elem = false;
    switch (op) {
      case ReduceOp::add: fn = is_64 ? &Derived::encode_addi64 : &Derived::encode_addi32; break;
      case ReduceOp::mul: fn = is_64 ? &Derived::encode_muli64 : &Derived::encode_muli32; break;
      case ReduceOp::land: fn = is_64 ? &Derived::encode_landi64 : &Derived::encode_landi32; break;
      case ReduceOp::lor: fn = is_64 ? &Derived::encode_lori64 : &Derived::encode_lori32; break;
      case ReduceOp::lxor: fn = is_64 ? &Derived::encode_lxori64 : &Derived::encode_lxori32; break;
      case ReduceOp::fadd:
        fn = is_64 ? &Derived::encode_addf64 : &Derived::encode_addf32;
        has_start_elem = true;
        break;
      case ReduceOp::fmul:
        fn = is_64 ? &Derived::encode_mulf64 : &Derived::encode_mulf32;
        has_start_elem = true;
        break;
      default:
        // Still missing: smin/smax/umin/umix/fmin/fmax/fminimum/fmaximum
        return false;
    }

    const IRValueRef src_op = inst.ops[has_start_elem ? 1 : 0];
    ValueRef src_ref = this->val_ref(src_op);
    ValueRef src_ref_disowned = src_ref.disowned();

    ValuePartRef elem{this, reg_bank_of_type(elem_ty)};
    ValuePartRef acc{this, reg_bank_of_type(elem_ty)};

    if (has_start_elem) {
      acc.set_value(this->val_ref(inst.ops[0]).part(0));
    } else {
      derived()->extract_element(src_ref_disowned, 0, elem_ty, acc);
    }

    const unsigned nelem = vector_info(this->adaptor->type_of_ref(src_op)).first;
    for (unsigned i = has_start_elem ? 0 : 1; i != nelem; i++) {
      derived()->extract_element(src_ref_disowned, i, elem_ty, elem);
      (derived()->*fn)(std::move(acc), std::move(elem), acc);
    }

    this->result_ref(inst.result).part(0).set_value(std::move(acc));

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_abort(RustAdaptor::IRInstRef, const ValInfo &, u64) {
    derived()->encode_trap();
    this->release_regs_after_return();
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_cmpxchg(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction& instr = this->adaptor->get_instruction(inst_ref);

    auto new_val = instr.ops[2];
    Type val_ty = this->adaptor->type_of_ref(new_val);
    unsigned width = size_of_type(val_ty);
    if (width > 64) {
      return false;
    }

    unsigned width_idx;
    switch (width) {
    case 8: width_idx = 0; break;
    case 16: width_idx = 1; break;
    case 32: width_idx = 2; break;
    case 64: width_idx = 3; break;
    default: return false;
    }

    // ptr, cmp, new_val, old_val, success
    using EncodeFnTy = bool (Derived::*)(GenericValuePart &&,
                                         GenericValuePart &&,
                                         GenericValuePart &&,
                                         ValuePart &&,
                                         ValuePart &&);
    static constexpr auto fns = []() constexpr {
      using enum AtomicOrdering;
      std::array<EncodeFnTy[size_t(LAST) + 1], 4> res{};
      res[0][u32(Monotonic)] = &Derived::encode_cmpxchg_u8_monotonic;
      res[1][u32(Monotonic)] = &Derived::encode_cmpxchg_u16_monotonic;
      res[2][u32(Monotonic)] = &Derived::encode_cmpxchg_u32_monotonic;
      res[3][u32(Monotonic)] = &Derived::encode_cmpxchg_u64_monotonic;
      res[0][u32(Acquire)] = &Derived::encode_cmpxchg_u8_acquire;
      res[1][u32(Acquire)] = &Derived::encode_cmpxchg_u16_acquire;
      res[2][u32(Acquire)] = &Derived::encode_cmpxchg_u32_acquire;
      res[3][u32(Acquire)] = &Derived::encode_cmpxchg_u64_acquire;
      res[0][u32(Release)] = &Derived::encode_cmpxchg_u8_release;
      res[1][u32(Release)] = &Derived::encode_cmpxchg_u16_release;
      res[2][u32(Release)] = &Derived::encode_cmpxchg_u32_release;
      res[3][u32(Release)] = &Derived::encode_cmpxchg_u64_release;
      res[0][u32(AcquireRelease)] = &Derived::encode_cmpxchg_u8_acqrel;
      res[1][u32(AcquireRelease)] = &Derived::encode_cmpxchg_u16_acqrel;
      res[2][u32(AcquireRelease)] = &Derived::encode_cmpxchg_u32_acqrel;
      res[3][u32(AcquireRelease)] = &Derived::encode_cmpxchg_u64_acqrel;
      res[0][u32(SequentiallyConsistent)] = &Derived::encode_cmpxchg_u8_seqcst;
      res[1][u32(SequentiallyConsistent)] = &Derived::encode_cmpxchg_u16_seqcst;
      res[2][u32(SequentiallyConsistent)] = &Derived::encode_cmpxchg_u32_seqcst;
      res[3][u32(SequentiallyConsistent)] = &Derived::encode_cmpxchg_u64_seqcst;
      return res;
    }();

    auto ptr_ref = this->val_ref(instr.ops[0]);
    auto cmp_ref = this->val_ref(instr.ops[1]);
    auto new_ref = this->val_ref(new_val);
    auto res_val = this->result_ref(instr.result);
    auto res = this->result_ref(this->adaptor->get_instruction(inst_ref.next()).result);

    auto order = static_cast<AtomicOrdering>(operands::content(instr.ops[3]));
    EncodeFnTy encode_fn = fns[width_idx][size_t(order)];
    assert(encode_fn && "invalid cmpxchg ordering");
    if (!(derived()->*encode_fn)(ptr_ref.part(0),
                                 cmp_ref.part(0),
                                 new_ref.part(0),
                                 res_val.part(0),
                                 res.part(0))) {
      return false;
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_atomicrmw(RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    Instruction& instr = this->adaptor->get_instruction(inst_ref);

    Type ty = this->adaptor->type_of_ref(instr.ops[2]);
    unsigned size = size_of_type(ty);
    // This is checked by the IR verifier.
    assert(size >= 8 && (size & (size - 1)) == 0 && "invalid atomicrmw size");

    auto bvt = val_info.type;

    // TODO: implement non-seq_cst orderings more efficiently
    // TODO: use more efficient implementation when the result is not used. On
    // x86-64, the current implementation gives many cmpxchg loops.
    bool (Derived::*fn)(GenericValuePart &&, GenericValuePart &&, ValuePart &&) =
        nullptr;
    switch (static_cast<AtomicRmwBinOp>(operands::content(instr.ops[0]))) {
    case AtomicRmwBinOp::Xchg:
      // TODO: support f32/f64
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_xchg_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_xchg_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_xchg_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_xchg_u64_seqcst; break;
      case ptr: fn = &Derived::encode_atomic_xchg_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::Add:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_add_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_add_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_add_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_add_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::Sub:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_sub_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_sub_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_sub_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_sub_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::And:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_and_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_and_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_and_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_and_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::Nand:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_nand_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_nand_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_nand_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_nand_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::Or:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_or_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_or_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_or_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_or_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::Xor:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_xor_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_xor_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_xor_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_xor_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::Min:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_min_i8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_min_i16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_min_i32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_min_i64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::Max:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_max_i8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_max_i16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_max_i32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_max_i64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::UMin:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_min_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_min_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_min_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_min_u64_seqcst; break;
      default: return false;
      }
      break;
    case AtomicRmwBinOp::UMax:
      switch (bvt) {
        using enum Type;
      case i8: fn = &Derived::encode_atomic_max_u8_seqcst; break;
      case i16: fn = &Derived::encode_atomic_max_u16_seqcst; break;
      case i32: fn = &Derived::encode_atomic_max_u32_seqcst; break;
      case i64: fn = &Derived::encode_atomic_max_u64_seqcst; break;
      default: return false;
      }
      break;
    default: return false;
    }

    auto ptr_ref = this->val_ref(instr.ops[1]);
    auto val_ref = this->val_ref(instr.ops[2]);
    auto res_ref = this->result_ref(instr.result);
    return (derived()->*fn)(ptr_ref.part(0), val_ref.part(0), res_ref.part(0));
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_fence(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction& instr = this->adaptor->get_instruction(inst_ref);

    if (operands::content(instr.ops[1]) == 0 /*= is single threaded*/) {
      // memory barrier only
      return true;
    }

    switch (static_cast<AtomicOrdering>(operands::content(instr.ops[0]))) {
      using enum AtomicOrdering;
      case Acquire: derived()->encode_fence_acq(); break;
      case Release: derived()->encode_fence_rel(); break;
      case AcquireRelease: derived()->encode_fence_acqrel(); break;
      case SequentiallyConsistent: derived()->encode_fence_seqcst(); break;
      default: return false;
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  RustCompilerBase<Adaptor, Derived, Config>::SymRef
  RustCompilerBase<Adaptor, Derived, Config>::get_libfunc_sym(LibFunc func) {
    assert(func < LibFunc::MAX);
    SymRef &sym = libfunc_syms[static_cast<size_t>(func)];
    if (sym.valid()) [[likely]] {
      return sym;
    }

    std::string_view name = "???";
    switch (func) {
      using enum LibFunc;
      case divti3: name = "__divti3";
        break;
      case udivti3: name = "__udivti3";
        break;
      case modti3: name = "__modti3";
        break;
      case umodti3: name = "__umodti3";
        break;
      case fmod: name = "fmod";
        break;
      case fmodf: name = "fmodf";
        break;
      case fmodf16: name = "fmodf16";
        break;
      case floorf: name = "floorf";
        break;
      case floor: name = "floor";
        break;
      case ceilf: name = "ceilf";
        break;
      case ceil: name = "ceil";
        break;
      case roundf: name = "roundf";
        break;
      case round: name = "round";
        break;
      case nearbyintf: name = "nearbyintf";
        break;
      case nearbyint: name = "nearbyint";
        break;
      case rintf: name = "rintf";
        break;
      case rint: name = "rint";
        break;
      case lround: name = "lround";
        break;
      case lroundf: name = "lroundf";
        break;
      case memcpy: name = "memcpy";
        break;
      case memset: name = "memset";
        break;
      case memmove: name = "memmove";
      break;
      case memcmp: name = "memcmp";
      break;
      case resume: name = "_Unwind_Resume";
        break;
      case powisf2: name = "__powisf2";
        break;
      case powidf2: name = "__powidf2";
        break;
      case trunc: name = "trunc";
        break;
      case truncf: name = "truncf";
        break;
      case fma: name = "fma";
        break;
      case fmaf: name = "fmaf";
        break;
      case pow: name = "pow";
        break;
      case powf: name = "powf";
        break;
      case sin: name = "sin";
        break;
      case sinf: name = "sinf";
        break;
      case cos: name = "cos";
        break;
      case cosf: name = "cosf";
        break;
      case tan: name = "tan";
        break;
      case tanf: name = "tanf";
        break;
      case asin: name = "asin";
        break;
      case asinf: name = "asinf";
        break;
      case acos: name = "acos";
        break;
      case acosf: name = "acosf";
        break;
      case atan: name = "atan";
        break;
      case atanf: name = "atanf";
        break;
      case atan2: name = "atan2";
        break;
      case atan2f: name = "atan2f";
        break;
      case sinh: name = "sinh";
        break;
      case sinhf: name = "sinhf";
        break;
      case cosh: name = "cosh";
        break;
      case coshf: name = "coshf";
        break;
      case tanh: name = "tanh";
        break;
      case tanhf: name = "tanhf";
        break;
      case log: name = "log";
        break;
      case logf: name = "logf";
        break;
      case logl: name = "logl";
        break;
      case logf128: name = "logf128";
        break;
      case log2: name = "log2";
        break;
      case log2f: name = "log2f";
        break;
      case log10: name = "log10";
        break;
      case log10f: name = "log10f";
        break;
      case exp: name = "exp";
        break;
      case expf: name = "expf";
        break;
      case exp2: name = "exp2";
        break;
      case exp2f: name = "exp2f";
        break;
      case modf: name = "modf";
        break;
      case modff: name = "modff";
        break;
      case frexp: name = "frexp";
        break;
      case frexpf: name = "frexpf";
        break;
      case trunctfsf2: name = "__trunctfsf2";
        break;
      case trunctfdf2: name = "__trunctfdf2";
        break;
      case extendsftf2: name = "__extendsftf2";
        break;
      case extenddftf2: name = "__extenddftf2";
        break;
      case eqtf2: name = "__eqtf2";
        break;
      case netf2: name = "__netf2";
        break;
      case gttf2: name = "__gttf2";
        break;
      case getf2: name = "__getf2";
        break;
      case lttf2: name = "__lttf2";
        break;
      case letf2: name = "__letf2";
        break;
      case unordtf2: name = "__unordtf2";
        break;
      case floatsitf: name = "__floatsitf";
        break;
      case floatditf: name = "__floatditf";
        break;
      case floatunsitf: name = "__floatunsitf";
        break;
      case floatunditf: name = "__floatunditf";
        break;
      case fixtfdi: name = "__fixtfdi";
        break;
      case fixunstfdi: name = "__fixunstfdi";
        break;
      case addtf3: name = "__addtf3";
        break;
      case subtf3: name = "__subtf3";
        break;
      case multf3: name = "__multf3";
        break;
      case divtf3: name = "__divtf3";
        break;
      default: TPDE_UNREACHABLE("invalid libfunc");
    }

    sym =
        this->assembler.sym_add_undef(name, tpde::Assembler::SymBinding::GLOBAL);
    return sym;
  }

  static tpde::Assembler::SymBinding convert_linkage(const Global &global) {
    if (global.flags.only_local)
      return tpde::Assembler::SymBinding::LOCAL;
    if (global.flags.weak_link)
      return tpde::Assembler::SymBinding::WEAK;
    return tpde::Assembler::SymBinding::GLOBAL;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::hook_post_func_sym_init() {
    global_symbols.clear();

    global_symbols.reserve(this->adaptor->mod->globals.size());
    for (const Global &global: this->adaptor->mod->globals) {
      std::string_view name(global.name.data(), global.name.size());

      auto binding = convert_linkage(global);
      SymRef ref;
      if (global.thread_loc) {
        ref = this->assembler.sym_predef_tls(name, binding);
      } else if (global.flags.extern_link) {
        ref = this->assembler.sym_add_undef(name, binding);
      } else {
        ref = this->assembler.sym_predef_data(name, binding);
      }
      global_symbols.push_back(ref);
    }

    for (size_t i = 0; i < global_symbols.size(); ++i) {
      Global global = this->adaptor->mod->globals[i];
      if (global.flags.extern_link)
        continue;

      SymRef &sym = global_symbols[i];

      tpde::SectionKind kind;
      {
        bool needs_relocs = !global.relocations.empty();
        bool init_zero = !global.init;
        bool read_only = global.read_only;
        if (global.thread_loc) {
          kind = init_zero ? tpde::SectionKind::ThreadBSS : tpde::SectionKind::ThreadData;
        } else if (!read_only && init_zero) {
          assert(!needs_relocs && "BSS section must not have relocations");
          kind = tpde::SectionKind::BSS;
        } else if (read_only) {
          kind = needs_relocs ? tpde::SectionKind::DataRelRO : tpde::SectionKind::ReadOnly;
        } else {
          kind = tpde::SectionKind::Data;
        }
      }
      SecRef sec = this->assembler.create_section(kind);

      if (global.init) {
        u32 off;
        this->assembler.sym_def_predef_data(sec, sym, global.data, global.align, &off);
        for (Relocation &reloc: global.relocations) {
          if (operands::is_global(reloc.slot)) {
            SymRef &target = global_symbols[operands::content(reloc.slot)];
            this->assembler.reloc_abs(sec, target, off + reloc.offset, 0);
          } else if (operands::is_func(reloc.slot)) {
            SymRef target = this->func_syms[operands::content(reloc.slot)];
            this->assembler.reloc_abs(sec, target, off + reloc.offset, 0);
          } else {
            assert(false);
          }
        }
      } else {
        this->assembler.sym_def_predef_zero(sec, sym, global.size, global.align);
      }
    }

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_br(RustAdaptor::IRInstRef instr, const ValInfo &, u64) {
    Instruction &bri = this->adaptor->get_instruction(instr);

    assert(operands::is_raw(bri.ops[0]));
    Base::generate_uncond_branch(operands::content(bri.ops[0]));

    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_switch(RustAdaptor::IRInstRef inst_ref, const ValInfo &, u64) {
    Instruction& inst = this->adaptor->get_instruction(inst_ref);

    auto cond_ref = inst.ops[0];
    u32 width = size_of_type(this->adaptor->type_of_ref(cond_ref));
    if (width > 64) {
      return false;
    }

    // Collect cases, their target block and sort them in ascending order.
    tpde::util::SmallVector<std::pair<u64, IRBlockRef>, 64> cases;
    size_t num_cases = inst.ops.size() / 2 - 1;
    assert(num_cases <= 200000);
    cases.reserve(num_cases);
    for (size_t i = 0; i < num_cases; ++i) {
      cases.push_back(std::make_pair(
          static_cast<u64>(operands::content(inst.ops[2 + 2 * i])),
          operands::content(inst.ops[2 + 2 * i + 1])));
    }
    std::sort(cases.begin(), cases.end(), [](const auto &lhs, const auto &rhs) {
      return lhs.first < rhs.first;
    });

    IRBlockRef def = operands::content(inst.ops[1]);

    // cond must be ref-counted before generate_switch.
    ScratchReg cond_scratch = this->val_ref(cond_ref).part(0).into_scratch();
    this->generate_switch(std::move(cond_scratch), width, def, cases);
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_select(RustAdaptor::IRInstRef inst_ref, const ValInfo &val_info, u64) {
    Instruction& inst = this->adaptor->get_instruction(inst_ref);

    auto [cond_vr, cond] = this->val_ref_single(inst.ops[0]);
    auto lhs = this->val_ref(inst.ops[1]);
    auto rhs = this->val_ref(inst.ops[2]);

    auto res = this->result_ref(inst.result);

    switch (val_info.type) {
      using enum Type;
      case Bool:
      case i8:
      case i16:
      case i32:
        derived()->encode_select_i32(
            std::move(cond), lhs.part(0), rhs.part(0), res.part(0));
        break;
      case i64:
      case ptr:
        derived()->encode_select_i64(
            std::move(cond), lhs.part(0), rhs.part(0), res.part(0));
        break;
      case f32:
        derived()->encode_select_f32(
            std::move(cond), lhs.part(0), rhs.part(0), res.part(0));
        break;
      case f64:
        derived()->encode_select_f64(
            std::move(cond), lhs.part(0), rhs.part(0), res.part(0));
        break;
      case i128: {
        derived()->encode_select_i128(std::move(cond),
                                      lhs.part(0),
                                      lhs.part(1),
                                      rhs.part(0),
                                      rhs.part(1),
                                      res.part(0),
                                      res.part(1));
        break;
      }
      default: TPDE_UNREACHABLE("invalid select basic type"); break;
    }
    return true;
  }

  template<typename Adaptor, typename Derived, typename Config>
  bool RustCompilerBase<Adaptor, Derived, Config>::compile_unreachable(RustAdaptor::IRInstRef, const ValInfo &, u64) {
    derived()->encode_trap();
    this->release_regs_after_return();
    return true;
  }
}
