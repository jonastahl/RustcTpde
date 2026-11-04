// MIR normally has a single return terminator, so custom MIR is used to get multiple real returns.
#![feature(core_intrinsics, custom_mir)]
#![allow(internal_features)]

use std::intrinsics::mir::*;

#[custom_mir(dialect = "runtime", phase = "optimized")]
#[inline(always)]
fn sign_u8(x: u8) -> i32 {
    mir! {
        {
            match x {
                0 => zero,
                1 => positive,
                _ => negative,
            }
        }
        zero = {
            RET = 0;
            Return()
        }
        positive = {
            RET = 1;
            Return()
        }
        negative = {
            RET = -1;
            Return()
        }
    }
}

// Two call sites in the same caller, the second one has to use its own phi
#[inline(never)]
#[unsafe(no_mangle)]
pub fn two_sites(a: u8, b: u8) -> i32 {
    sign_u8(a) * 10 + sign_u8(b)
}

#[custom_mir(dialect = "runtime", phase = "optimized")]
#[inline(always)]
fn pick<T: Copy>(first: bool, a: T, b: T) -> T {
    mir! {
        {
            match first {
                false => second,
                _ => other,
            }
        }
        other = {
            RET = a;
            Return()
        }
        second = {
            RET = b;
            Return()
        }
    }
}

// Runtime arguments passed through the inlined returns
#[inline(never)]
#[unsafe(no_mangle)]
pub fn pick_u32(first: bool, a: u32, b: u32) -> u32 {
    pick(first, a, b)
}

// Constant arguments passed through the inlined returns
#[inline(never)]
#[unsafe(no_mangle)]
pub fn pick_const(first: bool) -> u32 {
    pick(first, 7, 9)
}

// Constant folding kills one of the returns, so the phi has an unreachable predecessor
#[inline(never)]
#[unsafe(no_mangle)]
pub fn pick_folded(a: u32, b: u32) -> u32 {
    pick(1 + 1 == 3, a, b)
}

// Values consisting of multiple registers
#[inline(never)]
#[unsafe(no_mangle)]
pub fn pick_u128(first: bool, a: u128, b: u128) -> u128 {
    pick(first, a, b)
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn pick_f64(first: bool, a: f64, b: f64) -> f64 {
    pick(first, a, b)
}

pub static GLOBAL_A: u32 = 11;
pub static GLOBAL_B: u32 = 22;

// Addresses of globals passed through the inlined returns
#[inline(never)]
#[unsafe(no_mangle)]
pub fn pick_global(first: bool) -> u32 {
    *pick(first, &GLOBAL_A, &GLOBAL_B)
}

// The result is not used at all
#[inline(never)]
#[unsafe(no_mangle)]
pub fn pick_ignored(first: bool, a: u32, b: u32) -> u32 {
    pick(first, a, b);
    a + b
}

// Pairs need two phis in the same block
#[custom_mir(dialect = "runtime", phase = "optimized")]
#[inline(always)]
fn ordered(a: u64, b: u64) -> (u64, u64) {
    mir! {
        let swap: bool;
        {
            swap = a > b;
            match swap {
                false => keep,
                _ => swapped,
            }
        }
        keep = {
            RET = (a, b);
            Return()
        }
        swapped = {
            RET = (b, a);
            Return()
        }
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn ordered_diff(a: u64, b: u64) -> u64 {
    let (low, high) = ordered(a, b);
    high - low
}

// Multiple returns inlined into multiple returns inlined into a third function. The inner phis
// have to be remapped when the outer function gets inlined
#[custom_mir(dialect = "runtime", phase = "optimized")]
#[inline(always)]
fn nested(x: u8, y: u8) -> i32 {
    mir! {
        let inner: i32;
        {
            match x {
                0 => early,
                _ => call_inner,
            }
        }
        early = {
            RET = 100;
            Return()
        }
        call_inner = {
            Call(inner = sign_u8(y), ReturnTo(after), UnwindUnreachable())
        }
        after = {
            RET = inner;
            Return()
        }
    }
}

#[inline(never)]
#[unsafe(no_mangle)]
pub fn nested_twice(a: u8, b: u8) -> i32 {
    nested(a, b) * 1000 + nested(b, a)
}

// The phi is inside of a loop
#[inline(never)]
#[unsafe(no_mangle)]
pub fn sum_signs(n: u8) -> i32 {
    let mut sum = 0;
    let mut i = 0;
    while i < n {
        sum += sign_u8(i);
        i += 1;
    }
    sum
}
