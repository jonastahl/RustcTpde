
extern "Rust" {
  fn add_i8(a: i8, b: i8) -> i8;
  fn sub_i8(a: i8, b: i8) -> i8;
  fn add_i16(a: i16, b: i16) -> i16;
  fn sub_i16(a: i16, b: i16) -> i16;
  fn add_i32(a: i32, b: i32) -> i32;
  fn sub_i32(a: i32, b: i32) -> i32;
  fn add_i64(a: i64, b: i64) -> i64;
  fn sub_i64(a: i64, b: i64) -> i64;
  fn add_u8(a: u8, b: u8) -> u8;
  fn sub_u8(a: u8, b: u8) -> u8;
  fn add_u16(a: u16, b: u16) -> u16;
  fn sub_u16(a: u16, b: u16) -> u16;
  fn add_u32(a: u32, b: u32) -> u32;
  fn sub_u32(a: u32, b: u32) -> u32;
  fn add_u64(a: u64, b: u64) -> u64;
  fn sub_u64(a: u64, b: u64) -> u64;

  fn add3_u32(a: u32, b: u32, c: u32) -> u32;
  fn add_sub_u32(a: u32, b: u32) -> u32;
  fn add_sub_u8(a: u8, b: u8) -> u8;
  fn add8_args_u32(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32;

  fn mul_i8(a: i8, b: i8) -> i8;
  fn mul_i16(a: i16, b: i16) -> i16;
  fn mul_i32(a: i32, b: i32) -> i32;
  fn mul_i64(a: i64, b: i64) -> i64;
  fn mul_u8(a: u8, b: u8) -> u8;
  fn mul_u16(a: u16, b: u16) -> u16;
  fn mul_u32(a: u32, b: u32) -> u32;
  fn mul_u64(a: u64, b: u64) -> u64;

  fn shl_i8(a: i8, b: i8) -> i8;
  fn shl_i16(a: i16, b: i16) -> i16;
  fn shl_i32(a: i32, b: i32) -> i32;
  fn shl_i64(a: i64, b: i64) -> i64;
  fn shl_u8(a: u8, b: u8) -> u8;
  fn shl_u16(a: u16, b: u16) -> u16;
  fn shl_u32(a: u32, b: u32) -> u32;
  fn shl_u64(a: u64, b: u64) -> u64;

  fn shl3_u32(a: u32) -> u32;
  fn and_u8(a: u8, b: u8) -> u8;
  fn and_u16(a: u16, b: u16) -> u16;
  fn and_u32(a: u32, b: u32) -> u32;
  fn and_u64(a: u64, b: u64) -> u64;
  fn or_u8(a: u8, b: u8) -> u8;
  fn or_u16(a: u16, b: u16) -> u16;
  fn or_u32(a: u32, b: u32) -> u32;
  fn or_u64(a: u64, b: u64) -> u64;

  fn mul_add_u32(a: u32, b: u32, c: u32) -> u32;
  fn add_mul_u32(a: u32, b: u32, c: u32) -> u32;
  fn shl_mul_u32(a: u32, b: u32) -> u32;
  fn keep_low_u32(a: u32, b: u32) -> u32;
  fn mul_add_u8(a: u8, b: u8, c: u8) -> u8;

  fn div_i8(a: i8, b: i8) -> i8;
  fn div_i16(a: i16, b: i16) -> i16;
  fn div_i32(a: i32, b: i32) -> i32;
  fn div_i64(a: i64, b: i64) -> i64;
  fn div_u8(a: u8, b: u8) -> u8;
  fn div_u16(a: u16, b: u16) -> u16;
  fn div_u32(a: u32, b: u32) -> u32;
  fn div_u64(a: u64, b: u64) -> u64;

  fn rem_i8(a: i8, b: i8) -> i8;
  fn rem_i16(a: i16, b: i16) -> i16;
  fn rem_i32(a: i32, b: i32) -> i32;
  fn rem_i64(a: i64, b: i64) -> i64;
  fn rem_u8(a: u8, b: u8) -> u8;
  fn rem_u16(a: u16, b: u16) -> u16;
  fn rem_u32(a: u32, b: u32) -> u32;
  fn rem_u64(a: u64, b: u64) -> u64;

  fn shr_i8(a: i8, b: i8) -> i8;
  fn shr_i16(a: i16, b: i16) -> i16;
  fn shr_i32(a: i32, b: i32) -> i32;
  fn shr_i64(a: i64, b: i64) -> i64;
  fn shr_u8(a: u8, b: u8) -> u8;
  fn shr_u16(a: u16, b: u16) -> u16;
  fn shr_u32(a: u32, b: u32) -> u32;
  fn shr_u64(a: u64, b: u64) -> u64;

  fn xor_i8(a: i8, b: i8) -> i8;
  fn xor_i16(a: i16, b: i16) -> i16;
  fn xor_i32(a: i32, b: i32) -> i32;
  fn xor_i64(a: i64, b: i64) -> i64;
  fn xor_u8(a: u8, b: u8) -> u8;
  fn xor_u16(a: u16, b: u16) -> u16;
  fn xor_u32(a: u32, b: u32) -> u32;
  fn xor_u64(a: u64, b: u64) -> u64;

  fn shr3_u32(a: u32) -> u32;
  fn shr3_i32(a: i32) -> i32;

  fn not_i8(a: i8) -> i8;
  fn not_i16(a: i16) -> i16;
  fn not_i32(a: i32) -> i32;
  fn not_i64(a: i64) -> i64;
  fn not_u8(a: u8) -> u8;
  fn not_u16(a: u16) -> u16;
  fn not_u32(a: u32) -> u32;
  fn not_u64(a: u64) -> u64;
  fn neg_i8(a: i8) -> i8;
  fn neg_i16(a: i16) -> i16;
  fn neg_i32(a: i32) -> i32;
  fn neg_i64(a: i64) -> i64;
  fn not_bool(a: bool) -> bool;

  fn clear_low_u32(a: u32, b: u32) -> u32;
  fn div_rem_u32(a: u32, b: u32) -> u32;
  fn div_rem_i32(a: i32, b: i32) -> i32;
  fn shr_shl_u32(a: u32, b: u32) -> u32;
  fn de_morgan_u32(a: u32, b: u32) -> u32;
  fn xor_roundtrip_u8(a: u8, b: u8) -> u8;
  fn neg_via_not_i32(a: i32) -> i32;

  fn add_i128(a: i128, b: i128) -> i128;
  fn sub_i128(a: i128, b: i128) -> i128;
  fn add_u128(a: u128, b: u128) -> u128;
  fn sub_u128(a: u128, b: u128) -> u128;
  fn mul_i128(a: i128, b: i128) -> i128;
  fn mul_u128(a: u128, b: u128) -> u128;
  fn div_i128(a: i128, b: i128) -> i128;
  fn div_u128(a: u128, b: u128) -> u128;
  fn rem_i128(a: i128, b: i128) -> i128;
  fn rem_u128(a: u128, b: u128) -> u128;
  fn shl_i128(a: i128, b: i128) -> i128;
  fn shl_u128(a: u128, b: u128) -> u128;
  fn shr_i128(a: i128, b: i128) -> i128;
  fn shr_u128(a: u128, b: u128) -> u128;
  fn and_u128(a: u128, b: u128) -> u128;
  fn or_u128(a: u128, b: u128) -> u128;
  fn xor_i128(a: i128, b: i128) -> i128;
  fn xor_u128(a: u128, b: u128) -> u128;
  fn not_i128(a: i128) -> i128;
  fn not_u128(a: u128) -> u128;
  fn neg_i128(a: i128) -> i128;

  fn shl3_u128(a: u128) -> u128;
  fn shl64_u128(a: u128) -> u128;
  fn shr64_u128(a: u128) -> u128;
  fn shr100_i128(a: i128) -> i128;

  fn add3_u128(a: u128, b: u128, c: u128) -> u128;
  fn add_sub_u128(a: u128, b: u128) -> u128;
  fn add4_args_u128(a: u128, b: u128, c: u128, d: u128) -> u128;
  fn mixed_width_u128(a: u32, b: u128, c: u64, d: u128) -> u128;
  fn mul_add_u128(a: u128, b: u128, c: u128) -> u128;

  fn div_rem_u128(a: u128, b: u128) -> u128;
  fn div_rem_i128(a: i128, b: i128) -> i128;
  fn shr_shl_u128(a: u128, b: u128) -> u128;
  fn de_morgan_u128(a: u128, b: u128) -> u128;
  fn neg_via_not_i128(a: i128) -> i128;

  fn zext_u64_u128(a: u64) -> u128;
  fn sext_i64_i128(a: i64) -> i128;
  fn trunc_u128_u64(a: u128) -> u64;
  fn trunc_u128_u8(a: u128) -> u8;
  fn rotl_u8(a: u8, n: u32) -> u8;
  fn rotr_u8(a: u8, n: u32) -> u8;
  fn funnel_shl_u8(a: u8, b: u8, n: u32) -> u8;
  fn funnel_shr_u8(a: u8, b: u8, n: u32) -> u8;
  fn rotl_u16(a: u16, n: u32) -> u16;
  fn rotr_u16(a: u16, n: u32) -> u16;
  fn funnel_shl_u16(a: u16, b: u16, n: u32) -> u16;
  fn funnel_shr_u16(a: u16, b: u16, n: u32) -> u16;
  fn rotl_u32(a: u32, n: u32) -> u32;
  fn rotr_u32(a: u32, n: u32) -> u32;
  fn funnel_shl_u32(a: u32, b: u32, n: u32) -> u32;
  fn funnel_shr_u32(a: u32, b: u32, n: u32) -> u32;
  fn rotl_u64(a: u64, n: u32) -> u64;
  fn rotr_u64(a: u64, n: u32) -> u64;
  fn funnel_shl_u64(a: u64, b: u64, n: u32) -> u64;
  fn funnel_shr_u64(a: u64, b: u64, n: u32) -> u64;
}

/// Checks `add_$ty` and `sub_$ty` over every ordered pair drawn from `$vals`.
macro_rules! check_add_sub {
  ($ty:ty, $add:ident, $sub:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        assert_eq!(
          unsafe { $add(a, b) },
          a.wrapping_add(b),
          concat!(stringify!($add), "({}, {})"), a, b
        );
        assert_eq!(
          unsafe { $sub(a, b) },
          a.wrapping_sub(b),
          concat!(stringify!($sub), "({}, {})"), a, b
        );
      }
    }
  }};
}

/// Checks `mul_$ty` over every ordered pair drawn from `$vals`.
macro_rules! check_mul {
  ($ty:ty, $mul:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        assert_eq!(
          unsafe { $mul(a, b) },
          a.wrapping_mul(b),
          concat!(stringify!($mul), "({}, {})"), a, b
        );
      }
    }
  }};
}

/// Checks `shl_$ty` for every operand in `$vals` and every in-range shift
/// amount. Out-of-range amounts are left alone: the backend compiles an
/// unchecked shift, so they have no defined result to compare against.
macro_rules! check_shl {
  ($ty:ty, $shl:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for b in 0..(<$ty>::BITS as $ty) {
        assert_eq!(
          unsafe { $shl(a, b) },
          a.wrapping_shl(b as u32),
          concat!(stringify!($shl), "({}, {})"), a, b
        );
      }
    }
  }};
}

/// Checks `and_$ty` and `or_$ty` over every ordered pair drawn from `$vals`.
macro_rules! check_bitwise {
  ($ty:ty, $and:ident, $or:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        assert_eq!(unsafe { $and(a, b) }, a & b, concat!(stringify!($and), "({}, {})"), a, b);
        assert_eq!(unsafe { $or(a, b) }, a | b, concat!(stringify!($or), "({}, {})"), a, b);
      }
    }
  }};
}

/// Checks `div_$ty` and `rem_$ty` over every ordered pair drawn from `$vals`.
/// Pairs that would trap are skipped: a zero divisor, and `MIN / -1`, whose
/// quotient is not representable.
macro_rules! check_div_rem {
  ($ty:ty, $div:ident, $rem:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        if b == 0 || a.checked_div(b).is_none() {
          continue;
        }
        assert_eq!(
          unsafe { $div(a, b) }, a / b,
          concat!(stringify!($div), "({}, {})"), a, b
        );
        assert_eq!(
          unsafe { $rem(a, b) }, a % b,
          concat!(stringify!($rem), "({}, {})"), a, b
        );
      }
    }
  }};
}

/// Checks `shr_$ty` for every operand in `$vals` and every in-range shift
/// amount. As with `shl`, out-of-range amounts have no defined result.
macro_rules! check_shr {
  ($ty:ty, $shr:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for b in 0..(<$ty>::BITS as $ty) {
        assert_eq!(
          unsafe { $shr(a, b) },
          a.wrapping_shr(b as u32),
          concat!(stringify!($shr), "({}, {})"), a, b
        );
      }
    }
  }};
}

/// Checks `xor_$ty` over every ordered pair drawn from `$vals`.
macro_rules! check_xor {
  ($ty:ty, $xor:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        assert_eq!(unsafe { $xor(a, b) }, a ^ b, concat!(stringify!($xor), "({}, {})"), a, b);
      }
    }
  }};
}

/// Checks `not_$ty` over every value in `$vals`.
macro_rules! check_not {
  ($ty:ty, $not:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      assert_eq!(unsafe { $not(a) }, !a, concat!(stringify!($not), "({})"), a);
    }
  }};
}

/// Checks `neg_$ty` over every value in `$vals`, including `MIN`, which
/// wraps to itself.
macro_rules! check_neg {
  ($ty:ty, $neg:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      assert_eq!(unsafe { $neg(a) }, a.wrapping_neg(), concat!(stringify!($neg), "({})"), a);
    }
  }};
}

fn main() {
  check_add_sub!(i8, add_i8, sub_i8, [i8::MIN, i8::MIN + 1, -2, -1, 0, 1, 2, i8::MAX - 1, i8::MAX]);
  check_add_sub!(i16, add_i16, sub_i16, [i16::MIN, i16::MIN + 1, -2, -1, 0, 1, 2, i16::MAX - 1, i16::MAX]);
  check_add_sub!(i32, add_i32, sub_i32, [i32::MIN, i32::MIN + 1, -2, -1, 0, 1, 2, i32::MAX - 1, i32::MAX]);
  check_add_sub!(i64, add_i64, sub_i64, [i64::MIN, i64::MIN + 1, -2, -1, 0, 1, 2, i64::MAX - 1, i64::MAX]);
  check_add_sub!(u8, add_u8, sub_u8, [0, 1, 2, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_add_sub!(u16, add_u16, sub_u16, [0, 1, 2, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_add_sub!(u32, add_u32, sub_u32, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_add_sub!(u64, add_u64, sub_u64, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);

  // Chained arithmetic, including a carry out of the low word.
  assert_eq!(unsafe { add3_u32(1, 2, 3) }, 6);
  assert_eq!(unsafe { add3_u32(u32::MAX, 1, 1) }, 1);
  assert_eq!(unsafe { add3_u32(0x8000_0000, 0x8000_0000, 7) }, 7);

  // (a + b) - (a - b) == 2 * b, wrapping, for every operand pair.
  for a in [0u32, 1, 0x7FFF_FFFF, 0x8000_0000, u32::MAX] {
    for b in [0u32, 1, 0x7FFF_FFFF, 0x8000_0000, u32::MAX] {
      assert_eq!(unsafe { add_sub_u32(a, b) }, b.wrapping_mul(2), "add_sub_u32({}, {})", a, b);
    }
  }
  for a in [0u8, 1, 0x7F, 0x80, 0xFF] {
    for b in [0u8, 1, 0x7F, 0x80, 0xFF] {
      assert_eq!(unsafe { add_sub_u8(a, b) }, b.wrapping_mul(2), "add_sub_u8({}, {})", a, b);
    }
  }

  // Stack-passed arguments: the 7th and 8th must not be dropped or swapped.
  assert_eq!(unsafe { add8_args_u32(1, 2, 3, 4, 5, 6, 7, 8) }, 36);
  assert_eq!(unsafe { add8_args_u32(0, 0, 0, 0, 0, 0, 0, u32::MAX) }, u32::MAX);
  assert_eq!(unsafe { add8_args_u32(0, 0, 0, 0, 0, 0, u32::MAX, 0) }, u32::MAX);
  assert_eq!(unsafe { add8_args_u32(u32::MAX, 1, 0, 0, 0, 0, 0, 0) }, 0);
  assert_eq!(unsafe { add8_args_u32(1, 0, 0, 0, 0, 0, 0, u32::MAX) }, 0);

  check_mul!(i8, mul_i8, [i8::MIN, i8::MIN + 1, -16, -3, -2, -1, 0, 1, 2, 3, 16, i8::MAX - 1, i8::MAX]);
  check_mul!(i16, mul_i16, [i16::MIN, i16::MIN + 1, -256, -3, -2, -1, 0, 1, 2, 3, 256, i16::MAX - 1, i16::MAX]);
  check_mul!(i32, mul_i32, [i32::MIN, i32::MIN + 1, -65536, -3, -2, -1, 0, 1, 2, 3, 65536, i32::MAX - 1, i32::MAX]);
  check_mul!(i64, mul_i64, [i64::MIN, i64::MIN + 1, -(1 << 32), -3, -2, -1, 0, 1, 2, 3, 1 << 32, i64::MAX - 1, i64::MAX]);
  check_mul!(u8, mul_u8, [0, 1, 2, 3, 0x10, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_mul!(u16, mul_u16, [0, 1, 2, 3, 0x100, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_mul!(u32, mul_u32, [0, 1, 2, 3, 0x1_0000, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_mul!(u64, mul_u64, [0, 1, 2, 3, 1 << 32, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);

  // Shifts by every amount the type allows. The signed cases only use
  // non-negative amounts, since a negative shift is out of range.
  check_shl!(i8, shl_i8, [i8::MIN, -1, 0, 1, 2, i8::MAX]);
  check_shl!(i16, shl_i16, [i16::MIN, -1, 0, 1, 2, i16::MAX]);
  check_shl!(i32, shl_i32, [i32::MIN, -1, 0, 1, 2, i32::MAX]);
  check_shl!(i64, shl_i64, [i64::MIN, -1, 0, 1, 2, i64::MAX]);
  check_shl!(u8, shl_u8, [0, 1, 2, 0x7F, 0x80, 0xFF]);
  check_shl!(u16, shl_u16, [0, 1, 2, 0x7FFF, 0x8000, 0xFFFF]);
  check_shl!(u32, shl_u32, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF]);
  check_shl!(u64, shl_u64, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF]);

  // A constant shift amount, which the backend can fold into the encoding.
  for a in [0u32, 1, 3, 0x1FFF_FFFF, 0x2000_0000, 0x8000_0000, u32::MAX] {
    assert_eq!(unsafe { shl3_u32(a) }, a.wrapping_shl(3), "shl3_u32({})", a);
  }

  check_bitwise!(u8, and_u8, or_u8, [0, 1, 2, 0x0F, 0xF0, 0x55, 0xAA, 0xFF]);
  check_bitwise!(u16, and_u16, or_u16, [0, 1, 2, 0x00FF, 0xFF00, 0x5555, 0xAAAA, 0xFFFF]);
  check_bitwise!(u32, and_u32, or_u32, [0, 1, 2, 0x0000_FFFF, 0xFFFF_0000, 0x5555_5555, 0xAAAA_AAAA, 0xFFFF_FFFF]);
  check_bitwise!(u64, and_u64, or_u64, [0, 1, 2, 0x0000_0000_FFFF_FFFF, 0xFFFF_FFFF_0000_0000, 0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA, 0xFFFF_FFFF_FFFF_FFFF]);

  // Operator precedence and the width of each intermediate.
  for a in [0u32, 1, 3, 0x1_0000, 0x8000_0000, u32::MAX] {
    for b in [0u32, 1, 3, 0x1_0000, 0x8000_0000, u32::MAX] {
      for c in [0u32, 1, 7, u32::MAX] {
        assert_eq!(
          unsafe { mul_add_u32(a, b, c) },
          a.wrapping_mul(b).wrapping_add(c),
          "mul_add_u32({}, {}, {})", a, b, c
        );
        assert_eq!(
          unsafe { add_mul_u32(a, b, c) },
          a.wrapping_add(b.wrapping_mul(c)),
          "add_mul_u32({}, {}, {})", a, b, c
        );
      }
    }
  }
  for a in [0u8, 1, 3, 0x10, 0x80, 0xFF] {
    for b in [0u8, 1, 3, 0x10, 0x80, 0xFF] {
      for c in [0u8, 1, 7, 0xFF] {
        assert_eq!(
          unsafe { mul_add_u8(a, b, c) },
          a.wrapping_mul(b).wrapping_add(c),
          "mul_add_u8({}, {}, {})", a, b, c
        );
      }
    }
  }

  // `(a << b) - a * (1 << b)` is zero whenever both sides agree, which they
  // must for every in-range shift amount.
  for a in [0u32, 1, 3, 0x1_0000, 0x8000_0000, u32::MAX] {
    for b in 0..32u32 {
      assert_eq!(unsafe { shl_mul_u32(a, b) }, 0, "shl_mul_u32({}, {})", a, b);
    }
  }

  // Masking off the high bits: `b` of 32 would overflow the `1 << b`, so it
  // stops at 31.
  for a in [0u32, 1, 3, 0x5555_5555, 0xAAAA_AAAA, u32::MAX] {
    for b in 0..32u32 {
      assert_eq!(
        unsafe { keep_low_u32(a, b) },
        a & (1u32.wrapping_shl(b).wrapping_sub(1)),
        "keep_low_u32({}, {})", a, b
      );
    }
  }

  // Division and remainder, at every width. The value sets include `MIN` and
  // `-1` so that the pair the macro has to skip is actually present.
  check_div_rem!(i8, div_i8, rem_i8, [i8::MIN, i8::MIN + 1, -7, -2, -1, 0, 1, 2, 7, i8::MAX - 1, i8::MAX]);
  check_div_rem!(i16, div_i16, rem_i16, [i16::MIN, i16::MIN + 1, -7, -2, -1, 0, 1, 2, 7, i16::MAX - 1, i16::MAX]);
  check_div_rem!(i32, div_i32, rem_i32, [i32::MIN, i32::MIN + 1, -7, -2, -1, 0, 1, 2, 7, i32::MAX - 1, i32::MAX]);
  check_div_rem!(i64, div_i64, rem_i64, [i64::MIN, i64::MIN + 1, -7, -2, -1, 0, 1, 2, 7, i64::MAX - 1, i64::MAX]);
  check_div_rem!(u8, div_u8, rem_u8, [0, 1, 2, 7, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_div_rem!(u16, div_u16, rem_u16, [0, 1, 2, 7, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_div_rem!(u32, div_u32, rem_u32, [0, 1, 2, 7, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_div_rem!(u64, div_u64, rem_u64, [0, 1, 2, 7, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, 0xFFFF_FFFF_FFFF_FFFF]);

  // Right shifts. The signed cases include negative operands, where the sign
  // bit must be replicated rather than zero-filled.
  check_shr!(i8, shr_i8, [i8::MIN, -3, -1, 0, 1, 2, i8::MAX]);
  check_shr!(i16, shr_i16, [i16::MIN, -3, -1, 0, 1, 2, i16::MAX]);
  check_shr!(i32, shr_i32, [i32::MIN, -3, -1, 0, 1, 2, i32::MAX]);
  check_shr!(i64, shr_i64, [i64::MIN, -3, -1, 0, 1, 2, i64::MAX]);
  check_shr!(u8, shr_u8, [0, 1, 2, 0x7F, 0x80, 0xFF]);
  check_shr!(u16, shr_u16, [0, 1, 2, 0x7FFF, 0x8000, 0xFFFF]);
  check_shr!(u32, shr_u32, [0, 1, 2, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF]);
  check_shr!(u64, shr_u64, [0, 1, 2, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF]);

  // Constant shift amounts, logical and arithmetic.
  for a in [0u32, 1, 8, 0x8000_0000, u32::MAX] {
    assert_eq!(unsafe { shr3_u32(a) }, a >> 3, "shr3_u32({})", a);
  }
  for a in [i32::MIN, -8, -1, 0, 1, 8, i32::MAX] {
    assert_eq!(unsafe { shr3_i32(a) }, a >> 3, "shr3_i32({})", a);
  }

  check_xor!(i8, xor_i8, [i8::MIN, -1, 0, 1, 0x55, 0x7F]);
  check_xor!(i16, xor_i16, [i16::MIN, -1, 0, 1, 0x5555, i16::MAX]);
  check_xor!(i32, xor_i32, [i32::MIN, -1, 0, 1, 0x5555_5555, i32::MAX]);
  check_xor!(i64, xor_i64, [i64::MIN, -1, 0, 1, 0x5555_5555_5555_5555, i64::MAX]);
  check_xor!(u8, xor_u8, [0, 1, 0x0F, 0xF0, 0x55, 0xAA, 0xFF]);
  check_xor!(u16, xor_u16, [0, 1, 0x00FF, 0xFF00, 0x5555, 0xAAAA, 0xFFFF]);
  check_xor!(u32, xor_u32, [0, 1, 0x0000_FFFF, 0xFFFF_0000, 0x5555_5555, 0xAAAA_AAAA, 0xFFFF_FFFF]);
  check_xor!(u64, xor_u64, [0, 1, 0xFFFF_FFFF, 0xFFFF_FFFF_0000_0000, 0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA, 0xFFFF_FFFF_FFFF_FFFF]);

  // Bitwise not, which at the narrow widths must only invert the bits the
  // type has.
  check_not!(i8, not_i8, [i8::MIN, -1, 0, 1, 0x55, i8::MAX]);
  check_not!(i16, not_i16, [i16::MIN, -1, 0, 1, 0x5555, i16::MAX]);
  check_not!(i32, not_i32, [i32::MIN, -1, 0, 1, 0x5555_5555, i32::MAX]);
  check_not!(i64, not_i64, [i64::MIN, -1, 0, 1, 0x5555_5555_5555_5555, i64::MAX]);
  check_not!(u8, not_u8, [0, 1, 0x55, 0xAA, 0x7F, 0x80, 0xFF]);
  check_not!(u16, not_u16, [0, 1, 0x5555, 0xAAAA, 0x7FFF, 0x8000, 0xFFFF]);
  check_not!(u32, not_u32, [0, 1, 0x5555_5555, 0xAAAA_AAAA, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF]);
  check_not!(u64, not_u64, [0, 1, 0x5555_5555_5555_5555, 0xAAAA_AAAA_AAAA_AAAA, 0x8000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF]);

  // `not` on a bool is a logical negation of the low bit, not a bit flip of
  // the whole byte.
  assert_eq!(unsafe { not_bool(true) }, false, "not_bool(true)");
  assert_eq!(unsafe { not_bool(false) }, true, "not_bool(false)");

  // Wrapping negation, including `MIN`, which negates to itself.
  check_neg!(i8, neg_i8, [i8::MIN, i8::MIN + 1, -1, 0, 1, i8::MAX]);
  check_neg!(i16, neg_i16, [i16::MIN, i16::MIN + 1, -1, 0, 1, i16::MAX]);
  check_neg!(i32, neg_i32, [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX]);
  check_neg!(i64, neg_i64, [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX]);

  // Clearing the low `b` bits. `b` stops at 31, since `1 << 32` would be an
  // out-of-range shift.
  for a in [0u32, 1, 3, 0x5555_5555, 0xAAAA_AAAA, u32::MAX] {
    for b in 0..32u32 {
      assert_eq!(
        unsafe { clear_low_u32(a, b) },
        a & !(1u32.wrapping_shl(b).wrapping_sub(1)),
        "clear_low_u32({}, {})", a, b
      );
    }
  }

  // `(a / b) * b + (a % b)` reconstructs `a` exactly, whenever the division
  // itself is well defined.
  for a in [0u32, 1, 7, 0x1_0000, 0x8000_0000, u32::MAX] {
    for b in [1u32, 2, 7, 0x1_0000, 0x8000_0000, u32::MAX] {
      assert_eq!(unsafe { div_rem_u32(a, b) }, a, "div_rem_u32({}, {})", a, b);
    }
  }
  for a in [i32::MIN + 1, -7, -1, 0, 1, 7, i32::MAX] {
    for b in [-7i32, -2, -1, 1, 2, 7, i32::MAX] {
      assert_eq!(unsafe { div_rem_i32(a, b) }, a, "div_rem_i32({}, {})", a, b);
    }
  }

  // Shifting the low bits out and zeros back in.
  for a in [0u32, 1, 3, 0x5555_5555, 0xAAAA_AAAA, u32::MAX] {
    for b in 0..32u32 {
      assert_eq!(
        unsafe { shr_shl_u32(a, b) },
        (a >> b) << b,
        "shr_shl_u32({}, {})", a, b
      );
    }
  }

  // De Morgan's law: the two sides are equal, so their xor is zero.
  for a in [0u32, 1, 0x0000_FFFF, 0x5555_5555, 0xAAAA_AAAA, u32::MAX] {
    for b in [0u32, 1, 0xFFFF_0000, 0x5555_5555, 0xAAAA_AAAA, u32::MAX] {
      assert_eq!(unsafe { de_morgan_u32(a, b) }, 0, "de_morgan_u32({}, {})", a, b);
    }
  }

  // `a ^ b ^ b == a`.
  for a in [0u8, 1, 0x0F, 0x55, 0xAA, 0xFF] {
    for b in [0u8, 1, 0xF0, 0x55, 0xAA, 0xFF] {
      assert_eq!(unsafe { xor_roundtrip_u8(a, b) }, a, "xor_roundtrip_u8({}, {})", a, b);
    }
  }

  // Two's complement negation spelled out must match the `neg` instruction.
  for a in [i32::MIN, i32::MIN + 1, -7, -1, 0, 1, 7, i32::MAX] {
    assert_eq!(unsafe { neg_via_not_i32(a) }, a.wrapping_neg(), "neg_via_not_i32({})", a);
  }

  // The 128-bit types. Every value set below straddles the 64-bit word
  // boundary in at least one place, so an operation that only touches the low
  // word -- or that fails to carry into the high one -- produces a wrong
  // answer rather than a merely unusual one.
  check_add_sub!(i128, add_i128, sub_i128, [
    i128::MIN, i128::MIN + 1, -(1i128 << 64), -(1i128 << 64) + 1, -2, -1, 0, 1, 2,
    1i128 << 64, i128::MAX - 1, i128::MAX
  ]);
  check_add_sub!(u128, add_u128, sub_u128, [
    0, 1, 2, 0xFFFF_FFFF_FFFF_FFFF, 0x1_0000_0000_0000_0000,
    0x8000_0000_0000_0000_0000_0000_0000_0000, u128::MAX - 1, u128::MAX
  ]);

  check_mul!(i128, mul_i128, [
    i128::MIN, i128::MIN + 1, -(1i128 << 64), -3, -2, -1, 0, 1, 2, 3,
    1i128 << 64, 1i128 << 63, i128::MAX - 1, i128::MAX
  ]);
  check_mul!(u128, mul_u128, [
    0, 1, 2, 3, 1u128 << 63, 1u128 << 64, 0xFFFF_FFFF_FFFF_FFFF,
    0x8000_0000_0000_0000_0000_0000_0000_0000, u128::MAX - 1, u128::MAX
  ]);

  check_div_rem!(i128, div_i128, rem_i128, [
    i128::MIN, i128::MIN + 1, -(1i128 << 64), -7, -2, -1, 0, 1, 2, 7,
    1i128 << 64, i128::MAX - 1, i128::MAX
  ]);
  check_div_rem!(u128, div_u128, rem_u128, [
    0, 1, 2, 7, 0xFFFF_FFFF_FFFF_FFFF, 0x1_0000_0000_0000_0000,
    0x8000_0000_0000_0000_0000_0000_0000_0000, u128::MAX - 1, u128::MAX
  ]);

  // Shifts by every amount from 0 to 127, which includes the 64 that moves
  // one word wholesale into the other.
  check_shl!(i128, shl_i128, [i128::MIN, -1, 0, 1, 2, 1i128 << 64, i128::MAX]);
  check_shl!(u128, shl_u128, [
    0, 1, 2, 0xFFFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000_0000_0000_0000_0000, u128::MAX
  ]);
  check_shr!(i128, shr_i128, [i128::MIN, -3, -1, 0, 1, 2, 1i128 << 64, i128::MAX]);
  check_shr!(u128, shr_u128, [
    0, 1, 2, 0xFFFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000_0000_0000_0000_0000, u128::MAX
  ]);

  check_bitwise!(u128, and_u128, or_u128, [
    0, 1, 2, 0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000,
    0x5555_5555_5555_5555_5555_5555_5555_5555,
    0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA, u128::MAX
  ]);
  check_xor!(i128, xor_i128, [
    i128::MIN, -1, 0, 1, 0x5555_5555_5555_5555, 1i128 << 64, i128::MAX
  ]);
  check_xor!(u128, xor_u128, [
    0, 1, 0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000,
    0x5555_5555_5555_5555_5555_5555_5555_5555,
    0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA, u128::MAX
  ]);

  check_not!(i128, not_i128, [
    i128::MIN, -1, 0, 1, 0x5555_5555_5555_5555, 1i128 << 64, i128::MAX
  ]);
  check_not!(u128, not_u128, [
    0, 1, 0x5555_5555_5555_5555_5555_5555_5555_5555,
    0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA,
    0x8000_0000_0000_0000_0000_0000_0000_0000, u128::MAX
  ]);
  check_neg!(i128, neg_i128, [
    i128::MIN, i128::MIN + 1, -(1i128 << 64), -1, 0, 1, 1i128 << 64, i128::MAX
  ]);

  // Constant shift amounts, on both sides of the word boundary and exactly on
  // it.
  for a in [0u128, 1, 3, 1 << 63, 1 << 64, 0xFFFF_FFFF_FFFF_FFFF, u128::MAX] {
    assert_eq!(unsafe { shl3_u128(a) }, a.wrapping_shl(3), "shl3_u128({})", a);
    assert_eq!(unsafe { shl64_u128(a) }, a.wrapping_shl(64), "shl64_u128({})", a);
    assert_eq!(unsafe { shr64_u128(a) }, a.wrapping_shr(64), "shr64_u128({})", a);
  }
  for a in [i128::MIN, -(1i128 << 64), -7, -1, 0, 1, 1i128 << 64, i128::MAX] {
    assert_eq!(unsafe { shr100_i128(a) }, a >> 100, "shr100_i128({})", a);
  }

  // Chained arithmetic, with a carry that has to propagate out of the low
  // word twice in a row.
  assert_eq!(unsafe { add3_u128(1, 2, 3) }, 6);
  assert_eq!(unsafe { add3_u128(u128::MAX, 1, 1) }, 1);
  assert_eq!(unsafe { add3_u128(0xFFFF_FFFF_FFFF_FFFF, 1, 0) }, 1u128 << 64);
  assert_eq!(
    unsafe { add3_u128(1 << 127, 1 << 127, 7) },
    7
  );

  // (a + b) - (a - b) == 2 * b, wrapping.
  for a in [0u128, 1, 0xFFFF_FFFF_FFFF_FFFF, 1 << 64, 1 << 127, u128::MAX] {
    for b in [0u128, 1, 0xFFFF_FFFF_FFFF_FFFF, 1 << 64, 1 << 127, u128::MAX] {
      assert_eq!(
        unsafe { add_sub_u128(a, b) },
        b.wrapping_mul(2),
        "add_sub_u128({}, {})", a, b
      );
    }
  }

  // Four 128-bit arguments overflow the argument registers, so the tail
  // arrives on the stack and must not be dropped or swapped.
  assert_eq!(unsafe { add4_args_u128(1, 2, 3, 4) }, 10);
  assert_eq!(unsafe { add4_args_u128(0, 0, 0, u128::MAX) }, u128::MAX);
  assert_eq!(unsafe { add4_args_u128(0, 0, u128::MAX, 0) }, u128::MAX);
  assert_eq!(unsafe { add4_args_u128(u128::MAX, 1, 0, 0) }, 0);
  assert_eq!(
    unsafe { add4_args_u128(0xFFFF_FFFF_FFFF_FFFF, 1, 1 << 64, 0) },
    1u128 << 65
  );

  // A 128-bit value next to narrower ones in the same signature.
  assert_eq!(unsafe { mixed_width_u128(1, 2, 3, 4) }, 10);
  assert_eq!(
    unsafe { mixed_width_u128(u32::MAX, 0, u64::MAX, 0) },
    u32::MAX as u128 + u64::MAX as u128
  );
  assert_eq!(
    unsafe { mixed_width_u128(0, 1 << 127, 0, 1 << 127) },
    0
  );
  assert_eq!(
    unsafe { mixed_width_u128(1, u128::MAX, 0, 0) },
    0
  );

  for a in [0u128, 1, 3, 1 << 64, 1 << 127, u128::MAX] {
    for b in [0u128, 1, 3, 1 << 64, 1 << 127, u128::MAX] {
      for c in [0u128, 1, 7, u128::MAX] {
        assert_eq!(
          unsafe { mul_add_u128(a, b, c) },
          a.wrapping_mul(b).wrapping_add(c),
          "mul_add_u128({}, {}, {})", a, b, c
        );
      }
    }
  }

  // `(a / b) * b + (a % b)` reconstructs `a`.
  for a in [0u128, 1, 7, 1 << 64, 1 << 127, 0xFFFF_FFFF_FFFF_FFFF, u128::MAX] {
    for b in [1u128, 2, 7, 1 << 64, 0xFFFF_FFFF_FFFF_FFFF, u128::MAX] {
      assert_eq!(unsafe { div_rem_u128(a, b) }, a, "div_rem_u128({}, {})", a, b);
    }
  }
  for a in [i128::MIN + 1, -(1i128 << 64), -7, -1, 0, 1, 7, 1i128 << 64, i128::MAX] {
    for b in [-(1i128 << 64), -7i128, -2, -1, 1, 2, 7, 1i128 << 64, i128::MAX] {
      assert_eq!(unsafe { div_rem_i128(a, b) }, a, "div_rem_i128({}, {})", a, b);
    }
  }

  // Shifting the low bits out and zeros back in, across the word boundary.
  for a in [0u128, 1, 3, 0x5555_5555_5555_5555_5555_5555_5555_5555, 1 << 64, u128::MAX] {
    for b in 0..128u128 {
      assert_eq!(
        unsafe { shr_shl_u128(a, b) },
        (a >> b) << b,
        "shr_shl_u128({}, {})", a, b
      );
    }
  }

  // De Morgan's law at 128 bits: the two sides agree, so their xor is zero.
  for a in [0u128, 1, 0xFFFF_FFFF_FFFF_FFFF, 0x5555_5555_5555_5555_5555_5555_5555_5555, u128::MAX] {
    for b in [0u128, 1, 1 << 64, 0xAAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA_AAAA, u128::MAX] {
      assert_eq!(unsafe { de_morgan_u128(a, b) }, 0, "de_morgan_u128({}, {})", a, b);
    }
  }

  // Two's complement negation spelled out, where the `+ 1` may carry all the
  // way from the low word into the high one.
  for a in [i128::MIN, i128::MIN + 1, -(1i128 << 64), -7, -1, 0, 1, 7, 1i128 << 64, i128::MAX] {
    assert_eq!(unsafe { neg_via_not_i128(a) }, a.wrapping_neg(), "neg_via_not_i128({})", a);
  }

  // Widening into and truncating out of 128 bits.
  for a in [0u64, 1, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, u64::MAX] {
    assert_eq!(unsafe { zext_u64_u128(a) }, a as u128, "zext_u64_u128({})", a);
  }
  for a in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX] {
    assert_eq!(unsafe { sext_i64_i128(a) }, a as i128, "sext_i64_i128({})", a);
  }
  for a in [0u128, 1, 0xFFFF_FFFF_FFFF_FFFF, 1 << 64, 0x1234_5678_9ABC_DEF0_0FED_CBA9_8765_4321, u128::MAX] {
    assert_eq!(unsafe { trunc_u128_u64(a) }, a as u64, "trunc_u128_u64({})", a);
    assert_eq!(unsafe { trunc_u128_u8(a) }, a as u8, "trunc_u128_u8({})", a);
  }

  // Rotates and funnel shifts, including shift amounts of 0 and beyond the bit width.
  for a in [0u8, 1, 0x80, 0xA5, 0xFF] {
    for n in [0u32, 1, 3, 4, 7, 8, 9, 2 * 8 + 5] {
      assert_eq!(unsafe { rotl_u8(a, n) }, a.rotate_left(n), "rotl_u8({}, {})", a, n);
      assert_eq!(unsafe { rotr_u8(a, n) }, a.rotate_right(n), "rotr_u8({}, {})", a, n);
    }
    for b in [0u8, 1, 0x80, 0xA5, 0xFF] {
      for n in [0u32, 1, 3, 4, 7] {
        let wide_l = (((a as u128) << 8 | b as u128) << n >> 8) as u8;
        let wide_r = (((a as u128) << 8 | b as u128) >> n) as u8;
        assert_eq!(unsafe { funnel_shl_u8(a, b, n) }, wide_l, "funnel_shl_u8({}, {}, {})", a, b, n);
        assert_eq!(unsafe { funnel_shr_u8(a, b, n) }, wide_r, "funnel_shr_u8({}, {}, {})", a, b, n);
      }
    }
  }
  for a in [0u16, 1, 0x8000, 0xA5C3, 0xFFFF] {
    for n in [0u32, 1, 3, 8, 15, 16, 17, 2 * 16 + 5] {
      assert_eq!(unsafe { rotl_u16(a, n) }, a.rotate_left(n), "rotl_u16({}, {})", a, n);
      assert_eq!(unsafe { rotr_u16(a, n) }, a.rotate_right(n), "rotr_u16({}, {})", a, n);
    }
    for b in [0u16, 1, 0x8000, 0xA5C3, 0xFFFF] {
      for n in [0u32, 1, 3, 8, 15] {
        let wide_l = (((a as u128) << 16 | b as u128) << n >> 16) as u16;
        let wide_r = (((a as u128) << 16 | b as u128) >> n) as u16;
        assert_eq!(unsafe { funnel_shl_u16(a, b, n) }, wide_l, "funnel_shl_u16({}, {}, {})", a, b, n);
        assert_eq!(unsafe { funnel_shr_u16(a, b, n) }, wide_r, "funnel_shr_u16({}, {}, {})", a, b, n);
      }
    }
  }
  for a in [0u32, 1, 0x8000_0000, 0xA5C3_1E78, u32::MAX] {
    for n in [0u32, 1, 3, 16, 31, 32, 33, 2 * 32 + 5] {
      assert_eq!(unsafe { rotl_u32(a, n) }, a.rotate_left(n), "rotl_u32({}, {})", a, n);
      assert_eq!(unsafe { rotr_u32(a, n) }, a.rotate_right(n), "rotr_u32({}, {})", a, n);
    }
    for b in [0u32, 1, 0x8000_0000, 0xA5C3_1E78, u32::MAX] {
      for n in [0u32, 1, 3, 16, 31] {
        let wide_l = (((a as u128) << 32 | b as u128) << n >> 32) as u32;
        let wide_r = (((a as u128) << 32 | b as u128) >> n) as u32;
        assert_eq!(unsafe { funnel_shl_u32(a, b, n) }, wide_l, "funnel_shl_u32({}, {}, {})", a, b, n);
        assert_eq!(unsafe { funnel_shr_u32(a, b, n) }, wide_r, "funnel_shr_u32({}, {}, {})", a, b, n);
      }
    }
  }
  for a in [0u64, 1, 0x8000_0000_0000_0000, 0xA5C3_1E78_9B4D_0F26, u64::MAX] {
    for n in [0u32, 1, 3, 32, 63, 64, 65, 2 * 64 + 5] {
      assert_eq!(unsafe { rotl_u64(a, n) }, a.rotate_left(n), "rotl_u64({}, {})", a, n);
      assert_eq!(unsafe { rotr_u64(a, n) }, a.rotate_right(n), "rotr_u64({}, {})", a, n);
    }
    for b in [0u64, 1, 0x8000_0000_0000_0000, 0xA5C3_1E78_9B4D_0F26, u64::MAX] {
      for n in [0u32, 1, 3, 32, 63] {
        let wide_l = (((a as u128) << 64 | b as u128) << n >> 64) as u64;
        let wide_r = (((a as u128) << 64 | b as u128) >> n) as u64;
        assert_eq!(unsafe { funnel_shl_u64(a, b, n) }, wide_l, "funnel_shl_u64({}, {}, {})", a, b, n);
        assert_eq!(unsafe { funnel_shr_u64(a, b, n) }, wide_r, "funnel_shr_u64({}, {}, {})", a, b, n);
      }
    }
  }
}
