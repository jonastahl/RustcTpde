//! Thin wrapper around tpde-llvm (see cpp/wrapper.cpp).

unsafe extern "C" {
    fn tpde_llvm_compile_bitcode(data: *const u8, len: usize, out_len: *mut usize) -> *mut u8;
    fn tpde_llvm_free_buffer(buf: *mut u8);
}

/// Compiles LLVM bitcode to an ELF object file with tpde-llvm.
pub fn compile_bitcode(bitcode: &[u8]) -> Option<Vec<u8>> {
    let mut len = 0usize;
    unsafe {
        let ptr = tpde_llvm_compile_bitcode(bitcode.as_ptr(), bitcode.len(), &mut len);
        if ptr.is_null() {
            return None;
        }
        let obj = std::slice::from_raw_parts(ptr, len).to_vec();
        tpde_llvm_free_buffer(ptr);
        Some(obj)
    }
}
