cmake_minimum_required(VERSION 3.25)
project(tpde_llvm_wrapper_project CXX C)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(TPDE_TARGETS "x86_64")
#set(TPDE_TARGETS "x86_64;aarch64")

add_subdirectory(../deps/tpde tpde_build)


find_program(TPDE_CLANG REQUIRED NAMES clang-21 clang-20 clang-19 clang)
message(STATUS "Found Clang for encode template: ${TPDE_CLANG}")

add_library(tpde_llvm_wrapper STATIC
        cpp/wrapper.cpp $<TARGET_OBJECTS:tpde_llvm_impl>
)
target_link_libraries(tpde_llvm_wrapper PRIVATE tpde_llvm)
target_link_libraries(tpde-llc PRIVATE tpde)


install(TARGETS tpde_llvm_wrapper ARCHIVE DESTINATION lib)
