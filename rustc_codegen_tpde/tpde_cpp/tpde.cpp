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

std::pair<u32, Type> vector_info(Type type) {
    switch (type) {
        // Vectors narrower than 64 bit
        case Type::v2i8: return {2, Type::i8};
        case Type::v4i8: return {4, Type::i8};
        case Type::v2i16: return {2, Type::i16};

        // 64 bit vectors
        case Type::v8i8: return {8, Type::i8};
        case Type::v4i16: return {4, Type::i16};
        case Type::v2i32: return {2, Type::i32};
        case Type::v2f32: return {2, Type::f32};

        // 128 bit vectors
        case Type::v16i8: return {16, Type::i8};
        case Type::v8i16: return {8, Type::i16};
        case Type::v4i32: return {4, Type::i32};
        case Type::v2i64: return {2, Type::i64};
        case Type::v4f32: return {4, Type::f32};
        case Type::v2f64: return {2, Type::f64};

        // 256 bit vectors
        case Type::v32i8: return {32, Type::i8};
        case Type::v16i16: return {16, Type::i16};
        case Type::v8i32: return {8, Type::i32};
        case Type::v4i64: return {4, Type::i64};
        case Type::v8f32: return {8, Type::f32};
        case Type::v4f64: return {4, Type::f64};

        // 512 bit vectors
        case Type::v64i8: return {64, Type::i8};
        case Type::v32i16: return {32, Type::i16};
        case Type::v16i32: return {16, Type::i32};
        case Type::v8i64: return {8, Type::i64};
        case Type::v16f32: return {16, Type::f32};
        case Type::v8f64: return {8, Type::f64};

        // 1024 bit vectors
        case Type::v128i8: return {128, Type::i8};
        case Type::v64i16: return {64, Type::i16};
        case Type::v32i32: return {32, Type::i32};
        case Type::v16i64: return {16, Type::i64};
        case Type::v32f32: return {32, Type::f32};
        case Type::v16f64: return {16, Type::f64};

        // 2048 bit vectors
        case Type::v64i32: return {64, Type::i32};
        case Type::v32i64: return {32, Type::i64};

        // 4096 bit vectors
        case Type::v64i64: return {64, Type::i64};
        default:
            throw std::runtime_error("vector_info: not a vector type");
    }
}

bool is_vector(Type type) {
    switch (type) {
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
        case Type::v2i8:
        case Type::v4i8:
        case Type::v2i16:
        case Type::v32i8:
        case Type::v16i16:
        case Type::v8i32:
        case Type::v4i64:
        case Type::v8f32:
        case Type::v4f64:
        case Type::v64i8:
        case Type::v32i16:
        case Type::v16i32:
        case Type::v8i64:
        case Type::v16f32:
        case Type::v8f64:
        case Type::v128i8:
        case Type::v64i16:
        case Type::v32i32:
        case Type::v16i64:
        case Type::v32f32:
        case Type::v16f64:
        case Type::v64i32:
        case Type::v32i64:
        case Type::v64i64:
            return true;
        default:
            return false;
    }
}

std::pair<u32, Type> vector_parts(Type type) {
    const auto [nelem, elem] = vector_info(type);
    const u32 bits = nelem * size_of_type(elem);
    if (bits < 64) {
        return {nelem, elem};
    }
    if (bits <= 128) {
        return {1, type};
    }
    Type part;
    switch (elem) {
        case Type::i8: part = Type::v16i8; break;
        case Type::i16: part = Type::v8i16; break;
        case Type::i32: part = Type::v4i32; break;
        case Type::i64: part = Type::v2i64; break;
        case Type::f32: part = Type::v4f32; break;
        case Type::f64: part = Type::v2f64; break;
        default: throw std::runtime_error("vector_parts: unsupported element type");
    }
    return {bits / 128, part};
}

[[nodiscard]] u32 size_of_type(Type type) {
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
        default:
            if (is_vector(type)) {
                const auto [nelem, elem] = vector_info(type);
                return nelem * size_of_type(elem);
            }
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
