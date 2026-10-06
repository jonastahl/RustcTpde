cmake_minimum_required(VERSION 3.14)
project(tpde_wrapper_project)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

#set(TPDE_LOGGING OFF)

add_subdirectory(../deps/tpde tpde_build)

set(TPDE_TARGETS "x86_64")
#set(TPDE_TARGETS "x86_64;aarch64")

find_program(CXXBRIDGE cxxbridge REQUIRED HINTS $ENV{HOME}/.cargo/bin)

set(CXX_GEN_DIR ${CMAKE_CURRENT_BINARY_DIR}/cxxbridge)
file(MAKE_DIRECTORY ${CXX_GEN_DIR}/rust ${CXX_GEN_DIR}/rustc_codegen_tpde/src)

execute_process(COMMAND ${CXXBRIDGE} --header -o ${CXX_GEN_DIR}/rust/cxx.h)
execute_process(COMMAND ${CXXBRIDGE} ${CMAKE_CURRENT_SOURCE_DIR}/src/shared.rs --header
        -o ${CXX_GEN_DIR}/rustc_codegen_tpde/src/shared.rs.h)

find_program(TPDE_CLANG REQUIRED NAMES clang-21 clang-20 clang-19 clang)
message(STATUS "Found Clang for encode template: ${TPDE_CLANG}")

if ("x86_64" IN_LIST TPDE_TARGETS)
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/encode_template_x64.bc
        COMMAND ${TPDE_CLANG} -c -emit-llvm -ffreestanding -fcf-protection=none
            -O3 -fomit-frame-pointer -fno-math-errno
            --target=x86_64-linux -march=x86-64-v4
            -o ${CMAKE_CURRENT_BINARY_DIR}/encode_template_x64.bc
            ${CMAKE_CURRENT_SOURCE_DIR}/tpde_cpp/encode_template.c
        DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/tpde_cpp/encode_template.c
    )
    add_custom_command(
        OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/encode_template_x64.hpp
        COMMAND tpde_encodegen
            -o ${CMAKE_CURRENT_BINARY_DIR}/encode_template_x64.hpp
            ${CMAKE_CURRENT_BINARY_DIR}/encode_template_x64.bc
        DEPENDS
            tpde_encodegen
            ${CMAKE_CURRENT_BINARY_DIR}/encode_template_x64.bc
    )
endif()

add_library(tpde_cpp STATIC
        tpde_cpp/tpde.cpp
        tpde_cpp/RustAdaptor.cpp
        tpde_cpp/RustCompilerBase.cpp
        tpde_cpp/RustCompiler.cpp
        tpde_cpp/x64/RustCompilerX64.cpp
        ${CMAKE_CURRENT_BINARY_DIR}/encode_template_x64.hpp
)

target_include_directories(tpde_cpp PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${CXX_GEN_DIR}
        ${CMAKE_CURRENT_BINARY_DIR}
        ${CMAKE_CURRENT_SOURCE_DIR}/../deps/tpde/tpde/include
)

target_link_libraries(tpde_cpp PRIVATE tpde)

install(TARGETS tpde_cpp ARCHIVE DESTINATION lib)
