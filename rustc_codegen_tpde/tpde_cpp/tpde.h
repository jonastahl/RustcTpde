#pragma once
#include <cstdint>
#include <tpde/base.hpp>
#include <limits>
#include <span>

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

