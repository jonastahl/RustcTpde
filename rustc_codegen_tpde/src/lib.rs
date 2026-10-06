#![feature(rustc_private)]
#![allow(unused_variables, dead_code)]

extern crate rustc_codegen_llvm;
extern crate rustc_codegen_ssa;
extern crate rustc_driver;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_data_structures;
extern crate rustc_target;
extern crate rustc_abi;
extern crate rustc_hir;
extern crate alloc;
extern crate rustc_fs_util;
extern crate core;
extern crate rustc_hashes;
extern crate rustc_symbol_mangling;

use rustc_codegen_ssa::traits::CodegenBackend;
use crate::backend::TpdeCodegenBackend;

mod backend;
mod shared;
mod base;
mod context;
mod builder;
mod asm;
mod debuginfo;
mod abi;
mod consts;
mod common;
mod type_;
mod mono_item;
pub mod back;
mod allocator;

#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(TpdeCodegenBackend::new())
}

#[cfg(test)]
mod tests {
}