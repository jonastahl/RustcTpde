abi-cafe \
  --add-rustc-codegen-backend=tpde:$(pwd)/target/release/librustc_codegen_tpde.so \
  --pairs rustc_calls_tpde \
  --pairs tpde_calls_rustc \
  --pairs tpde_calls_tpde \
  --pairs cc_calls_tpde \
  --pairs tpde_calls_cc \
  "$@"