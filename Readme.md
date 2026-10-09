
# TPDE Backend for Rust

First, install the nightly version of rustc
```bash
rustup toolchain install nightly
```

Then, build the library
```bash
cargo build --release
```

Use in rustc
```bash
rustc -Zcodegen-backend=target/release/librustc_codegen_tpde_dylib.so <files>
```
Use in cargo
```bash
WORKDIR=$(pwd) # replace by this directory
RUSTFLAGS="-Zcodegen-backend=$WORKDIR/target/release/librustc_codegen_tpde_dylib.so" cargo build
```

To test the difference 
```bash
./target/release/rustc_tpde <files>
```

## LLVM + TPDE backend (`rustc_codegen_llvm_tpde`)

Rustc's LLVM generates the IR, TPDE compiles it. TPDE links against the libLLVM that ships with the
toolchain, but the toolchain has no LLVM headers. `cargo build` therefore generates them from the
`deps/rust-llvm-project` submodule (rust-lang/llvm-project, only the headers are generated, LLVM is not
compiled). The submodule is fetched automatically by `build.rs` if needed.

Headers and library must match exactly. The submodule is pinned to the LLVM commit of the nightly in
`rust-toolchain.toml` (`rustc -vV` commit-hash, then the `src/llvm-project` submodule of that commit in
rust-lang/rust). When changing the nightly, update the submodule and `LLVM_HEADERS_FOR_RUSTC` in
`rustc_codegen_llvm_tpde/build.rs`; the build warns if they differ.
