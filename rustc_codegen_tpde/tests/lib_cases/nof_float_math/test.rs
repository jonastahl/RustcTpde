
extern "Rust" {
  fn add_f32(a: f32, b: f32) -> f32;
  fn sub_f32(a: f32, b: f32) -> f32;
  fn mul_f32(a: f32, b: f32) -> f32;
  fn div_f32(a: f32, b: f32) -> f32;
  fn rem_f32(a: f32, b: f32) -> f32;
  fn add_f64(a: f64, b: f64) -> f64;
  fn sub_f64(a: f64, b: f64) -> f64;
  fn mul_f64(a: f64, b: f64) -> f64;
  fn div_f64(a: f64, b: f64) -> f64;
  fn rem_f64(a: f64, b: f64) -> f64;

  fn neg_f32(a: f32) -> f32;
  fn neg_f64(a: f64) -> f64;

  fn cmp_eq_f32(a: f32, b: f32) -> bool;
  fn cmp_ne_f32(a: f32, b: f32) -> bool;
  fn cmp_lt_f32(a: f32, b: f32) -> bool;
  fn cmp_le_f32(a: f32, b: f32) -> bool;
  fn cmp_gt_f32(a: f32, b: f32) -> bool;
  fn cmp_ge_f32(a: f32, b: f32) -> bool;

  fn cmp_eq_f64(a: f64, b: f64) -> bool;
  fn cmp_ne_f64(a: f64, b: f64) -> bool;
  fn cmp_lt_f64(a: f64, b: f64) -> bool;
  fn cmp_le_f64(a: f64, b: f64) -> bool;
  fn cmp_gt_f64(a: f64, b: f64) -> bool;
  fn cmp_ge_f64(a: f64, b: f64) -> bool;

  fn branch_eq_f32(a: f32, b: f32) -> i32;
  fn branch_ne_f32(a: f32, b: f32) -> i32;
  fn branch_lt_f32(a: f32, b: f32) -> i32;
  fn branch_le_f32(a: f32, b: f32) -> i32;
  fn branch_gt_f32(a: f32, b: f32) -> i32;
  fn branch_ge_f32(a: f32, b: f32) -> i32;

  fn branch_eq_f64(a: f64, b: f64) -> i32;
  fn branch_ne_f64(a: f64, b: f64) -> i32;
  fn branch_lt_f64(a: f64, b: f64) -> i32;
  fn branch_le_f64(a: f64, b: f64) -> i32;
  fn branch_gt_f64(a: f64, b: f64) -> i32;
  fn branch_ge_f64(a: f64, b: f64) -> i32;

  fn f32_to_f64(a: f32) -> f64;
  fn f64_to_f32(a: f64) -> f32;

  fn f32_to_i32(a: f32) -> i32;
  fn f32_to_i64(a: f32) -> i64;
  fn f32_to_u32(a: f32) -> u32;
  fn f32_to_u64(a: f32) -> u64;
  fn f64_to_i32(a: f64) -> i32;
  fn f64_to_i64(a: f64) -> i64;
  fn f64_to_u32(a: f64) -> u32;
  fn f64_to_u64(a: f64) -> u64;
  fn i32_to_f32(a: i32) -> f32;
  fn i32_to_f64(a: i32) -> f64;
  fn i64_to_f32(a: i64) -> f32;
  fn i64_to_f64(a: i64) -> f64;
  fn u32_to_f32(a: u32) -> f32;
  fn u32_to_f64(a: u32) -> f64;
  fn u64_to_f32(a: u64) -> f32;
  fn u64_to_f64(a: u64) -> f64;

  fn mul_add_f64(a: f64, b: f64, c: f64) -> f64;
  fn mul_add_f32(a: f32, b: f32, c: f32) -> f32;
  fn add_mul_f64(a: f64, b: f64, c: f64) -> f64;
  fn add3_f64(a: f64, b: f64, c: f64) -> f64;
  fn add_sub_f64(a: f64, b: f64) -> f64;
  fn add9_args_f64(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, i: f64) -> f64;
  fn mixed_args(a: f64, b: u64, c: f64, d: u64, e: f64, f: u64) -> f64;

  fn add_const_f32(a: f32) -> f32;
  fn sub_const_f32(a: f32) -> f32;
  fn mul_const_f32(a: f32) -> f32;
  fn div_const_f32(a: f32) -> f32;
  fn add_const_f64(a: f64) -> f64;
  fn sub_const_f64(a: f64) -> f64;
  fn mul_const_f64(a: f64) -> f64;
  fn div_const_f64(a: f64) -> f64;

  fn add_zero_f64(a: f64) -> f64;
  fn add_neg_zero_f64(a: f64) -> f64;
  fn mul_one_f64(a: f64) -> f64;
  fn mul_neg_one_f64(a: f64) -> f64;
  fn add_inf_f64(a: f64) -> f64;
  fn add_neg_inf_f64(a: f64) -> f64;
  fn mul_pi_f64(a: f64) -> f64;
  fn add_big_f64(a: f64) -> f64;
  fn const_sub_f64(a: f64) -> f64;
  fn const_div_f64(a: f64) -> f64;

  fn cmp_const_f64(a: f64) -> bool;
  fn branch_const_f64(a: f64) -> i32;
  fn ret_const_f32() -> f32;
  fn ret_const_f64() -> f64;
}

/// Compares two floats bit for bit, so that `-0.0` and `0.0` are treated as
/// different and NaN compares equal to itself. Every NaN counts as equal to
/// every other NaN: the operations here are free to pick any payload, and the
/// reference implementation and the backend need not pick the same one.
macro_rules! assert_float_eq {
  ($got:expr, $want:expr, $($fmt:tt)+) => {{
    let got = $got;
    let want = $want;
    // Ties the two to the same float type, so a bare literal on the `want`
    // side picks up the width of the value under test instead of defaulting.
    let [got, want] = [got, want];
    let same =if got.is_nan() || want.is_nan() {
      got.is_nan() && want.is_nan()
    } else {
      got.to_bits() == want.to_bits()
    };
    assert!(same, "{}: got {:?} ({:#x}), want {:?} ({:#x})",
      format_args!($($fmt)+), got, got.to_bits(), want, want.to_bits());
  }};
}

/// Checks the five arithmetic operators over every ordered pair drawn from
/// `$vals`, against the same expression evaluated by the host compiler.
macro_rules! check_arith {
  ($ty:ty, $add:ident, $sub:ident, $mul:ident, $div:ident, $rem:ident, $neg:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      assert_float_eq!(unsafe { $neg(a) }, -a, concat!(stringify!($neg), "({:?})"), a);
      for &b in vals {
        assert_float_eq!(unsafe { $add(a, b) }, a + b, concat!(stringify!($add), "({:?}, {:?})"), a, b);
        assert_float_eq!(unsafe { $sub(a, b) }, a - b, concat!(stringify!($sub), "({:?}, {:?})"), a, b);
        assert_float_eq!(unsafe { $mul(a, b) }, a * b, concat!(stringify!($mul), "({:?}, {:?})"), a, b);
        assert_float_eq!(unsafe { $div(a, b) }, a / b, concat!(stringify!($div), "({:?}, {:?})"), a, b);
        assert_float_eq!(unsafe { $rem(a, b) }, a % b, concat!(stringify!($rem), "({:?}, {:?})"), a, b);
      }
    }
  }};
}

/// Checks the six ordered comparisons over every ordered pair drawn from
/// `$vals`.
macro_rules! check_cmp {
  ($ty:ty, $eq:ident, $ne:ident, $lt:ident, $le:ident, $gt:ident, $ge:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        assert_eq!(unsafe { $eq(a, b) }, a == b, concat!(stringify!($eq), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $ne(a, b) }, a != b, concat!(stringify!($ne), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $lt(a, b) }, a < b, concat!(stringify!($lt), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $le(a, b) }, a <= b, concat!(stringify!($le), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $gt(a, b) }, a > b, concat!(stringify!($gt), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $ge(a, b) }, a >= b, concat!(stringify!($ge), "({:?}, {:?})"), a, b);
      }
    }
  }};
}

macro_rules! check_branch_cmp {
  ($ty:ty, $eq:ident, $ne:ident, $lt:ident, $le:ident, $gt:ident, $ge:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        assert_eq!(unsafe { $eq(a, b) }, if a == b { 10 } else { 20 }, concat!(stringify!($eq), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $ne(a, b) }, if a != b { 10 } else { 20 }, concat!(stringify!($ne), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $lt(a, b) }, if a < b { 10 } else { 20 }, concat!(stringify!($lt), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $le(a, b) }, if a <= b { 10 } else { 20 }, concat!(stringify!($le), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $gt(a, b) }, if a > b { 10 } else { 20 }, concat!(stringify!($gt), "({:?}, {:?})"), a, b);
        assert_eq!(unsafe { $ge(a, b) }, if a >= b { 10 } else { 20 }, concat!(stringify!($ge), "({:?}, {:?})"), a, b);
      }
    }
  }};
}

/// The interesting f32 values: both zeros, both infinities, a NaN, the
/// subnormal and normal extremes, and a few values that need every mantissa
/// bit.
const F32_VALS: [f32; 15] = [
  0.0,
  -0.0,
  1.0,
  -1.0,
  0.5,
  -2.5,
  3.0,
  f32::MIN_POSITIVE,
  f32::MIN_POSITIVE / 2.0, // subnormal
  f32::MAX,
  f32::MIN,
  f32::EPSILON,
  1.0 + f32::EPSILON,
  f32::INFINITY,
  f32::NEG_INFINITY,
];

/// The same set at double width, plus values that only round-trip in f64.
const F64_VALS: [f64; 17] = [
  0.0,
  -0.0,
  1.0,
  -1.0,
  0.5,
  -2.5,
  3.0,
  0.1,
  f64::MIN_POSITIVE,
  f64::MIN_POSITIVE / 2.0, // subnormal
  f64::MAX,
  f64::MIN,
  f64::EPSILON,
  1.0 + f64::EPSILON,
  9007199254740993.0, // 2^53 + 1, not representable: rounds down
  f64::INFINITY,
  f64::NEG_INFINITY,
];

fn main() {
  // NaN is kept out of the const arrays above so that each loop can decide
  // whether to include it; the arithmetic and comparison checks both do.
  let f32_vals: Vec<f32> = F32_VALS.iter().copied().chain([f32::NAN]).collect();
  let f64_vals: Vec<f64> = F64_VALS.iter().copied().chain([f64::NAN]).collect();

  check_arith!(f32, add_f32, sub_f32, mul_f32, div_f32, rem_f32, neg_f32, f32_vals);
  check_arith!(f64, add_f64, sub_f64, mul_f64, div_f64, rem_f64, neg_f64, f64_vals);

  check_cmp!(f32, cmp_eq_f32, cmp_ne_f32, cmp_lt_f32, cmp_le_f32, cmp_gt_f32, cmp_ge_f32, f32_vals);
  check_cmp!(f64, cmp_eq_f64, cmp_ne_f64, cmp_lt_f64, cmp_le_f64, cmp_gt_f64, cmp_ge_f64, f64_vals);

  check_branch_cmp!(f32, branch_eq_f32, branch_ne_f32, branch_lt_f32, branch_le_f32, branch_gt_f32, branch_ge_f32, f32_vals);
  check_branch_cmp!(f64, branch_eq_f64, branch_ne_f64, branch_lt_f64, branch_le_f64, branch_gt_f64, branch_ge_f64, f64_vals);

  // Widening is exact, so the round trip through f64 changes nothing.
  for &a in &f32_vals {
    assert_float_eq!(unsafe { f32_to_f64(a) }, a as f64, "f32_to_f64({:?})", a);
  }
  // Narrowing rounds to nearest-even, and overflows to infinity.
  for &a in &f64_vals {
    assert_float_eq!(unsafe { f64_to_f32(a) }, a as f32, "f64_to_f32({:?})", a);
  }

  // Float-to-int, including the saturating edges: values past the range clamp
  // rather than wrap, and NaN becomes 0.
  for &a in &f32_vals {
    assert_eq!(unsafe { f32_to_i32(a) }, a as i32, "f32_to_i32({:?})", a);
    assert_eq!(unsafe { f32_to_i64(a) }, a as i64, "f32_to_i64({:?})", a);
    assert_eq!(unsafe { f32_to_u32(a) }, a as u32, "f32_to_u32({:?})", a);
    assert_eq!(unsafe { f32_to_u64(a) }, a as u64, "f32_to_u64({:?})", a);
  }
  for &a in &f64_vals {
    assert_eq!(unsafe { f64_to_i32(a) }, a as i32, "f64_to_i32({:?})", a);
    assert_eq!(unsafe { f64_to_i64(a) }, a as i64, "f64_to_i64({:?})", a);
    assert_eq!(unsafe { f64_to_u32(a) }, a as u32, "f64_to_u32({:?})", a);
    assert_eq!(unsafe { f64_to_u64(a) }, a as u64, "f64_to_u64({:?})", a);
  }

  // Int-to-float. The 64-bit cases include values with more significant bits
  // than the mantissa can hold, so they have to round.
  for a in [i32::MIN, -1, 0, 1, 0x00FF_FFFF, 0x0100_0001, i32::MAX] {
    assert_float_eq!(unsafe { i32_to_f32(a) }, a as f32, "i32_to_f32({})", a);
    assert_float_eq!(unsafe { i32_to_f64(a) }, a as f64, "i32_to_f64({})", a);
  }
  for a in [i64::MIN, -1, 0, 1, (1 << 53) + 1, i64::MAX] {
    assert_float_eq!(unsafe { i64_to_f32(a) }, a as f32, "i64_to_f32({})", a);
    assert_float_eq!(unsafe { i64_to_f64(a) }, a as f64, "i64_to_f64({})", a);
  }
  for a in [0u32, 1, 0x00FF_FFFF, 0x0100_0001, 0x8000_0000, u32::MAX] {
    assert_float_eq!(unsafe { u32_to_f32(a) }, a as f32, "u32_to_f32({})", a);
    assert_float_eq!(unsafe { u32_to_f64(a) }, a as f64, "u32_to_f64({})", a);
  }
  for a in [0u64, 1, (1 << 53) + 1, 1 << 63, u64::MAX] {
    assert_float_eq!(unsafe { u64_to_f32(a) }, a as f32, "u64_to_f32({})", a);
    assert_float_eq!(unsafe { u64_to_f64(a) }, a as f64, "u64_to_f64({})", a);
  }

  // Chained arithmetic. The operand sets are chosen so that a fused
  // multiply-add, or any reassociation, would give a different answer than
  // the strict left-to-right evaluation Rust specifies.
  let triples_f64: [(f64, f64, f64); 8] = [
    (1.0, 2.0, 3.0),
    (0.1, 0.2, 0.3),
    (1.0 + f64::EPSILON, 1.0 + f64::EPSILON, -1.0),
    (f64::MAX, 2.0, f64::MIN),
    (f64::MAX, f64::MAX, f64::NEG_INFINITY),
    (1.0, f64::INFINITY, f64::NEG_INFINITY),
    (f64::NAN, 1.0, 1.0),
    (1e300, 1e300, -f64::INFINITY),
  ];
  for (a, b, c) in triples_f64 {
    assert_float_eq!(unsafe { mul_add_f64(a, b, c) }, a * b + c, "mul_add_f64({:?}, {:?}, {:?})", a, b, c);
    assert_float_eq!(unsafe { add_mul_f64(a, b, c) }, a + b * c, "add_mul_f64({:?}, {:?}, {:?})", a, b, c);
    assert_float_eq!(unsafe { add3_f64(a, b, c) }, a + b + c, "add3_f64({:?}, {:?}, {:?})", a, b, c);
  }
  let triples_f32: [(f32, f32, f32); 5] = [
    (1.0, 2.0, 3.0),
    (0.1, 0.2, 0.3),
    (1.0 + f32::EPSILON, 1.0 + f32::EPSILON, -1.0),
    (f32::MAX, 2.0, f32::MIN),
    (f32::NAN, 1.0, 1.0),
  ];
  for (a, b, c) in triples_f32 {
    assert_float_eq!(unsafe { mul_add_f32(a, b, c) }, a * b + c, "mul_add_f32({:?}, {:?}, {:?})", a, b, c);
  }

  for &a in &f64_vals {
    for &b in &f64_vals {
      assert_float_eq!(unsafe { add_sub_f64(a, b) }, (a + b) - (a - b), "add_sub_f64({:?}, {:?})", a, b);
    }
  }

  // Stack-passed float argument: the 9th must not be dropped or swapped.
  assert_float_eq!(unsafe { add9_args_f64(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0) }, 45.0, "add9_args_f64 sum");
  assert_float_eq!(unsafe { add9_args_f64(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0) }, 1.0, "add9_args_f64 last only");
  assert_float_eq!(unsafe { add9_args_f64(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0) }, 1.0, "add9_args_f64 eighth only");
  assert_float_eq!(
    unsafe { add9_args_f64(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, f64::NAN) },
    f64::NAN,
    "add9_args_f64 nan last"
  );

  // Floats and integers are assigned from separate register classes.
  assert_float_eq!(unsafe { mixed_args(1.0, 2, 3.0, 4, 5.0, 6) }, 9.0 * 12.0, "mixed_args");
  assert_float_eq!(unsafe { mixed_args(0.5, 1, 0.25, 0, 0.25, 0) }, 1.0, "mixed_args fractional");

  // Constant operands, checked over the same value sets as the two-argument
  // forms. Every expectation is the same expression evaluated by the host
  // compiler, so a constant that was materialized with the wrong bit pattern
  // shows up as a mismatch rather than as a plausible-looking answer.
  for &a in &f32_vals {
    assert_float_eq!(unsafe { add_const_f32(a) }, a + 1.5, "add_const_f32({:?})", a);
    assert_float_eq!(unsafe { sub_const_f32(a) }, a - 0.1, "sub_const_f32({:?})", a);
    assert_float_eq!(unsafe { mul_const_f32(a) }, a * 3.0, "mul_const_f32({:?})", a);
    assert_float_eq!(unsafe { div_const_f32(a) }, a / 7.0, "div_const_f32({:?})", a);
  }
  for &a in &f64_vals {
    assert_float_eq!(unsafe { add_const_f64(a) }, a + 1.5, "add_const_f64({:?})", a);
    assert_float_eq!(unsafe { sub_const_f64(a) }, a - 0.1, "sub_const_f64({:?})", a);
    assert_float_eq!(unsafe { mul_const_f64(a) }, a * 3.0, "mul_const_f64({:?})", a);
    assert_float_eq!(unsafe { div_const_f64(a) }, a / 7.0, "div_const_f64({:?})", a);

    // The constants that look like identities but are not: `+ 0.0` normalizes
    // `-0.0`, and `* -1.0` flips the sign of the zeros and infinities too.
    assert_float_eq!(unsafe { add_zero_f64(a) }, a + 0.0, "add_zero_f64({:?})", a);
    assert_float_eq!(unsafe { add_neg_zero_f64(a) }, a + -0.0, "add_neg_zero_f64({:?})", a);
    assert_float_eq!(unsafe { mul_one_f64(a) }, a * 1.0, "mul_one_f64({:?})", a);
    assert_float_eq!(unsafe { mul_neg_one_f64(a) }, a * -1.0, "mul_neg_one_f64({:?})", a);

    // Constants with awkward bit patterns.
    assert_float_eq!(unsafe { add_inf_f64(a) }, a + f64::INFINITY, "add_inf_f64({:?})", a);
    assert_float_eq!(unsafe { add_neg_inf_f64(a) }, a + f64::NEG_INFINITY, "add_neg_inf_f64({:?})", a);
    assert_float_eq!(unsafe { mul_pi_f64(a) }, a * 3.141592653589793, "mul_pi_f64({:?})", a);
    assert_float_eq!(unsafe { add_big_f64(a) }, a + 1.7976931348623157e308, "add_big_f64({:?})", a);

    // The constant on the left of a non-commutative operator.
    assert_float_eq!(unsafe { const_sub_f64(a) }, 2.5 - a, "const_sub_f64({:?})", a);
    assert_float_eq!(unsafe { const_div_f64(a) }, 1.0 / a, "const_div_f64({:?})", a);

    // Comparing against a constant, plain and fused with a branch.
    assert_eq!(unsafe { cmp_const_f64(a) }, a > 0.5, "cmp_const_f64({:?})", a);
    assert_eq!(
      unsafe { branch_const_f64(a) },
      if a < -1.25 { 10 } else { 20 },
      "branch_const_f64({:?})", a
    );
  }

  // A constant with no argument to hide behind.
  assert_float_eq!(unsafe { ret_const_f32() }, 0.15625f32, "ret_const_f32");
  assert_float_eq!(unsafe { ret_const_f64() }, -0.1f64, "ret_const_f64");
}

