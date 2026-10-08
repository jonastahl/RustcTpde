use std::ptr;

// Copies with a runtime length: lowered to a call to memcpy/memmove/memset.
#[no_mangle]
pub unsafe extern "C" fn do_memcpy(dst: *mut u8, src: *const u8, len: usize) {
    ptr::copy_nonoverlapping(src, dst, len);
}

#[no_mangle]
pub unsafe extern "C" fn do_memmove(dst: *mut u8, src: *const u8, len: usize) {
    ptr::copy(src, dst, len);
}

#[no_mangle]
pub unsafe extern "C" fn do_memset(dst: *mut u8, val: u8, len: usize) {
    ptr::write_bytes(dst, val, len);
}

// Element sizes larger than one byte: the byte count is len * size_of::<T>().
#[no_mangle]
pub unsafe extern "C" fn do_memcpy_u64(dst: *mut u64, src: *const u64, len: usize) {
    ptr::copy_nonoverlapping(src, dst, len);
}

#[no_mangle]
pub unsafe extern "C" fn do_memmove_u32(dst: *mut u32, src: *const u32, len: usize) {
    ptr::copy(src, dst, len);
}

#[no_mangle]
pub unsafe extern "C" fn do_memset_u16(dst: *mut u16, val: u8, len: usize) {
    ptr::write_bytes(dst, val, len);
}

// Constant lengths: may be inlined or kept as a call, both must be correct.
#[no_mangle]
pub unsafe extern "C" fn memcpy_const_8(dst: *mut u8, src: *const u8) {
    ptr::copy_nonoverlapping(src, dst, 8);
}

#[no_mangle]
pub unsafe extern "C" fn memcpy_const_100(dst: *mut u8, src: *const u8) {
    ptr::copy_nonoverlapping(src, dst, 100);
}

#[no_mangle]
pub unsafe extern "C" fn memmove_const_64(dst: *mut u8, src: *const u8) {
    ptr::copy(src, dst, 64);
}

#[no_mangle]
pub unsafe extern "C" fn memset_const_0(dst: *mut u8) {
    ptr::write_bytes(dst, 0, 33);
}

#[no_mangle]
pub unsafe extern "C" fn memset_const_val(dst: *mut u8) {
    ptr::write_bytes(dst, 0xAB, 200);
}

// Zero length must not touch memory.
#[no_mangle]
pub unsafe extern "C" fn memcpy_zero(dst: *mut u8, src: *const u8) {
    ptr::copy_nonoverlapping(src, dst, 0);
}

// Overlapping move inside a single buffer, forward and backward.
#[no_mangle]
pub unsafe extern "C" fn shift_up(buf: *mut u8, len: usize, by: usize) {
    ptr::copy(buf, buf.add(by), len);
}

#[no_mangle]
pub unsafe extern "C" fn shift_down(buf: *mut u8, len: usize, by: usize) {
    ptr::copy(buf.add(by), buf, len);
}

// Aggregate copies and zero-initialisation are typically lowered to memcpy/memset.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Big {
    pub data: [u64; 32],
}

#[no_mangle]
pub unsafe extern "C" fn copy_big(dst: *mut Big, src: *const Big) {
    *dst = *src;
}

#[no_mangle]
pub unsafe extern "C" fn zero_big(dst: *mut Big) {
    *dst = Big { data: [0; 32] };
}

#[no_mangle]
pub unsafe extern "C" fn fill_big(dst: *mut Big, v: u64) {
    *dst = Big { data: [v; 32] };
}

// Return value use after the call: the destination must survive the libcall.
#[no_mangle]
pub unsafe extern "C" fn memcpy_then_read(dst: *mut u8, src: *const u8, len: usize) -> u8 {
    ptr::copy_nonoverlapping(src, dst, len);
    *dst
}

#[no_mangle]
pub unsafe extern "C" fn memset_then_sum(dst: *mut u8, len: usize) -> u64 {
    ptr::write_bytes(dst, 3, len);
    let mut acc = 0u64;
    let mut i = 0usize;
    while i < len {
        acc += *dst.add(i) as u64;
        i += 1;
    }
    acc
}
