// Drives the leading/trailing zero counts in `source.rs`, including the
// 128 bit variants which are split into two registers.

use std::num::NonZero;

extern "Rust" {
  fn lz_u8(a: u8) -> u32;
  fn tz_u8(a: u8) -> u32;
  fn lz_u32(a: u32) -> u32;
  fn tz_u32(a: u32) -> u32;
  fn lz_u64(a: u64) -> u32;
  fn tz_u64(a: u64) -> u32;
  fn lz_u128(a: u128) -> u32;
  fn tz_u128(a: u128) -> u32;
  fn lz_nz_u128(a: NonZero<u128>) -> u32;
  fn tz_nz_u128(a: NonZero<u128>) -> u32;
}

fn main() {
  for a in [0u8, 1, 2, 0x80, 0xFF] {
    assert_eq!(unsafe { lz_u8(a) }, a.leading_zeros(), "lz_u8({a})");
    assert_eq!(unsafe { tz_u8(a) }, a.trailing_zeros(), "tz_u8({a})");
  }
  for a in [0u32, 1, 2, 0x8000_0000, 0xA5C3_1E78, u32::MAX] {
    assert_eq!(unsafe { lz_u32(a) }, a.leading_zeros(), "lz_u32({a})");
    assert_eq!(unsafe { tz_u32(a) }, a.trailing_zeros(), "tz_u32({a})");
  }
  for a in [0u64, 1, 2, 1 << 32, 0x8000_0000_0000_0000, 0xA5C3_1E78_9B4D_0F26, u64::MAX] {
    assert_eq!(unsafe { lz_u64(a) }, a.leading_zeros(), "lz_u64({a})");
    assert_eq!(unsafe { tz_u64(a) }, a.trailing_zeros(), "tz_u64({a})");
  }
  let vals128 = [
    0u128, 1, 2, 1 << 63, 1 << 64, 1 << 65, 1 << 100, 1 << 127,
    0xA5C3_1E78_9B4D_0F26_0000_0000_0000_0000, 0x0000_0000_0000_0000_A5C3_1E78_9B4D_0F26,
    u128::MAX,
  ];
  for a in vals128 {
    assert_eq!(unsafe { lz_u128(a) }, a.leading_zeros(), "lz_u128({a})");
    assert_eq!(unsafe { tz_u128(a) }, a.trailing_zeros(), "tz_u128({a})");
    if let Some(nz) = NonZero::new(a) {
      assert_eq!(unsafe { lz_nz_u128(nz) }, a.leading_zeros(), "lz_nz_u128({a})");
      assert_eq!(unsafe { tz_nz_u128(nz) }, a.trailing_zeros(), "tz_nz_u128({a})");
    }
  }
}
