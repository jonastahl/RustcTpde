#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Big { pub a: u64, pub b: u64, pub c: u64, pub d: u32 }

#[inline(always)]
fn helper(x: u32) -> u32 { x.wrapping_mul(3) }

#[inline(always)]
fn big(x: u64) -> Big { Big { a: x, b: x + 1, c: x + 2, d: x as u32 + 3 } }

// The inlinable call and the repeat loop can end up in the same block, which then gets split
#[inline(never)]
#[unsafe(no_mangle)]
pub fn repeat_after_call(x: u32) -> [u32; 5] {
    [helper(x); 5]
}

// Also calls after the loop
#[inline(never)]
#[unsafe(no_mangle)]
pub fn repeat_between_calls(x: u32) -> u32 {
    let a = helper(x);
    let arr = [a; 7];
    let b = helper(arr[6]);
    let brr = [b; 3];
    helper(arr[0]) + brr[2]
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn repeat_pair(a: u64, b: u64) -> [(u64, u64); 4] {
    [(a, b); 4]
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn repeat_u128(a: u128) -> [u128; 3] {
    [a; 3]
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn repeat_f64(a: f64) -> [f64; 6] {
    [a; 6]
}

// The element is stored with a memcpy
#[inline(never)]
#[unsafe(no_mangle)]
pub fn repeat_big(x: u64) -> [Big; 3] {
    [big(x); 3]
}

// The loop gets inlined itself, twice into the same caller
#[inline(always)]
fn sum_repeated(x: u32) -> u32 {
    let arr = [helper(x); 4];
    arr[0] + arr[1] + arr[2] + arr[3]
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn inlined_loops(a: u32, b: u32) -> u32 {
    sum_repeated(a) + sum_repeated(b)
}

// The repeat loop in a loop
#[inline(never)]
#[unsafe(no_mangle)]
pub fn repeat_in_loop(n: u32) -> u32 {
    let mut sum = 0;
    let mut i = 0;
    while i < n {
        let arr = [helper(i); 3];
        sum += arr[2];
        i += 1;
    }
    sum
}
