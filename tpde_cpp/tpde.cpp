#include "tpde.h"

#include <cstdlib>
#include <fstream>
#include <mutex>

#include "RustCompiler.h"
#include "tpde/RegisterFile.hpp"

namespace {
    // Dont recreate the backend on every compilation request
    // This pool holds as many tpde instances as the maximum repquests at a time
    std::mutex compiler_pool_mutex;
    std::vector<std::unique_ptr<tpde_rust::RustCompiler>> compiler_pool;

    std::unique_ptr<tpde_rust::RustCompiler> acquire_compiler() {
        std::unique_ptr<tpde_rust::RustCompiler> compiler;
        {
            std::lock_guard lock(compiler_pool_mutex);
            if (!compiler_pool.empty()) {
                compiler = std::move(compiler_pool.back());
                compiler_pool.pop_back();
            }
        }
        if (!compiler) {
            return tpde_rust::RustCompiler::create();
        }

        compiler->reset();
        return compiler;
    }

    void release_compiler(std::unique_ptr<tpde_rust::RustCompiler> compiler) {
        std::lock_guard lock(compiler_pool_mutex);
        compiler_pool.push_back(std::move(compiler));
    }

    unsigned repeat_obj_emit() {
        static const unsigned count = [] {
            const char *env = std::getenv("REPEAT_OBJ_EMIT");
            const unsigned long n = env ? std::strtoul(env, nullptr, 10) : 1;
            return n == 0 ? 1u : static_cast<unsigned>(n);
        }();
        return count;
    }
}

bool compile_to_file(ModuleTpde& module, const rust::Str path) {
    std::vector<uint8_t> buf;
    for (unsigned i = 0, n = repeat_obj_emit(); i < n; ++i) {
        auto compiler = acquire_compiler();
        const bool ok = compiler->compile_to_elf(module, buf);
        release_compiler(std::move(compiler));
        if (!ok) {
            return false;
        }
    }

    {
        std::string file_path(path.data(), path.size());
        std::ofstream out_file(file_path, std::ios::binary);

        if (!out_file) {
            return 0;
        }

        out_file.write(reinterpret_cast<const char*>(buf.data()), buf.size());
        out_file.close();
    }

    return true;
}
