#pragma once
#include <cstdint>
#include <memory>
#include <vector>

#include "tpde.h"

namespace tpde_rust {
  class RustCompiler {
  protected:
    RustCompiler() = default;

  public:
    virtual ~RustCompiler();

    RustCompiler(const RustCompiler &) = delete;
    RustCompiler &operator=(const RustCompiler &) = delete;

    static std::unique_ptr<RustCompiler> create();

    virtual bool compile_to_elf(ModuleTpde &mod, std::vector<uint8_t> &buf) = 0;
  };
}
