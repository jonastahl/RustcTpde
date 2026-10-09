// C interface around tpde-llvm: bitcode in, ELF object out.
#include <tpde-llvm/LLVMCompiler.hpp>

#include <llvm/Bitcode/BitcodeReader.h>
#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>
#include <llvm/Support/MemoryBuffer.h>
#include <llvm/TargetParser/Triple.h>

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

extern "C" {

/// Compiles the bitcode to an ELF object. Returns a malloc'ed buffer (release
/// with tpde_llvm_free_buffer) or nullptr on failure.
uint8_t *tpde_llvm_compile_bitcode(const uint8_t *data, size_t len, size_t *out_len) {
  llvm::LLVMContext ctx;
  llvm::MemoryBufferRef ref(
      llvm::StringRef(reinterpret_cast<const char *>(data), len), "module");
  auto mod = llvm::parseBitcodeFile(ref, ctx);
  if (!mod) {
    std::fprintf(stderr, "tpde-llvm: failed to parse bitcode: %s\n",
                 llvm::toString(mod.takeError()).c_str());
    return nullptr;
  }

  llvm::Triple triple((*mod)->getTargetTriple());
  auto compiler = tpde_llvm::LLVMCompiler::create(triple);
  if (!compiler) {
    std::fprintf(stderr, "tpde-llvm: unsupported target triple %s\n",
                 triple.str().c_str());
    return nullptr;
  }

  std::vector<uint8_t> buf;
  if (!compiler->compile_to_elf(**mod, buf)) {
    std::fprintf(stderr, "tpde-llvm: compilation failed\n");
    return nullptr;
  }

  auto *out = static_cast<uint8_t *>(std::malloc(buf.size()));
  std::memcpy(out, buf.data(), buf.size());
  *out_len = buf.size();
  return out;
}

void tpde_llvm_free_buffer(uint8_t *buf) { std::free(buf); }
}
