#![feature(rustc_private)]
#![allow(unused_variables, dead_code)]

extern crate alloc;
extern crate core;
extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_attr_ir;
extern crate rustc_codegen_llvm;
extern crate rustc_codegen_ssa;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fs_util;
extern crate rustc_hashes;
extern crate rustc_hir;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_symbol_mangling;
extern crate rustc_target;

use crate::backend::TpdeCodegenBackend;
use rustc_codegen_ssa::traits::CodegenBackend;

mod abi;
mod allocator;
mod asm;
pub mod back;
mod backend;
mod base;
mod builder;
mod common;
mod consts;
mod context;
mod debuginfo;
mod mono_item;
mod shared;
mod type_;

#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(TpdeCodegenBackend::new())
}
