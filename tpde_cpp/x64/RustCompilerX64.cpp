#include "RustCompilerX64.h"

#include "encode_template_x64.hpp"

namespace tpde_rust::x64 {

  struct CompilerConfig : tpde::x64::PlatformConfig {
    static constexpr bool DEFAULT_VAR_REF_HANDLING = false;
  };

  struct RustCompilerX64 :
    tpde::x64::CompilerX64<
      RustAdaptor,
      RustCompilerX64,
      RustCompilerBase,
      CompilerConfig
    >,
    tpde_encodegen::EncodeCompiler<
      RustAdaptor,
      RustCompilerX64,
      RustCompilerBase,
      CompilerConfig
    >  {
    using Base = tpde::x64::CompilerX64<
      RustAdaptor,
      RustCompilerX64,
      RustCompilerBase,
      CompilerConfig
    >;

    using ScratchReg = Base::ScratchReg;
    using ValuePartRef = Base::ValuePartRef;
    using ValuePart = Base::ValuePart;
    using ValueRef = Base::ValueRef;
    using GenericValuePart = Base::GenericValuePart;

    using AsmReg = Base::AsmReg;

    std::unique_ptr<RustAdaptor> adaptor;

    std::variant<std::monostate, tpde::x64::CCAssignerSysV> cc_assigners;

    static constexpr std::array<AsmReg, 2> LANDING_PAD_RES_REGS = {AsmReg::AX,
                                                                   AsmReg::DX};

    explicit RustCompilerX64(std::unique_ptr<RustAdaptor> &&adaptor)
      : Base{adaptor.get()},
        adaptor(std::move(adaptor)) {
      static_assert(tpde::Compiler<RustCompilerX64, CompilerConfig>);
    }

    void reset() {
      Base::reset();
      EncodeCompiler::reset();
    }

    bool arg_allow_split_reg_stack_passing(IRValueRef value) const {
      // All types except i128 can be split across registers/stack.
      return this->adaptor->type_of_ref(value) != Type::i128;
    }

    bool compile_cmp(RustAdaptor::IRInstRef inst, const ValInfo &, u64);
    bool compile_condbr(RustAdaptor::IRInstRef, const ValInfo &, u64);
    bool compile_overflow_jump(Instruction&, InstructionKind, bool);

    bool compile_pause(RustAdaptor::IRInstRef, const ValInfo &, u64);

    static GenericValuePart create_addr_for_alloca(tpde::AssignmentPartRef ap);

    void create_helper_call(std::span<IRValueRef> args,
                        ValueRef *result,
                        SymRef sym);

    std::optional<CallBuilder> create_call_builder();

    void load_address_of_var_reference(tpde::x64::AsmReg dst, tpde::AssignmentPartRef ap);

    bool handle_overflow_intrin_128(OverflowOp op,
                                GenericValuePart &&lhs_lo,
                                GenericValuePart &&lhs_hi,
                                GenericValuePart &&rhs_lo,
                                GenericValuePart &&rhs_hi,
                                ValuePart &&res_lo,
                                ValuePart &&res_hi,
                                ValuePart &&res_of);
  };

  std::unique_ptr<RustCompiler> create_compiler() {
    auto adaptor = std::make_unique<RustAdaptor>();
    return std::make_unique<RustCompilerX64>(std::move(adaptor));
  }

  bool RustCompilerX64::compile_cmp(const RustAdaptor::IRInstRef inst, const ValInfo &, u64) {
    const Instruction cmpi = this->adaptor->get_instruction(inst);

    Type type = this->adaptor->type_of_ref(cmpi.ops[0]);
    if (is_vector(type)) {
      return compile_icmp_vector(this->adaptor->get_instruction(inst));
    }
    u32 int_width = size_of_type(type);

    Jump jump;
    bool is_signed = false;
    switch (cmpi.kind) {
      case InstructionKind::CMPeq: jump = Jump::je; break;
      case InstructionKind::CMPne: jump = Jump::jne; break;
      case InstructionKind::CMPugt: jump = Jump::ja; break;
      case InstructionKind::CMPuge: jump = Jump::jae; break;
      case InstructionKind::CMPult: jump = Jump::jb; break;
      case InstructionKind::CMPule: jump = Jump::jbe; break;
      case InstructionKind::CMPsgt:
        jump = Jump::jg;
        is_signed = true;
        break;
      case InstructionKind::CMPsge:
        jump = Jump::jge;
        is_signed = true;
        break;
      case InstructionKind::CMPslt:
        jump = Jump::jl;
        is_signed = true;
        break;
      case InstructionKind::CMPsle:
        jump = Jump::jle;
        is_signed = true;
        break;
      default: TPDE_UNREACHABLE("invalid icmp predicate");
    }

    const Instruction& jmp_instr = this->adaptor->get_instruction(inst.next());
    bool fuse_br = jmp_instr.kind == InstructionKind::CondBr && jmp_instr.ops[0] == cmpi.result;

    const auto local_idx = this->adaptor->val_local_idx(cmpi.result);
    const bool single_use = this->analyzer.liveness_info(local_idx).ref_count == 2;

    auto lhs = this->val_ref(cmpi.ops[0]);
    auto rhs = this->val_ref(cmpi.ops[1]);

    if (int_width > 64) {
      assert(int_width <= 128);
      // for 128 bit compares, we need to swap the operands sometimes
      if ((jump == Jump::ja) || (jump == Jump::jbe) || (jump == Jump::jle) ||
          (jump == Jump::jg)) {
        std::swap(lhs, rhs);
        jump = swap_jump(jump);
      }

      auto rhs_lo = rhs.part(0);
      auto rhs_hi = rhs.part(1);
      auto lhs_hi = lhs.part(1);
      if (int_width < 128) {
        lhs_hi = std::move(lhs_hi).into_extended(is_signed, int_width - 64, 64);
        rhs_hi = std::move(rhs_hi).into_extended(is_signed, int_width - 64, 64);
      }
      lhs_hi = std::move(lhs_hi).into_temporary();
      auto rhs_reg_lo = rhs_lo.load_to_reg();
      auto rhs_reg_hi = rhs_hi.cur_reg_or_load();

      // Compare the ints using carried subtraction
      if ((jump == Jump::je) || (jump == Jump::jne)) {
        // for eq,neq do something a bit quicker
        auto lhs_lo = lhs.part(0).into_temporary();
        ASM(XOR64rr, lhs_lo.cur_reg(), rhs_reg_lo);
        ASM(XOR64rr, lhs_hi.cur_reg(), rhs_reg_hi);
        ASM(OR64rr, lhs_lo.cur_reg(), lhs_hi.cur_reg());
      } else {
        auto lhs_lo = lhs.part(0);
        auto lhs_reg_lo = lhs_lo.load_to_reg();
        ASM(CMP64rr, lhs_reg_lo, rhs_reg_lo);
        ASM(SBB64rr, lhs_hi.cur_reg(), rhs_reg_hi);
      }
    } else {
      ValuePartRef lhs_op = lhs.part(0);
      ValuePartRef rhs_op = rhs.part(0);

      if (lhs_op.is_const() && !rhs_op.is_const()) {
        std::swap(lhs_op, rhs_op);
        jump = swap_jump(jump);
      }

      if (int_width < 8 || (int_width & (int_width - 1))) {
        // We could handle comparisons of integers <32 bit against zero with
        // TESTri. They occur very rarely and are not worth the effort.
        unsigned ext_bits = tpde::util::align_up(int_width, 32);
        lhs_op = std::move(lhs_op).into_extended(is_signed, int_width, ext_bits);
        rhs_op = std::move(rhs_op).into_extended(is_signed, int_width, ext_bits);
        int_width = ext_bits;
      }

      // We can do comparisons against small immediates more efficiently.
      i64 rhs_val = rhs_op.is_const() ? rhs_op.const_data()[0] : 0;
      if (rhs_op.is_const() && (int_width <= 32 || i32(rhs_val) == rhs_val)) {
        // Comparison of 8/16/32/64-bit can use CMPmi. Only do so if the value
        // doesn't reside in a register.
        if (lhs_op.has_assignment()) {
          tpde::AssignmentPartRef ap = lhs_op.assignment();
          if (!ap.register_valid() && ap.stack_valid()) {
            FeMem mem = FE_MEM(FE_BP, 0, FE_NOREG, ap.frame_off());
            switch (int_width) {
            case 8: ASM(CMP8mi, mem, i8(rhs_val)); goto done_compare;
            case 16: ASM(CMP16mi, mem, i16(rhs_val)); goto done_compare;
            case 32: ASM(CMP32mi, mem, i32(rhs_val)); goto done_compare;
            case 64: ASM(CMP64mi, mem, rhs_val); goto done_compare;
            default: TPDE_UNREACHABLE("impossible int bit width");
            }
          }
        }

        auto lhs_reg = lhs_op.has_reg() ? lhs_op.cur_reg() : lhs_op.load_to_reg();
        if (rhs_val == 0) {
          // Comparison of register with zero is TESTrr/TESTri.
          switch (int_width) {
          case 8: ASM(TEST8rr, lhs_reg, lhs_reg); break;
          case 16: ASM(TEST16rr, lhs_reg, lhs_reg); break;
          case 32: ASM(TEST32rr, lhs_reg, lhs_reg); break;
          case 64: ASM(TEST64rr, lhs_reg, lhs_reg); break;
          default: TPDE_UNREACHABLE("impossible int bit width");
          }
        } else {
          // Comparison of 8/16/32/64-bit is CMPri.
          switch (int_width) {
          case 8: ASM(CMP8ri, lhs_reg, i8(rhs_val)); break;
          case 16: ASM(CMP16ri, lhs_reg, i16(rhs_val)); break;
          case 32: ASM(CMP32ri, lhs_reg, i32(rhs_val)); break;
          case 64: ASM(CMP64ri, lhs_reg, rhs_val); break;
          default: TPDE_UNREACHABLE("impossible int bit width");
          }
        }
      } else {
        auto lhs_reg = lhs_op.has_reg() ? lhs_op.cur_reg() : lhs_op.load_to_reg();
        auto rhs_reg = rhs_op.has_reg() ? rhs_op.cur_reg() : rhs_op.load_to_reg();
        switch (int_width) {
        case 8: ASM(CMP8rr, lhs_reg, rhs_reg); break;
        case 16: ASM(CMP16rr, lhs_reg, rhs_reg); break;
        case 32: ASM(CMP32rr, lhs_reg, rhs_reg); break;
        case 64: ASM(CMP64rr, lhs_reg, rhs_reg); break;
        default: TPDE_UNREACHABLE("impossible int bit width");
        }
      }

    done_compare:;
    }

    // No need for set_preserve_flags; we don't call helpers that could
    // potentially clobber them.

    // ref-count, otherwise phi assignment will think that value is still used
    lhs.reset();
    rhs.reset();

    if (fuse_br) {
      this->adaptor->next_fused = true;
      if (!single_use) {
        (void)result_ref(cmpi.result); // ref-count for branch
        generate_raw_set(
            jump, result_ref(cmpi.result).part(0).alloc_reg(), /*zext=*/false);
      }
      assert(operands::is_raw(jmp_instr.ops[1]));
      assert(operands::is_raw(jmp_instr.ops[2]));
      generate_cond_branch(jump, operands::content(jmp_instr.ops[1]), operands::content(jmp_instr.ops[2]));
    } else {
      auto [_, res_ref] = result_ref_single(cmpi.result);
      generate_raw_set(jump, res_ref.alloc_reg(), /*zext=*/false);
    }

    return true;
  }

  bool RustCompilerX64::compile_condbr(RustAdaptor::IRInstRef instr, const ValInfo &, u64) {
    Instruction& condbr = adaptor->get_instruction(instr);
    assert(this->adaptor->type_of_ref(condbr.ops[0]) == Type::Bool);

    ValueRef cond = this->val_ref(condbr.ops[0]);
    ValuePartRef cond_op = cond.part(0);
    const AsmReg cond_reg = cond_op.cur_reg_or_load();

    ASM(TEST8ri, cond_reg, 1);
    generate_cond_branch(Jump::jne, operands::content(condbr.ops[1]), operands::content(condbr.ops[2]));
    return true;
  }

  bool RustCompilerX64::compile_overflow_jump(Instruction& jmpi, InstructionKind kind, bool is_signed) {
    Jump jump;
    switch (kind) {
      case InstructionKind::Add:
      case InstructionKind::Sub:
        jump = is_signed ? Jump::jo : Jump::jb;
        break;
      case InstructionKind::Mul:
        jump = Jump::jo;
        break;
      default: TPDE_UNREACHABLE("Invalid op for overflow");
    }
    generate_cond_branch(jump, operands::content(jmpi.ops[1]), operands::content(jmpi.ops[2]));
    return true;
  }

  bool RustCompilerX64::compile_pause(RustAdaptor::IRInstRef, const ValInfo &, u64) {
    ASM(PAUSE);
    return true;
  }

  RustCompilerX64::GenericValuePart
    RustCompilerX64::create_addr_for_alloca(tpde::AssignmentPartRef ap) {
    return GenericValuePart::Expr{AsmReg::BP, ap.variable_stack_off()};
  }

  void RustCompilerX64::create_helper_call(std::span<IRValueRef> args, ValueRef *result, SymRef sym) {
    tpde::util::SmallVector<CallArg, 8> arg_vec{};
    for (auto arg : args) {
      arg_vec.push_back(CallArg{arg});
    }

    generate_call(sym, arg_vec, result);
  }

  std::optional<tpde::x64::CompilerX64<RustAdaptor, RustCompilerX64, RustCompilerBase, CompilerConfig>::CallBuilder>
  RustCompilerX64::create_call_builder() {
    cc_assigners = tpde::x64::CCAssignerSysV(false);
    return CallBuilder{*this, std::get<tpde::x64::CCAssignerSysV>(cc_assigners)};
  }

  void RustCompilerX64::load_address_of_var_reference(tpde::x64::AsmReg dst, tpde::AssignmentPartRef ap) {
    const uint32_t glob_ptr_start = this->adaptor->mod->globals.size();
    const uint32_t func_start = glob_ptr_start + this->adaptor->mod->global_ptrs.size();

    uint32_t glob_id = ap.variable_ref_data();
    uint32_t offset = 0;

    if (glob_id >= func_start) {
      // A function used as a value: load the address of its symbol.
      const uint32_t func_id = glob_id - func_start;
      assert(func_id < this->adaptor->mod->functions.size());
      assert(func_id < this->func_syms.size());
      const auto sym = this->func_syms[func_id];
      assert(sym.valid());

      if (this->adaptor->mod->functions[func_id].flags.extern_link) {
        // mov the ptr from the GOT
        ASM(MOV64rm, dst, FE_MEM(FE_IP, 0, FE_NOREG, -1));
        reloc_text(sym, tpde::elf::R_X86_64_GOTPCREL, text_writer.offset() - 4, -4);
      } else {
        // emit lea with relocation
        ASM(LEA64rm, dst, FE_MEM(FE_IP, 0, FE_NOREG, -1));
        reloc_text(sym, tpde::elf::R_X86_64_PC32, text_writer.offset() - 4, -4);
      }
      return;
    }

    if (glob_id >= glob_ptr_start) {
      auto &[id, off] = this->adaptor->mod->global_ptrs[glob_id - glob_ptr_start];
      glob_id = id;
      offset = off;
    }
    assert(glob_id < this->adaptor->mod->globals.size());
    assert(glob_id < this->global_symbols.size());
    const Global& global = this->adaptor->mod->globals[glob_id];
    const auto sym = this->global_symbols[glob_id];
    assert(sym.valid());

    if (global.flags.extern_link) {
      // mov the ptr from the GOT
      ASM(MOV64rm, dst, FE_MEM(FE_IP, 0, FE_NOREG, -1));
      reloc_text(sym, tpde::elf::R_X86_64_GOTPCREL, text_writer.offset() - 4, - 4);
      if (offset != 0) {
        // TODO assert, dass conversion zu i32 geht
        // TODO assert, dass i32 insgesamt passt
        // TODO assert, dass das kein thread local ist
        ASM(LEA64rm, dst, FE_MEM(dst, 0, FE_NOREG, static_cast<int32_t>(offset)));
      }
    } else {
      // emit lea with relocation
      ASM(LEA64rm, dst, FE_MEM(FE_IP, 0, FE_NOREG, -1));
      reloc_text(sym, tpde::elf::R_X86_64_PC32, text_writer.offset() - 4, static_cast<int64_t>(offset) - 4);
    }
  }

  bool RustCompilerX64::handle_overflow_intrin_128(OverflowOp op, GenericValuePart &&lhs_lo, GenericValuePart &&lhs_hi,
    GenericValuePart &&rhs_lo, GenericValuePart &&rhs_hi, ValuePart &&res_lo, ValuePart &&res_hi, ValuePart &&res_of) {

    using EncodeFnTy = bool (RustCompilerX64::*)(GenericValuePart &&,
                                                 GenericValuePart &&,
                                                 GenericValuePart &&,
                                                 GenericValuePart &&,
                                                 ValuePart &,
                                                 ValuePart &,
                                                 ValuePart &);
    EncodeFnTy encode_fn = nullptr;
    switch (op) {
      using enum OverflowOp::Value;
      case uadd:
        encode_fn = &RustCompilerX64::encode_of_add_u128;
        break;
      case sadd:
        encode_fn = &RustCompilerX64::encode_of_add_i128;
        break;
      case usub:
        encode_fn = &RustCompilerX64::encode_of_sub_u128;
        break;
      case ssub:
        encode_fn = &RustCompilerX64::encode_of_sub_i128;
        break;
      case umul:
        encode_fn = &RustCompilerX64::encode_of_mul_u128;
        break;
      case smul:
        encode_fn = &RustCompilerX64::encode_of_mul_i128;
        break;
      default: TPDE_UNREACHABLE("invalid operation");
    }

    return (this->*encode_fn)(std::move(lhs_lo),
                              std::move(lhs_hi),
                              std::move(rhs_lo),
                              std::move(rhs_hi),
                              res_lo,
                              res_hi,
                              res_of);
  }
}
