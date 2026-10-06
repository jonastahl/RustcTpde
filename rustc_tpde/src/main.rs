#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_interface;

use std::process::ExitCode;

use rustc_driver::{Callbacks, catch_with_exit_code, run_compiler};
use rustc_interface::Config;
use rustc_codegen_tpde::__rustc_codegen_backend;

use tikv_jemalloc_sys as _;

struct BackendCallbacks;

impl Callbacks for BackendCallbacks {
    fn config(&mut self, config: &mut Config) {
        config.make_codegen_backend = Some(Box::new(|_options| {
            __rustc_codegen_backend()
        }));
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let mut callbacks = BackendCallbacks;

    catch_with_exit_code(|| run_compiler(&args, &mut callbacks))
}