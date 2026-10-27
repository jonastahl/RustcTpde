
fn main() {
    let no_cache = std::env::var_os("CARGO_FEATURE_NO_CACHE").is_some();

    let mut cmake_cfg = cmake::Config::new(".");
    cmake_cfg
        .define("CMAKE_C_COMPILER", "clang")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .define("TPDE_ENABLE_LLVM_PLUGIN", "OFF")
        .define("TPDE_INCLUDE_TESTS", "OFF")
        .define("TPDE_LOGGING", "ON")
        .define("TPDE_RUST_NO_CACHE", if no_cache { "ON" } else { "OFF" });
    // default selection would be -O2 so we need to force him to -O3
    if matches!(std::env::var("OPT_LEVEL").as_deref(), Ok("1" | "2" | "3")) {
        cmake_cfg.profile("Release");
        if std::env::var("DEBUG").as_deref() == Ok("true") {
            cmake_cfg.cflag("-gline-tables-only").cxxflag("-gline-tables-only");
        }
    }
    let dst = cmake_cfg.build();

    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());
    println!("cargo:rustc-link-search=native={}", dst.join("lib64").display());
    let spdlog_dir = dst.join("build/tpde_build/tpde/deps/spdlog");
    println!("cargo:rustc-link-search=native={}", spdlog_dir.display());
    
    let spdlog_lib = if spdlog_dir.join("libspdlogd.a").exists() { "spdlogd" } else { "spdlog" };
    println!("cargo:rustc-link-lib=static=tpde_cpp");
    println!("cargo:rustc-link-lib=static=tpde");
    println!("cargo:rustc-link-lib=static={spdlog_lib}");
    println!("cargo:rustc-link-lib=static=fadec");
    println!("cargo:rustc-link-lib=static=disarm64");

    println!("cargo:rustc-link-arg=-Wl,-z,defs");

    let mut bridge = cxx_build::bridge("src/shared.rs");
    if no_cache {
        bridge.define("TPDE_RUST_NO_CACHE", None);
    }
    bridge
        .include(".")
        .include("../deps/tpde/tpde/include")
        .flag_if_supported("-std=c++23")
        .flag_if_supported("-fno-exceptions")
        // silence warning in generated cxx code
        .flag_if_supported("-Wno-maybe-uninitialized")
        .compile("rustc_codegen_tpde");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=tpde_cpp");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
}