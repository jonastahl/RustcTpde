#[no_mangle]
fn lz_u8(a: u8) -> u32 {
    a.leading_zeros()
}

#[no_mangle]
fn tz_u8(a: u8) -> u32 {
    a.trailing_zeros()
}

#[no_mangle]
fn lz_u32(a: u32) -> u32 {
    a.leading_zeros()
}

#[no_mangle]
fn tz_u32(a: u32) -> u32 {
    a.trailing_zeros()
}

#[no_mangle]
fn lz_u64(a: u64) -> u32 {
    a.leading_zeros()
}

#[no_mangle]
fn tz_u64(a: u64) -> u32 {
    a.trailing_zeros()
}

#[no_mangle]
fn lz_u128(a: u128) -> u32 {
    a.leading_zeros()
}

#[no_mangle]
fn tz_u128(a: u128) -> u32 {
    a.trailing_zeros()
}

// Zero is poison for the `_nonzero` intrinsics, `NonZero` uses those.
#[no_mangle]
fn lz_nz_u128(a: std::num::NonZero<u128>) -> u32 {
    a.leading_zeros()
}

#[no_mangle]
fn tz_nz_u128(a: std::num::NonZero<u128>) -> u32 {
    a.trailing_zeros()
}
