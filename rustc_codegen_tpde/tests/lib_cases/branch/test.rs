const TAKEN: u32 = 0xF0F0F0F0;
const NOT_TAKEN: u32 = 0x0F0F0F0F;

extern "Rust" {
  fn simplebranch(a: u32) -> u32;
  fn cmp_eq_i8(a: i8, b: i8) -> u32;
  fn cmp_ne_i8(a: i8, b: i8) -> u32;
  fn cmp_gt_i8(a: i8, b: i8) -> u32;
  fn cmp_ge_i8(a: i8, b: i8) -> u32;
  fn cmp_lt_i8(a: i8, b: i8) -> u32;
  fn cmp_le_i8(a: i8, b: i8) -> u32;
  fn cmp_eq_i16(a: i16, b: i16) -> u32;
  fn cmp_ne_i16(a: i16, b: i16) -> u32;
  fn cmp_gt_i16(a: i16, b: i16) -> u32;
  fn cmp_ge_i16(a: i16, b: i16) -> u32;
  fn cmp_lt_i16(a: i16, b: i16) -> u32;
  fn cmp_le_i16(a: i16, b: i16) -> u32;
  fn cmp_eq_i32(a: i32, b: i32) -> u32;
  fn cmp_ne_i32(a: i32, b: i32) -> u32;
  fn cmp_gt_i32(a: i32, b: i32) -> u32;
  fn cmp_ge_i32(a: i32, b: i32) -> u32;
  fn cmp_lt_i32(a: i32, b: i32) -> u32;
  fn cmp_le_i32(a: i32, b: i32) -> u32;
  fn cmp_eq_i64(a: i64, b: i64) -> u32;
  fn cmp_ne_i64(a: i64, b: i64) -> u32;
  fn cmp_gt_i64(a: i64, b: i64) -> u32;
  fn cmp_ge_i64(a: i64, b: i64) -> u32;
  fn cmp_lt_i64(a: i64, b: i64) -> u32;
  fn cmp_le_i64(a: i64, b: i64) -> u32;
  fn cmp_eq_u8(a: u8, b: u8) -> u32;
  fn cmp_ne_u8(a: u8, b: u8) -> u32;
  fn cmp_gt_u8(a: u8, b: u8) -> u32;
  fn cmp_ge_u8(a: u8, b: u8) -> u32;
  fn cmp_lt_u8(a: u8, b: u8) -> u32;
  fn cmp_le_u8(a: u8, b: u8) -> u32;
  fn cmp_eq_u16(a: u16, b: u16) -> u32;
  fn cmp_ne_u16(a: u16, b: u16) -> u32;
  fn cmp_gt_u16(a: u16, b: u16) -> u32;
  fn cmp_ge_u16(a: u16, b: u16) -> u32;
  fn cmp_lt_u16(a: u16, b: u16) -> u32;
  fn cmp_le_u16(a: u16, b: u16) -> u32;
  fn cmp_eq_u32(a: u32, b: u32) -> u32;
  fn cmp_ne_u32(a: u32, b: u32) -> u32;
  fn cmp_gt_u32(a: u32, b: u32) -> u32;
  fn cmp_ge_u32(a: u32, b: u32) -> u32;
  fn cmp_lt_u32(a: u32, b: u32) -> u32;
  fn cmp_le_u32(a: u32, b: u32) -> u32;
  fn cmp_eq_u64(a: u64, b: u64) -> u32;
  fn cmp_ne_u64(a: u64, b: u64) -> u32;
  fn cmp_gt_u64(a: u64, b: u64) -> u32;
  fn cmp_ge_u64(a: u64, b: u64) -> u32;
  fn cmp_lt_u64(a: u64, b: u64) -> u32;
  fn cmp_le_u64(a: u64, b: u64) -> u32;

  fn cmp_imm_u8(a: u8) -> u32;
  fn cmp_imm_i8(a: i8) -> u32;
  fn cmp_imm_u16(a: u16) -> u32;
  fn cmp_imm_u32(a: u32) -> u32;
  fn cmp_imm_i64(a: i64) -> u32;
  fn cmp_imm_i64_fits(a: i64) -> u32;
  fn cmp_imm_i64_wide(a: i64) -> u32;
  fn cmp_imm_u64_wide(a: u64) -> u32;
  fn cmp_nested_i32(a: i32, b: i32) -> u32;
}

fn marker(cond: bool) -> u32 {
  if cond { TAKEN } else { NOT_TAKEN }
}

/// Checks one comparison function against the corresponding Rust operator over
/// every ordered pair drawn from `$vals`.
macro_rules! check_cmp {
  ($ty:ty, $f:ident, $op:tt, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        assert_eq!(
          unsafe { $f(a, b) },
          marker(a $op b),
          concat!(stringify!($f), "({}, {}): expected a ", stringify!($op), " b == {}"),
          a, b, a $op b
        );
      }
    }
  }};
}

fn main() {
  // The original hand-written case, kept as a regression check.
  assert_eq!(unsafe { simplebranch(101) }, TAKEN);
  assert_eq!(unsafe { simplebranch(u32::MAX) }, TAKEN);
  assert_eq!(unsafe { simplebranch(0x8000_0000) }, TAKEN);
  assert_eq!(unsafe { simplebranch(100) }, NOT_TAKEN);
  assert_eq!(unsafe { simplebranch(99) }, NOT_TAKEN);
  assert_eq!(unsafe { simplebranch(0) }, NOT_TAKEN);

  // i8
  check_cmp!(i8, cmp_eq_i8, ==, [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, 2, i8::MAX - 1, i8::MAX]);
  check_cmp!(i8, cmp_ne_i8, !=, [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, 2, i8::MAX - 1, i8::MAX]);
  check_cmp!(i8, cmp_gt_i8, >, [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, 2, i8::MAX - 1, i8::MAX]);
  check_cmp!(i8, cmp_ge_i8, >=, [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, 2, i8::MAX - 1, i8::MAX]);
  check_cmp!(i8, cmp_lt_i8, <, [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, 2, i8::MAX - 1, i8::MAX]);
  check_cmp!(i8, cmp_le_i8, <=, [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, 2, i8::MAX - 1, i8::MAX]);

  // i16
  check_cmp!(i16, cmp_eq_i16, ==, [i16::MIN, i16::MIN + 1, -2, -1, 0, 1, 2, i16::MAX - 1, i16::MAX]);
  check_cmp!(i16, cmp_ne_i16, !=, [i16::MIN, i16::MIN + 1, -2, -1, 0, 1, 2, i16::MAX - 1, i16::MAX]);
  check_cmp!(i16, cmp_gt_i16, >, [i16::MIN, i16::MIN + 1, -2, -1, 0, 1, 2, i16::MAX - 1, i16::MAX]);
  check_cmp!(i16, cmp_ge_i16, >=, [i16::MIN, i16::MIN + 1, -2, -1, 0, 1, 2, i16::MAX - 1, i16::MAX]);
  check_cmp!(i16, cmp_lt_i16, <, [i16::MIN, i16::MIN + 1, -2, -1, 0, 1, 2, i16::MAX - 1, i16::MAX]);
  check_cmp!(i16, cmp_le_i16, <=, [i16::MIN, i16::MIN + 1, -2, -1, 0, 1, 2, i16::MAX - 1, i16::MAX]);

  // i32
  check_cmp!(i32, cmp_eq_i32, ==, [i32::MIN, i32::MIN + 1, -2, -1, 0, 1, 2, i32::MAX - 1, i32::MAX]);
  check_cmp!(i32, cmp_ne_i32, !=, [i32::MIN, i32::MIN + 1, -2, -1, 0, 1, 2, i32::MAX - 1, i32::MAX]);
  check_cmp!(i32, cmp_gt_i32, >, [i32::MIN, i32::MIN + 1, -2, -1, 0, 1, 2, i32::MAX - 1, i32::MAX]);
  check_cmp!(i32, cmp_ge_i32, >=, [i32::MIN, i32::MIN + 1, -2, -1, 0, 1, 2, i32::MAX - 1, i32::MAX]);
  check_cmp!(i32, cmp_lt_i32, <, [i32::MIN, i32::MIN + 1, -2, -1, 0, 1, 2, i32::MAX - 1, i32::MAX]);
  check_cmp!(i32, cmp_le_i32, <=, [i32::MIN, i32::MIN + 1, -2, -1, 0, 1, 2, i32::MAX - 1, i32::MAX]);

  // i64
  check_cmp!(i64, cmp_eq_i64, ==, [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, 2, i64::MAX - 1, i64::MAX]);
  check_cmp!(i64, cmp_ne_i64, !=, [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, 2, i64::MAX - 1, i64::MAX]);
  check_cmp!(i64, cmp_gt_i64, >, [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, 2, i64::MAX - 1, i64::MAX]);
  check_cmp!(i64, cmp_ge_i64, >=, [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, 2, i64::MAX - 1, i64::MAX]);
  check_cmp!(i64, cmp_lt_i64, <, [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, 2, i64::MAX - 1, i64::MAX]);
  check_cmp!(i64, cmp_le_i64, <=, [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, 2, i64::MAX - 1, i64::MAX]);

  // u8
  check_cmp!(u8, cmp_eq_u8, ==, [0, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_cmp!(u8, cmp_ne_u8, !=, [0, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_cmp!(u8, cmp_gt_u8, >, [0, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_cmp!(u8, cmp_ge_u8, >=, [0, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_cmp!(u8, cmp_lt_u8, <, [0, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_cmp!(u8, cmp_le_u8, <=, [0, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);

  // u16
  check_cmp!(u16, cmp_eq_u16, ==, [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_cmp!(u16, cmp_ne_u16, !=, [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_cmp!(u16, cmp_gt_u16, >, [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_cmp!(u16, cmp_ge_u16, >=, [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_cmp!(u16, cmp_lt_u16, <, [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_cmp!(u16, cmp_le_u16, <=, [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);

  // u32
  check_cmp!(u32, cmp_eq_u32, ==, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_cmp!(u32, cmp_ne_u32, !=, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_cmp!(u32, cmp_gt_u32, >, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_cmp!(u32, cmp_ge_u32, >=, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_cmp!(u32, cmp_lt_u32, <, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_cmp!(u32, cmp_le_u32, <=, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);

  // u64
  check_cmp!(u64, cmp_eq_u64, ==, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);
  check_cmp!(u64, cmp_ne_u64, !=, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);
  check_cmp!(u64, cmp_gt_u64, >, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);
  check_cmp!(u64, cmp_ge_u64, >=, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);
  check_cmp!(u64, cmp_lt_u64, <, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);
  check_cmp!(u64, cmp_le_u64, <=, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);

  // Comparison against a constant (immediate form of CMP).
  for a in [0u8, 1, 0x7F, 0x80, 0x81, 0xFE, 0xFF] {
    assert_eq!(unsafe { cmp_imm_u8(a) }, marker(a > 0x80), "cmp_imm_u8({})", a);
  }
  for a in [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, i8::MAX] {
    assert_eq!(unsafe { cmp_imm_i8(a) }, marker(a > -1), "cmp_imm_i8({})", a);
  }
  for a in [0u16, 1, 0x7FFF, 0x8000, 0x8001, u16::MAX] {
    assert_eq!(unsafe { cmp_imm_u16(a) }, marker(a >= 0x8000), "cmp_imm_u16({})", a);
  }
  for a in [0u32, 1, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, u32::MAX] {
    assert_eq!(unsafe { cmp_imm_u32(a) }, marker(a >= 0x8000_0000), "cmp_imm_u32({})", a);
  }
  for a in [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, i64::MAX] {
    assert_eq!(unsafe { cmp_imm_i64(a) }, marker(a < -1), "cmp_imm_i64({})", a);
  }
  // Largest immediate that still fits in a sign-extended imm32.
  for a in [i64::MIN, -1, 0, 0x7FFF_FFFE, 0x7FFF_FFFF, 0x8000_0000, i64::MAX] {
    assert_eq!(unsafe { cmp_imm_i64_fits(a) }, marker(a >= 0x7FFF_FFFF), "cmp_imm_i64_fits({})", a);
  }

  // Nested branches.
  for a in [i32::MIN, -1, 0, 1, i32::MAX] {
    for b in [i32::MIN, -1, 0, 1, i32::MAX] {
      let expected = if a < b {
        if a < 0 { 1 } else { 2 }
      } else if a > b {
        if b < 0 { 3 } else { 4 }
      } else {
        5
      };
      assert_eq!(unsafe { cmp_nested_i32(a, b) }, expected, "cmp_nested_i32({}, {})", a, b);
    }
  }

  for a in [i64::MIN, -1, 0, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, i64::MAX] {
    assert_eq!(unsafe { cmp_imm_i64_wide(a) }, marker(a >= 0x8000_0000), "cmp_imm_i64_wide({})", a);
  }
  for a in [0u64, 1, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, u64::MAX] {
    assert_eq!(unsafe { cmp_imm_u64_wide(a) }, marker(a >= 0x8000_0000_0000_0000), "cmp_imm_u64_wide({})", a);
  }
}
