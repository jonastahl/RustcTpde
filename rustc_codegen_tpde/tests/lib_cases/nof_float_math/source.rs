// Floating-point arithmetic for the two widths the backend supports. Nothing
// here can overflow in the integer sense, so the `nof_` prefix only keeps this
// case off the overflow-check path -- the results are plain IEEE-754.

// The five arithmetic operators, at both widths.
#[no_mangle]
pub fn add_f32(a: f32, b: f32) -> f32 {
    a + b
}

#[no_mangle]
pub fn sub_f32(a: f32, b: f32) -> f32 {
    a - b
}

#[no_mangle]
pub fn mul_f32(a: f32, b: f32) -> f32 {
    a * b
}

#[no_mangle]
pub fn div_f32(a: f32, b: f32) -> f32 {
    a / b
}

#[no_mangle]
pub fn rem_f32(a: f32, b: f32) -> f32 {
    a % b
}

#[no_mangle]
pub fn add_f64(a: f64, b: f64) -> f64 {
    a + b
}

#[no_mangle]
pub fn sub_f64(a: f64, b: f64) -> f64 {
    a - b
}

#[no_mangle]
pub fn mul_f64(a: f64, b: f64) -> f64 {
    a * b
}

#[no_mangle]
pub fn div_f64(a: f64, b: f64) -> f64 {
    a / b
}

#[no_mangle]
pub fn rem_f64(a: f64, b: f64) -> f64 {
    a % b
}

// Negation, which flips the sign bit and so is defined on zeros and NaNs too.
#[no_mangle]
pub fn neg_f32(a: f32) -> f32 {
    -a
}

#[no_mangle]
pub fn neg_f64(a: f64) -> f64 {
    -a
}

// Comparisons. These are the ordered predicates, so every one of them is
// false when either operand is NaN -- including `le` and `ge`, which are not
// the negations of `gt` and `lt`.
#[no_mangle]
pub fn cmp_eq_f32(a: f32, b: f32) -> bool {
    a == b
}

#[no_mangle]
pub fn cmp_ne_f32(a: f32, b: f32) -> bool {
    a != b
}

#[no_mangle]
pub fn cmp_lt_f32(a: f32, b: f32) -> bool {
    a < b
}

#[no_mangle]
pub fn cmp_le_f32(a: f32, b: f32) -> bool {
    a <= b
}

#[no_mangle]
pub fn cmp_gt_f32(a: f32, b: f32) -> bool {
    a > b
}

#[no_mangle]
pub fn cmp_ge_f32(a: f32, b: f32) -> bool {
    a >= b
}

#[no_mangle]
pub fn cmp_eq_f64(a: f64, b: f64) -> bool {
    a == b
}

#[no_mangle]
pub fn cmp_ne_f64(a: f64, b: f64) -> bool {
    a != b
}

#[no_mangle]
pub fn cmp_lt_f64(a: f64, b: f64) -> bool {
    a < b
}

#[no_mangle]
pub fn cmp_le_f64(a: f64, b: f64) -> bool {
    a <= b
}

#[no_mangle]
pub fn cmp_gt_f64(a: f64, b: f64) -> bool {
    a > b
}

#[no_mangle]
pub fn cmp_ge_f64(a: f64, b: f64) -> bool {
    a >= b
}

// ==========================================
// Fused F32 Comparisons + Branches / Selects
// ==========================================

#[no_mangle]
pub fn branch_eq_f32(a: f32, b: f32) -> i32 {
    if a == b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_ne_f32(a: f32, b: f32) -> i32 {
    if a != b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_lt_f32(a: f32, b: f32) -> i32 {
    if a < b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_le_f32(a: f32, b: f32) -> i32 {
    if a <= b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_gt_f32(a: f32, b: f32) -> i32 {
    if a > b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_ge_f32(a: f32, b: f32) -> i32 {
    if a >= b { 10 } else { 20 }
}

// ==========================================
// Fused F64 Comparisons + Branches / Selects
// ==========================================

#[no_mangle]
pub fn branch_eq_f64(a: f64, b: f64) -> i32 {
    if a == b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_ne_f64(a: f64, b: f64) -> i32 {
    if a != b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_lt_f64(a: f64, b: f64) -> i32 {
    if a < b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_le_f64(a: f64, b: f64) -> i32 {
    if a <= b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_gt_f64(a: f64, b: f64) -> i32 {
    if a > b { 10 } else { 20 }
}

#[no_mangle]
pub fn branch_ge_f64(a: f64, b: f64) -> i32 {
    if a >= b { 10 } else { 20 }
}

// Casts between the two float widths. Widening is exact; narrowing rounds,
// and saturates to infinity once the value leaves the f32 range.
#[no_mangle]
pub fn f32_to_f64(a: f32) -> f64 {
    a as f64
}

#[no_mangle]
pub fn f64_to_f32(a: f64) -> f32 {
    a as f32
}

// Casts between floats and integers. Rust defines these as saturating: a
// value past the integer's range clamps to `MIN` or `MAX`, and NaN becomes 0.
// That is not what the bare hardware instruction does, so the backend has to
// emit the clamping itself.
#[no_mangle]
pub fn f32_to_i32(a: f32) -> i32 {
    a as i32
}

#[no_mangle]
pub fn f32_to_i64(a: f32) -> i64 {
    a as i64
}

#[no_mangle]
pub fn f32_to_u32(a: f32) -> u32 {
    a as u32
}

#[no_mangle]
pub fn f32_to_u64(a: f32) -> u64 {
    a as u64
}

#[no_mangle]
pub fn f64_to_i32(a: f64) -> i32 {
    a as i32
}

#[no_mangle]
pub fn f64_to_i64(a: f64) -> i64 {
    a as i64
}

#[no_mangle]
pub fn f64_to_u32(a: f64) -> u32 {
    a as u32
}

#[no_mangle]
pub fn f64_to_u64(a: f64) -> u64 {
    a as u64
}

#[no_mangle]
pub fn i32_to_f32(a: i32) -> f32 {
    a as f32
}

#[no_mangle]
pub fn i32_to_f64(a: i32) -> f64 {
    a as f64
}

#[no_mangle]
pub fn i64_to_f32(a: i64) -> f32 {
    a as f32
}

#[no_mangle]
pub fn i64_to_f64(a: i64) -> f64 {
    a as f64
}

#[no_mangle]
pub fn u32_to_f32(a: u32) -> f32 {
    a as f32
}

#[no_mangle]
pub fn u32_to_f64(a: u32) -> f64 {
    a as f64
}

#[no_mangle]
pub fn u64_to_f32(a: u64) -> f32 {
    a as f32
}

#[no_mangle]
pub fn u64_to_f64(a: u64) -> f64 {
    a as f64
}

// Chained arithmetic, where each intermediate has to round before the next
// operation sees it. `a * b + c` must NOT be contracted into a single fused
// multiply-add: fusing it would skip a rounding step and change the result.
#[no_mangle]
pub fn mul_add_f64(a: f64, b: f64, c: f64) -> f64 {
    a * b + c
}

#[no_mangle]
pub fn mul_add_f32(a: f32, b: f32, c: f32) -> f32 {
    a * b + c
}

// Operator precedence: `a + b * c`, not `(a + b) * c`.
#[no_mangle]
pub fn add_mul_f64(a: f64, b: f64, c: f64) -> f64 {
    a + b * c
}

// Left-associative, and not reassociated: for these operands the two
// groupings give different answers.
#[no_mangle]
pub fn add3_f64(a: f64, b: f64, c: f64) -> f64 {
    a + b + c
}

// Mixed add/sub with both arguments used twice. Unlike the integer case this
// is not exactly `2 * b`, since each half rounds separately.
#[no_mangle]
pub fn add_sub_f64(a: f64, b: f64) -> f64 {
    (a + b) - (a - b)
}

// More float arguments than the SysV C ABI has SSE argument registers (8), so
// the last one arrives on the stack.
#[no_mangle]
pub fn add9_args_f64(
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    e: f64,
    f: f64,
    g: f64,
    h: f64,
    i: f64,
) -> f64 {
    a + b + c + d + e + f + g + h + i
}

// Integer and float arguments are counted against separate register classes,
// so these all stay in registers despite there being ten of them.
#[no_mangle]
pub fn mixed_args(a: f64, b: u64, c: f64, d: u64, e: f64, f: u64) -> f64 {
    (a + c + e) * ((b + d + f) as f64)
}

// Constant operands. Unlike an integer constant, a float constant has no
// immediate form: it has to be materialized, either from a constant pool or
// by building the bit pattern in a general-purpose register and moving it
// across. The constants below cover the cases where a backend is most likely
// to take a shortcut -- 0.0 and 1.0, which have tempting special encodings,
// -0.0 and the infinities, whose bit patterns differ from the "obvious" ones,
// and a value that needs every mantissa bit.
#[no_mangle]
pub fn add_const_f32(a: f32) -> f32 {
    a + 1.5
}

#[no_mangle]
pub fn sub_const_f32(a: f32) -> f32 {
    a - 0.1
}

#[no_mangle]
pub fn mul_const_f32(a: f32) -> f32 {
    a * 3.0
}

#[no_mangle]
pub fn div_const_f32(a: f32) -> f32 {
    a / 7.0
}

#[no_mangle]
pub fn add_const_f64(a: f64) -> f64 {
    a + 1.5
}

#[no_mangle]
pub fn sub_const_f64(a: f64) -> f64 {
    a - 0.1
}

#[no_mangle]
pub fn mul_const_f64(a: f64) -> f64 {
    a * 3.0
}

#[no_mangle]
pub fn div_const_f64(a: f64) -> f64 {
    a / 7.0
}

// Adding zero is not the identity: it turns -0.0 into +0.0, so it must not be
// folded away.
#[no_mangle]
pub fn add_zero_f64(a: f64) -> f64 {
    a + 0.0
}

// Adding -0.0 *is* the identity on everything except -0.0 itself.
#[no_mangle]
pub fn add_neg_zero_f64(a: f64) -> f64 {
    a + -0.0
}

// Multiplying by 1.0 is the identity except on NaN payloads; by -1.0 it is a
// sign flip that, unlike `neg`, is a real multiply and so quiets a NaN.
#[no_mangle]
pub fn mul_one_f64(a: f64) -> f64 {
    a * 1.0
}

#[no_mangle]
pub fn mul_neg_one_f64(a: f64) -> f64 {
    a * -1.0
}

// Constants with awkward bit patterns: the infinities, and a value that only
// round-trips if the full 53-bit mantissa survives materialization.
#[no_mangle]
pub fn add_inf_f64(a: f64) -> f64 {
    a + f64::INFINITY
}

#[no_mangle]
pub fn add_neg_inf_f64(a: f64) -> f64 {
    a + f64::NEG_INFINITY
}

#[no_mangle]
pub fn mul_pi_f64(a: f64) -> f64 {
    a * 3.141592653589793
}

// A large constant, where a narrower materialization would lose bits.
#[no_mangle]
pub fn add_big_f64(a: f64) -> f64 {
    a + 1.7976931348623157e308
}

// A constant on the left of a non-commutative operator, so the operand order
// matters as well as the value.
#[no_mangle]
pub fn const_sub_f64(a: f64) -> f64 {
    2.5 - a
}

#[no_mangle]
pub fn const_div_f64(a: f64) -> f64 {
    1.0 / a
}

// Comparison against a constant, where the constant is one side of the fused
// compare-and-branch.
#[no_mangle]
pub fn cmp_const_f64(a: f64) -> bool {
    a > 0.5
}

#[no_mangle]
pub fn branch_const_f64(a: f64) -> i32 {
    if a < -1.25 { 10 } else { 20 }
}

// No argument at all: the whole body is the constant.
#[no_mangle]
pub fn ret_const_f32() -> f32 {
    0.15625
}

#[no_mangle]
pub fn ret_const_f64() -> f64 {
    -0.1
}
