#![feature(core_intrinsics, funnel_shifts)]
#![allow(internal_features)]

// Addition and subtraction for every integer width the backend supports,
// signed and unsigned. Compiled with `overflow-checks=no`, so each of these
// is a wrapping operation.

#[no_mangle]
pub fn add_i8(a: i8, b: i8) -> i8 {
    a + b
}

#[no_mangle]
pub fn sub_i8(a: i8, b: i8) -> i8 {
    a - b
}

#[no_mangle]
pub fn add_i16(a: i16, b: i16) -> i16 {
    a + b
}

#[no_mangle]
pub fn sub_i16(a: i16, b: i16) -> i16 {
    a - b
}

#[no_mangle]
pub fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub fn sub_i32(a: i32, b: i32) -> i32 {
    a - b
}

#[no_mangle]
pub fn add_i64(a: i64, b: i64) -> i64 {
    a + b
}

#[no_mangle]
pub fn sub_i64(a: i64, b: i64) -> i64 {
    a - b
}

#[no_mangle]
pub fn add_u8(a: u8, b: u8) -> u8 {
    a + b
}

#[no_mangle]
pub fn sub_u8(a: u8, b: u8) -> u8 {
    a - b
}

#[no_mangle]
pub fn add_u16(a: u16, b: u16) -> u16 {
    a + b
}

#[no_mangle]
pub fn sub_u16(a: u16, b: u16) -> u16 {
    a - b
}

#[no_mangle]
pub fn add_u32(a: u32, b: u32) -> u32 {
    a + b
}

#[no_mangle]
pub fn sub_u32(a: u32, b: u32) -> u32 {
    a - b
}

#[no_mangle]
pub fn add_u64(a: u64, b: u64) -> u64 {
    a + b
}

#[no_mangle]
pub fn sub_u64(a: u64, b: u64) -> u64 {
    a - b
}

// Chained arithmetic: the intermediate result needs a slot of its own.
#[no_mangle]
pub fn add3_u32(a: u32, b: u32, c: u32) -> u32 {
    a + b + c
}

// Mixed add/sub, with both arguments used twice. Equals `2 * b` for all inputs.
#[no_mangle]
pub fn add_sub_u32(a: u32, b: u32) -> u32 {
    (a + b) - (a - b)
}

// Same, at the narrowest width, where a lost truncation is most visible.
#[no_mangle]
pub fn add_sub_u8(a: u8, b: u8) -> u8 {
    (a + b) - (a - b)
}

// More arguments than the SysV C ABI has argument registers (6), so the last
// two arrive on the stack.
#[no_mangle]
pub fn add8_args_u32(
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
    g: u32,
    h: u32,
) -> u32 {
    a + b + c + d + e + f + g + h
}

// Multiplication, at every width. `mul` is the first operation here whose
// result depends on more than the low bits of one operand, so a missing
// truncation shows up as a wrong answer rather than just a wrong flag.
#[no_mangle]
pub fn mul_i8(a: i8, b: i8) -> i8 {
    a * b
}

#[no_mangle]
pub fn mul_i16(a: i16, b: i16) -> i16 {
    a * b
}

#[no_mangle]
pub fn mul_i32(a: i32, b: i32) -> i32 {
    a * b
}

#[no_mangle]
pub fn mul_i64(a: i64, b: i64) -> i64 {
    a * b
}

#[no_mangle]
pub fn mul_u8(a: u8, b: u8) -> u8 {
    a * b
}

#[no_mangle]
pub fn mul_u16(a: u16, b: u16) -> u16 {
    a * b
}

#[no_mangle]
pub fn mul_u32(a: u32, b: u32) -> u32 {
    a * b
}

#[no_mangle]
pub fn mul_u64(a: u64, b: u64) -> u64 {
    a * b
}

// Left shift. The shift amount has the same type as the operand, so no
// widening or narrowing of the RHS is needed; callers must keep it below the
// operand width, since `overflow-checks=no` turns an out-of-range shift into
// an unchecked one.
#[no_mangle]
pub fn shl_i8(a: i8, b: i8) -> i8 {
    a << b
}

#[no_mangle]
pub fn shl_i16(a: i16, b: i16) -> i16 {
    a << b
}

#[no_mangle]
pub fn shl_i32(a: i32, b: i32) -> i32 {
    a << b
}

#[no_mangle]
pub fn shl_i64(a: i64, b: i64) -> i64 {
    a << b
}

#[no_mangle]
pub fn shl_u8(a: u8, b: u8) -> u8 {
    a << b
}

#[no_mangle]
pub fn shl_u16(a: u16, b: u16) -> u16 {
    a << b
}

#[no_mangle]
pub fn shl_u32(a: u32, b: u32) -> u32 {
    a << b
}

#[no_mangle]
pub fn shl_u64(a: u64, b: u64) -> u64 {
    a << b
}

// A shift by a constant: the amount folds into the instruction encoding
// instead of arriving in a register.
#[no_mangle]
pub fn shl3_u32(a: u32) -> u32 {
    a << 3
}

// Bitwise and/or. Included at the narrow widths too, where the operands live
// in registers wider than the type and the high garbage must not leak out.
#[no_mangle]
pub fn and_u8(a: u8, b: u8) -> u8 {
    a & b
}

#[no_mangle]
pub fn and_u16(a: u16, b: u16) -> u16 {
    a & b
}

#[no_mangle]
pub fn and_u32(a: u32, b: u32) -> u32 {
    a & b
}

#[no_mangle]
pub fn and_u64(a: u64, b: u64) -> u64 {
    a & b
}

#[no_mangle]
pub fn or_u8(a: u8, b: u8) -> u8 {
    a | b
}

#[no_mangle]
pub fn or_u16(a: u16, b: u16) -> u16 {
    a | b
}

#[no_mangle]
pub fn or_u32(a: u32, b: u32) -> u32 {
    a | b
}

#[no_mangle]
pub fn or_u64(a: u64, b: u64) -> u64 {
    a | b
}

// Mixed-operator expressions, where the operand order and the width of each
// intermediate both matter.
#[no_mangle]
pub fn mul_add_u32(a: u32, b: u32, c: u32) -> u32 {
    a * b + c
}

// Precedence: `a + b * c`, not `(a + b) * c`.
#[no_mangle]
pub fn add_mul_u32(a: u32, b: u32, c: u32) -> u32 {
    a + b * c
}

// `a * 2^b`, computed the long way. Equals `mul` for every in-range `b`.
#[no_mangle]
pub fn shl_mul_u32(a: u32, b: u32) -> u32 {
    (a << b) - (a * (1u32 << b))
}

// Bit twiddling: keeps only the low `b` bits of `a`.
#[no_mangle]
pub fn keep_low_u32(a: u32, b: u32) -> u32 {
    a & ((1u32 << b) - 1)
}

// And at the narrowest width, where every intermediate must be truncated to 8
// bits before the next operation sees it.
#[no_mangle]
pub fn mul_add_u8(a: u8, b: u8, c: u8) -> u8 {
    a * b + c
}

// Division and remainder. Unlike the operations above these can trap, so the
// callers below never pass a zero divisor, and never pass `MIN / -1`.
#[no_mangle]
pub fn div_i8(a: i8, b: i8) -> i8 {
    a / b
}

#[no_mangle]
pub fn div_i16(a: i16, b: i16) -> i16 {
    a / b
}

#[no_mangle]
pub fn div_i32(a: i32, b: i32) -> i32 {
    a / b
}

#[no_mangle]
pub fn div_i64(a: i64, b: i64) -> i64 {
    a / b
}

#[no_mangle]
pub fn div_u8(a: u8, b: u8) -> u8 {
    a / b
}

#[no_mangle]
pub fn div_u16(a: u16, b: u16) -> u16 {
    a / b
}

#[no_mangle]
pub fn div_u32(a: u32, b: u32) -> u32 {
    a / b
}

#[no_mangle]
pub fn div_u64(a: u64, b: u64) -> u64 {
    a / b
}

#[no_mangle]
pub fn rem_i8(a: i8, b: i8) -> i8 {
    a % b
}

#[no_mangle]
pub fn rem_i16(a: i16, b: i16) -> i16 {
    a % b
}

#[no_mangle]
pub fn rem_i32(a: i32, b: i32) -> i32 {
    a % b
}

#[no_mangle]
pub fn rem_i64(a: i64, b: i64) -> i64 {
    a % b
}

#[no_mangle]
pub fn rem_u8(a: u8, b: u8) -> u8 {
    a % b
}

#[no_mangle]
pub fn rem_u16(a: u16, b: u16) -> u16 {
    a % b
}

#[no_mangle]
pub fn rem_u32(a: u32, b: u32) -> u32 {
    a % b
}

#[no_mangle]
pub fn rem_u64(a: u64, b: u64) -> u64 {
    a % b
}


// Right shift: logical for the unsigned types, arithmetic for the signed
// ones, so the sign bit has to be replicated for the latter. Same in-range
// requirement on the shift amount as `shl`.
#[no_mangle]
pub fn shr_i8(a: i8, b: i8) -> i8 {
    a >> b
}

#[no_mangle]
pub fn shr_i16(a: i16, b: i16) -> i16 {
    a >> b
}

#[no_mangle]
pub fn shr_i32(a: i32, b: i32) -> i32 {
    a >> b
}

#[no_mangle]
pub fn shr_i64(a: i64, b: i64) -> i64 {
    a >> b
}

#[no_mangle]
pub fn shr_u8(a: u8, b: u8) -> u8 {
    a >> b
}

#[no_mangle]
pub fn shr_u16(a: u16, b: u16) -> u16 {
    a >> b
}

#[no_mangle]
pub fn shr_u32(a: u32, b: u32) -> u32 {
    a >> b
}

#[no_mangle]
pub fn shr_u64(a: u64, b: u64) -> u64 {
    a >> b
}


// A right shift by a constant amount.
#[no_mangle]
pub fn shr3_u32(a: u32) -> u32 {
    a >> 3
}

#[no_mangle]
pub fn shr3_i32(a: i32) -> i32 {
    a >> 3
}


// Exclusive or, which unlike `and` and `or` cannot be answered from either
// operand alone.
#[no_mangle]
pub fn xor_i8(a: i8, b: i8) -> i8 {
    a ^ b
}

#[no_mangle]
pub fn xor_i16(a: i16, b: i16) -> i16 {
    a ^ b
}

#[no_mangle]
pub fn xor_i32(a: i32, b: i32) -> i32 {
    a ^ b
}

#[no_mangle]
pub fn xor_i64(a: i64, b: i64) -> i64 {
    a ^ b
}

#[no_mangle]
pub fn xor_u8(a: u8, b: u8) -> u8 {
    a ^ b
}

#[no_mangle]
pub fn xor_u16(a: u16, b: u16) -> u16 {
    a ^ b
}

#[no_mangle]
pub fn xor_u32(a: u32, b: u32) -> u32 {
    a ^ b
}

#[no_mangle]
pub fn xor_u64(a: u64, b: u64) -> u64 {
    a ^ b
}


// The two unary operators. `not` inverts every bit of the type -- including,
// at the narrow widths, only the bits the type actually has. `neg` is a
// wrapping negation here, so `MIN` maps to itself.
#[no_mangle]
pub fn not_i8(a: i8) -> i8 {
    !a
}

#[no_mangle]
pub fn not_i16(a: i16) -> i16 {
    !a
}

#[no_mangle]
pub fn not_i32(a: i32) -> i32 {
    !a
}

#[no_mangle]
pub fn not_i64(a: i64) -> i64 {
    !a
}

#[no_mangle]
pub fn not_u8(a: u8) -> u8 {
    !a
}

#[no_mangle]
pub fn not_u16(a: u16) -> u16 {
    !a
}

#[no_mangle]
pub fn not_u32(a: u32) -> u32 {
    !a
}

#[no_mangle]
pub fn not_u64(a: u64) -> u64 {
    !a
}

#[no_mangle]
pub fn neg_i8(a: i8) -> i8 {
    -a
}

#[no_mangle]
pub fn neg_i16(a: i16) -> i16 {
    -a
}

#[no_mangle]
pub fn neg_i32(a: i32) -> i32 {
    -a
}

#[no_mangle]
pub fn neg_i64(a: i64) -> i64 {
    -a
}

#[no_mangle]
pub fn not_bool(a: bool) -> bool {
    !a
}


// Bit twiddling that leans on `not`: clears the low `b` bits of `a`.
#[no_mangle]
pub fn clear_low_u32(a: u32, b: u32) -> u32 {
    a & !((1u32 << b) - 1)
}


// Identities that only hold if several operations are each correct at the
// full width of the type.
#[no_mangle]
pub fn div_rem_u32(a: u32, b: u32) -> u32 {
    (a / b) * b + (a % b)
}

#[no_mangle]
pub fn div_rem_i32(a: i32, b: i32) -> i32 {
    (a / b) * b + (a % b)
}

// `a` with its low `b` bits shifted out and back in as zeros.
#[no_mangle]
pub fn shr_shl_u32(a: u32, b: u32) -> u32 {
    (a >> b) << b
}

// De Morgan: `!(a & b)` and `!a | !b` must agree bit for bit.
#[no_mangle]
pub fn de_morgan_u32(a: u32, b: u32) -> u32 {
    !(a & b) ^ (!a | !b)
}

// `a ^ b ^ b` is `a` again, for every pair.
#[no_mangle]
pub fn xor_roundtrip_u8(a: u8, b: u8) -> u8 {
    a ^ b ^ b
}

// Wrapping negation via `!a + 1`, which must match the `neg` above.
#[no_mangle]
pub fn neg_via_not_i32(a: i32) -> i32 {
    !a + 1
}


// The 128-bit types. These are wider than a register, so every operation
// below is either an instruction pair over a low/high word -- where a carry
// or borrow has to travel between the two -- or a call into the compiler
// builtins. They are also passed and returned differently from the narrower
// types: an argument occupies two registers, and the return value comes back
// in a register pair.
#[no_mangle]
pub fn add_i128(a: i128, b: i128) -> i128 {
    a + b
}

#[no_mangle]
pub fn sub_i128(a: i128, b: i128) -> i128 {
    a - b
}

#[no_mangle]
pub fn add_u128(a: u128, b: u128) -> u128 {
    a + b
}

#[no_mangle]
pub fn sub_u128(a: u128, b: u128) -> u128 {
    a - b
}

#[no_mangle]
pub fn mul_i128(a: i128, b: i128) -> i128 {
    a * b
}

#[no_mangle]
pub fn mul_u128(a: u128, b: u128) -> u128 {
    a * b
}

#[no_mangle]
pub fn div_i128(a: i128, b: i128) -> i128 {
    a / b
}

#[no_mangle]
pub fn div_u128(a: u128, b: u128) -> u128 {
    a / b
}

#[no_mangle]
pub fn rem_i128(a: i128, b: i128) -> i128 {
    a % b
}

#[no_mangle]
pub fn rem_u128(a: u128, b: u128) -> u128 {
    a % b
}

#[no_mangle]
pub fn shl_i128(a: i128, b: i128) -> i128 {
    a << b
}

#[no_mangle]
pub fn shl_u128(a: u128, b: u128) -> u128 {
    a << b
}

#[no_mangle]
pub fn shr_i128(a: i128, b: i128) -> i128 {
    a >> b
}

#[no_mangle]
pub fn shr_u128(a: u128, b: u128) -> u128 {
    a >> b
}

#[no_mangle]
pub fn and_u128(a: u128, b: u128) -> u128 {
    a & b
}

#[no_mangle]
pub fn or_u128(a: u128, b: u128) -> u128 {
    a | b
}

#[no_mangle]
pub fn xor_i128(a: i128, b: i128) -> i128 {
    a ^ b
}

#[no_mangle]
pub fn xor_u128(a: u128, b: u128) -> u128 {
    a ^ b
}

#[no_mangle]
pub fn not_i128(a: i128) -> i128 {
    !a
}

#[no_mangle]
pub fn not_u128(a: u128) -> u128 {
    !a
}

#[no_mangle]
pub fn neg_i128(a: i128) -> i128 {
    -a
}

// Shifts by a constant. 64 crosses the word boundary exactly, so the result
// is a pure word swap plus a fill; 3 and 100 land on either side of it.
#[no_mangle]
pub fn shl3_u128(a: u128) -> u128 {
    a << 3
}

#[no_mangle]
pub fn shl64_u128(a: u128) -> u128 {
    a << 64
}

#[no_mangle]
pub fn shr64_u128(a: u128) -> u128 {
    a >> 64
}

#[no_mangle]
pub fn shr100_i128(a: i128) -> i128 {
    a >> 100
}

// Chained arithmetic at 128 bits: the intermediate needs a slot of its own,
// two registers wide.
#[no_mangle]
pub fn add3_u128(a: u128, b: u128, c: u128) -> u128 {
    a + b + c
}

// (a + b) - (a - b) == 2 * b, where both the carry and the borrow have to
// cross the word boundary correctly for the two to cancel.
#[no_mangle]
pub fn add_sub_u128(a: u128, b: u128) -> u128 {
    (a + b) - (a - b)
}

// Four 128-bit arguments is eight registers' worth, more than the six the
// SysV C ABI has, so the tail of this list arrives on the stack.
#[no_mangle]
pub fn add4_args_u128(a: u128, b: u128, c: u128, d: u128) -> u128 {
    a + b + c + d
}

// A mixed-width signature: the 128-bit values must not disturb the placement
// of the narrower ones around them.
#[no_mangle]
pub fn mixed_width_u128(a: u32, b: u128, c: u64, d: u128) -> u128 {
    a as u128 + b + c as u128 + d
}

#[no_mangle]
pub fn mul_add_u128(a: u128, b: u128, c: u128) -> u128 {
    a * b + c
}

// Identities, each of which only holds if every operation is correct across
// the full 128 bits rather than just the low word.
#[no_mangle]
pub fn div_rem_u128(a: u128, b: u128) -> u128 {
    (a / b) * b + (a % b)
}

#[no_mangle]
pub fn div_rem_i128(a: i128, b: i128) -> i128 {
    (a / b) * b + (a % b)
}

#[no_mangle]
pub fn shr_shl_u128(a: u128, b: u128) -> u128 {
    (a >> b) << b
}

#[no_mangle]
pub fn de_morgan_u128(a: u128, b: u128) -> u128 {
    !(a & b) ^ (!a | !b)
}

#[no_mangle]
pub fn neg_via_not_i128(a: i128) -> i128 {
    !a + 1
}

// Widening and narrowing across the 64/128 boundary: sign extension has to
// fill the whole high word, and a truncation has to drop it.
#[no_mangle]
pub fn zext_u64_u128(a: u64) -> u128 {
    a as u128
}

#[no_mangle]
pub fn sext_i64_i128(a: i64) -> i128 {
    a as i128
}

#[no_mangle]
pub fn trunc_u128_u64(a: u128) -> u64 {
    a as u64
}

#[no_mangle]
pub fn trunc_u128_u8(a: u128) -> u8 {
    a as u8
}

// Rotates and funnel shifts (the shift amount is taken modulo the bit width for
// rotates and must be < bit width for the unchecked funnel shifts).

#[no_mangle]
pub fn rotl_u8(a: u8, n: u32) -> u8 {
    core::intrinsics::rotate_left(a, n)
}

#[no_mangle]
pub fn rotr_u8(a: u8, n: u32) -> u8 {
    core::intrinsics::rotate_right(a, n)
}

#[no_mangle]
pub fn funnel_shl_u8(a: u8, b: u8, n: u32) -> u8 {
    unsafe { core::intrinsics::unchecked_funnel_shl(a, b, n) }
}

#[no_mangle]
pub fn funnel_shr_u8(a: u8, b: u8, n: u32) -> u8 {
    unsafe { core::intrinsics::unchecked_funnel_shr(a, b, n) }
}

#[no_mangle]
pub fn rotl_u16(a: u16, n: u32) -> u16 {
    core::intrinsics::rotate_left(a, n)
}

#[no_mangle]
pub fn rotr_u16(a: u16, n: u32) -> u16 {
    core::intrinsics::rotate_right(a, n)
}

#[no_mangle]
pub fn funnel_shl_u16(a: u16, b: u16, n: u32) -> u16 {
    unsafe { core::intrinsics::unchecked_funnel_shl(a, b, n) }
}

#[no_mangle]
pub fn funnel_shr_u16(a: u16, b: u16, n: u32) -> u16 {
    unsafe { core::intrinsics::unchecked_funnel_shr(a, b, n) }
}

#[no_mangle]
pub fn rotl_u32(a: u32, n: u32) -> u32 {
    core::intrinsics::rotate_left(a, n)
}

#[no_mangle]
pub fn rotr_u32(a: u32, n: u32) -> u32 {
    core::intrinsics::rotate_right(a, n)
}

#[no_mangle]
pub fn funnel_shl_u32(a: u32, b: u32, n: u32) -> u32 {
    unsafe { core::intrinsics::unchecked_funnel_shl(a, b, n) }
}

#[no_mangle]
pub fn funnel_shr_u32(a: u32, b: u32, n: u32) -> u32 {
    unsafe { core::intrinsics::unchecked_funnel_shr(a, b, n) }
}

#[no_mangle]
pub fn rotl_u64(a: u64, n: u32) -> u64 {
    core::intrinsics::rotate_left(a, n)
}

#[no_mangle]
pub fn rotr_u64(a: u64, n: u32) -> u64 {
    core::intrinsics::rotate_right(a, n)
}

#[no_mangle]
pub fn funnel_shl_u64(a: u64, b: u64, n: u32) -> u64 {
    unsafe { core::intrinsics::unchecked_funnel_shl(a, b, n) }
}

#[no_mangle]
pub fn funnel_shr_u64(a: u64, b: u64, n: u32) -> u64 {
    unsafe { core::intrinsics::unchecked_funnel_shr(a, b, n) }
}
