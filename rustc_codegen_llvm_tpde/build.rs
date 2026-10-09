
fn main() {
    let dst = cmake::Config::new(".")
        .define("CMAKE_C_COMPILER", "clang")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .define("TPDE_ENABLE_LLVM_PLUGIN", "OFF")
        .define("TPDE_INCLUDE_TESTS", "OFF")
        .define("TPDE_LOGGING", "OFF")
        .build();

    // build_target() only builds, it does not install, so the archives live in the build tree
    let build = dst.join("build");
    for dir in ["", "tpde_build/tpde", "tpde_build/tpde/deps/fadec", "tpde_build/tpde/deps/disarm"] {
        println!("cargo:rustc-link-search=native={}", build.join(dir).display());
    }

    println!("cargo:rustc-link-lib=static=tpde_llvm_wrapper");
    println!("cargo:rustc-link-lib=static=tpde");
    println!("cargo:rustc-link-lib=static=fadec");
    println!("cargo:rustc-link-lib=static=disarm64");

    println!("cargo:rustc-link-arg=-Wl,-z,defs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
}
