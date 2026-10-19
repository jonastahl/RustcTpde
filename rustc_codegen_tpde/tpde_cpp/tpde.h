#pragma once
#include <array>
#include <cassert>
#include <cstdint>
#include <tpde/base.hpp>
#include <limits>
#include <span>

#include "../deps/tpde/tpde-llvm/src/base.hpp"
#include "rustc_codegen_tpde/src/shared.rs.h"
#include "tpde/RegisterFile.hpp"

bool compile_to_file(ModuleTpde& module, rust::Str path);

constexpr std::pair<u32, Type> vector_layout(Type type) {
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
        default: return {0, Type::Void};
    }
}

constexpr u32 scalar_bits(Type type) {
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
        default: return 0;
    }
}

inline constexpr tpde::RegBank NO_REG_BANK{0xff};
constexpr u8 scalar_bank(Type type) {
    switch (type) {
        case Type::Bool:
        case Type::i8:
        case Type::i16:
        case Type::i32:
        case Type::i64:
        case Type::ptr:
            return 0;
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
            return 1;
        default:
            return 0xff;
    }
}

constexpr Type vector_part_128(Type elem) {
    switch (elem) {
        case Type::i8: return Type::v16i8;
        case Type::i16: return Type::v8i16;
        case Type::i32: return Type::v4i32;
        case Type::i64: return Type::v2i64;
        case Type::f32: return Type::v4f32;
        case Type::f64: return Type::v2f64;
        default: return Type::Void;
    }
}

struct TypeInfo {
    u16 bits = 0;              // 0: type has no size
    tpde::RegBank reg_bank = NO_REG_BANK;

    u8 nelem = 0;              // 0: not a vector
    Type elem = Type::Void;    // repeated element
    Type part = Type::Void;    // 128 bit part for splitting
    u8 num_parts = 0;          // number of parts for splitting
};

inline constexpr auto type_infos = []() constexpr {
    std::array<TypeInfo, static_cast<size_t>(Type::Last)> res{};
    for (size_t i = 0; i < res.size(); ++i) {
        const Type type = static_cast<Type>(i);
        TypeInfo &info = res[i];

        const auto [nelem, elem] = vector_layout(type);
        if (nelem == 0) {
            info.bits = scalar_bits(type);
            info.reg_bank = tpde::RegBank{scalar_bank(type)};
            continue;
        }

        const u32 bits = nelem * scalar_bits(elem);
        info.bits = bits;
        info.nelem = nelem;
        info.elem = elem;
        if (bits < 64) {
            info.num_parts = nelem;
            info.part = elem;
        } else if (bits <= 128) {
            info.num_parts = 1;
            info.part = type;
            info.reg_bank = tpde::RegBank{1};
        } else {
            info.num_parts = bits / 128;
            info.part = vector_part_128(elem);
        }
    }
    return res;
}();

[[nodiscard]] inline const TypeInfo &type_info(Type type) {
    assert(static_cast<size_t>(type) < type_infos.size());
    return type_infos[static_cast<size_t>(type)];
}

inline std::pair<u32, Type> vector_info(Type type) {
    const auto &info = type_info(type);
    if (info.nelem == 0) [[unlikely]] {
        TPDE_FATAL("vector_info: not a vector type");
    }
    return {info.nelem, info.elem};
}

inline bool is_vector(Type type) {
    return type_info(type).nelem != 0;
}

[[nodiscard]] inline u32 size_of_type(Type type) {
    const u32 bits = type_info(type).bits;
    if (bits == 0) [[unlikely]] {
        TPDE_FATAL("size_of_type: unsupported type");
    }
    return bits;
}

inline std::pair<u32, Type> vector_parts(Type type) {
    const auto &info = type_info(type);
    if (info.num_parts == 0) [[unlikely]] {
        TPDE_FATAL("vector_info: not a vector type");
    }
    return {info.num_parts, info.part};
}

[[nodiscard]] inline tpde::RegBank reg_bank_of_type(Type type) {
    const tpde::RegBank bank = type_info(type).reg_bank;
    if (bank == NO_REG_BANK) [[unlikely]] {
        TPDE_FATAL("reg_bank_of_type: unsupported type");
    }
    return bank;
}

inline bool is_integer(Type type) {
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

inline bool is_float(Type type) {
    switch (type) {
        case Type::f32:
        case Type::f64:
            return true;
        default:
            return false;
    }
}
