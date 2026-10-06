
fn main() {

    let dst = cmake::Config::new(".")
        .define("CMAKE_C_COMPILER", "clang")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .define("TPDE_ENABLE_LLVM_PLUGIN", "OFF")
        .define("TPDE_INCLUDE_TESTS", "OFF")
        .define("TPDE_LOGGING", "OFF")
        .build();

    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-search=native={}", dst.join("lib64").display());

    println!("cargo:rustc-link-lib=static=tpde_cpp");
    println!("cargo:rustc-link-lib=static=tpde");
    println!("cargo:rustc-link-lib=static=fadec");
    println!("cargo:rustc-link-lib=static=disarm64");

    println!("cargo:rustc-link-arg=-Wl,-z,defs");

    cxx_build::bridge("src/shared.rs")
        .include(".")
        .include("../deps/tpde/tpde/include")
        .flag_if_supported("-std=c++23")
        // silence warning in generated cxx code
        .flag_if_supported("-Wno-maybe-uninitialized")
        .compile("rustc_codegen_tpde");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=tpde_cpp");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
}