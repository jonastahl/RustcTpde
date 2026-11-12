
#include "RustCompiler.h"

#include "x64/RustCompilerX64.h"

namespace tpde_rust {
  RustCompiler::~RustCompiler() = default;

  std::unique_ptr<RustCompiler> RustCompiler::create() {
    // TODO only support x64 for now
    return x64::create_compiler();
  }
}
