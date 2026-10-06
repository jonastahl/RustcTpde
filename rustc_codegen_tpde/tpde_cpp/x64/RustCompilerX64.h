#pragma once
#include <tpde/x64/CompilerX64.hpp>

#include "tpde_cpp/RustAdaptor.h"
#include "tpde_cpp/RustCompilerBase.h"

namespace tpde_rust::x64 {
  std::unique_ptr<RustCompiler> create_compiler();
}
