// Drives the vector functions in `source.rs`. Every result is compared
// against a lane-by-lane scalar reference computed here, so the reference
// does not depend on the backend's vector lowering.

#![feature(portable_simd, simd_ffi)]

use std::fmt::Debug;
use std::simd::{f32x4, i32x4, u8x16};
use std::simd::{f32x16, f32x8, f64x4, f64x8, i16x16, i16x32, i32x16, i32x8, i64x4, i64x8, i8x32, i8x64, u8x64};

extern "Rust" {
  fn add_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn sub_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn mul_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn div_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn rem_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn div_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn rem_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn neg_i32x4(a: [i32; 4]) -> [i32; 4];
  fn add_u8x16(a: [u8; 16], b: [u8; 16]) -> [u8; 16];
  fn mul_i16x8(a: [i16; 8], b: [i16; 8]) -> [i16; 8];
  fn add_i64x2(a: [i64; 2], b: [i64; 2]) -> [i64; 2];
  fn mul_i64x2(a: [i64; 2], b: [i64; 2]) -> [i64; 2];
  fn saturating_add_u8x16(a: [u8; 16], b: [u8; 16]) -> [u8; 16];
  fn saturating_sub_i8x16(a: [i8; 16], b: [i8; 16]) -> [i8; 16];
  fn abs_i32x4(a: [i32; 4]) -> [i32; 4];

  fn and_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn or_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn xor_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn not_u32x4(a: [u32; 4]) -> [u32; 4];
  fn shl_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn shr_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn shr_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn shl_splat_u8x16(a: [u8; 16], s: u8) -> [u8; 16];

  fn add_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4];
  fn sub_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4];
  fn mul_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4];
  fn div_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4];
  fn neg_f32x4(a: [f32; 4]) -> [f32; 4];
  fn abs_f32x4(a: [f32; 4]) -> [f32; 4];
  fn add_f64x2(a: [f64; 2], b: [f64; 2]) -> [f64; 2];
  fn mul_f64x2(a: [f64; 2], b: [f64; 2]) -> [f64; 2];
  fn min_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4];
  fn max_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4];

  fn min_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn max_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn clamp_i32x4(a: [i32; 4], lo: i32, hi: i32) -> [i32; 4];

  fn eq_i32x4(a: [i32; 4], b: [i32; 4]) -> u64;
  fn ne_i32x4(a: [i32; 4], b: [i32; 4]) -> u64;
  fn lt_i32x4(a: [i32; 4], b: [i32; 4]) -> u64;
  fn lt_u32x4(a: [u32; 4], b: [u32; 4]) -> u64;
  fn ge_u8x16(a: [u8; 16], b: [u8; 16]) -> u64;
  fn le_f32x4(a: [f32; 4], b: [f32; 4]) -> u64;
  fn is_nan_f32x4(a: [f32; 4]) -> u64;
  fn select_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn select_bitmask_u32x4(m: u64, a: [u32; 4], b: [u32; 4]) -> [u32; 4];
  fn any_eq_u8x16(a: [u8; 16], needle: u8) -> bool;
  fn all_lt_i32x4(a: [i32; 4], bound: i32) -> bool;

  fn reduce_sum_i32x4(a: [i32; 4]) -> i32;
  fn reduce_product_i32x4(a: [i32; 4]) -> i32;
  fn reduce_sum_u8x16(a: [u8; 16]) -> u8;
  fn reduce_min_i32x4(a: [i32; 4]) -> i32;
  fn reduce_max_u32x4(a: [u32; 4]) -> u32;
  fn reduce_and_u32x4(a: [u32; 4]) -> u32;
  fn reduce_or_u32x4(a: [u32; 4]) -> u32;
  fn reduce_xor_u32x4(a: [u32; 4]) -> u32;
  fn reduce_sum_f32x4(a: [f32; 4]) -> f32;
  fn reduce_max_f32x4(a: [f32; 4]) -> f32;

  fn splat_i32x4(x: i32) -> [i32; 4];
  fn extract_i32x4(a: [i32; 4], i: usize) -> i32;
  fn insert_i32x4(a: [i32; 4], i: usize, x: i32) -> [i32; 4];
  fn reverse_i32x4(a: [i32; 4]) -> [i32; 4];
  fn rotate_left_u8x16(a: [u8; 16]) -> [u8; 16];
  fn swizzle_i32x4(a: [i32; 4]) -> [i32; 4];
  fn shuffle2_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4];
  fn interleave_i32x4(a: [i32; 4], b: [i32; 4]) -> ([i32; 4], [i32; 4]);
  fn widen_shuffle_i32x4(a: [i32; 4]) -> [i32; 8];
  fn swizzle_dyn_u8x16(a: [u8; 16], idx: [u8; 16]) -> [u8; 16];

  fn cast_i8x16_to_i16_low(a: [i8; 16]) -> [i16; 8];
  fn cast_u8x16_to_u32_low(a: [u8; 16]) -> [u32; 4];
  fn cast_i32x4_to_i8(a: [i32; 4]) -> [i8; 4];
  fn cast_i32x4_to_f32(a: [i32; 4]) -> [f32; 4];
  fn cast_f32x4_to_i32(a: [f32; 4]) -> [i32; 4];
  fn cast_f32x4_to_f64_low(a: [f32; 4]) -> [f64; 2];
  fn to_bits_f32x4(a: [f32; 4]) -> [u32; 4];

  fn load_or_default_i32x4(s: &[i32]) -> [i32; 4];
  fn gather_i32x4(s: &[i32], idx: [usize; 4]) -> [i32; 4];
  fn scatter_i32x4(s: &mut [i32], idx: [usize; 4], v: [i32; 4]);
  fn sum_slice_i32(s: &[i32]) -> i32;
  fn dot_f32(a: &[f32], b: &[f32]) -> f32;

  fn add_i32x4_by_value(a: i32x4, b: i32x4) -> i32x4;
  fn mul_f32x4_by_value(a: f32x4, b: f32x4) -> f32x4;
  fn add_u8x16_by_value(a: u8x16, b: u8x16) -> u8x16;
  fn by_value_calls(a: i32x4, b: i32x4) -> i32x4;
}

// Declarations shared by every wide integer vector type in `source.rs`.
macro_rules! wide_int_decls {
  ($v:ident, $t:ty, $n:literal, $add:ident, $sub:ident, $mul:ident, $div:ident, $neg:ident,
   $and:ident, $xor:ident, $shl:ident, $shr:ident, $min:ident, $lt:ident, $sel:ident,
   $rsum:ident, $rmax:ident, $rxor:ident, $rev:ident, $rot:ident, $splat:ident,
   $extract:ident, $insert:ident, $sum_slice:ident, $add_bv:ident) => {
    extern "Rust" {
      fn $add(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $sub(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $mul(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $div(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $neg(a: [$t; $n]) -> [$t; $n];
      fn $and(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $xor(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $shl(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $shr(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $min(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $lt(a: [$t; $n], b: [$t; $n]) -> u64;
      fn $sel(m: u64, a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $rsum(a: [$t; $n]) -> $t;
      fn $rmax(a: [$t; $n]) -> $t;
      fn $rxor(a: [$t; $n]) -> $t;
      fn $rev(a: [$t; $n]) -> [$t; $n];
      fn $rot(a: [$t; $n]) -> [$t; $n];
      fn $splat(x: $t) -> [$t; $n];
      fn $extract(a: [$t; $n], i: usize) -> $t;
      fn $insert(a: [$t; $n], i: usize, x: $t) -> [$t; $n];
      fn $sum_slice(s: &[$t]) -> $t;
      fn $add_bv(a: $v, b: $v) -> $v;
    }
  };
}

// Checks every operation of one wide integer type against a scalar reference.
macro_rules! wide_int_checks {
  ($f:expr, $v:ident, $t:ty, $n:literal, $add:ident, $sub:ident, $mul:ident, $div:ident, $neg:ident,
   $and:ident, $xor:ident, $shl:ident, $shr:ident, $min:ident, $lt:ident, $sel:ident,
   $rsum:ident, $rmax:ident, $rxor:ident, $rev:ident, $rot:ident, $splat:ident,
   $extract:ident, $insert:ident, $sum_slice:ident, $add_bv:ident) => {{
    let f: &mut Vec<String> = $f;
    let name = stringify!($v);
    let bits = <$t>::BITS;
    // Lane-distinct inputs covering both signs and the extremes.
    let inputs: [[$t; $n]; 3] = [
      std::array::from_fn(|i| (i as $t).wrapping_mul(3).wrapping_sub(5)),
      std::array::from_fn(|i| match i % 4 { 0 => <$t>::MIN, 1 => <$t>::MAX, 2 => -1, _ => (i as $t).wrapping_mul(0x35) }),
      std::array::from_fn(|i| wide_seed(i) as $t),
    ];
    for &a in &inputs {
      for &b in &inputs {
        check(f, &format!("add_{name}({a:?}, {b:?})"), unsafe { $add(a, b) }, zip(a, b, <$t>::wrapping_add));
        check(f, &format!("sub_{name}({a:?}, {b:?})"), unsafe { $sub(a, b) }, zip(a, b, <$t>::wrapping_sub));
        check(f, &format!("mul_{name}({a:?}, {b:?})"), unsafe { $mul(a, b) }, zip(a, b, <$t>::wrapping_mul));
        check(f, &format!("and_{name}({a:?}, {b:?})"), unsafe { $and(a, b) }, zip(a, b, |x, y| x & y));
        check(f, &format!("xor_{name}({a:?}, {b:?})"), unsafe { $xor(a, b) }, zip(a, b, |x, y| x ^ y));
        check(f, &format!("min_{name}({a:?}, {b:?})"), unsafe { $min(a, b) }, zip(a, b, <$t>::min));
        check(f, &format!("lt_{name}({a:?}, {b:?})"), unsafe { $lt(a, b) }, bitmask(zip(a, b, |x, y| x < y)));
      }
      check(f, &format!("neg_{name}({a:?})"), unsafe { $neg(a) }, a.map(<$t>::wrapping_neg));
      check(f, &format!("reduce_sum_{name}({a:?})"), unsafe { $rsum(a) }, a.iter().fold(0, |s: $t, &x| s.wrapping_add(x)));
      check(f, &format!("reduce_max_{name}({a:?})"), unsafe { $rmax(a) }, *a.iter().max().unwrap());
      check(f, &format!("reduce_xor_{name}({a:?})"), unsafe { $rxor(a) }, a.iter().fold(0, |s: $t, &x| s ^ x));
      check(f, &format!("reverse_{name}({a:?})"), unsafe { $rev(a) }, std::array::from_fn(|i| a[$n - 1 - i]));
      check(f, &format!("rotate_left_{name}({a:?})"), unsafe { $rot(a) }, std::array::from_fn(|i| a[(i + 3) % $n]));
      // Shift amounts cover 0, 1, the middle and the top bit.
      let sh: [$t; $n] = std::array::from_fn(|i| [0, 1, bits / 2, bits - 1][i % 4] as $t);
      check(f, &format!("shl_{name}({a:?})"), unsafe { $shl(a, sh) }, zip(a, sh, |x, s| x << s));
      check(f, &format!("shr_{name}({a:?})"), unsafe { $shr(a, sh) }, zip(a, sh, |x, s| x >> s));
      for i in [0, $n / 2 - 1, $n / 2, $n - 1] {
        check(f, &format!("extract_{name}({a:?}, {i})"), unsafe { $extract(a, i) }, a[i]);
        let mut want = a;
        want[i] = 42;
        check(f, &format!("insert_{name}({a:?}, {i})"), unsafe { $insert(a, i, 42) }, want);
      }
    }
    // Divisors are non-zero and never `MIN / -1`.
    let num: [$t; $n] = std::array::from_fn(|i| match i % 4 { 0 => <$t>::MIN, 1 => <$t>::MAX, 2 => -100, _ => i as $t });
    let den: [$t; $n] = std::array::from_fn(|i| [3, -7, 7, 2][i % 4]);
    check(f, &format!("div_{name}"), unsafe { $div(num, den) }, zip(num, den, |x, y| x / y));
    let m: u64 = 0xA5C3_0F96_5A3C_F069 & (u64::MAX >> (64 - $n));
    let (a, b) = (inputs[0], inputs[1]);
    check(f, &format!("select_bitmask_{name}"), unsafe { $sel(m, a, b) },
      std::array::from_fn(|i| if m >> i & 1 != 0 { a[i] } else { b[i] }));
    check(f, &format!("splat_{name}"), unsafe { $splat(-9) }, [-9; $n]);
    let data: Vec<$t> = (0..$n * 3 + 5).map(|x| (x as $t).wrapping_mul(7).wrapping_sub(20)).collect();
    for len in [0, 1, $n - 1, $n, $n + 1, 2 * $n, data.len()] {
      check(f, &format!("sum_slice_{name}(len {len})"), unsafe { $sum_slice(&data[..len]) },
        data[..len].iter().fold(0, |s: $t, &x| s.wrapping_add(x)));
    }
    check(f, &format!("add_{name}_by_value"), unsafe { $add_bv($v::from_array(a), $v::from_array(b)) }.to_array(),
      zip(a, b, <$t>::wrapping_add));
  }};
}

macro_rules! wide_float_decls {
  ($v:ident, $t:ty, $n:literal, $add:ident, $mul:ident, $div:ident, $neg:ident, $abs:ident,
   $min:ident, $le:ident, $is_nan:ident, $rsum:ident, $rmax:ident, $dot:ident, $mul_bv:ident) => {
    extern "Rust" {
      fn $add(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $mul(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $div(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $neg(a: [$t; $n]) -> [$t; $n];
      fn $abs(a: [$t; $n]) -> [$t; $n];
      fn $min(a: [$t; $n], b: [$t; $n]) -> [$t; $n];
      fn $le(a: [$t; $n], b: [$t; $n]) -> u64;
      fn $is_nan(a: [$t; $n]) -> u64;
      fn $rsum(a: [$t; $n]) -> $t;
      fn $rmax(a: [$t; $n]) -> $t;
      fn $dot(a: &[$t], b: &[$t]) -> $t;
      fn $mul_bv(a: $v, b: $v) -> $v;
    }
  };
}

macro_rules! wide_float_checks {
  ($f:expr, $check:ident, $v:ident, $t:ty, $n:literal, $add:ident, $mul:ident, $div:ident, $neg:ident, $abs:ident,
   $min:ident, $le:ident, $is_nan:ident, $rsum:ident, $rmax:ident, $dot:ident, $mul_bv:ident) => {{
    let f: &mut Vec<String> = $f;
    let name = stringify!($v);
    let specials: [$t; 8] = [1.5, -2.25, 0.0, -0.0, <$t>::INFINITY, <$t>::NAN, 1e-30, -7.0];
    let a: [$t; $n] = std::array::from_fn(|i| specials[i % 8] * (1 + i / 8) as $t);
    let b: [$t; $n] = std::array::from_fn(|i| specials[(i + 3) % 8] + i as $t);
    $check(f, &format!("add_{name}"), unsafe { $add(a, b) }, zip(a, b, |x, y| x + y));
    $check(f, &format!("mul_{name}"), unsafe { $mul(a, b) }, zip(a, b, |x, y| x * y));
    $check(f, &format!("div_{name}"), unsafe { $div(a, b) }, zip(a, b, |x, y| x / y));
    $check(f, &format!("neg_{name}"), unsafe { $neg(a) }, a.map(|x| -x));
    $check(f, &format!("abs_{name}"), unsafe { $abs(a) }, a.map(<$t>::abs));
    check(f, &format!("le_{name}"), unsafe { $le(a, b) }, bitmask(zip(a, b, |x, y| x <= y)));
    check(f, &format!("is_nan_{name}"), unsafe { $is_nan(a) }, bitmask(a.map(<$t>::is_nan)));
    // No NaN and no signed zeros here, so min / max / sum are exact and unambiguous.
    let c: [$t; $n] = std::array::from_fn(|i| ((i * 7 % 11) as $t) - 4.5);
    let d: [$t; $n] = std::array::from_fn(|i| ((i * 5 % 13) as $t) - 6.0);
    $check(f, &format!("min_{name}"), unsafe { $min(c, d) }, zip(c, d, <$t>::min));
    check(f, &format!("reduce_sum_{name}"), unsafe { $rsum(c) }.to_bits(), c.iter().sum::<$t>().to_bits());
    check(f, &format!("reduce_max_{name}"), unsafe { $rmax(c) }.to_bits(), c.iter().copied().fold(<$t>::NEG_INFINITY, <$t>::max).to_bits());
    let xa: Vec<$t> = (0..$n * 2 + 3).map(|x| x as $t).collect();
    let xb: Vec<$t> = (0..$n * 2 + 3).map(|x| (x % 3) as $t).collect();
    check(f, &format!("dot_{name}"), unsafe { $dot(&xa, &xb) }, xa.iter().zip(&xb).map(|(a, b)| a * b).sum());
    $check(f, &format!("mul_{name}_by_value"), unsafe { $mul_bv($v::from_array(c), $v::from_array(d)) }.to_array(),
      zip(c, d, |x, y| x * y));
  }};
}

wide_int_decls!(i32x8, i32, 8, add_i32x8, sub_i32x8, mul_i32x8, div_i32x8, neg_i32x8,
  and_i32x8, xor_i32x8, shl_i32x8, shr_i32x8, min_i32x8, lt_i32x8, select_bitmask_i32x8,
  reduce_sum_i32x8, reduce_max_i32x8, reduce_xor_i32x8, reverse_i32x8, rotate_left_i32x8,
  splat_i32x8, extract_i32x8, insert_i32x8, sum_slice_i32x8, add_i32x8_by_value);
wide_int_decls!(i32x16, i32, 16, add_i32x16, sub_i32x16, mul_i32x16, div_i32x16, neg_i32x16,
  and_i32x16, xor_i32x16, shl_i32x16, shr_i32x16, min_i32x16, lt_i32x16, select_bitmask_i32x16,
  reduce_sum_i32x16, reduce_max_i32x16, reduce_xor_i32x16, reverse_i32x16, rotate_left_i32x16,
  splat_i32x16, extract_i32x16, insert_i32x16, sum_slice_i32x16, add_i32x16_by_value);
wide_int_decls!(i64x4, i64, 4, add_i64x4, sub_i64x4, mul_i64x4, div_i64x4, neg_i64x4,
  and_i64x4, xor_i64x4, shl_i64x4, shr_i64x4, min_i64x4, lt_i64x4, select_bitmask_i64x4,
  reduce_sum_i64x4, reduce_max_i64x4, reduce_xor_i64x4, reverse_i64x4, rotate_left_i64x4,
  splat_i64x4, extract_i64x4, insert_i64x4, sum_slice_i64x4, add_i64x4_by_value);
wide_int_decls!(i64x8, i64, 8, add_i64x8, sub_i64x8, mul_i64x8, div_i64x8, neg_i64x8,
  and_i64x8, xor_i64x8, shl_i64x8, shr_i64x8, min_i64x8, lt_i64x8, select_bitmask_i64x8,
  reduce_sum_i64x8, reduce_max_i64x8, reduce_xor_i64x8, reverse_i64x8, rotate_left_i64x8,
  splat_i64x8, extract_i64x8, insert_i64x8, sum_slice_i64x8, add_i64x8_by_value);
wide_int_decls!(i16x16, i16, 16, add_i16x16, sub_i16x16, mul_i16x16, div_i16x16, neg_i16x16,
  and_i16x16, xor_i16x16, shl_i16x16, shr_i16x16, min_i16x16, lt_i16x16, select_bitmask_i16x16,
  reduce_sum_i16x16, reduce_max_i16x16, reduce_xor_i16x16, reverse_i16x16, rotate_left_i16x16,
  splat_i16x16, extract_i16x16, insert_i16x16, sum_slice_i16x16, add_i16x16_by_value);
wide_int_decls!(i16x32, i16, 32, add_i16x32, sub_i16x32, mul_i16x32, div_i16x32, neg_i16x32,
  and_i16x32, xor_i16x32, shl_i16x32, shr_i16x32, min_i16x32, lt_i16x32, select_bitmask_i16x32,
  reduce_sum_i16x32, reduce_max_i16x32, reduce_xor_i16x32, reverse_i16x32, rotate_left_i16x32,
  splat_i16x32, extract_i16x32, insert_i16x32, sum_slice_i16x32, add_i16x32_by_value);
wide_int_decls!(i8x32, i8, 32, add_i8x32, sub_i8x32, mul_i8x32, div_i8x32, neg_i8x32,
  and_i8x32, xor_i8x32, shl_i8x32, shr_i8x32, min_i8x32, lt_i8x32, select_bitmask_i8x32,
  reduce_sum_i8x32, reduce_max_i8x32, reduce_xor_i8x32, reverse_i8x32, rotate_left_i8x32,
  splat_i8x32, extract_i8x32, insert_i8x32, sum_slice_i8x32, add_i8x32_by_value);
wide_int_decls!(i8x64, i8, 64, add_i8x64, sub_i8x64, mul_i8x64, div_i8x64, neg_i8x64,
  and_i8x64, xor_i8x64, shl_i8x64, shr_i8x64, min_i8x64, lt_i8x64, select_bitmask_i8x64,
  reduce_sum_i8x64, reduce_max_i8x64, reduce_xor_i8x64, reverse_i8x64, rotate_left_i8x64,
  splat_i8x64, extract_i8x64, insert_i8x64, sum_slice_i8x64, add_i8x64_by_value);

wide_float_decls!(f32x8, f32, 8, add_f32x8, mul_f32x8, div_f32x8, neg_f32x8, abs_f32x8,
  min_f32x8, le_f32x8, is_nan_f32x8, reduce_sum_f32x8, reduce_max_f32x8, dot_f32x8, mul_f32x8_by_value);
wide_float_decls!(f32x16, f32, 16, add_f32x16, mul_f32x16, div_f32x16, neg_f32x16, abs_f32x16,
  min_f32x16, le_f32x16, is_nan_f32x16, reduce_sum_f32x16, reduce_max_f32x16, dot_f32x16, mul_f32x16_by_value);
wide_float_decls!(f64x4, f64, 4, add_f64x4, mul_f64x4, div_f64x4, neg_f64x4, abs_f64x4,
  min_f64x4, le_f64x4, is_nan_f64x4, reduce_sum_f64x4, reduce_max_f64x4, dot_f64x4, mul_f64x4_by_value);
wide_float_decls!(f64x8, f64, 8, add_f64x8, mul_f64x8, div_f64x8, neg_f64x8, abs_f64x8,
  min_f64x8, le_f64x8, is_nan_f64x8, reduce_sum_f64x8, reduce_max_f64x8, dot_f64x8, mul_f64x8_by_value);

extern "Rust" {
  fn div_u32x8(a: [u32; 8], b: [u32; 8]) -> [u32; 8];
  fn div_u32x16(a: [u32; 16], b: [u32; 16]) -> [u32; 16];
  fn shr_u32x8(a: [u32; 8], b: [u32; 8]) -> [u32; 8];
  fn shr_u32x16(a: [u32; 16], b: [u32; 16]) -> [u32; 16];
  fn lt_u32x8(a: [u32; 8], b: [u32; 8]) -> u64;
  fn lt_u32x16(a: [u32; 16], b: [u32; 16]) -> u64;
  fn max_u32x16(a: [u32; 16], b: [u32; 16]) -> [u32; 16];
  fn saturating_add_u8x32(a: [u8; 32], b: [u8; 32]) -> [u8; 32];
  fn saturating_add_u8x64(a: [u8; 64], b: [u8; 64]) -> [u8; 64];
  fn ge_u8x64(a: [u8; 64], b: [u8; 64]) -> u64;
  fn any_eq_u8x64(a: [u8; 64], needle: u8) -> bool;
  fn all_lt_u8x32(a: [u8; 32], bound: u8) -> bool;
  fn reduce_sum_u8x64(a: [u8; 64]) -> u8;
  fn swizzle_dyn_u8x32(a: [u8; 32], idx: [u8; 32]) -> [u8; 32];
  fn swizzle_dyn_u8x64(a: [u8; 64], idx: [u8; 64]) -> [u8; 64];

  fn swizzle_i32x8(a: [i32; 8]) -> [i32; 8];
  fn swizzle_i32x16(a: [i32; 16]) -> [i32; 16];
  fn shuffle2_i32x8(a: [i32; 8], b: [i32; 8]) -> [i32; 8];
  fn interleave_i32x8(a: [i32; 8], b: [i32; 8]) -> ([i32; 8], [i32; 8]);
  fn deinterleave_i32x16(a: [i32; 16], b: [i32; 16]) -> ([i32; 16], [i32; 16]);
  fn concat_i32x8(a: [i32; 8], b: [i32; 8]) -> [i32; 16];
  fn high_half_i32x16(a: [i32; 16]) -> [i32; 8];

  fn cast_i8x16_to_i16x16(a: [i8; 16]) -> [i16; 16];
  fn cast_u8x16_to_u32x16(a: [u8; 16]) -> [u32; 16];
  fn cast_i32x8_to_i64x8(a: [i32; 8]) -> [i64; 8];
  fn cast_i32x16_to_i8x16(a: [i32; 16]) -> [i8; 16];
  fn cast_i32x8_to_f32x8(a: [i32; 8]) -> [f32; 8];
  fn cast_f32x16_to_i32x16(a: [f32; 16]) -> [i32; 16];
  fn cast_f32x8_to_f64x8(a: [f32; 8]) -> [f64; 8];
  fn cast_f64x8_to_f32x8(a: [f64; 8]) -> [f32; 8];
  fn to_bits_f64x4(a: [f64; 4]) -> [u64; 4];

  fn load_or_default_i32x16(s: &[i32]) -> [i32; 16];
  fn gather_i32x8(s: &[i32], idx: [usize; 8]) -> [i32; 8];
  fn scatter_i32x16(s: &mut [i32], idx: [usize; 16], v: [i32; 16]);

  fn add_u8x64_by_value(a: u8x64, b: u8x64) -> u8x64;
  fn by_value_calls_x16(a: i32x16, b: i32x16) -> i32x16;
  fn store_zero_i64x2(out: &mut [i64; 2]);
  fn store_const_i32x4(out: &mut [i32; 4]);
  fn store_const_u8x16(out: &mut [u8; 16]);
}

/// Collects mismatches instead of asserting so one broken operation does not
/// hide the rest.
fn check<T: PartialEq + Debug>(failures: &mut Vec<String>, label: &str, got: T, want: T) {
  if got != want {
    failures.push(format!("{label}: got {got:?}, expected {want:?}"));
  }
}

/// Float arrays are compared bitwise so that NaN and -0.0 are checked too.
fn check_f32<const N: usize>(failures: &mut Vec<String>, label: &str, got: [f32; N], want: [f32; N]) {
  if got.map(f32::to_bits) != want.map(f32::to_bits) {
    failures.push(format!("{label}: got {got:?}, expected {want:?}"));
  }
}

fn check_f64<const N: usize>(failures: &mut Vec<String>, label: &str, got: [f64; N], want: [f64; N]) {
  if got.map(f64::to_bits) != want.map(f64::to_bits) {
    failures.push(format!("{label}: got {got:?}, expected {want:?}"));
  }
}

fn zip<A: Copy, B: Copy, R, const N: usize>(a: [A; N], b: [B; N], f: impl Fn(A, B) -> R) -> [R; N] {
  std::array::from_fn(|i| f(a[i], b[i]))
}

fn bitmask<const N: usize>(lanes: [bool; N]) -> u64 {
  lanes.iter().enumerate().fold(0, |m, (i, &b)| m | ((b as u64) << i))
}

fn bytes(start: u8, step: u8) -> [u8; 16] {
  std::array::from_fn(|i| start.wrapping_add(step.wrapping_mul(i as u8)))
}

/// Deterministic pseudo-random lane values.
fn wide_seed(i: usize) -> u64 {
  (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(17)
}

fn wide_bytes<const N: usize>(start: u8, step: u8) -> [u8; N] {
  std::array::from_fn(|i| start.wrapping_add(step.wrapping_mul(i as u8)))
}

fn main() {
  let f = &mut Vec::new();

  // Distinct values per lane so a lane mix-up is visible, plus the extremes.
  let i32_inputs: &[[i32; 4]] = &[
    [1, 2, 3, 4],
    [-1, -2, -3, -4],
    [i32::MIN, i32::MAX, 0, -1],
    [0x1234_5678, -0x0765_4321, 7, 1 << 30],
  ];
  let u32_inputs: &[[u32; 4]] = &[
    [1, 2, 3, 4],
    [0, u32::MAX, 0x8000_0000, 0x7FFF_FFFF],
    [0xDEAD_BEEF, 0x0F0F_0F0F, 0xF0F0_F0F0, 31],
  ];

  // ---- Integer arithmetic (wrapping, as without overflow checks on vectors) ----
  for &a in i32_inputs {
    for &b in i32_inputs {
      check(f, &format!("add_i32x4({a:?}, {b:?})"), unsafe { add_i32x4(a, b) }, zip(a, b, i32::wrapping_add));
      check(f, &format!("sub_i32x4({a:?}, {b:?})"), unsafe { sub_i32x4(a, b) }, zip(a, b, i32::wrapping_sub));
      check(f, &format!("mul_i32x4({a:?}, {b:?})"), unsafe { mul_i32x4(a, b) }, zip(a, b, i32::wrapping_mul));
      check(f, &format!("min_i32x4({a:?}, {b:?})"), unsafe { min_i32x4(a, b) }, zip(a, b, i32::min));
      check(f, &format!("select_i32x4({a:?}, {b:?})"), unsafe { select_i32x4(a, b) }, zip(a, b, |x, y| if x > y { x } else { y }));
      check(f, &format!("eq_i32x4({a:?}, {b:?})"), unsafe { eq_i32x4(a, b) }, bitmask(zip(a, b, |x, y| x == y)));
      check(f, &format!("ne_i32x4({a:?}, {b:?})"), unsafe { ne_i32x4(a, b) }, bitmask(zip(a, b, |x, y| x != y)));
      check(f, &format!("lt_i32x4({a:?}, {b:?})"), unsafe { lt_i32x4(a, b) }, bitmask(zip(a, b, |x, y| x < y)));
    }
    check(f, &format!("neg_i32x4({a:?})"), unsafe { neg_i32x4(a) }, a.map(i32::wrapping_neg));
    check(f, &format!("abs_i32x4({a:?})"), unsafe { abs_i32x4(a) }, a.map(i32::wrapping_abs));
    check(f, &format!("clamp_i32x4({a:?}, -3, 3)"), unsafe { clamp_i32x4(a, -3, 3) }, a.map(|x| x.clamp(-3, 3)));
  }
  for &a in u32_inputs {
    for &b in u32_inputs {
      check(f, &format!("and_u32x4({a:?}, {b:?})"), unsafe { and_u32x4(a, b) }, zip(a, b, |x, y| x & y));
      check(f, &format!("or_u32x4({a:?}, {b:?})"), unsafe { or_u32x4(a, b) }, zip(a, b, |x, y| x | y));
      check(f, &format!("xor_u32x4({a:?}, {b:?})"), unsafe { xor_u32x4(a, b) }, zip(a, b, |x, y| x ^ y));
      check(f, &format!("max_u32x4({a:?}, {b:?})"), unsafe { max_u32x4(a, b) }, zip(a, b, u32::max));
      check(f, &format!("lt_u32x4({a:?}, {b:?})"), unsafe { lt_u32x4(a, b) }, bitmask(zip(a, b, |x, y| x < y)));
    }
    check(f, &format!("not_u32x4({a:?})"), unsafe { not_u32x4(a) }, a.map(|x| !x));
  }

  // Division: divisors are non-zero and never `MIN / -1`.
  let num = [100, -100, i32::MIN, 7];
  let den = [7, 7, 3, -2];
  check(f, "div_i32x4", unsafe { div_i32x4(num, den) }, zip(num, den, |x, y| x / y));
  check(f, "rem_i32x4", unsafe { rem_i32x4(num, den) }, zip(num, den, |x, y| x % y));
  let unum = [100, u32::MAX, 0x8000_0000, 5];
  let uden = [7, 3, 0x8000_0001, 10];
  check(f, "div_u32x4", unsafe { div_u32x4(unum, uden) }, zip(unum, uden, |x, y| x / y));
  check(f, "rem_u32x4", unsafe { rem_u32x4(unum, uden) }, zip(unum, uden, |x, y| x % y));

  // Other element widths.
  let ba = bytes(0, 17);
  let bb = bytes(250, 3);
  check(f, "add_u8x16", unsafe { add_u8x16(ba, bb) }, zip(ba, bb, u8::wrapping_add));
  check(f, "saturating_add_u8x16", unsafe { saturating_add_u8x16(ba, bb) }, zip(ba, bb, u8::saturating_add));
  let sa = ba.map(|x| x as i8);
  let sb = bb.map(|x| x as i8);
  check(f, "saturating_sub_i8x16", unsafe { saturating_sub_i8x16(sa, sb) }, zip(sa, sb, i8::saturating_sub));
  check(f, "ge_u8x16", unsafe { ge_u8x16(ba, bb) }, bitmask(zip(ba, bb, |x, y| x >= y)));
  let ha: [i16; 8] = [1, -2, 300, i16::MAX, i16::MIN, 255, -1, 0x1234];
  let hb: [i16; 8] = [5, 7, 300, 2, -1, 256, -1, 0x10];
  check(f, "mul_i16x8", unsafe { mul_i16x8(ha, hb) }, zip(ha, hb, i16::wrapping_mul));
  let la = [i64::MAX, 1 << 40];
  let lb = [1, -(1 << 33)];
  check(f, "add_i64x2", unsafe { add_i64x2(la, lb) }, zip(la, lb, i64::wrapping_add));
  check(f, "mul_i64x2", unsafe { mul_i64x2(la, lb) }, zip(la, lb, i64::wrapping_mul));

  // ---- Shifts ----
  let sh = [0, 1, 16, 31];
  for &a in u32_inputs {
    check(f, &format!("shl_u32x4({a:?})"), unsafe { shl_u32x4(a, sh) }, zip(a, sh, |x, s| x << s));
    check(f, &format!("shr_u32x4({a:?})"), unsafe { shr_u32x4(a, sh) }, zip(a, sh, |x, s| x >> s));
  }
  let ish = [0, 1, 16, 31];
  for &a in i32_inputs {
    check(f, &format!("shr_i32x4({a:?})"), unsafe { shr_i32x4(a, ish) }, zip(a, ish, |x, s| x >> s));
  }
  for s in [0u8, 1, 4, 7] {
    check(f, &format!("shl_splat_u8x16(.., {s})"), unsafe { shl_splat_u8x16(ba, s) }, ba.map(|x| x << s));
  }

  // ---- Floats ----
  let fa = [1.5f32, -2.0, 0.0, f32::INFINITY];
  let fb = [0.25f32, 4.0, -0.0, 1.0];
  check_f32(f, "add_f32x4", unsafe { add_f32x4(fa, fb) }, zip(fa, fb, |x, y| x + y));
  check_f32(f, "sub_f32x4", unsafe { sub_f32x4(fa, fb) }, zip(fa, fb, |x, y| x - y));
  check_f32(f, "mul_f32x4", unsafe { mul_f32x4(fa, fb) }, zip(fa, fb, |x, y| x * y));
  check_f32(f, "div_f32x4", unsafe { div_f32x4(fa, fb) }, zip(fa, fb, |x, y| x / y));
  check_f32(f, "neg_f32x4", unsafe { neg_f32x4(fa) }, fa.map(|x| -x));
  check_f32(f, "abs_f32x4", unsafe { abs_f32x4([-1.0, -0.0, 3.0, f32::NEG_INFINITY]) }, [1.0, 0.0, 3.0, f32::INFINITY]);
  check_f32(f, "min_f32x4", unsafe { min_f32x4(fa, fb) }, [0.25, -2.0, 0.0f32.min(-0.0), 1.0]);
  check_f32(f, "max_f32x4", unsafe { max_f32x4(fa, fb) }, [1.5, 4.0, 0.0f32.max(-0.0), f32::INFINITY]);
  // simd_min/simd_max return the non-NaN operand like f32::min/max.
  check_f32(f, "min_f32x4 NaN", unsafe { min_f32x4([f32::NAN, 1.0, 2.0, 3.0], [5.0, f32::NAN, 1.0, 4.0]) }, [5.0, 1.0, 1.0, 3.0]);
  let da = [1.5f64, -1e300];
  let db = [2.25f64, 1e10];
  check_f64(f, "add_f64x2", unsafe { add_f64x2(da, db) }, zip(da, db, |x, y| x + y));
  check_f64(f, "mul_f64x2", unsafe { mul_f64x2(da, db) }, zip(da, db, |x, y| x * y));

  let fc = [1.0f32, f32::NAN, 3.0, -0.0];
  let fd = [1.0f32, 1.0, 2.0, 0.0];
  check(f, "le_f32x4", unsafe { le_f32x4(fc, fd) }, bitmask(zip(fc, fd, |x, y| x <= y)));
  check(f, "is_nan_f32x4", unsafe { is_nan_f32x4(fc) }, bitmask(fc.map(f32::is_nan)));

  // ---- Masks ----
  for m in 0..16u64 {
    let a = [10, 20, 30, 40];
    let b = [1, 2, 3, 4];
    let want = std::array::from_fn(|i| if m & (1 << i) != 0 { a[i] } else { b[i] });
    check(f, &format!("select_bitmask_u32x4({m:#b})"), unsafe { select_bitmask_u32x4(m, a, b) }, want);
  }
  check(f, "any_eq_u8x16 hit", unsafe { any_eq_u8x16(ba, ba[15]) }, true);
  check(f, "any_eq_u8x16 miss", unsafe { any_eq_u8x16([0; 16], 1) }, false);
  check(f, "all_lt_i32x4 true", unsafe { all_lt_i32x4([1, 2, 3, 4], 5) }, true);
  check(f, "all_lt_i32x4 false", unsafe { all_lt_i32x4([1, 2, 5, 4], 5) }, false);

  // ---- Reductions ----
  for &a in i32_inputs {
    check(f, &format!("reduce_sum_i32x4({a:?})"), unsafe { reduce_sum_i32x4(a) }, a.iter().fold(0i32, |s, &x| s.wrapping_add(x)));
    check(f, &format!("reduce_product_i32x4({a:?})"), unsafe { reduce_product_i32x4(a) }, a.iter().fold(1i32, |s, &x| s.wrapping_mul(x)));
    check(f, &format!("reduce_min_i32x4({a:?})"), unsafe { reduce_min_i32x4(a) }, *a.iter().min().unwrap());
  }
  for &a in u32_inputs {
    check(f, &format!("reduce_max_u32x4({a:?})"), unsafe { reduce_max_u32x4(a) }, *a.iter().max().unwrap());
    check(f, &format!("reduce_and_u32x4({a:?})"), unsafe { reduce_and_u32x4(a) }, a.iter().fold(!0, |s, &x| s & x));
    check(f, &format!("reduce_or_u32x4({a:?})"), unsafe { reduce_or_u32x4(a) }, a.iter().fold(0, |s, &x| s | x));
    check(f, &format!("reduce_xor_u32x4({a:?})"), unsafe { reduce_xor_u32x4(a) }, a.iter().fold(0, |s, &x| s ^ x));
  }
  check(f, "reduce_sum_u8x16", unsafe { reduce_sum_u8x16(ba) }, ba.iter().fold(0u8, |s, &x| s.wrapping_add(x)));
  // Exactly representable, so the summation order does not matter.
  check(f, "reduce_sum_f32x4", unsafe { reduce_sum_f32x4([1.0, 2.5, -0.5, 8.0]) }, 11.0);
  check(f, "reduce_max_f32x4", unsafe { reduce_max_f32x4([1.0, 9.5, -0.5, 8.0]) }, 9.5);

  // ---- Lanes and shuffles ----
  check(f, "splat_i32x4", unsafe { splat_i32x4(-7) }, [-7; 4]);
  let v = [10, 20, 30, 40];
  for i in 0..4 {
    check(f, &format!("extract_i32x4({i})"), unsafe { extract_i32x4(v, i) }, v[i]);
    let mut want = v;
    want[i] = 99;
    check(f, &format!("insert_i32x4({i})"), unsafe { insert_i32x4(v, i, 99) }, want);
  }
  check(f, "reverse_i32x4", unsafe { reverse_i32x4(v) }, [40, 30, 20, 10]);
  let mut rot = ba;
  rot.rotate_left(3);
  check(f, "rotate_left_u8x16", unsafe { rotate_left_u8x16(ba) }, rot);
  check(f, "swizzle_i32x4", unsafe { swizzle_i32x4(v) }, [30, 10, 40, 40]);
  let w = [1, 2, 3, 4];
  check(f, "shuffle2_i32x4", unsafe { shuffle2_i32x4(v, w) }, [10, 1, 20, 2]);
  check(f, "interleave_i32x4", unsafe { interleave_i32x4(v, w) }, ([10, 1, 20, 2], [30, 3, 40, 4]));
  check(f, "widen_shuffle_i32x4", unsafe { widen_shuffle_i32x4(v) }, [10, 20, 30, 40, 40, 30, 20, 10]);
  // Out-of-range indices yield 0.
  let idx: [u8; 16] = [15, 14, 0, 1, 16, 255, 3, 3, 8, 9, 10, 11, 12, 13, 2, 200];
  let want = idx.map(|i| if (i as usize) < 16 { ba[i as usize] } else { 0 });
  check(f, "swizzle_dyn_u8x16", unsafe { swizzle_dyn_u8x16(ba, idx) }, want);

  // ---- Casts ----
  let sb16: [i8; 16] = std::array::from_fn(|i| (i as i8 - 8) * 15);
  check(f, "cast_i8x16_to_i16_low", unsafe { cast_i8x16_to_i16_low(sb16) }, std::array::from_fn(|i| sb16[i] as i16));
  let ub16 = bytes(0xF0, 1);
  check(f, "cast_u8x16_to_u32_low", unsafe { cast_u8x16_to_u32_low(ub16) }, std::array::from_fn(|i| ub16[i] as u32));
  for &a in i32_inputs {
    check(f, &format!("cast_i32x4_to_i8({a:?})"), unsafe { cast_i32x4_to_i8(a) }, a.map(|x| x as i8));
    check_f32(f, &format!("cast_i32x4_to_f32({a:?})"), unsafe { cast_i32x4_to_f32(a) }, a.map(|x| x as f32));
  }
  let fs = [1.9f32, -1.9, f32::NAN, 1e20];
  check(f, "cast_f32x4_to_i32", unsafe { cast_f32x4_to_i32(fs) }, fs.map(|x| x as i32));
  check_f64(f, "cast_f32x4_to_f64_low", unsafe { cast_f32x4_to_f64_low([0.1, -3.5, 0.0, 0.0]) }, [0.1f32 as f64, -3.5]);
  check(f, "to_bits_f32x4", unsafe { to_bits_f32x4(fc) }, fc.map(f32::to_bits));

  // ---- Memory ----
  let data: Vec<i32> = (0..11).map(|x| x * 3 - 5).collect();
  check(f, "load_or_default_i32x4 full", unsafe { load_or_default_i32x4(&data) }, [data[0], data[1], data[2], data[3]]);
  check(f, "load_or_default_i32x4 short", unsafe { load_or_default_i32x4(&data[9..]) }, [data[9], data[10], 0, 0]);
  check(f, "gather_i32x4", unsafe { gather_i32x4(&data, [10, 0, 99, 5]) }, [data[10], data[0], -1, data[5]]);
  let mut buf = [0i32; 6];
  unsafe { scatter_i32x4(&mut buf, [5, 0, 100, 2], [1, 2, 3, 4]) };
  check(f, "scatter_i32x4", buf, [2, 0, 4, 0, 0, 1]);
  for n in [0, 1, 3, 4, 5, 8, 11] {
    check(f, &format!("sum_slice_i32(len {n})"), unsafe { sum_slice_i32(&data[..n]) }, data[..n].iter().sum());
  }
  let xa: Vec<f32> = (0..10).map(|x| x as f32).collect();
  let xb: Vec<f32> = (0..10).map(|x| (x % 3) as f32).collect();
  check(f, "dot_f32", unsafe { dot_f32(&xa, &xb) }, xa.iter().zip(&xb).map(|(a, b)| a * b).sum());

  // ---- By value ----
  let a = i32x4::from_array([1, -2, i32::MAX, 4]);
  let b = i32x4::from_array([10, 20, 1, -4]);
  check(f, "add_i32x4_by_value", unsafe { add_i32x4_by_value(a, b) }.to_array(), zip(a.to_array(), b.to_array(), i32::wrapping_add));
  check(f, "by_value_calls", unsafe { by_value_calls(a, b) }.to_array(), zip(a.to_array(), b.to_array(), |x, y| x.wrapping_add(y).wrapping_mul(2)));
  check_f32(f, "mul_f32x4_by_value", unsafe { mul_f32x4_by_value(f32x4::from_array(fa), f32x4::from_array(fb)) }.to_array(), zip(fa, fb, |x, y| x * y));
  check(f, "add_u8x16_by_value", unsafe { add_u8x16_by_value(u8x16::from_array(ba), u8x16::from_array(bb)) }.to_array(), zip(ba, bb, u8::wrapping_add));

  // ======== 256-bit and 512-bit vectors ========
  wide_int_checks!(f, i32x8, i32, 8, add_i32x8, sub_i32x8, mul_i32x8, div_i32x8, neg_i32x8,
    and_i32x8, xor_i32x8, shl_i32x8, shr_i32x8, min_i32x8, lt_i32x8, select_bitmask_i32x8,
    reduce_sum_i32x8, reduce_max_i32x8, reduce_xor_i32x8, reverse_i32x8, rotate_left_i32x8,
    splat_i32x8, extract_i32x8, insert_i32x8, sum_slice_i32x8, add_i32x8_by_value);
  wide_int_checks!(f, i32x16, i32, 16, add_i32x16, sub_i32x16, mul_i32x16, div_i32x16, neg_i32x16,
    and_i32x16, xor_i32x16, shl_i32x16, shr_i32x16, min_i32x16, lt_i32x16, select_bitmask_i32x16,
    reduce_sum_i32x16, reduce_max_i32x16, reduce_xor_i32x16, reverse_i32x16, rotate_left_i32x16,
    splat_i32x16, extract_i32x16, insert_i32x16, sum_slice_i32x16, add_i32x16_by_value);
  wide_int_checks!(f, i64x4, i64, 4, add_i64x4, sub_i64x4, mul_i64x4, div_i64x4, neg_i64x4,
    and_i64x4, xor_i64x4, shl_i64x4, shr_i64x4, min_i64x4, lt_i64x4, select_bitmask_i64x4,
    reduce_sum_i64x4, reduce_max_i64x4, reduce_xor_i64x4, reverse_i64x4, rotate_left_i64x4,
    splat_i64x4, extract_i64x4, insert_i64x4, sum_slice_i64x4, add_i64x4_by_value);
  wide_int_checks!(f, i64x8, i64, 8, add_i64x8, sub_i64x8, mul_i64x8, div_i64x8, neg_i64x8,
    and_i64x8, xor_i64x8, shl_i64x8, shr_i64x8, min_i64x8, lt_i64x8, select_bitmask_i64x8,
    reduce_sum_i64x8, reduce_max_i64x8, reduce_xor_i64x8, reverse_i64x8, rotate_left_i64x8,
    splat_i64x8, extract_i64x8, insert_i64x8, sum_slice_i64x8, add_i64x8_by_value);
  wide_int_checks!(f, i16x16, i16, 16, add_i16x16, sub_i16x16, mul_i16x16, div_i16x16, neg_i16x16,
    and_i16x16, xor_i16x16, shl_i16x16, shr_i16x16, min_i16x16, lt_i16x16, select_bitmask_i16x16,
    reduce_sum_i16x16, reduce_max_i16x16, reduce_xor_i16x16, reverse_i16x16, rotate_left_i16x16,
    splat_i16x16, extract_i16x16, insert_i16x16, sum_slice_i16x16, add_i16x16_by_value);
  wide_int_checks!(f, i16x32, i16, 32, add_i16x32, sub_i16x32, mul_i16x32, div_i16x32, neg_i16x32,
    and_i16x32, xor_i16x32, shl_i16x32, shr_i16x32, min_i16x32, lt_i16x32, select_bitmask_i16x32,
    reduce_sum_i16x32, reduce_max_i16x32, reduce_xor_i16x32, reverse_i16x32, rotate_left_i16x32,
    splat_i16x32, extract_i16x32, insert_i16x32, sum_slice_i16x32, add_i16x32_by_value);
  wide_int_checks!(f, i8x32, i8, 32, add_i8x32, sub_i8x32, mul_i8x32, div_i8x32, neg_i8x32,
    and_i8x32, xor_i8x32, shl_i8x32, shr_i8x32, min_i8x32, lt_i8x32, select_bitmask_i8x32,
    reduce_sum_i8x32, reduce_max_i8x32, reduce_xor_i8x32, reverse_i8x32, rotate_left_i8x32,
    splat_i8x32, extract_i8x32, insert_i8x32, sum_slice_i8x32, add_i8x32_by_value);
  wide_int_checks!(f, i8x64, i8, 64, add_i8x64, sub_i8x64, mul_i8x64, div_i8x64, neg_i8x64,
    and_i8x64, xor_i8x64, shl_i8x64, shr_i8x64, min_i8x64, lt_i8x64, select_bitmask_i8x64,
    reduce_sum_i8x64, reduce_max_i8x64, reduce_xor_i8x64, reverse_i8x64, rotate_left_i8x64,
    splat_i8x64, extract_i8x64, insert_i8x64, sum_slice_i8x64, add_i8x64_by_value);

  wide_float_checks!(f, check_f32, f32x8, f32, 8, add_f32x8, mul_f32x8, div_f32x8, neg_f32x8, abs_f32x8,
    min_f32x8, le_f32x8, is_nan_f32x8, reduce_sum_f32x8, reduce_max_f32x8, dot_f32x8, mul_f32x8_by_value);
  wide_float_checks!(f, check_f32, f32x16, f32, 16, add_f32x16, mul_f32x16, div_f32x16, neg_f32x16, abs_f32x16,
    min_f32x16, le_f32x16, is_nan_f32x16, reduce_sum_f32x16, reduce_max_f32x16, dot_f32x16, mul_f32x16_by_value);
  wide_float_checks!(f, check_f64, f64x4, f64, 4, add_f64x4, mul_f64x4, div_f64x4, neg_f64x4, abs_f64x4,
    min_f64x4, le_f64x4, is_nan_f64x4, reduce_sum_f64x4, reduce_max_f64x4, dot_f64x4, mul_f64x4_by_value);
  wide_float_checks!(f, check_f64, f64x8, f64, 8, add_f64x8, mul_f64x8, div_f64x8, neg_f64x8, abs_f64x8,
    min_f64x8, le_f64x8, is_nan_f64x8, reduce_sum_f64x8, reduce_max_f64x8, dot_f64x8, mul_f64x8_by_value);

  // ---- Wide unsigned ----
  let ua: [u32; 16] = std::array::from_fn(|i| wide_seed(i) as u32 | if i % 3 == 0 { 0x8000_0000 } else { 0 });
  let ub: [u32; 16] = std::array::from_fn(|i| (wide_seed(i + 40) as u32 >> (i % 5)).max(1));
  let ua8: [u32; 8] = std::array::from_fn(|i| ua[i]);
  let ub8: [u32; 8] = std::array::from_fn(|i| ub[i]);
  let ush: [u32; 16] = std::array::from_fn(|i| [0, 1, 16, 31][i % 4]);
  let ush8: [u32; 8] = std::array::from_fn(|i| ush[i]);
  check(f, "div_u32x8", unsafe { div_u32x8(ua8, ub8) }, zip(ua8, ub8, |x, y| x / y));
  check(f, "div_u32x16", unsafe { div_u32x16(ua, ub) }, zip(ua, ub, |x, y| x / y));
  check(f, "shr_u32x8", unsafe { shr_u32x8(ua8, ush8) }, zip(ua8, ush8, |x, s| x >> s));
  check(f, "shr_u32x16", unsafe { shr_u32x16(ua, ush) }, zip(ua, ush, |x, s| x >> s));
  check(f, "lt_u32x8", unsafe { lt_u32x8(ua8, ub8) }, bitmask(zip(ua8, ub8, |x, y| x < y)));
  check(f, "lt_u32x16", unsafe { lt_u32x16(ua, ub) }, bitmask(zip(ua, ub, |x, y| x < y)));
  check(f, "max_u32x16", unsafe { max_u32x16(ua, ub) }, zip(ua, ub, u32::max));

  let wa32: [u8; 32] = wide_bytes(0, 9);
  let wb32: [u8; 32] = wide_bytes(250, 5);
  let wa64: [u8; 64] = wide_bytes(3, 11);
  let wb64: [u8; 64] = wide_bytes(200, 7);
  check(f, "saturating_add_u8x32", unsafe { saturating_add_u8x32(wa32, wb32) }, zip(wa32, wb32, u8::saturating_add));
  check(f, "saturating_add_u8x64", unsafe { saturating_add_u8x64(wa64, wb64) }, zip(wa64, wb64, u8::saturating_add));
  check(f, "ge_u8x64", unsafe { ge_u8x64(wa64, wb64) }, bitmask(zip(wa64, wb64, |x, y| x >= y)));
  check(f, "ge_u8x64 all", unsafe { ge_u8x64(wa64, wa64) }, u64::MAX);
  // The needle only appears in the last lane.
  let mut hay = [1u8; 64];
  check(f, "any_eq_u8x64 absent", unsafe { any_eq_u8x64(hay, 7) }, false);
  hay[63] = 7;
  check(f, "any_eq_u8x64 last lane", unsafe { any_eq_u8x64(hay, 7) }, true);
  check(f, "all_lt_u8x32 true", unsafe { all_lt_u8x32([5; 32], 6) }, true);
  let mut low = [5u8; 32];
  low[31] = 6;
  check(f, "all_lt_u8x32 last lane", unsafe { all_lt_u8x32(low, 6) }, false);
  check(f, "reduce_sum_u8x64", unsafe { reduce_sum_u8x64(wa64) }, wa64.iter().fold(0, |s, &x| s.wrapping_add(x)));
  // Indices out of range give 0.
  let idx32: [u8; 32] = std::array::from_fn(|i| [31, 0, 16, 15, 40, 3, 255, 20][i % 8] ^ (i as u8 & 1));
  check(f, "swizzle_dyn_u8x32", unsafe { swizzle_dyn_u8x32(wa32, idx32) },
    idx32.map(|j| if (j as usize) < 32 { wa32[j as usize] } else { 0 }));
  let idx64: [u8; 64] = std::array::from_fn(|i| (wide_seed(i) % 80) as u8);
  check(f, "swizzle_dyn_u8x64", unsafe { swizzle_dyn_u8x64(wa64, idx64) },
    idx64.map(|j| if (j as usize) < 64 { wa64[j as usize] } else { 0 }));

  // ---- Wide shuffles ----
  let a8: [i32; 8] = std::array::from_fn(|i| i as i32 * 10 + 1);
  let b8: [i32; 8] = std::array::from_fn(|i| -(i as i32) * 10 - 2);
  let a16: [i32; 16] = std::array::from_fn(|i| i as i32 * 100 + 3);
  let b16: [i32; 16] = std::array::from_fn(|i| -(i as i32) * 100 - 4);
  check(f, "swizzle_i32x8", unsafe { swizzle_i32x8(a8) }, [7, 0, 5, 2, 3, 3, 6, 1].map(|i| a8[i]));
  check(f, "swizzle_i32x16", unsafe { swizzle_i32x16(a16) },
    [15, 0, 9, 4, 12, 3, 3, 7, 8, 1, 14, 2, 10, 6, 11, 5].map(|i| a16[i]));
  let ab8: Vec<i32> = a8.iter().chain(&b8).copied().collect();
  check(f, "shuffle2_i32x8", unsafe { shuffle2_i32x8(a8, b8) }, [0, 8, 7, 15, 3, 12, 4, 11].map(|i| ab8[i]));
  check(f, "interleave_i32x8", unsafe { interleave_i32x8(a8, b8) },
    (std::array::from_fn(|i| if i % 2 == 0 { a8[i / 2] } else { b8[i / 2] }),
     std::array::from_fn(|i| if i % 2 == 0 { a8[4 + i / 2] } else { b8[4 + i / 2] })));
  let ab16: Vec<i32> = a16.iter().chain(&b16).copied().collect();
  check(f, "deinterleave_i32x16", unsafe { deinterleave_i32x16(a16, b16) },
    (std::array::from_fn(|i| ab16[2 * i]), std::array::from_fn(|i| ab16[2 * i + 1])));
  check(f, "concat_i32x8", unsafe { concat_i32x8(a8, b8) }, std::array::from_fn(|i| ab8[i]));
  check(f, "high_half_i32x16", unsafe { high_half_i32x16(a16) }, std::array::from_fn(|i| a16[8 + i]));

  // ---- Wide casts ----
  let sb16: [i8; 16] = std::array::from_fn(|i| wide_seed(i) as i8);
  let ub16: [u8; 16] = sb16.map(|x| x as u8);
  let wi8: [i32; 8] = [i32::MIN, i32::MAX, -1, 0, 255, -129, 0x1234_5678, 7];
  let wi16: [i32; 16] = std::array::from_fn(|i| wide_seed(i) as i32);
  check(f, "cast_i8x16_to_i16x16", unsafe { cast_i8x16_to_i16x16(sb16) }, sb16.map(|x| x as i16));
  check(f, "cast_u8x16_to_u32x16", unsafe { cast_u8x16_to_u32x16(ub16) }, ub16.map(|x| x as u32));
  check(f, "cast_i32x8_to_i64x8", unsafe { cast_i32x8_to_i64x8(wi8) }, wi8.map(|x| x as i64));
  check(f, "cast_i32x16_to_i8x16", unsafe { cast_i32x16_to_i8x16(wi16) }, wi16.map(|x| x as i8));
  check_f32(f, "cast_i32x8_to_f32x8", unsafe { cast_i32x8_to_f32x8(wi8) }, wi8.map(|x| x as f32));
  let wf16: [f32; 16] = std::array::from_fn(|i| [1.9, -1.9, f32::NAN, 1e20, -1e20, f32::INFINITY, -0.5, 2147483520.0][i % 8] * (1 + i / 8) as f32);
  check(f, "cast_f32x16_to_i32x16", unsafe { cast_f32x16_to_i32x16(wf16) }, wf16.map(|x| x as i32));
  let wf8: [f32; 8] = std::array::from_fn(|i| wf16[i]);
  check_f64(f, "cast_f32x8_to_f64x8", unsafe { cast_f32x8_to_f64x8(wf8) }, wf8.map(|x| x as f64));
  let wd8: [f64; 8] = [0.1, -1e300, f64::NAN, 1e-50, -0.0, 3.5, f64::INFINITY, 16777217.0];
  check_f32(f, "cast_f64x8_to_f32x8", unsafe { cast_f64x8_to_f32x8(wd8) }, wd8.map(|x| x as f32));
  let wd4 = [0.1, -0.0, f64::NAN, -1e300];
  check(f, "to_bits_f64x4", unsafe { to_bits_f64x4(wd4) }, wd4.map(f64::to_bits));

  // ---- Wide memory ----
  let wdata: Vec<i32> = (0..21).map(|x| x * 3 - 5).collect();
  check(f, "load_or_default_i32x16 full", unsafe { load_or_default_i32x16(&wdata) }, std::array::from_fn(|i| wdata[i]));
  check(f, "load_or_default_i32x16 short", unsafe { load_or_default_i32x16(&wdata[11..]) },
    std::array::from_fn(|i| wdata.get(11 + i).copied().unwrap_or(0)));
  let gidx = [20, 0, 99, 5, 13, usize::MAX, 7, 8];
  check(f, "gather_i32x8", unsafe { gather_i32x8(&wdata, gidx) }, gidx.map(|i| wdata.get(i).copied().unwrap_or(-1)));
  let mut wbuf = [0i32; 12];
  let sidx: [usize; 16] = [11, 0, 100, 2, 3, 4, 50, 6, 7, 8, 9, 10, 1, 12, 5, 99];
  let sval: [i32; 16] = std::array::from_fn(|i| i as i32 + 1);
  unsafe { scatter_i32x16(&mut wbuf, sidx, sval) };
  let mut want = [0i32; 12];
  for (&i, &v) in sidx.iter().zip(&sval) {
    if i < want.len() {
      want[i] = v;
    }
  }
  check(f, "scatter_i32x16", wbuf, want);

  // ---- Wide by value ----
  check(f, "add_u8x64_by_value", unsafe { add_u8x64_by_value(u8x64::from_array(wa64), u8x64::from_array(wb64)) }.to_array(),
    zip(wa64, wb64, u8::wrapping_add));
  check(f, "by_value_calls_x16", unsafe { by_value_calls_x16(i32x16::from_array(wi16), i32x16::from_array(a16)) }.to_array(),
    zip(wi16, a16, |x, y| x.wrapping_add(y).wrapping_mul(2)));

  let mut zero = [-1i64; 2];
  unsafe { store_zero_i64x2(&mut zero) };
  check(f, "store_zero_i64x2", zero, [0, 0]);
  let mut c32 = [0i32; 4];
  unsafe { store_const_i32x4(&mut c32) };
  check(f, "store_const_i32x4", c32, [1, -2, 3, i32::MIN]);
  let mut c8 = [0u8; 16];
  unsafe { store_const_u8x16(&mut c8) };
  check(f, "store_const_u8x16", c8, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255]);

  if !f.is_empty() {
    for line in f.iter() {
      eprintln!("{line}");
    }
    panic!("{} simd failures", f.len());
  }
}
