#![feature(rustc_private)]
#![allow(unused_variables, dead_code)]

mod backend;
mod tpde;

extern crate rustc_driver;
extern crate rustc_codegen_ssa;
extern crate rustc_session;
extern crate rustc_middle;
extern crate rustc_data_structures;
extern crate rustc_codegen_llvm;
extern crate rustc_errors;
extern crate rustc_ast;
extern crate rustc_span;

use rustc_codegen_ssa::traits::CodegenBackend;
use crate::backend::LLvmTpdeCodegenBackend;

#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(LLvmTpdeCodegenBackend::new())
}

