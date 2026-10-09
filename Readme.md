
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
