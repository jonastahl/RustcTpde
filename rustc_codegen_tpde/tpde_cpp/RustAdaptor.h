#pragma once

#include "tpde.h"

#include <../deps/tpde/tpde-llvm/src/base.hpp>
#include <tpde/IRAdaptor.hpp>
#include <tpde/RegisterFile.hpp>
#include <generator>
#include <span>
#include <unordered_set>

#include "tpde/util/SmallVector.hpp"

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
    using IRValueRef = uint32_t;
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

    static constexpr IRValueRef INVALID_VALUE_REF = -1;
    static constexpr IRBlockRef INVALID_BLOCK_REF = ~0u;
    static constexpr IRFuncRef INVALID_FUNC_REF = nullptr;

    static constexpr bool TPDE_PROVIDES_HIGHEST_VAL_IDX = true;
    static constexpr bool TPDE_LIVENESS_VISIT_ARGS = true;

    ModuleTpde *mod = nullptr;
    Function *cur_func = nullptr;

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

    template<typename T>
    [[nodiscard]] static std::span<T> view(rust::Vec<T> &vec) {
      return {vec.data(), vec.size()};
    }

    // wrapper for handling inlined constant values
    [[nodiscard]] Value const_value(const IRValueRef value) const {
      const u32 c = operands::content(value);
      if (c < operands::INLINE_CONST_END) {
        return Value{
          .ty = static_cast<Type>(c >> operands::INLINE_CONST_BITS),
          .data1 = 0,
          .data2 = c & operands::INLINE_CONST_MASK,
        };
      }
      return mod_data.consts[c - operands::INLINE_CONST_END];
    }

    [[nodiscard]] Type type_of_ref(const IRValueRef value) const {
      if (operands::is_val(value))
        return cur_data.slots[operands::content(value)].ty;
      if (operands::is_const(value))
        return const_value(value).ty;
      if (operands::is_const_vector(value))
        return mod_data.const_vectors[operands::content(value)].ty;
      if (operands::is_alloc(value) || operands::is_global(value) || operands::is_global_ptr(value)
          || operands::is_func(value))
        return Type::ptr;
      assert(false && "invalid value ref");
    }

    /// Element type for vectors, the type itself otherwise.
    [[nodiscard]] Type elem_type_of_ref(const IRValueRef value) const {
      const Type ty = type_of_ref(value);
      return is_vector(ty) ? vector_info(ty).second : ty;
    }

    [[nodiscard]] BasicBlock &get_basic_block(const IRBlockRef block) const {
      return cur_data.basic_blocks[block];
    }

    [[nodiscard]] Instruction &get_instruction(const IRInstRef inst) const {
      return cur_data.instructions[get_basic_block(inst.block).insts_start + inst.inst];
    }

    [[nodiscard]] std::span<Instruction> block_instructions(const IRBlockRef block) const {
      const auto &bb = get_basic_block(block);
      return cur_data.instructions.subspan(bb.insts_start, bb.insts_len);
    }

    [[nodiscard]] std::span<const u32> ops(const Instruction &inst) const {
      return cur_data.operands.subspan(inst.ops_start, inst.ops_len);
    }

    struct ValInfo {
      Type type;
    };

    [[nodiscard]] u32 func_count() const { return mod_data.functions.size(); }

    [[nodiscard]] auto funcs() const {
      return mod_data.functions | std::views::transform([](Function &fn) { return &fn; });
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
      return cur_data.allocas.size() + cur_data.slots.size() + mod_data.globals.size() + mod_data.global_ptrs.size()
             + mod_data.functions.size();
    }

    [[nodiscard]] auto cur_args() const {
      return std::views::iota(0u, static_cast<u32>(cur_data.args.size()));
    }

    [[nodiscard]] bool cur_arg_is_byval(const u32 idx) const {
      return cur_data.args[idx].kind == ArgKind::ByVal;
    }

    [[nodiscard]] u32 cur_arg_byval_size(const u32 idx) const {
      return cur_data.args[idx].size;
    }

    [[nodiscard]] u32 cur_arg_byval_align(const u32 idx) const {
      return cur_data.args[idx].align;
    }

    [[nodiscard]] u32 cur_arg_is_sret(const u32 idx) const {
      return cur_data.args[idx].kind == ArgKind::sRet;
    }

    [[nodiscard]] auto cur_static_allocas() const {
      return std::ranges::views::iota(static_cast<IRValueRef>(0), static_cast<IRValueRef>(cur_data.allocas.size()))
        | std::views::transform([](const IRValueRef idx) { return operands::MARKER_ALLOC | idx; });
    }

    [[nodiscard]] static auto cur_has_dynamic_alloca() {
      // TODO
      return false;
    }

    [[nodiscard]] IRBlockRef cur_entry_block() const {
      return 0;
    }

    [[nodiscard]] auto cur_blocks() const {
      return std::ranges::views::iota(0ul, cur_data.basic_blocks.size());
    }

    [[nodiscard]] auto block_succs(IRBlockRef bb) const {
      size_t offset, count;
      size_t stride = 1;

      const auto insts = block_instructions(bb);
      auto* br_instr = &insts.back();
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


      return ops(*br_instr)
             | std::ranges::views::drop(offset)
             | std::ranges::views::take(count)
             | std::ranges::views::stride(stride)
             | std::ranges::views::transform([](uint32_t op) { return operands::content(op); });
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
      if (operands::is_alloc(ir_value))
        return static_cast<tpde::ValLocalIdx>(operands::content(ir_value));
      size_t prev = cur_data.allocas.size();

      if (operands::is_val(ir_value))
        return static_cast<tpde::ValLocalIdx>(operands::content(ir_value) + prev);
      prev += cur_data.slots.size();

      if (operands::is_global(ir_value))
        return static_cast<tpde::ValLocalIdx>(operands::content(ir_value) + prev);
      prev += mod_data.globals.size();

      if (operands::is_global_ptr(ir_value))
        return static_cast<tpde::ValLocalIdx>(operands::content(ir_value) + prev);
      prev += mod_data.global_ptrs.size();

      // Function references are materialized as variable refs holding the
      // address of the function symbol.
      if (operands::is_func(ir_value))
        return static_cast<tpde::ValLocalIdx>(operands::content(ir_value) + prev);
      prev += mod_data.functions.size();

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
      if (operands::is_alloc(val)) {
        return cur_data.allocas[operands::content(val)].size;
      }
      TPDE_FATAL("not a alloc");
    }

    [[nodiscard]] u32 val_alloca_align(IRValueRef val) const {
      if (operands::is_alloc(val)) {
        // TODO: over-aligned allocas. TPDE handles them as dynamic allocas,
        // which need liveness info our allocas don't have. 16 byte suffice for
        // the 128 bit vector parts.
        return std::min<u32>(cur_data.allocas[operands::content(val)].align, 16);
      }
      TPDE_FATAL("not a alloc");
    }

    [[nodiscard]] std::string value_fmt_ref(const IRValueRef val) const {
      // TODO
      return "value";
    }

    [[nodiscard]] auto inst_operands(IRInstRef inst) const {
      return ops(get_instruction(inst))
             | std::views::filter([](auto op) {
               return operands::is_val(op);
             });
    }

    [[nodiscard]] auto inst_results(const IRInstRef instref) const {
      const auto &inst = get_instruction(instref);
      return std::views::single(inst.has_result ? inst.result : INVALID_VALUE_REF)
             | std::views::take(inst.has_result ? 1 : 0);
    }

    bool next_fused = false;
    [[nodiscard]] bool inst_fused(const IRInstRef inst) {
      const auto& cur = get_instruction(inst);
      switch (cur.kind) {
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

    ValInfo val_info(const Instruction *inst) const {
      return ValInfo{inst->has_result ? cur_data.slots[inst->result].ty : Type::Void};
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
      cur_data = FuncData{
        .args = view(func->args),
        .slots = view(func->slots),
        .operands = view(func->operands),
        .instructions = view(func->instructions),
        .allocas = view(func->allocas),
        .basic_blocks = view(func->basic_blocks),
        .callee_infos = view(func->callee_infos),
      };
      return true;
    }

    bool switch_module(ModuleTpde &mod) {
      this->mod = &mod;
      mod_data = ModuleData{
        .functions = view(mod.functions),
        .consts = view(mod.consts),
        .globals = view(mod.globals),
        .global_ptrs = view(mod.global_ptrs),
        .const_vectors = view(mod.const_vectors),
      };
      return true;
    }

    void reset() {
      mod = nullptr;
      mod_data = {};
      cur_func = INVALID_FUNC_REF;
      cur_data = {};
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
