
#[no_mangle]
fn add_i8(a: i8, b: i8) -> i8 {
    a + b
}

#[no_mangle]
fn sub_i8(a: i8, b: i8) -> i8 {
    a - b
}

#[no_mangle]
fn mul_i8(a: i8, b: i8) -> i8 {
    a * b
}

#[no_mangle]
fn add_i16(a: i16, b: i16) -> i16 {
    a + b
}

#[no_mangle]
fn sub_i16(a: i16, b: i16) -> i16 {
    a - b
}

#[no_mangle]
fn mul_i16(a: i16, b: i16) -> i16 {
    a * b
}

#[no_mangle]
fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
fn sub_i32(a: i32, b: i32) -> i32 {
    a - b
}

#[no_mangle]
fn mul_i32(a: i32, b: i32) -> i32 {
    a * b
}

#[no_mangle]
fn add_i64(a: i64, b: i64) -> i64 {
    a + b
}

#[no_mangle]
fn sub_i64(a: i64, b: i64) -> i64 {
    a - b
}

#[no_mangle]
fn mul_i64(a: i64, b: i64) -> i64 {
    a * b
}

#[no_mangle]
fn add_u8(a: u8, b: u8) -> u8 {
    a + b
}

#[no_mangle]
fn sub_u8(a: u8, b: u8) -> u8 {
    a - b
}

#[no_mangle]
fn mul_u8(a: u8, b: u8) -> u8 {
    a * b
}

#[no_mangle]
fn add_u16(a: u16, b: u16) -> u16 {
    a + b
}

#[no_mangle]
fn sub_u16(a: u16, b: u16) -> u16 {
    a - b
}

#[no_mangle]
fn mul_u16(a: u16, b: u16) -> u16 {
    a * b
}

#[no_mangle]
fn add_u32(a: u32, b: u32) -> u32 {
    a + b
}

#[no_mangle]
fn sub_u32(a: u32, b: u32) -> u32 {
    a - b
}

#[no_mangle]
fn mul_u32(a: u32, b: u32) -> u32 {
    a * b
}

#[no_mangle]
fn add_u64(a: u64, b: u64) -> u64 {
    a + b
}

#[no_mangle]
fn sub_u64(a: u64, b: u64) -> u64 {
    a - b
}

#[no_mangle]
fn mul_u64(a: u64, b: u64) -> u64 {
    a * b
}

#[no_mangle]
fn add_usize(a: usize, b: usize) -> usize {
    a + b
}

#[no_mangle]
fn sub_usize(a: usize, b: usize) -> usize {
    a - b
}

#[no_mangle]
fn mul_isize(a: isize, b: isize) -> isize {
    a * b
}

// Chained arithmetic: only the intermediate `a + b` overflows, so the check on
// the first operation must not be folded away by the following subtraction.
#[no_mangle]
fn add_then_sub_u32(a: u32, b: u32, c: u32) -> u32 {
    (a + b) - c
}

// Same shape at the narrowest width, where a missing truncation before the
// check would hide the overflow entirely.
#[no_mangle]
fn add_then_sub_u8(a: u8, b: u8, c: u8) -> u8 {
    (a + b) - c
}

// Mixed operators in one expression: `a * b` is checked before the add.
#[no_mangle]
fn mul_add_i32(a: i32, b: i32, c: i32) -> i32 {
    a * b + c
}

// Compound assignment goes through the same checked operations.
#[no_mangle]
fn accumulate_u8(a: u8, b: u8) -> u8 {
    let mut acc = a;
    acc += b;
    acc *= 2;
    acc -= 1;
    acc
}

// A checked operation whose operands arrive past the six SysV argument
// registers: the 7th and 8th are on the stack.
#[no_mangle]
fn add8_args_u32(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32 {
    a + b + c + d + e + f + g + h
}

// A check inside a loop: the panic must be reachable from a non-entry block,
// and the successful path must fall through each iteration.
#[no_mangle]
fn sum_to_u8(n: u8) -> u8 {
    let mut acc: u8 = 0;
    let mut i: u8 = 1;
    while i <= n {
        acc += i;
        i += 1;
    }
    acc
}

// Overflow on one arm of a branch only, so the checks must be per-block.
#[no_mangle]
fn branchy_i32(a: i32, b: i32, take_mul: bool) -> i32 {
    if take_mul { a * b } else { a - b }
}

// The operation succeeds but the result is only observed through a call, so
// the value has to survive the check's control flow join.
#[no_mangle]
fn add_twice_u32(a: u32, b: u32) -> u32 {
    let first = add_u32(a, b);
    add_u32(first, b)
}
