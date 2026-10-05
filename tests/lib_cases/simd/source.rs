#![feature(portable_simd)]

use std::simd::cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd};
use std::simd::num::{SimdFloat, SimdInt, SimdUint};
use std::simd::{simd_swizzle, Select, f32x4, f64x2, i16x8, i32x4, i64x2, i8x16, u32x4, u8x16, Mask};
use std::simd::{f32x8, f64x4, i16x16, i32x8, i64x4, i8x32, u32x8, u8x32};
use std::simd::{f32x16, f64x8, i16x32, i32x16, i64x8, i8x64, u32x16, u8x64};

// Most functions take and return plain arrays so that the test does not
// depend on how vectors are passed across the Rust ABI; the vector work
// happens in between. The `*_by_value` functions at the end cover the ABI.

// ---- Integer arithmetic ----

#[no_mangle]
fn add_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    (i32x4::from_array(a) + i32x4::from_array(b)).to_array()
}

#[no_mangle]
fn sub_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    (i32x4::from_array(a) - i32x4::from_array(b)).to_array()
}

#[no_mangle]
fn mul_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    (i32x4::from_array(a) * i32x4::from_array(b)).to_array()
}

#[no_mangle]
fn div_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    (i32x4::from_array(a) / i32x4::from_array(b)).to_array()
}

#[no_mangle]
fn rem_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    (i32x4::from_array(a) % i32x4::from_array(b)).to_array()
}

#[no_mangle]
fn div_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    (u32x4::from_array(a) / u32x4::from_array(b)).to_array()
}

#[no_mangle]
fn rem_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    (u32x4::from_array(a) % u32x4::from_array(b)).to_array()
}

#[no_mangle]
fn neg_i32x4(a: [i32; 4]) -> [i32; 4] {
    (-i32x4::from_array(a)).to_array()
}

#[no_mangle]
fn add_u8x16(a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    (u8x16::from_array(a) + u8x16::from_array(b)).to_array()
}

#[no_mangle]
fn mul_i16x8(a: [i16; 8], b: [i16; 8]) -> [i16; 8] {
    (i16x8::from_array(a) * i16x8::from_array(b)).to_array()
}

#[no_mangle]
fn add_i64x2(a: [i64; 2], b: [i64; 2]) -> [i64; 2] {
    (i64x2::from_array(a) + i64x2::from_array(b)).to_array()
}

#[no_mangle]
fn mul_i64x2(a: [i64; 2], b: [i64; 2]) -> [i64; 2] {
    (i64x2::from_array(a) * i64x2::from_array(b)).to_array()
}

#[no_mangle]
fn saturating_add_u8x16(a: [u8; 16], b: [u8; 16]) -> [u8; 16] {
    u8x16::from_array(a).saturating_add(u8x16::from_array(b)).to_array()
}

#[no_mangle]
fn saturating_sub_i8x16(a: [i8; 16], b: [i8; 16]) -> [i8; 16] {
    i8x16::from_array(a).saturating_sub(i8x16::from_array(b)).to_array()
}

#[no_mangle]
fn abs_i32x4(a: [i32; 4]) -> [i32; 4] {
    i32x4::from_array(a).abs().to_array()
}

// ---- Bitwise and shifts ----

#[no_mangle]
fn and_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    (u32x4::from_array(a) & u32x4::from_array(b)).to_array()
}

#[no_mangle]
fn or_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    (u32x4::from_array(a) | u32x4::from_array(b)).to_array()
}

#[no_mangle]
fn xor_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    (u32x4::from_array(a) ^ u32x4::from_array(b)).to_array()
}

#[no_mangle]
fn not_u32x4(a: [u32; 4]) -> [u32; 4] {
    (!u32x4::from_array(a)).to_array()
}

// Per-lane shift amounts.
#[no_mangle]
fn shl_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    (u32x4::from_array(a) << u32x4::from_array(b)).to_array()
}

// Logical shift right.
#[no_mangle]
fn shr_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    (u32x4::from_array(a) >> u32x4::from_array(b)).to_array()
}

// Arithmetic shift right.
#[no_mangle]
fn shr_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    (i32x4::from_array(a) >> i32x4::from_array(b)).to_array()
}

// Shift by a scalar, splatted to all lanes.
#[no_mangle]
fn shl_splat_u8x16(a: [u8; 16], s: u8) -> [u8; 16] {
    (u8x16::from_array(a) << u8x16::splat(s)).to_array()
}

// ---- Float arithmetic ----

#[no_mangle]
fn add_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    (f32x4::from_array(a) + f32x4::from_array(b)).to_array()
}

#[no_mangle]
fn sub_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    (f32x4::from_array(a) - f32x4::from_array(b)).to_array()
}

#[no_mangle]
fn mul_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    (f32x4::from_array(a) * f32x4::from_array(b)).to_array()
}

#[no_mangle]
fn div_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    (f32x4::from_array(a) / f32x4::from_array(b)).to_array()
}

#[no_mangle]
fn neg_f32x4(a: [f32; 4]) -> [f32; 4] {
    (-f32x4::from_array(a)).to_array()
}

#[no_mangle]
fn abs_f32x4(a: [f32; 4]) -> [f32; 4] {
    f32x4::from_array(a).abs().to_array()
}

#[no_mangle]
fn add_f64x2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    (f64x2::from_array(a) + f64x2::from_array(b)).to_array()
}

#[no_mangle]
fn mul_f64x2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    (f64x2::from_array(a) * f64x2::from_array(b)).to_array()
}

#[no_mangle]
fn min_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    f32x4::from_array(a).simd_min(f32x4::from_array(b)).to_array()
}

#[no_mangle]
fn max_f32x4(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    f32x4::from_array(a).simd_max(f32x4::from_array(b)).to_array()
}

// ---- Integer min / max / clamp ----

#[no_mangle]
fn min_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    i32x4::from_array(a).simd_min(i32x4::from_array(b)).to_array()
}

#[no_mangle]
fn max_u32x4(a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    u32x4::from_array(a).simd_max(u32x4::from_array(b)).to_array()
}

#[no_mangle]
fn clamp_i32x4(a: [i32; 4], lo: i32, hi: i32) -> [i32; 4] {
    i32x4::from_array(a).simd_clamp(i32x4::splat(lo), i32x4::splat(hi)).to_array()
}

// ---- Comparisons and masks ----

// Masks are returned as a bitmask so the test can check every lane.
#[no_mangle]
fn eq_i32x4(a: [i32; 4], b: [i32; 4]) -> u64 {
    i32x4::from_array(a).simd_eq(i32x4::from_array(b)).to_bitmask()
}

#[no_mangle]
fn ne_i32x4(a: [i32; 4], b: [i32; 4]) -> u64 {
    i32x4::from_array(a).simd_ne(i32x4::from_array(b)).to_bitmask()
}

// Signed compare.
#[no_mangle]
fn lt_i32x4(a: [i32; 4], b: [i32; 4]) -> u64 {
    i32x4::from_array(a).simd_lt(i32x4::from_array(b)).to_bitmask()
}

// Unsigned compare: lanes with the top bit set must count as large.
#[no_mangle]
fn lt_u32x4(a: [u32; 4], b: [u32; 4]) -> u64 {
    u32x4::from_array(a).simd_lt(u32x4::from_array(b)).to_bitmask()
}

#[no_mangle]
fn ge_u8x16(a: [u8; 16], b: [u8; 16]) -> u64 {
    u8x16::from_array(a).simd_ge(u8x16::from_array(b)).to_bitmask()
}

// Ordered float compare: NaN lanes are false.
#[no_mangle]
fn le_f32x4(a: [f32; 4], b: [f32; 4]) -> u64 {
    f32x4::from_array(a).simd_le(f32x4::from_array(b)).to_bitmask()
}

#[no_mangle]
fn is_nan_f32x4(a: [f32; 4]) -> u64 {
    f32x4::from_array(a).is_nan().to_bitmask()
}

#[no_mangle]
fn select_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    let a = i32x4::from_array(a);
    let b = i32x4::from_array(b);
    a.simd_gt(b).select(a, b).to_array()
}

#[no_mangle]
fn select_bitmask_u32x4(m: u64, a: [u32; 4], b: [u32; 4]) -> [u32; 4] {
    Mask::<i32, 4>::from_bitmask(m)
        .select(u32x4::from_array(a), u32x4::from_array(b))
        .to_array()
}

#[no_mangle]
fn any_eq_u8x16(a: [u8; 16], needle: u8) -> bool {
    u8x16::from_array(a).simd_eq(u8x16::splat(needle)).any()
}

#[no_mangle]
fn all_lt_i32x4(a: [i32; 4], bound: i32) -> bool {
    i32x4::from_array(a).simd_lt(i32x4::splat(bound)).all()
}

// ---- Reductions ----

#[no_mangle]
fn reduce_sum_i32x4(a: [i32; 4]) -> i32 {
    i32x4::from_array(a).reduce_sum()
}

#[no_mangle]
fn reduce_product_i32x4(a: [i32; 4]) -> i32 {
    i32x4::from_array(a).reduce_product()
}

#[no_mangle]
fn reduce_sum_u8x16(a: [u8; 16]) -> u8 {
    u8x16::from_array(a).reduce_sum()
}

#[no_mangle]
fn reduce_min_i32x4(a: [i32; 4]) -> i32 {
    i32x4::from_array(a).reduce_min()
}

#[no_mangle]
fn reduce_max_u32x4(a: [u32; 4]) -> u32 {
    u32x4::from_array(a).reduce_max()
}

#[no_mangle]
fn reduce_and_u32x4(a: [u32; 4]) -> u32 {
    u32x4::from_array(a).reduce_and()
}

#[no_mangle]
fn reduce_or_u32x4(a: [u32; 4]) -> u32 {
    u32x4::from_array(a).reduce_or()
}

#[no_mangle]
fn reduce_xor_u32x4(a: [u32; 4]) -> u32 {
    u32x4::from_array(a).reduce_xor()
}

#[no_mangle]
fn reduce_sum_f32x4(a: [f32; 4]) -> f32 {
    f32x4::from_array(a).reduce_sum()
}

#[no_mangle]
fn reduce_max_f32x4(a: [f32; 4]) -> f32 {
    f32x4::from_array(a).reduce_max()
}

// ---- Construction, lane access and shuffles ----

#[no_mangle]
fn splat_i32x4(x: i32) -> [i32; 4] {
    i32x4::splat(x).to_array()
}

#[no_mangle]
fn extract_i32x4(a: [i32; 4], i: usize) -> i32 {
    i32x4::from_array(a)[i]
}

#[no_mangle]
fn insert_i32x4(a: [i32; 4], i: usize, x: i32) -> [i32; 4] {
    let mut v = i32x4::from_array(a);
    v[i] = x;
    v.to_array()
}

#[no_mangle]
fn reverse_i32x4(a: [i32; 4]) -> [i32; 4] {
    i32x4::from_array(a).reverse().to_array()
}

#[no_mangle]
fn rotate_left_u8x16(a: [u8; 16]) -> [u8; 16] {
    u8x16::from_array(a).rotate_elements_left::<3>().to_array()
}

#[no_mangle]
fn swizzle_i32x4(a: [i32; 4]) -> [i32; 4] {
    simd_swizzle!(i32x4::from_array(a), [2, 0, 3, 3]).to_array()
}

// Two-input shuffle: picks lanes from both vectors.
#[no_mangle]
fn shuffle2_i32x4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    simd_swizzle!(i32x4::from_array(a), i32x4::from_array(b), [0, 4, 1, 5]).to_array()
}

#[no_mangle]
fn interleave_i32x4(a: [i32; 4], b: [i32; 4]) -> ([i32; 4], [i32; 4]) {
    let (lo, hi) = i32x4::from_array(a).interleave(i32x4::from_array(b));
    (lo.to_array(), hi.to_array())
}

// Shuffle into a vector of a different length.
#[no_mangle]
fn widen_shuffle_i32x4(a: [i32; 4]) -> [i32; 8] {
    simd_swizzle!(i32x4::from_array(a), [0, 1, 2, 3, 3, 2, 1, 0]).to_array()
}

// Dynamic, index-vector driven shuffle.
#[no_mangle]
fn swizzle_dyn_u8x16(a: [u8; 16], idx: [u8; 16]) -> [u8; 16] {
    u8x16::from_array(a).swizzle_dyn(u8x16::from_array(idx)).to_array()
}

// ---- Casts ----

// Sign extension.
#[no_mangle]
fn cast_i8x16_to_i16_low(a: [i8; 16]) -> [i16; 8] {
    let v = i8x16::from_array(a);
    let low = simd_swizzle!(v, [0, 1, 2, 3, 4, 5, 6, 7]);
    low.cast::<i16>().to_array()
}

// Zero extension.
#[no_mangle]
fn cast_u8x16_to_u32_low(a: [u8; 16]) -> [u32; 4] {
    let v = u8x16::from_array(a);
    let low = simd_swizzle!(v, [0, 1, 2, 3]);
    low.cast::<u32>().to_array()
}

// Truncation.
#[no_mangle]
fn cast_i32x4_to_i8(a: [i32; 4]) -> [i8; 4] {
    i32x4::from_array(a).cast::<i8>().to_array()
}

#[no_mangle]
fn cast_i32x4_to_f32(a: [i32; 4]) -> [f32; 4] {
    i32x4::from_array(a).cast::<f32>().to_array()
}

// Saturating float -> int cast (`as` semantics).
#[no_mangle]
fn cast_f32x4_to_i32(a: [f32; 4]) -> [i32; 4] {
    f32x4::from_array(a).cast::<i32>().to_array()
}

#[no_mangle]
fn cast_f32x4_to_f64_low(a: [f32; 4]) -> [f64; 2] {
    simd_swizzle!(f32x4::from_array(a), [0, 1]).cast::<f64>().to_array()
}

#[no_mangle]
fn to_bits_f32x4(a: [f32; 4]) -> [u32; 4] {
    f32x4::from_array(a).to_bits().to_array()
}

// ---- Memory ----

#[no_mangle]
fn load_or_default_i32x4(s: &[i32]) -> [i32; 4] {
    i32x4::load_or_default(s).to_array()
}

#[no_mangle]
fn gather_i32x4(s: &[i32], idx: [usize; 4]) -> [i32; 4] {
    i32x4::gather_or(s, std::simd::usizex4::from_array(idx), i32x4::splat(-1)).to_array()
}

#[no_mangle]
fn scatter_i32x4(s: &mut [i32], idx: [usize; 4], v: [i32; 4]) {
    i32x4::from_array(v).scatter(s, std::simd::usizex4::from_array(idx));
}

// Sum of a slice with a vector accumulator and a scalar tail.
#[no_mangle]
fn sum_slice_i32(s: &[i32]) -> i32 {
    let (chunks, tail) = s.as_chunks::<4>();
    let mut acc = i32x4::splat(0);
    for c in chunks {
        acc += i32x4::from_array(*c);
    }
    acc.reduce_sum() + tail.iter().sum::<i32>()
}

// Dot product of two slices, accumulated in vectors.
#[no_mangle]
fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len()) / 4 * 4;
    let mut acc = f32x4::splat(0.0);
    let mut i = 0;
    while i < n {
        acc += f32x4::from_slice(&a[i..]) * f32x4::from_slice(&b[i..]);
        i += 4;
    }
    let mut sum = acc.reduce_sum();
    while i < a.len().min(b.len()) {
        sum += a[i] * b[i];
        i += 1;
    }
    sum
}

// ---- Vectors across the ABI ----

#[no_mangle]
fn add_i32x4_by_value(a: i32x4, b: i32x4) -> i32x4 {
    a + b
}

#[no_mangle]
fn mul_f32x4_by_value(a: f32x4, b: f32x4) -> f32x4 {
    a * b
}

#[no_mangle]
fn add_u8x16_by_value(a: u8x16, b: u8x16) -> u8x16 {
    a + b
}

#[no_mangle]
fn by_value_calls(a: i32x4, b: i32x4) -> i32x4 {
    let s = add_i32x4_by_value(a, b);
    add_i32x4_by_value(s, s)
}

// ======== 256-bit and 512-bit vectors ========
// Same operations as above on wider vectors. Without AVX / AVX-512 enabled
// these have to be split into several 128-bit registers.

macro_rules! wide_int_ops {
    ($v:ident, $t:ty, $n:literal, $add:ident, $sub:ident, $mul:ident, $div:ident, $neg:ident,
     $and:ident, $xor:ident, $shl:ident, $shr:ident, $min:ident, $lt:ident, $sel:ident,
     $rsum:ident, $rmax:ident, $rxor:ident, $rev:ident, $rot:ident, $splat:ident,
     $extract:ident, $insert:ident, $sum_slice:ident, $add_bv:ident) => {
        #[no_mangle]
        fn $add(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) + $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $sub(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) - $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $mul(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) * $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $div(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) / $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $neg(a: [$t; $n]) -> [$t; $n] {
            (-$v::from_array(a)).to_array()
        }
        #[no_mangle]
        fn $and(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) & $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $xor(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) ^ $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $shl(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) << $v::from_array(b)).to_array()
        }
        // Arithmetic shift right.
        #[no_mangle]
        fn $shr(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) >> $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $min(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            $v::from_array(a).simd_min($v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $lt(a: [$t; $n], b: [$t; $n]) -> u64 {
            $v::from_array(a).simd_lt($v::from_array(b)).to_bitmask()
        }
        #[no_mangle]
        fn $sel(m: u64, a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            Mask::<$t, $n>::from_bitmask(m)
                .select($v::from_array(a), $v::from_array(b))
                .to_array()
        }
        #[no_mangle]
        fn $rsum(a: [$t; $n]) -> $t {
            $v::from_array(a).reduce_sum()
        }
        #[no_mangle]
        fn $rmax(a: [$t; $n]) -> $t {
            $v::from_array(a).reduce_max()
        }
        #[no_mangle]
        fn $rxor(a: [$t; $n]) -> $t {
            $v::from_array(a).reduce_xor()
        }
        #[no_mangle]
        fn $rev(a: [$t; $n]) -> [$t; $n] {
            $v::from_array(a).reverse().to_array()
        }
        // Rotation crosses 128-bit boundaries.
        #[no_mangle]
        fn $rot(a: [$t; $n]) -> [$t; $n] {
            $v::from_array(a).rotate_elements_left::<3>().to_array()
        }
        #[no_mangle]
        fn $splat(x: $t) -> [$t; $n] {
            $v::splat(x).to_array()
        }
        #[no_mangle]
        fn $extract(a: [$t; $n], i: usize) -> $t {
            $v::from_array(a)[i]
        }
        #[no_mangle]
        fn $insert(a: [$t; $n], i: usize, x: $t) -> [$t; $n] {
            let mut v = $v::from_array(a);
            v[i] = x;
            v.to_array()
        }
        #[no_mangle]
        fn $sum_slice(s: &[$t]) -> $t {
            let (chunks, tail) = s.as_chunks::<$n>();
            let mut acc = $v::splat(0);
            for c in chunks {
                acc += $v::from_array(*c);
            }
            acc.reduce_sum().wrapping_add(tail.iter().fold(0, |a: $t, &b| a.wrapping_add(b)))
        }
        #[no_mangle]
        fn $add_bv(a: $v, b: $v) -> $v {
            a + b
        }
    };
}

wide_int_ops!(i32x8, i32, 8, add_i32x8, sub_i32x8, mul_i32x8, div_i32x8, neg_i32x8,
    and_i32x8, xor_i32x8, shl_i32x8, shr_i32x8, min_i32x8, lt_i32x8, select_bitmask_i32x8,
    reduce_sum_i32x8, reduce_max_i32x8, reduce_xor_i32x8, reverse_i32x8, rotate_left_i32x8,
    splat_i32x8, extract_i32x8, insert_i32x8, sum_slice_i32x8, add_i32x8_by_value);
wide_int_ops!(i32x16, i32, 16, add_i32x16, sub_i32x16, mul_i32x16, div_i32x16, neg_i32x16,
    and_i32x16, xor_i32x16, shl_i32x16, shr_i32x16, min_i32x16, lt_i32x16, select_bitmask_i32x16,
    reduce_sum_i32x16, reduce_max_i32x16, reduce_xor_i32x16, reverse_i32x16, rotate_left_i32x16,
    splat_i32x16, extract_i32x16, insert_i32x16, sum_slice_i32x16, add_i32x16_by_value);
wide_int_ops!(i64x4, i64, 4, add_i64x4, sub_i64x4, mul_i64x4, div_i64x4, neg_i64x4,
    and_i64x4, xor_i64x4, shl_i64x4, shr_i64x4, min_i64x4, lt_i64x4, select_bitmask_i64x4,
    reduce_sum_i64x4, reduce_max_i64x4, reduce_xor_i64x4, reverse_i64x4, rotate_left_i64x4,
    splat_i64x4, extract_i64x4, insert_i64x4, sum_slice_i64x4, add_i64x4_by_value);
wide_int_ops!(i64x8, i64, 8, add_i64x8, sub_i64x8, mul_i64x8, div_i64x8, neg_i64x8,
    and_i64x8, xor_i64x8, shl_i64x8, shr_i64x8, min_i64x8, lt_i64x8, select_bitmask_i64x8,
    reduce_sum_i64x8, reduce_max_i64x8, reduce_xor_i64x8, reverse_i64x8, rotate_left_i64x8,
    splat_i64x8, extract_i64x8, insert_i64x8, sum_slice_i64x8, add_i64x8_by_value);
wide_int_ops!(i16x16, i16, 16, add_i16x16, sub_i16x16, mul_i16x16, div_i16x16, neg_i16x16,
    and_i16x16, xor_i16x16, shl_i16x16, shr_i16x16, min_i16x16, lt_i16x16, select_bitmask_i16x16,
    reduce_sum_i16x16, reduce_max_i16x16, reduce_xor_i16x16, reverse_i16x16, rotate_left_i16x16,
    splat_i16x16, extract_i16x16, insert_i16x16, sum_slice_i16x16, add_i16x16_by_value);
wide_int_ops!(i16x32, i16, 32, add_i16x32, sub_i16x32, mul_i16x32, div_i16x32, neg_i16x32,
    and_i16x32, xor_i16x32, shl_i16x32, shr_i16x32, min_i16x32, lt_i16x32, select_bitmask_i16x32,
    reduce_sum_i16x32, reduce_max_i16x32, reduce_xor_i16x32, reverse_i16x32, rotate_left_i16x32,
    splat_i16x32, extract_i16x32, insert_i16x32, sum_slice_i16x32, add_i16x32_by_value);
wide_int_ops!(i8x32, i8, 32, add_i8x32, sub_i8x32, mul_i8x32, div_i8x32, neg_i8x32,
    and_i8x32, xor_i8x32, shl_i8x32, shr_i8x32, min_i8x32, lt_i8x32, select_bitmask_i8x32,
    reduce_sum_i8x32, reduce_max_i8x32, reduce_xor_i8x32, reverse_i8x32, rotate_left_i8x32,
    splat_i8x32, extract_i8x32, insert_i8x32, sum_slice_i8x32, add_i8x32_by_value);
wide_int_ops!(i8x64, i8, 64, add_i8x64, sub_i8x64, mul_i8x64, div_i8x64, neg_i8x64,
    and_i8x64, xor_i8x64, shl_i8x64, shr_i8x64, min_i8x64, lt_i8x64, select_bitmask_i8x64,
    reduce_sum_i8x64, reduce_max_i8x64, reduce_xor_i8x64, reverse_i8x64, rotate_left_i8x64,
    splat_i8x64, extract_i8x64, insert_i8x64, sum_slice_i8x64, add_i8x64_by_value);

// ---- Wide unsigned ----

#[no_mangle]
fn div_u32x8(a: [u32; 8], b: [u32; 8]) -> [u32; 8] {
    (u32x8::from_array(a) / u32x8::from_array(b)).to_array()
}

#[no_mangle]
fn div_u32x16(a: [u32; 16], b: [u32; 16]) -> [u32; 16] {
    (u32x16::from_array(a) / u32x16::from_array(b)).to_array()
}

// Logical shift right.
#[no_mangle]
fn shr_u32x8(a: [u32; 8], b: [u32; 8]) -> [u32; 8] {
    (u32x8::from_array(a) >> u32x8::from_array(b)).to_array()
}

#[no_mangle]
fn shr_u32x16(a: [u32; 16], b: [u32; 16]) -> [u32; 16] {
    (u32x16::from_array(a) >> u32x16::from_array(b)).to_array()
}

// Unsigned compare.
#[no_mangle]
fn lt_u32x8(a: [u32; 8], b: [u32; 8]) -> u64 {
    u32x8::from_array(a).simd_lt(u32x8::from_array(b)).to_bitmask()
}

#[no_mangle]
fn lt_u32x16(a: [u32; 16], b: [u32; 16]) -> u64 {
    u32x16::from_array(a).simd_lt(u32x16::from_array(b)).to_bitmask()
}

#[no_mangle]
fn max_u32x16(a: [u32; 16], b: [u32; 16]) -> [u32; 16] {
    u32x16::from_array(a).simd_max(u32x16::from_array(b)).to_array()
}

#[no_mangle]
fn saturating_add_u8x32(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
    u8x32::from_array(a).saturating_add(u8x32::from_array(b)).to_array()
}

#[no_mangle]
fn saturating_add_u8x64(a: [u8; 64], b: [u8; 64]) -> [u8; 64] {
    u8x64::from_array(a).saturating_add(u8x64::from_array(b)).to_array()
}

// All 64 bits of the bitmask are used.
#[no_mangle]
fn ge_u8x64(a: [u8; 64], b: [u8; 64]) -> u64 {
    u8x64::from_array(a).simd_ge(u8x64::from_array(b)).to_bitmask()
}

#[no_mangle]
fn any_eq_u8x64(a: [u8; 64], needle: u8) -> bool {
    u8x64::from_array(a).simd_eq(u8x64::splat(needle)).any()
}

#[no_mangle]
fn all_lt_u8x32(a: [u8; 32], bound: u8) -> bool {
    u8x32::from_array(a).simd_lt(u8x32::splat(bound)).all()
}

#[no_mangle]
fn reduce_sum_u8x64(a: [u8; 64]) -> u8 {
    u8x64::from_array(a).reduce_sum()
}

#[no_mangle]
fn swizzle_dyn_u8x32(a: [u8; 32], idx: [u8; 32]) -> [u8; 32] {
    u8x32::from_array(a).swizzle_dyn(u8x32::from_array(idx)).to_array()
}

#[no_mangle]
fn swizzle_dyn_u8x64(a: [u8; 64], idx: [u8; 64]) -> [u8; 64] {
    u8x64::from_array(a).swizzle_dyn(u8x64::from_array(idx)).to_array()
}

// ---- Wide float ----

macro_rules! wide_float_ops {
    ($v:ident, $t:ty, $n:literal, $add:ident, $mul:ident, $div:ident, $neg:ident, $abs:ident,
     $min:ident, $le:ident, $is_nan:ident, $rsum:ident, $rmax:ident, $dot:ident, $mul_bv:ident) => {
        #[no_mangle]
        fn $add(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) + $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $mul(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) * $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $div(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            ($v::from_array(a) / $v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $neg(a: [$t; $n]) -> [$t; $n] {
            (-$v::from_array(a)).to_array()
        }
        #[no_mangle]
        fn $abs(a: [$t; $n]) -> [$t; $n] {
            $v::from_array(a).abs().to_array()
        }
        #[no_mangle]
        fn $min(a: [$t; $n], b: [$t; $n]) -> [$t; $n] {
            $v::from_array(a).simd_min($v::from_array(b)).to_array()
        }
        #[no_mangle]
        fn $le(a: [$t; $n], b: [$t; $n]) -> u64 {
            $v::from_array(a).simd_le($v::from_array(b)).to_bitmask()
        }
        #[no_mangle]
        fn $is_nan(a: [$t; $n]) -> u64 {
            $v::from_array(a).is_nan().to_bitmask()
        }
        #[no_mangle]
        fn $rsum(a: [$t; $n]) -> $t {
            $v::from_array(a).reduce_sum()
        }
        #[no_mangle]
        fn $rmax(a: [$t; $n]) -> $t {
            $v::from_array(a).reduce_max()
        }
        #[no_mangle]
        fn $dot(a: &[$t], b: &[$t]) -> $t {
            let n = a.len().min(b.len()) / $n * $n;
            let mut acc = $v::splat(0.0);
            let mut i = 0;
            while i < n {
                acc += $v::from_slice(&a[i..]) * $v::from_slice(&b[i..]);
                i += $n;
            }
            let mut sum = acc.reduce_sum();
            while i < a.len().min(b.len()) {
                sum += a[i] * b[i];
                i += 1;
            }
            sum
        }
        #[no_mangle]
        fn $mul_bv(a: $v, b: $v) -> $v {
            a * b
        }
    };
}

wide_float_ops!(f32x8, f32, 8, add_f32x8, mul_f32x8, div_f32x8, neg_f32x8, abs_f32x8,
    min_f32x8, le_f32x8, is_nan_f32x8, reduce_sum_f32x8, reduce_max_f32x8, dot_f32x8, mul_f32x8_by_value);
wide_float_ops!(f32x16, f32, 16, add_f32x16, mul_f32x16, div_f32x16, neg_f32x16, abs_f32x16,
    min_f32x16, le_f32x16, is_nan_f32x16, reduce_sum_f32x16, reduce_max_f32x16, dot_f32x16, mul_f32x16_by_value);
wide_float_ops!(f64x4, f64, 4, add_f64x4, mul_f64x4, div_f64x4, neg_f64x4, abs_f64x4,
    min_f64x4, le_f64x4, is_nan_f64x4, reduce_sum_f64x4, reduce_max_f64x4, dot_f64x4, mul_f64x4_by_value);
wide_float_ops!(f64x8, f64, 8, add_f64x8, mul_f64x8, div_f64x8, neg_f64x8, abs_f64x8,
    min_f64x8, le_f64x8, is_nan_f64x8, reduce_sum_f64x8, reduce_max_f64x8, dot_f64x8, mul_f64x8_by_value);

// ---- Wide shuffles ----

// Lanes are taken from all 128-bit quarters.
#[no_mangle]
fn swizzle_i32x8(a: [i32; 8]) -> [i32; 8] {
    simd_swizzle!(i32x8::from_array(a), [7, 0, 5, 2, 3, 3, 6, 1]).to_array()
}

#[no_mangle]
fn swizzle_i32x16(a: [i32; 16]) -> [i32; 16] {
    simd_swizzle!(i32x16::from_array(a), [15, 0, 9, 4, 12, 3, 3, 7, 8, 1, 14, 2, 10, 6, 11, 5]).to_array()
}

#[no_mangle]
fn shuffle2_i32x8(a: [i32; 8], b: [i32; 8]) -> [i32; 8] {
    simd_swizzle!(i32x8::from_array(a), i32x8::from_array(b), [0, 8, 7, 15, 3, 12, 4, 11]).to_array()
}

#[no_mangle]
fn interleave_i32x8(a: [i32; 8], b: [i32; 8]) -> ([i32; 8], [i32; 8]) {
    let (lo, hi) = i32x8::from_array(a).interleave(i32x8::from_array(b));
    (lo.to_array(), hi.to_array())
}

#[no_mangle]
fn deinterleave_i32x16(a: [i32; 16], b: [i32; 16]) -> ([i32; 16], [i32; 16]) {
    let (even, odd) = i32x16::from_array(a).deinterleave(i32x16::from_array(b));
    (even.to_array(), odd.to_array())
}

// Concatenate two 256-bit vectors into one 512-bit vector.
#[no_mangle]
fn concat_i32x8(a: [i32; 8], b: [i32; 8]) -> [i32; 16] {
    simd_swizzle!(i32x8::from_array(a), i32x8::from_array(b),
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]).to_array()
}

// Extract the upper 256 bits of a 512-bit vector.
#[no_mangle]
fn high_half_i32x16(a: [i32; 16]) -> [i32; 8] {
    simd_swizzle!(i32x16::from_array(a), [8, 9, 10, 11, 12, 13, 14, 15]).to_array()
}

// ---- Wide casts ----

#[no_mangle]
fn cast_i8x16_to_i16x16(a: [i8; 16]) -> [i16; 16] {
    i8x16::from_array(a).cast::<i16>().to_array()
}

#[no_mangle]
fn cast_u8x16_to_u32x16(a: [u8; 16]) -> [u32; 16] {
    u8x16::from_array(a).cast::<u32>().to_array()
}

#[no_mangle]
fn cast_i32x8_to_i64x8(a: [i32; 8]) -> [i64; 8] {
    i32x8::from_array(a).cast::<i64>().to_array()
}

#[no_mangle]
fn cast_i32x16_to_i8x16(a: [i32; 16]) -> [i8; 16] {
    i32x16::from_array(a).cast::<i8>().to_array()
}

#[no_mangle]
fn cast_i32x8_to_f32x8(a: [i32; 8]) -> [f32; 8] {
    i32x8::from_array(a).cast::<f32>().to_array()
}

#[no_mangle]
fn cast_f32x16_to_i32x16(a: [f32; 16]) -> [i32; 16] {
    f32x16::from_array(a).cast::<i32>().to_array()
}

#[no_mangle]
fn cast_f32x8_to_f64x8(a: [f32; 8]) -> [f64; 8] {
    f32x8::from_array(a).cast::<f64>().to_array()
}

#[no_mangle]
fn cast_f64x8_to_f32x8(a: [f64; 8]) -> [f32; 8] {
    f64x8::from_array(a).cast::<f32>().to_array()
}

#[no_mangle]
fn to_bits_f64x4(a: [f64; 4]) -> [u64; 4] {
    f64x4::from_array(a).to_bits().to_array()
}

// ---- Wide memory ----

#[no_mangle]
fn load_or_default_i32x16(s: &[i32]) -> [i32; 16] {
    i32x16::load_or_default(s).to_array()
}

#[no_mangle]
fn gather_i32x8(s: &[i32], idx: [usize; 8]) -> [i32; 8] {
    i32x8::gather_or(s, std::simd::usizex8::from_array(idx), i32x8::splat(-1)).to_array()
}

#[no_mangle]
fn scatter_i32x16(s: &mut [i32], idx: [usize; 16], v: [i32; 16]) {
    i32x16::from_array(v).scatter(s, std::simd::usizex16::from_array(idx));
}

// ---- Wide vectors across the ABI ----

#[no_mangle]
fn add_u8x64_by_value(a: u8x64, b: u8x64) -> u8x64 {
    a + b
}

#[no_mangle]
fn by_value_calls_x16(a: i32x16, b: i32x16) -> i32x16 {
    let s = add_i32x16_by_value(a, b);
    add_i32x16_by_value(s, s)
}

// ---- Constant vectors stored to memory ----

#[no_mangle]
fn store_zero_i64x2(out: &mut [i64; 2]) {
    *out = i64x2::splat(0).to_array();
    unsafe { std::ptr::write(out as *mut [i64; 2] as *mut i64x2, i64x2::splat(0)) };
}

#[no_mangle]
fn store_const_i32x4(out: &mut [i32; 4]) {
    unsafe { std::ptr::write(out as *mut [i32; 4] as *mut i32x4, i32x4::from_array([1, -2, 3, i32::MIN])) };
}

#[no_mangle]
fn store_const_u8x16(out: &mut [u8; 16]) {
    unsafe { std::ptr::write(out as *mut [u8; 16] as *mut u8x16, u8x16::from_array([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 255])) };
}
