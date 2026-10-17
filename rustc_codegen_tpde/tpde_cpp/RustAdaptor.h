#pragma once

#include "tpde.h"

#include <../deps/tpde/tpde-llvm/src/base.hpp>
#include <tpde/IRAdaptor.hpp>
#include <tpde/RegisterFile.hpp>
#include <generator>
#include <span>
#include <unordered_set>
#include <type_traits>

#include "tpde/util/SmallVector.hpp"

namespace tpde_rust {
  struct RustAdaptor;
}

namespace IR {
  inline constexpr uint32_t MARKER_BLOCK = size_t{7} << (std::numeric_limits<std::uint32_t>::digits - 3);

  inline constexpr uint32_t MARKER_VAL = size_t{0} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_CONST = size_t{1} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_RAW = size_t{2} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_ALLOC = size_t{3} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_FUNC = size_t{4} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_GLOBAL = size_t{5} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_GLOBAL_PTR = size_t{6} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_CONST_VECTOR = size_t{7} << (std::numeric_limits<std::uint32_t>::digits - 3);

  inline constexpr uint32_t INLINE_CONST_BITS = 6;
  inline constexpr uint32_t INLINE_CONST_MASK = (uint32_t{1} << INLINE_CONST_BITS) - 1;
  inline constexpr uint32_t INLINE_CONST_END = static_cast<uint32_t>(Type::Last) << INLINE_CONST_BITS;

  class Operand {
    u32 value;

  public:
    constexpr explicit Operand(u32 op) : value(op) {}

  private:
    [[nodiscard]] constexpr bool is(uint32_t marker) const {
      return (value & MARKER_BLOCK) == marker;
    }

  public:
    [[nodiscard]] constexpr bool is_val() const {
      return is(MARKER_VAL);
    }

    [[nodiscard]] constexpr bool is_const() const {
      return is(MARKER_CONST);
    }

    [[nodiscard]] constexpr bool is_raw() const {
      return is(MARKER_RAW);
    }

    [[nodiscard]] constexpr bool is_alloc() const {
      return is(MARKER_ALLOC);
    }

    [[nodiscard]] constexpr bool is_func() const {
      return is(MARKER_FUNC);
    }

    [[nodiscard]] constexpr bool is_global() const {
      return is(MARKER_GLOBAL);
    }

    [[nodiscard]] constexpr bool is_global_ptr() const {
      return is(MARKER_GLOBAL_PTR);
    }

    [[nodiscard]] constexpr bool is_const_vector() const {
      return is(MARKER_CONST_VECTOR);
    }

    [[nodiscard]] constexpr uint32_t content() const {
      return value & ~MARKER_BLOCK;
    }

    [[nodiscard]] bool operator==(const Operand &other) const {
      return this->value == other.value;
    }
  };

  struct ToOperand {
    constexpr Operand operator()(const u32 op) const { return Operand(op); }
  };
  using OperandView = std::ranges::transform_view<std::span<const u32>, ToOperand>;

  static_assert(sizeof(Operand) == sizeof(u32) && std::is_trivially_copyable_v<Operand>);

  class Instr {
    const Instruction& instruction;
    const tpde_rust::RustAdaptor& adaptor;

  public:
    Instr(const Instruction& instruction, const tpde_rust::RustAdaptor& adaptor)
      : instruction(instruction), adaptor(adaptor) {}

    [[nodiscard]] InstructionKind kind() const {
      return instruction.kind;
    }

    [[nodiscard]] bool is(InstructionKind kind) const {
      return instruction.kind == kind;
    }

    [[nodiscard]] bool has_result() const {
      return instruction.has_result;
    }

    [[nodiscard]] Operand result() const {
      return Operand(instruction.result);
    }

    [[nodiscard]] u32 num_ops() const {
      return instruction.ops_len;
    }

    [[nodiscard]] Operand op(size_t i) const;
    [[nodiscard]] OperandView ops() const;
  };
}

namespace tpde_rust {
  // doing this with a lambda throws magical errors
  struct PointifyFunctor {
    template<typename T>
    auto operator()(T &item) const {
      return &item;
    }
  };

  constexpr auto view_pointify = std::views::transform(PointifyFunctor{});

  struct RustAdaptor {
    using IRValueRef = IR::Operand;
    using IRBlockRef = uint32_t;

    struct IRInstRef {
      size_t inst;
      IRBlockRef block;

      [[nodiscard]] IRInstRef next() const {
        return {.inst = inst + 1, .block = block};
      }

      [[nodiscard]] IRInstRef prev() const {
        return {.inst = inst - 1, .block = block};
      }
    };

    using IRFuncRef = Function *;

    static constexpr IRValueRef INVALID_VALUE_REF = IR::Operand(static_cast<u32>(-1));
    static constexpr IRBlockRef INVALID_BLOCK_REF = ~0u;
    static constexpr IRFuncRef INVALID_FUNC_REF = nullptr;

    static constexpr bool TPDE_PROVIDES_HIGHEST_VAL_IDX = true;
    static constexpr bool TPDE_LIVENESS_VISIT_ARGS = true;

    ModuleTpde *mod = nullptr;
    Function *cur_func = nullptr;

    template<typename T>
    [[nodiscard]] static std::span<T> view(rust::Vec<T> &vec) {
      return {vec.data(), vec.size()};
    }

#ifndef TPDE_RUST_NO_CACHE
    // Every call to the cxxbridge taxes a lot of time, wile nothing is change in the IR while generating
    // We can just buffer when switching the function and use it for generation
    struct ModuleData {
      std::span<Function> functions;
      std::span<Value> consts;
      std::span<Global> globals;
      std::span<GlobalPtr> global_ptrs;
      std::span<ConstVector> const_vectors;
    } mod_data;

    struct FuncData {
      std::span<ArgInfo> args;
      std::span<Slot> slots;
      std::span<const u32> operands;
      std::span<Instruction> instructions;
      std::span<Alloca> allocas;
      std::span<BasicBlock> basic_blocks;
      std::span<CalleeInfo> callee_infos;
    } cur_data;

    [[nodiscard]] std::span<Function> get_functions() const { return mod_data.functions; }
    [[nodiscard]] std::span<Value> get_consts() const { return mod_data.consts; }
    [[nodiscard]] std::span<Global> get_globals() const { return mod_data.globals; }
    [[nodiscard]] std::span<GlobalPtr> get_global_ptrs() const { return mod_data.global_ptrs; }
    [[nodiscard]] std::span<ConstVector> get_const_vectors() const { return mod_data.const_vectors; }

    [[nodiscard]] std::span<ArgInfo> get_args() const { return cur_data.args; }
    [[nodiscard]] std::span<Slot> get_slots() const { return cur_data.slots; }
    [[nodiscard]] std::span<const u32> get_operands() const { return cur_data.operands; }
    [[nodiscard]] std::span<Instruction> get_instructions() const { return cur_data.instructions; }
    [[nodiscard]] std::span<Alloca> get_allocas() const { return cur_data.allocas; }
    [[nodiscard]] std::span<BasicBlock> get_basic_blocks() const { return cur_data.basic_blocks; }
    [[nodiscard]] std::span<CalleeInfo> get_callee_infos() const { return cur_data.callee_infos; }
#else
    // no-cache: go through the cxxbridge on every access, to measure what the caching gains
    [[nodiscard]] std::span<Function> get_functions() const { return view(mod->functions); }
    [[nodiscard]] std::span<Value> get_consts() const { return view(mod->consts); }
    [[nodiscard]] std::span<Global> get_globals() const { return view(mod->globals); }
    [[nodiscard]] std::span<GlobalPtr> get_global_ptrs() const { return view(mod->global_ptrs); }
    [[nodiscard]] std::span<ConstVector> get_const_vectors() const { return view(mod->const_vectors); }

    [[nodiscard]] std::span<ArgInfo> get_args() const { return view(cur_func->args); }
    [[nodiscard]] std::span<Slot> get_slots() const { return view(cur_func->slots); }
    [[nodiscard]] std::span<const u32> get_operands() const { return view(cur_func->operands); }
    [[nodiscard]] std::span<Instruction> get_instructions() const { return view(cur_func->instructions); }
    [[nodiscard]] std::span<Alloca> get_allocas() const { return view(cur_func->allocas); }
    [[nodiscard]] std::span<BasicBlock> get_basic_blocks() const { return view(cur_func->basic_blocks); }
    [[nodiscard]] std::span<CalleeInfo> get_callee_infos() const { return view(cur_func->callee_infos); }
#endif

    // wrapper for handling inlined constant values
    [[nodiscard]] Value const_value(const IRValueRef value) const {
      const u32 c = value.content();
      if (c < IR::INLINE_CONST_END) {
        return Value{
          .ty = static_cast<Type>(c >> IR::INLINE_CONST_BITS),
          .data1 = 0,
          .data2 = c & IR::INLINE_CONST_MASK,
        };
      }
      return get_consts()[c - IR::INLINE_CONST_END];
    }

    [[nodiscard]] Type type_of_ref(const IRValueRef value) const {
      if (value.is_val())
        return get_slots()[value.content()].ty;
      if (value.is_const())
        return const_value(value).ty;
      if (value.is_const_vector())
        return get_const_vectors()[value.content()].ty;
      if (value.is_alloc() || value.is_global() || value.is_global_ptr()
          || value.is_func())
        return Type::ptr;
      assert(false && "invalid value ref");
    }

    /// Element type for vectors, the type itself otherwise.
    [[nodiscard]] Type elem_type_of_ref(const IRValueRef value) const {
      const Type ty = type_of_ref(value);
      return is_vector(ty) ? vector_info(ty).second : ty;
    }

    [[nodiscard]] BasicBlock &get_basic_block(const IRBlockRef block) const {
      return get_basic_blocks()[block];
    }

    [[nodiscard]] IR::Instr get_instruction(const IRInstRef inst) const {
      return IR::Instr(get_instructions()[get_basic_block(inst.block).insts_start + inst.inst], *this);
    }

  private:
    [[nodiscard]] std::span<Instruction> block_instructions(const IRBlockRef block) const {
      const auto &bb = get_basic_block(block);
      return get_instructions().subspan(bb.insts_start, bb.insts_len);
    }
  public:

    struct ValInfo {
      Type type;
    };

    [[nodiscard]] u32 func_count() const { return get_functions().size(); }

    [[nodiscard]] auto funcs() const {
      return get_functions() | std::views::transform([](Function &fn) { return &fn; });
    }

    [[nodiscard]] auto funcs_to_compile() const { return funcs(); }

    [[nodiscard]] static std::string_view func_link_name(const IRFuncRef func) {
      return func->name.c_str();
    }

    [[nodiscard]] static bool func_extern(const IRFuncRef func) {
      return func->flags.extern_link;
    }

    [[nodiscard]] static bool func_only_local(const IRFuncRef func) {
      return func->flags.only_local;
    }

    [[nodiscard]] static bool func_has_weak_linkage(const IRFuncRef func) {
      return func->flags.weak_link;
    }

    [[nodiscard]] static bool cur_needs_unwind_info() {
      // TODO cur_func need unwind?
      return true;
    }

    [[nodiscard]] static bool cur_is_vararg() {
      // TODO cur_func is vararg?
      return false;
    }

    [[nodiscard]] u32 cur_highest_val_idx() const {
      return get_allocas().size() + get_slots().size() + get_globals().size() + get_global_ptrs().size()
             + get_functions().size();
    }

    [[nodiscard]] auto cur_args() const {
      return std::views::iota(0u, static_cast<u32>(get_args().size()))
        | std::views::transform([](const u32 idx) { return IR::Operand(IR::MARKER_VAL | idx); });
    }

    [[nodiscard]] bool cur_arg_is_byval(const u32 idx) const {
      return get_args()[idx].kind == ArgKind::ByVal;
    }

    [[nodiscard]] u32 cur_arg_byval_size(const u32 idx) const {
      return get_args()[idx].size;
    }

    [[nodiscard]] u32 cur_arg_byval_align(const u32 idx) const {
      return get_args()[idx].align;
    }

    [[nodiscard]] u32 cur_arg_is_sret(const u32 idx) const {
      return get_args()[idx].kind == ArgKind::sRet;
    }

    [[nodiscard]] auto cur_static_allocas() const {
      return std::ranges::views::iota(static_cast<u32>(0), static_cast<u32>(get_allocas().size()))
        | std::views::transform([](const u32 idx) { return IR::Operand(IR::MARKER_ALLOC | idx); });
    }

    [[nodiscard]] static auto cur_has_dynamic_alloca() {
      // TODO
      return false;
    }

    [[nodiscard]] IRBlockRef cur_entry_block() const {
      return 0;
    }

    [[nodiscard]] auto cur_blocks() const {
      return std::ranges::views::iota(0ul, get_basic_blocks().size());
    }

    [[nodiscard]] auto block_succs(IRBlockRef bb) const {
      size_t offset, count;
      size_t stride = 1;

      const auto insts = block_instructions(bb);
      const Instruction* br_instr = &insts.back();
      if (br_instr->kind == InstructionKind::AddRet) {
        br_instr = &insts[insts.size() - 2];
      }

      switch (br_instr->kind) {
        case InstructionKind::Ret:
          offset = count = 0;
          break;
        case InstructionKind::Br:
          offset = 0;
          count = 1;
          break;
        case InstructionKind::CondBr:
          offset = 1;
          count = 2;
          break;
        case InstructionKind::Unreachable:
          offset = count = 0;
          break;
        case InstructionKind::Invoke:
          offset = 1;
          count = 2;
          break;
        case InstructionKind::Switch:
          offset = 1;
          count = br_instr->ops_len - offset;
          stride = 2;
          break;
        case InstructionKind::Resume:
          offset = count = 0;
          break;
        default:
          TPDE_FATAL("Invalid branching instruction");
      }

      return IR::Instr(*br_instr, *this).ops()
             | std::ranges::views::drop(offset)
             | std::ranges::views::take(count)
             | std::ranges::views::stride(stride)
             | std::ranges::views::transform([](IR::Operand op) { return op.content(); });
    }

    [[nodiscard]] auto block_insts(const IRBlockRef bb) {
      return std::ranges::views::iota(0u, static_cast<uint32_t>(get_basic_block(bb).insts_len))
             | std::ranges::views::transform([bb](uint32_t idx) { return IRInstRef{.inst = idx, .block = bb}; });
    }

    [[nodiscard]] static auto block_phis(const IRBlockRef bb) {
      // TODO
      return std::views::empty<IRValueRef>;
    }

    [[nodiscard]] u32 block_info(const IRBlockRef bb) {
      return get_basic_block(bb).info1;
    }

    void block_set_info(IRBlockRef bb, const u32 info) {
      get_basic_block(bb).info1 = info;
    }

    [[nodiscard]] u32 block_info2(const IRBlockRef bb) {
      return get_basic_block(bb).info2;
    }

    void block_set_info2(IRBlockRef bb, const u32 info) {
      get_basic_block(bb).info2 = info;
    }

    [[nodiscard]] std::string block_fmt_ref(IRBlockRef bb) {
      // TODO
      return std::string(get_basic_block(bb).name);
    }

    [[nodiscard]] tpde::ValLocalIdx val_local_idx(IRValueRef ir_value) {
      if (ir_value.is_alloc())
        return static_cast<tpde::ValLocalIdx>(ir_value.content());
      size_t prev = get_allocas().size();

      if (ir_value.is_val())
        return static_cast<tpde::ValLocalIdx>(ir_value.content() + prev);
      prev += get_slots().size();

      if (ir_value.is_global())
        return static_cast<tpde::ValLocalIdx>(ir_value.content() + prev);
      prev += get_globals().size();

      if (ir_value.is_global_ptr())
        return static_cast<tpde::ValLocalIdx>(ir_value.content() + prev);
      prev += get_global_ptrs().size();

      // Function references are materialized as variable refs holding the
      // address of the function symbol.
      if (ir_value.is_func())
        return static_cast<tpde::ValLocalIdx>(ir_value.content() + prev);
      prev += get_functions().size();

      assert(false);
    }

    [[nodiscard]] static bool val_ignore_in_liveness_analysis(const IRValueRef value) {
      // TODO
      return false;
    }

    [[nodiscard]] bool val_is_phi(const IRValueRef value) const {
      // TODO needed when phis nodes are added
      return false;
    }

    [[nodiscard]] auto val_as_phi(const IRValueRef value) const {
      struct PHIRef {
        [[nodiscard]] u32 incoming_count() const {
          return 0;
        }

        [[nodiscard]] IRValueRef incoming_val_for_slot(const u32 slot) const {
          TPDE_FATAL("not implemented");
        }

        [[nodiscard]] IRBlockRef incoming_block_for_slot(const u32 slot) const {
          TPDE_FATAL("not implemented");
        }

        [[nodiscard]] IRValueRef incoming_val_for_block(const IRBlockRef block) const {
          TPDE_FATAL("not implemented");
        }
      };

      return PHIRef{};
    }

    [[nodiscard]] u32 val_alloca_size(IRValueRef val) const {
      if (val.is_alloc()) {
        return get_allocas()[val.content()].size;
      }
      TPDE_FATAL("not a alloc");
    }

    [[nodiscard]] u32 val_alloca_align(IRValueRef val) const {
      if (val.is_alloc()) {
        // TODO: over-aligned allocas. TPDE handles them as dynamic allocas,
        // which need liveness info our allocas don't have. 16 byte suffice for
        // the 128 bit vector parts.
        return std::min<u32>(get_allocas()[val.content()].align, 16);
      }
      TPDE_FATAL("not a alloc");
    }

    [[nodiscard]] std::string value_fmt_ref(const IRValueRef val) const {
      // TODO
      return "value";
    }

    [[nodiscard]] auto inst_operands(IRInstRef inst) const {
      return get_instruction(inst).ops()
             | std::views::filter([](const IR::Operand op) {
               return op.is_val();
             });
    }

    [[nodiscard]] auto inst_results(const IRInstRef instref) const {
      const IR::Instr inst = get_instruction(instref);
      return std::views::single(inst.has_result() ? inst.result() : INVALID_VALUE_REF)
             | std::views::take(inst.has_result() ? 1 : 0);
    }

    bool next_fused = false;
    [[nodiscard]] bool inst_fused(const IRInstRef inst) {
      const IR::Instr cur = get_instruction(inst);
      switch (cur.kind()) {
        case InstructionKind::AddRet:
        case InstructionKind::OverflowCheck:
        case InstructionKind::Unreachable:
          return true;
        default:
      }
      if (next_fused) {
        next_fused = false;
        return true;
      }
      return false;
    }

    ValInfo val_info(const IR::Instr inst) const {
      return ValInfo{inst.has_result() ? get_slots()[inst.result().content()].ty : Type::Void};
    }

    [[nodiscard]] std::string inst_fmt_ref(IRInstRef inst) const {
      return "Instance";
    }

    static void start_compile() {
    }

    static void end_compile() {
    }

    [[nodiscard]] bool switch_func(IRFuncRef func) {
      cur_func = func;
#ifndef TPDE_RUST_NO_CACHE
      cur_data = FuncData{
        .args = view(func->args),
        .slots = view(func->slots),
        .operands = view(func->operands),
        .instructions = view(func->instructions),
        .allocas = view(func->allocas),
        .basic_blocks = view(func->basic_blocks),
        .callee_infos = view(func->callee_infos),
      };
#endif
      return true;
    }

    bool switch_module(ModuleTpde &mod) {
      this->mod = &mod;
#ifndef TPDE_RUST_NO_CACHE
      mod_data = ModuleData{
        .functions = view(mod.functions),
        .consts = view(mod.consts),
        .globals = view(mod.globals),
        .global_ptrs = view(mod.global_ptrs),
        .const_vectors = view(mod.const_vectors),
      };
#endif
      return true;
    }

    void reset() {
      mod = nullptr;
      cur_func = INVALID_FUNC_REF;
#ifndef TPDE_RUST_NO_CACHE
      mod_data = {};
      cur_data = {};
#endif
    }

    // things for compiler

    struct ValueParts {
      Type ty;

      ValueParts() = delete;
      ValueParts(const Type t) : ty(t) {}

      [[nodiscard]] u32 count() const {
        switch (ty) {
          case Type::i128:
            return 2;
          default:
            return is_vector(ty) ? vector_parts(ty).first : 1;
        }
      }

      [[nodiscard]] Type type(u32 n) const {
        switch (ty) {
          case Type::i128:
            return Type::i64;
          default:
            return is_vector(ty) ? vector_parts(ty).second : ty;
        }
      }

      [[nodiscard]] u32 size_bytes(const u32 n) const {
        return size_of_type(type(n)) / 8;
      }

      [[nodiscard]] tpde::RegBank reg_bank(u32 n) const {
        return reg_bank_of_type(type(n));
      }
    };

    [[nodiscard]] ValueParts val_parts(const IRValueRef value) const {
      return ValueParts{type_of_ref(value)};
    }

    static ValueParts val_parts(const ValInfo &info) {
      return ValueParts{info.type};
    }

    static ValueParts val_parts(const Type ty) {
      return ValueParts{ty};
    }
  };

  static_assert(tpde::IRAdaptor<RustAdaptor>);
}

inline IR::Operand IR::Instr::op(size_t i) const {
  assert(i < instruction.ops_len);
  return IR::Operand(adaptor.get_operands()[instruction.ops_start + i]);
}

inline IR::OperandView IR::Instr::ops() const {
  return OperandView(adaptor.get_operands().subspan(instruction.ops_start, instruction.ops_len), ToOperand{});
}
