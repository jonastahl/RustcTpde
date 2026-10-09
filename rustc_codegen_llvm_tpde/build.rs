use std::path::{Path, PathBuf};
use std::process::Command;

/// rustc commit whose LLVM the `deps/rust-llvm-project` submodule is pinned to.
const LLVM_HEADERS_FOR_RUSTC: &str = "e71c0f1e3395b10a8c331317be1a5c107bdf7b2e";

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// The `lib` directory of the toolchain's sysroot, which contains rustc's own libLLVM.
fn sysroot_lib() -> PathBuf {
    let out = Command::new(rustc()).args(["--print", "sysroot"]).output().expect("cannot run rustc");
    PathBuf::from(String::from_utf8(out.stdout).unwrap().trim()).join("lib")
}

/// rustc's libLLVM, e.g. `libLLVM.so.23.1-rust-1.100.0-nightly`.
fn rustc_libllvm(lib_dir: &Path) -> PathBuf {
    std::fs::read_dir(lib_dir)
        .expect("cannot read sysroot lib dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            name.starts_with("libLLVM.so.") && name.contains("rust")
        })
        .expect("rustc's libLLVM.so.* not found in the sysroot")
}

/// Fetches the LLVM sources (only needed for the headers) if the submodule is not checked out yet.
fn ensure_llvm_sources(repo_root: &Path) -> PathBuf {
    let src = repo_root.join("deps/rust-llvm-project");
    if !src.join("llvm/CMakeLists.txt").exists() {
        let status = Command::new("git")
            .current_dir(repo_root)
            .args(["submodule", "update", "--init", "--depth", "1", "deps/rust-llvm-project"])
            .status()
            .expect("cannot run git");
        assert!(status.success(), "failed to fetch the deps/rust-llvm-project submodule");
    }
    src
}

/// Warns if the headers do not belong to the rustc that is compiling us, they must match exactly.
fn check_rustc_commit() {
    let out = Command::new(rustc()).arg("-vV").output().expect("cannot run rustc");
    let out = String::from_utf8(out.stdout).unwrap();
    let commit = out.lines().find_map(|l| l.strip_prefix("commit-hash: ")).unwrap_or("");
    if commit != LLVM_HEADERS_FOR_RUSTC {
        println!(
            "cargo:warning=deps/rust-llvm-project is pinned for rustc {LLVM_HEADERS_FOR_RUSTC}, but rustc is \
            {commit}; update the submodule to the LLVM commit of this rustc and LLVM_HEADERS_FOR_RUSTC in build.rs"
        );
    }
}

/// Configures LLVM and generates the headers (`*.inc`, `llvm/Config/*.h`) that TPDE includes. LLVM
/// itself is not compiled. Returns the directory with `LLVMConfig.cmake`.
fn generate_llvm_headers(llvm_src: &Path, out_dir: &Path) -> PathBuf {
    let dst = cmake::Config::new(llvm_src.join("llvm"))
        .out_dir(out_dir.join("llvm-headers"))
        .profile("Release")
        .define("LLVM_TARGETS_TO_BUILD", "X86")
        // must match rustc's LLVM (assertions off, no ABI breaking checks)
        .define("LLVM_ENABLE_ASSERTIONS", "OFF")
        .define("LLVM_INCLUDE_TESTS", "OFF")
        .define("LLVM_INCLUDE_EXAMPLES", "OFF")
        .define("LLVM_INCLUDE_BENCHMARKS", "OFF")
        .define("LLVM_INCLUDE_DOCS", "OFF")
        .define("CMAKE_C_COMPILER", "clang")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .build_target("intrinsics_gen")
        .build();
    let build = dst.join("build");

    let status = Command::new("cmake")
        .arg("--build")
        .arg(&build)
        .args(["--target", "omp_gen", "acc_gen", "vt_gen", "--parallel"])
        .status()
        .expect("cannot run cmake");
    assert!(status.success(), "generating the LLVM headers failed");

    build.join("lib/cmake/llvm")
}

fn main() {
    // We link against the libLLVM that ships with rustc, so TPDE reads and compiles the module with
    // the very LLVM that produced it, and no second LLVM ends up in the process. The toolchain does
    // not ship LLVM's headers, so they are generated from the rust-lang/llvm-project submodule.
    check_rustc_commit();
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let llvm_src = ensure_llvm_sources(&repo_root);
    let llvm_dir = generate_llvm_headers(&llvm_src, &out_dir);
    let libllvm = rustc_libllvm(&sysroot_lib());

    let dst = cmake::Config::new(".")
        .define("CMAKE_C_COMPILER", "clang")
        .define("CMAKE_CXX_COMPILER", "clang++")
        .define("TPDE_ENABLE_LLVM_PLUGIN", "OFF")
        .define("TPDE_INCLUDE_TESTS", "OFF")
        .define("TPDE_LOGGING", "OFF")
        .define("TPDE_LINK_LLVM_STATIC", "OFF")
        .define("LLVM_DIR", &llvm_dir)
        .define("RUSTC_LIBLLVM", &libllvm)
        // rustc's libLLVM embeds its own libstdc++ and exports its symbols. Host tools (tpde_encodegen)
        // must bind std:: to the system libstdc++ instead (it has to come first in the lookup order),
        // otherwise they crash on exit in std::locale.
        .define("CMAKE_EXE_LINKER_FLAGS", "-Wl,--push-state,--no-as-needed -lstdc++ -Wl,--pop-state")
        // only what we link, not TPDE's command line tools and tests
        .build_target("tpde_llvm_wrapper")
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

    // libLLVM itself is already on the link line (rustc_driver depends on it), the symbols resolve
    // against it. rustc is linked with -nodefaultlibs, so the C++ runtime is needed explicitly.
    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rustc-link-arg=-Wl,-z,defs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cpp");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
}
