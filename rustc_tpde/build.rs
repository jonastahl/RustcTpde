
use std::process::Command;

fn main() {
    let output = Command::new("rustc")
        .arg("+nightly-2026-08-19") // Ensure we query the nightly compiler
        .args(["--print", "sysroot"])
        .output()
        .expect("Failed to get rustc sysroot");

    let sysroot = String::from_utf8(output.stdout).unwrap();
    let sysroot = sysroot.trim();

    println!("cargo:rustc-link-arg=-Wl,-rpath,{}/lib", sysroot);
}