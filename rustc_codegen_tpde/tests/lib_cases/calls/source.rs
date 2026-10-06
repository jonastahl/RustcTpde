

#[no_mangle]
fn fib(n: u64) -> u64 {
    if n <= 2 {
        return n;
    }

    fib(n - 1) + fib(n - 2)
}

pub struct User {
    pub id: u8,
    pub age: u8
}


#[no_mangle]
pub fn editor(user: User) -> User {
    User {
        id: user.id + 1,
        age: user.age
    }
}

#[no_mangle]
pub fn user() -> User {
    editor(User {
        id: 1,
        age: 2
    })
}

// ==========================================
// 2. Register Exhaustion (Many Arguments)
// ==========================================
// Most ABIs (like System V AMD64) pass the first ~6 arguments in registers.
// This function forces the backend to push the remaining arguments to the stack.

#[no_mangle]
pub fn many_args(
    a: i8, b: i16, c: i32, d: i64,
    e: f32, f: f64, g: u8, h: u64, i: i64
) -> i64 {
    a as i64 + b as i64 + c as i64 + d + e as i64 + f as i64 + g as i64 + h as i64 + i
}

// ==========================================
// 3. Zero-Sized Types (ZST)
// ==========================================
// ZSTs take up no space and should not consume a register or stack slot in the ABI.
// If your backend isn't careful, it might misalign the `val` argument.

pub struct Zst;

#[no_mangle]
pub fn pass_zst(_zst1: Zst, val: i64, _zst2: Zst) -> i64 {
    val
}

// ==========================================
// 4. Large Structs (Hidden `sret` Pointer)
// ==========================================
// Structs that are too large to fit in return registers are usually handled
// by the caller allocating stack space and passing a hidden pointer as the first arg.

#[repr(C)]
pub struct BigStruct {
    pub data: [u64; 8],
}

#[no_mangle]
pub fn return_big_struct(seed: u64) -> BigStruct {
    BigStruct {
        data: [seed, seed+1, seed+2, seed+3, seed+4, seed+5, seed+6, seed+7],
    }
}

#[no_mangle]
pub fn consume_big_struct(big: BigStruct) -> u64 {
    big.data[0] + big.data[7]
}

// ==========================================
// 5. Heterogeneous Structs (Int/Float Splitting)
// ==========================================
// In ABIs like SysV, structs with a mix of floats and ints might be split
// across integer (GPR) and floating-point (XMM) registers.

#[repr(C)]
pub struct MixedTypes {
    pub integer: i32,
    pub floating: f64,
}

#[no_mangle]
pub fn split_registers(mixed: MixedTypes) -> f64 {
    (mixed.integer as f64) + mixed.floating
}

// ==========================================
// 6. C ABI & byval Arguments
// ==========================================
// Using extern "C" forces the C ABI. Passing a struct by value in C
// often translates to `byval` pointers in IRs like LLVM or specific
// stack copies in custom backends.

#[repr(C)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

extern "C" {
    // A dummy external C function. Your backend should emit a call to this,
    // handling the Point3D argument according to standard C ABI (often byval/stack).
    fn external_c_computation(p: Point3D, scale: f64) -> f64;
}

#[no_mangle]
pub extern "C" fn test_c_abi_call() -> f64 {
    let p = Point3D { x: 1.5, y: 2.5, z: 3.5 };
    unsafe {
        // Calls the extern function
        external_c_computation(p, 2.0)
    }
}

// Tests receiving a struct byval via the C ABI and returning it.
#[no_mangle]
pub extern "C" fn c_abi_point_transformer(p: Point3D) -> Point3D {
    Point3D {
        x: p.x * 2.0,
        y: p.y * 2.0,
        z: p.z * 2.0,
    }
}

// ==========================================
// 7. Tuple / Multiple Returns
// ==========================================
// Rust natively returns tuples in a specific way.
// A small tuple might fit in two registers (e.g. RAX and RDX on x86_64).

#[no_mangle]
pub fn min_max(a: i64, b: i64) -> (i64, i64) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

// ==========================================
// 8. Mutual Recursion / Deep Call Chains
// ==========================================
// Forces the backend to handle call frames that alternate between two
// functions, plus a non-trivially deep chain (ackermann).

#[no_mangle]
pub fn is_even(n: u64) -> bool {
    if n == 0 { true } else { is_odd(n - 1) }
}

#[no_mangle]
pub fn is_odd(n: u64) -> bool {
    if n == 0 { false } else { is_even(n - 1) }
}

#[no_mangle]
pub fn ackermann(m: u64, n: u64) -> u64 {
    if m == 0 {
        n + 1
    } else if n == 0 {
        ackermann(m - 1, 1)
    } else {
        ackermann(m - 1, ackermann(m, n - 1))
    }
}

// Calls nested directly inside the argument list of another call:
// the backend must keep intermediate results alive across calls.
#[no_mangle]
pub fn nested_calls(a: i64, b: i64) -> i64 {
    add3(add3(a, b, 1), add3(b, a, 2), add3(a, a, 3))
}

#[no_mangle]
pub fn add3(a: i64, b: i64, c: i64) -> i64 {
    a + b + c
}

// ==========================================
// 9. Float Register Exhaustion
// ==========================================
// SysV passes the first 8 floats in XMM0-XMM7; the rest go on the stack.

#[no_mangle]
pub fn many_floats(
    a: f32, b: f32, c: f32, d: f32,
    e: f64, f: f64, g: f64, h: f64,
    i: f64, j: f32, k: f64,
) -> f64 {
    a as f64 + b as f64 + c as f64 + d as f64 + e + f + g + h + i + j as f64 + k
}

// Interleaved ints and floats: GPRs and XMMs are consumed independently,
// so the stack spill point differs per class.
#[no_mangle]
pub fn interleaved_args(
    a: i64, b: f64, c: i64, d: f64,
    e: i64, f: f64, g: i64, h: f64,
    i: i64, j: f64, k: i64, l: f64,
) -> f64 {
    (a + c + e + g + i + k) as f64 + b + d + f + h + j + l
}

// ==========================================
// 10. Awkwardly Sized Structs
// ==========================================
// 3 bytes, 9 bytes, 16 bytes and 17 bytes all hit different ABI classes:
// packed into one GPR, split over two GPRs, or spilled to memory.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bytes3 {
    pub a: u8,
    pub b: u8,
    pub c: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Struct9 {
    pub big: u64,
    pub tail: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Struct16 {
    pub lo: u64,
    pub hi: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Struct17 {
    pub lo: u64,
    pub hi: u64,
    pub tail: u8,
}

#[no_mangle]
pub extern "C" fn bytes3_sum(v: Bytes3) -> u32 {
    v.a as u32 + v.b as u32 + v.c as u32
}

#[no_mangle]
pub extern "C" fn bytes3_bump(v: Bytes3) -> Bytes3 {
    Bytes3 { a: v.a + 1, b: v.b + 2, c: v.c + 3 }
}

#[no_mangle]
pub extern "C" fn struct9_sum(v: Struct9) -> u64 {
    v.big + v.tail as u64
}

#[no_mangle]
pub extern "C" fn struct16_swap(v: Struct16) -> Struct16 {
    Struct16 { lo: v.hi, hi: v.lo }
}

#[no_mangle]
pub extern "C" fn struct17_sum(v: Struct17) -> u64 {
    v.lo + v.hi + v.tail as u64
}

// Mixing a memory-class struct with trailing register arguments: the
// trailing args must not be shifted by the struct's stack copy.
#[no_mangle]
pub extern "C" fn struct17_and_args(v: Struct17, x: u64, y: f64, z: u64) -> f64 {
    (v.lo + v.hi + v.tail as u64 + x + z) as f64 + y
}

// Same, but with the big (sret-sized) struct as a byval argument.
#[no_mangle]
pub fn consume_big_and_args(big: BigStruct, x: u64, y: f64) -> f64 {
    let mut acc = 0u64;
    let mut i = 0usize;
    while i < 8 {
        acc += big.data[i];
        i += 1;
    }
    (acc + x) as f64 + y
}

// sret plus additional arguments, where the hidden pointer shifts every
// other argument by one register.
#[no_mangle]
pub fn big_struct_from_args(a: u64, b: u64, c: f64) -> BigStruct {
    let base = a + b + c as u64;
    BigStruct {
        data: [base, base + 1, base + 2, base + 3, base + 4, base + 5, base + 6, base + 7],
    }
}

// Struct returned by a callee, immediately fed into another call.
#[no_mangle]
pub fn big_struct_roundtrip(seed: u64) -> u64 {
    consume_big_struct(return_big_struct(seed))
}

// ==========================================
// 11. 128-bit Integers
// ==========================================
// i128/u128 are passed in register *pairs* and returned in RAX:RDX.

#[no_mangle]
pub fn u128_add(a: u128, b: u128) -> u128 {
    a.wrapping_add(b)
}

#[no_mangle]
pub fn i128_mix(a: i128, b: i64, c: i128) -> i128 {
    a + b as i128 + c
}

// Forces a 128-bit value to be passed after the registers are exhausted.
#[no_mangle]
pub fn u128_late(a: u64, b: u64, c: u64, d: u64, e: u64, f: u128) -> u128 {
    (a + b + c + d + e) as u128 + f
}

// ==========================================
// 12. Small Scalar Types (Narrow Arguments)
// ==========================================
// bool/char/u8/i16 must be correctly zero/sign-extended at call boundaries.

#[no_mangle]
pub fn narrow_args(a: bool, b: u8, c: i8, d: u16, e: i16, f: char) -> i64 {
    (if a { 1 } else { 0 }) + b as i64 + c as i64 + d as i64 + e as i64 + f as i64
}

#[no_mangle]
pub fn select_bool(cond: bool, a: i32, b: i32) -> i32 {
    if cond { a } else { b }
}

// Narrow return values: the upper bits of the return register are garbage
// unless the callee explicitly truncates/extends.
#[no_mangle]
pub fn narrow_ret_u8(x: u64) -> u8 {
    (x & 0xFF) as u8
}

#[no_mangle]
pub fn narrow_ret_i16(x: i64) -> i16 {
    (x & 0xFFFF) as i16
}

// ==========================================
// 13. Indirect Calls (Function Pointers)
// ==========================================

#[no_mangle]
pub extern "C" fn apply_binary(f: extern "C" fn(i64, i64) -> i64, a: i64, b: i64) -> i64 {
    f(a, b)
}

#[no_mangle]
pub extern "C" fn apply_twice(f: extern "C" fn(i64, i64) -> i64, a: i64, b: i64) -> i64 {
    f(f(a, b), f(b, a))
}

#[no_mangle]
pub extern "C" fn callback_target(a: i64, b: i64) -> i64 {
    a * 10 + b
}

// Returns a pointer to one of our own functions so the caller can invoke it.
#[no_mangle]
pub extern "C" fn get_callback() -> extern "C" fn(i64, i64) -> i64 {
    callback_target
}

// Indirect call through a function pointer taking/returning a struct.
#[no_mangle]
pub extern "C" fn apply_point(
    f: extern "C" fn(Point3D) -> Point3D,
    p: Point3D,
) -> Point3D {
    f(f(p))
}

// ==========================================
// 14. Raw Pointer Arguments
// ==========================================

#[no_mangle]
pub unsafe extern "C" fn sum_ptr(ptr: *const i64, len: usize) -> i64 {
    let mut acc = 0i64;
    let mut i = 0usize;
    while i < len {
        acc += *ptr.add(i);
        i += 1;
    }
    acc
}

#[no_mangle]
pub unsafe extern "C" fn write_out_params(a: *mut i64, b: *mut f64, value: i64) {
    *a = value * 2;
    *b = value as f64 / 2.0;
}

// Pointer returned from a call and dereferenced by the caller.
#[no_mangle]
pub unsafe extern "C" fn nth_ptr(ptr: *const i64, n: usize) -> *const i64 {
    ptr.add(n)
}

// ==========================================
// 15. Arrays and Nested Aggregates
// ==========================================

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Inner {
    pub a: i32,
    pub b: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Outer {
    pub first: Inner,
    pub second: Inner,
    pub scale: f64,
}

#[no_mangle]
pub extern "C" fn nested_struct_sum(o: Outer) -> f64 {
    ((o.first.a + o.first.b + o.second.a + o.second.b) as f64) * o.scale
}

#[no_mangle]
pub extern "C" fn nested_struct_build(a: i32, scale: f64) -> Outer {
    Outer {
        first: Inner { a, b: a + 1 },
        second: Inner { a: a + 2, b: a + 3 },
        scale,
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Array4 {
    pub data: [i32; 4],
}

#[no_mangle]
pub extern "C" fn array4_sum(v: Array4) -> i32 {
    v.data[0] + v.data[1] + v.data[2] + v.data[3]
}

// All-float struct: classified as SSE and passed in XMM registers.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[no_mangle]
pub extern "C" fn vec2_dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

#[no_mangle]
pub extern "C" fn vec2_scale(v: Vec2, s: f32) -> Vec2 {
    Vec2 { x: v.x * s, y: v.y * s }
}

// ==========================================
// 16. Tuples and Unit Returns
// ==========================================

#[no_mangle]
pub fn triple(a: i32, b: f64, c: i64) -> (i32, f64, i64) {
    (a + 1, b * 2.0, c - 1)
}

#[no_mangle]
pub fn nested_tuple(a: i64, b: f64) -> ((i64, i64), (f64, f64)) {
    ((a, a + 1), (b, b + 1.0))
}

#[no_mangle]
pub fn returns_unit(_a: i64) {}

// Consumes a tuple produced by another call in this module.
#[no_mangle]
pub fn min_max_spread(a: i64, b: i64) -> i64 {
    let (lo, hi) = min_max(a, b);
    hi - lo
}

// ==========================================
// 17. Calling Out Into the C ABI
// ==========================================
// These exercise the *caller* side of stack argument setup, byval copies
// and sret pointers when the callee lives in another object file.

extern "C" {
    fn external_many_args(
        a: i64, b: i64, c: i64, d: i64, e: i64, f: i64, g: i64, h: i64,
        i: f64, j: f64,
    ) -> i64;

    fn external_make_point(scale: f64) -> Point3D;

    fn external_big(big: BigStruct) -> u64;
}

#[no_mangle]
pub extern "C" fn call_external_many_args() -> i64 {
    unsafe { external_many_args(1, 2, 3, 4, 5, 6, 7, 8, 9.0, 10.0) }
}

// Result of one external call flows straight into another external call.
#[no_mangle]
pub extern "C" fn call_external_chain(scale: f64) -> f64 {
    unsafe {
        let p = external_make_point(scale);
        external_c_computation(p, 2.0)
    }
}

#[no_mangle]
pub extern "C" fn call_external_big(seed: u64) -> u64 {
    unsafe { external_big(return_big_struct(seed)) }
}
