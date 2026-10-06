#pragma once
#include <cstdint>
#include <tpde/base.hpp>
#include <limits>

#include "../deps/tpde/tpde-llvm/src/base.hpp"
#include "rustc_codegen_tpde/src/shared.rs.h"
#include "tpde/RegisterFile.hpp"

bool compile_to_file(ModuleTpde& module, rust::Str path);

u32 size_of_type(Type type);
tpde::RegBank reg_bank_of_type(Type type);
bool is_integer(Type type);
bool is_float(Type type);
bool is_vector(Type type);
std::pair<u32, Type> vector_info(Type type);
std::pair<u32, Type> vector_parts(Type type);

namespace operands {
  inline constexpr uint32_t MARKER_BLOCK = size_t{7} << (std::numeric_limits<std::uint32_t>::digits - 3);

  inline constexpr uint32_t MARKER_VAL = size_t{0} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_CONST = size_t{1} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_RAW = size_t{2} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_ALLOC = size_t{3} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_FUNC = size_t{4} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_GLOBAL = size_t{5} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_GLOBAL_PTR = size_t{6} << (std::numeric_limits<std::uint32_t>::digits - 3);
  inline constexpr uint32_t MARKER_CONST_VECTOR = size_t{7} << (std::numeric_limits<std::uint32_t>::digits - 3);

  inline bool is(uint32_t op, uint32_t marker) {
    return (op & MARKER_BLOCK) == marker;
  }

  inline bool is_val(uint32_t op) {
    return is(op, MARKER_VAL);
  }

  inline bool is_const(uint32_t op) {
    return is(op, MARKER_CONST);
  }

  inline bool is_raw(uint32_t op) {
    return is(op, MARKER_RAW);
  }

  inline bool is_alloc(uint32_t op) {
    return is(op, MARKER_ALLOC);
  }
  
  inline bool is_func(uint32_t op) {
    return is(op, MARKER_FUNC);
  }

  inline bool is_global(uint32_t op) {
    return is(op, MARKER_GLOBAL);
  }

  inline bool is_global_ptr(uint32_t op) {
    return is(op, MARKER_GLOBAL_PTR);
  }

  inline bool is_const_vector(uint32_t op) {
    return is(op, MARKER_CONST_VECTOR);
  }

  inline uint32_t content(size_t op) {
    return op & ~MARKER_BLOCK;
  }
}
