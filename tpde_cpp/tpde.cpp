#include "tpde.h"

#include <fstream>

#include "RustCompiler.h"
#include "tpde/RegisterFile.hpp"


bool compile_to_file(ModuleTpde& module, const rust::Str path) {
    // TODO move this out, don't want to initialize it every time separately
    const auto compiler = tpde_rust::RustCompiler::create();

    std::vector<uint8_t> buf;
    if (!compiler->compile_to_elf(module, buf)) {
        return false;
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

[[nodiscard]] tpde::u32 size_of_type(Type type) {
    switch (type) {
        case Type::Bool:
        case Type::i8: return 8;
        case Type::i16: return 16;
        case Type::i32: return 32;
        case Type::i64: return 64;
        case Type::i128: return 128;
        case Type::ptr: return 64;
        case Type::f32: return 32;
        case Type::f64: return 64;
        case Type::v8i8:
        case Type::v4i16:
        case Type::v2i32:
        case Type::v2f32: return 64;
        case Type::v16i8:
        case Type::v8i16:
        case Type::v4i32:
        case Type::v2i64:
        case Type::v4f32:
        case Type::v2f64: return 128;
        default:
            throw std::runtime_error("size_of_type: unsupported type");
    }
}

[[nodiscard]] tpde::RegBank reg_bank_of_type(Type type) {
    switch (type) {
        case Type::Bool:
        case Type::i8:
        case Type::i16:
        case Type::i32:
        case Type::i64:
        case Type::ptr:
            return tpde::RegBank{0};
        case Type::f32:
        case Type::f64:
        case Type::v8i8:
        case Type::v16i8:
        case Type::v4i16:
        case Type::v8i16:
        case Type::v2i32:
        case Type::v4i32:
        case Type::v2i64:
        case Type::v2f32:
        case Type::v4f32:
        case Type::v2f64:
            return tpde::RegBank{1};
        default:
            throw std::runtime_error("reg_bank_of_type: unsupported type");
    }
}

bool is_integer(Type type) {
    switch (type) {
        case Type::i8:
        case Type::i16:
        case Type::i32:
        case Type::i64:
        case Type::i128:
            return true;
        default:
            return false;
    }
}

bool is_float(Type type) {
    switch (type) {
        case Type::f32:
        case Type::f64:
            return true;
        default:
            return false;
    }
}
