// ==========================================
// Type Definitions (Matching your backend)
// ==========================================

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct User {
    pub id: u8,
    pub age: u8,
}

pub struct Zst;

#[repr(C)]
pub struct BigStruct {
    pub data: [u64; 8],
}

#[repr(C)]
pub struct MixedTypes {
    pub integer: i32,
    pub floating: f64,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Bytes3 {
    pub a: u8,
    pub b: u8,
    pub c: u8,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Struct9 {
    pub big: u64,
    pub tail: u8,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Struct16 {
    pub lo: u64,
    pub hi: u64,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Struct17 {
    pub lo: u64,
    pub hi: u64,
    pub tail: u8,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Inner {
    pub a: i32,
    pub b: i32,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Outer {
    pub first: Inner,
    pub second: Inner,
    pub scale: f64,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Array4 {
    pub data: [i32; 4],
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

// ==========================================
// Linker implementation for the C ABI test
// ==========================================
// Your backend's `test_c_abi_call` expects to link against this function.
// We implement it here so standard `rustc` can provide it to the linker.
#[no_mangle]
pub extern "C" fn external_c_computation(p: Point3D, scale: f64) -> f64 {
    (p.x + p.y + p.z) * scale
}

#[no_mangle]
pub extern "C" fn external_many_args(
    a: i64, b: i64, c: i64, d: i64, e: i64, f: i64, g: i64, h: i64,
    i: f64, j: f64,
) -> i64 {
    a + b + c + d + e + f + g + h + i as i64 + j as i64
}

#[no_mangle]
pub extern "C" fn external_make_point(scale: f64) -> Point3D {
    Point3D { x: scale, y: scale * 2.0, z: scale * 3.0 }
}

#[no_mangle]
pub extern "C" fn external_big(big: BigStruct) -> u64 {
    let mut acc = 0u64;
    let mut i = 0usize;
    while i < 8 {
        acc += big.data[i];
        i += 1;
    }
    acc
}

// Callbacks handed to the backend's indirect-call helpers.
#[no_mangle]
pub extern "C" fn host_sub(a: i64, b: i64) -> i64 {
    a - b
}

#[no_mangle]
pub extern "C" fn host_point_shift(p: Point3D) -> Point3D {
    Point3D { x: p.x + 1.0, y: p.y + 2.0, z: p.z + 3.0 }
}

// ==========================================
// Extern Declarations (Your Backend's outputs)
// ==========================================

extern "Rust" {
    fn fib(n: u64) -> u64;

    fn editor(user: User) -> User;
    fn user() -> User;

    fn many_args(a: i8, b: i16, c: i32, d: i64, e: f32, f: f64, g: u8, h: u64, i: i64) -> i64;

    fn pass_zst(zst1: Zst, val: i64, zst2: Zst) -> i64;

    fn return_big_struct(seed: u64) -> BigStruct;
    fn consume_big_struct(big: BigStruct) -> u64;

    fn split_registers(mixed: MixedTypes) -> f64;

    fn min_max(a: i64, b: i64) -> (i64, i64);

    fn is_even(n: u64) -> bool;
    fn is_odd(n: u64) -> bool;
    fn ackermann(m: u64, n: u64) -> u64;
    fn nested_calls(a: i64, b: i64) -> i64;
    fn add3(a: i64, b: i64, c: i64) -> i64;

    fn many_floats(
        a: f32, b: f32, c: f32, d: f32,
        e: f64, f: f64, g: f64, h: f64,
        i: f64, j: f32, k: f64,
    ) -> f64;
    fn interleaved_args(
        a: i64, b: f64, c: i64, d: f64,
        e: i64, f: f64, g: i64, h: f64,
        i: i64, j: f64, k: i64, l: f64,
    ) -> f64;

    fn consume_big_and_args(big: BigStruct, x: u64, y: f64) -> f64;
    fn big_struct_from_args(a: u64, b: u64, c: f64) -> BigStruct;
    fn big_struct_roundtrip(seed: u64) -> u64;

    fn u128_add(a: u128, b: u128) -> u128;
    fn i128_mix(a: i128, b: i64, c: i128) -> i128;
    fn u128_late(a: u64, b: u64, c: u64, d: u64, e: u64, f: u128) -> u128;

    fn narrow_args(a: bool, b: u8, c: i8, d: u16, e: i16, f: char) -> i64;
    fn select_bool(cond: bool, a: i32, b: i32) -> i32;
    fn narrow_ret_u8(x: u64) -> u8;
    fn narrow_ret_i16(x: i64) -> i16;

    fn triple(a: i32, b: f64, c: i64) -> (i32, f64, i64);
    fn nested_tuple(a: i64, b: f64) -> ((i64, i64), (f64, f64));
    fn returns_unit(a: i64);
    fn min_max_spread(a: i64, b: i64) -> i64;
}

extern "C" {
    fn test_c_abi_call() -> f64;
    fn c_abi_point_transformer(p: Point3D) -> Point3D;

    fn bytes3_sum(v: Bytes3) -> u32;
    fn bytes3_bump(v: Bytes3) -> Bytes3;
    fn struct9_sum(v: Struct9) -> u64;
    fn struct16_swap(v: Struct16) -> Struct16;
    fn struct17_sum(v: Struct17) -> u64;
    fn struct17_and_args(v: Struct17, x: u64, y: f64, z: u64) -> f64;

    fn apply_binary(f: unsafe extern "C" fn(i64, i64) -> i64, a: i64, b: i64) -> i64;
    fn apply_twice(f: extern "C" fn(i64, i64) -> i64, a: i64, b: i64) -> i64;
    fn callback_target(a: i64, b: i64) -> i64;
    fn get_callback() -> extern "C" fn(i64, i64) -> i64;
    fn apply_point(f: extern "C" fn(Point3D) -> Point3D, p: Point3D) -> Point3D;

    fn sum_ptr(ptr: *const i64, len: usize) -> i64;
    fn write_out_params(a: *mut i64, b: *mut f64, value: i64);
    fn nth_ptr(ptr: *const i64, n: usize) -> *const i64;

    fn nested_struct_sum(o: Outer) -> f64;
    fn nested_struct_build(a: i32, scale: f64) -> Outer;
    fn array4_sum(v: Array4) -> i32;
    fn vec2_dot(a: Vec2, b: Vec2) -> f32;
    fn vec2_scale(v: Vec2, s: f32) -> Vec2;

    fn call_external_many_args() -> i64;
    fn call_external_chain(scale: f64) -> f64;
    fn call_external_big(seed: u64) -> u64;
}

// ==========================================
// The Test Runner
// ==========================================

fn main() {
    unsafe {
        // 1. Primitive ABI (Existing tests)
        assert_eq!(fib(1), 1);
        assert_eq!(fib(2), 2);
        assert_eq!(fib(3), 3);
        assert_eq!(fib(4), 5);
        assert_eq!(fib(5), 8);
        assert_eq!(fib(11), 144);

        // 2. Small Structs (Returned in registers or combined)
        let u_default = user();
        assert_eq!(u_default, User { id: 2, age: 2 });

        let u_edited = editor(User { id: 42, age: 100 });
        assert_eq!(u_edited, User { id: 43, age: 100 });

        // 3. Register Exhaustion (Forces stack usage)
        // a:1 + b:2 + c:3 + d:4 + e:5.0 + f:6.0 + g:7 + h:8 + i:9 = 45
        let sum = many_args(1, 2, 3, 4, 5.0, 6.0, 7, 8, 9);
        assert_eq!(sum, 45);

        // 4. Zero-Sized Types (Should not misalign the real arguments)
        let zst_val = pass_zst(Zst, 777, Zst);
        assert_eq!(zst_val, 777);

        // 5. Large Structs (Testing hidden `sret` pointer allocation)
        let big = return_big_struct(10);
        assert_eq!(big.data, [10, 11, 12, 13, 14, 15, 16, 17]);

        let sum_ends = consume_big_struct(big);
        assert_eq!(sum_ends, 10 + 17); // 27

        // 6. Heterogeneous Structs (Testing GPR/XMM split)
        let mixed = MixedTypes { integer: 10, floating: 2.5 };
        let float_sum = split_registers(mixed);
        assert_eq!(float_sum, 12.5);

        // 7. Extern "C" ABI (Calling out of custom backend into standard rust)
        // Inside custom backend: Point3D(1.5, 2.5, 3.5), scale: 2.0
        // (1.5 + 2.5 + 3.5) * 2.0 = 7.5 * 2.0 = 15.0
        let c_result = test_c_abi_call();
        assert_eq!(c_result, 15.0);

        // 8. Extern "C" byval passing
        let p_in = Point3D { x: 1.0, y: 2.0, z: 3.0 };
        let p_out = c_abi_point_transformer(p_in);
        assert_eq!(p_out, Point3D { x: 2.0, y: 4.0, z: 6.0 });

        // 9. Tuple returns
        assert_eq!(min_max(10, 5), (5, 10));
        assert_eq!(min_max(2, 8), (2, 8));

        // 10. Mutual recursion and deep call chains
        assert!(is_even(0));
        assert!(!is_odd(0));
        assert!(is_even(10));
        assert!(is_odd(11));
        assert!(!is_even(7));
        assert_eq!(ackermann(0, 0), 1);
        assert_eq!(ackermann(2, 3), 9);
        assert_eq!(ackermann(3, 3), 61);

        // 11. Calls nested inside argument lists
        assert_eq!(add3(1, 2, 3), 6);
        // add3(3,4,1)=8, add3(4,3,2)=9, add3(3,3,3)=9 => 26
        assert_eq!(nested_calls(3, 4), 26);

        // 12. Float register exhaustion (1..=11 => 66)
        let f_sum = many_floats(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0);
        assert_eq!(f_sum, 66.0);

        // 13. Interleaved int/float args: ints 1+3+5+7+9+11=36, floats 2+4+..+12=42
        let inter = interleaved_args(
            1, 2.0, 3, 4.0, 5, 6.0, 7, 8.0, 9, 10.0, 11, 12.0,
        );
        assert_eq!(inter, 78.0);

        // 14. Awkwardly sized structs
        assert_eq!(bytes3_sum(Bytes3 { a: 1, b: 2, c: 3 }), 6);
        assert_eq!(
            bytes3_bump(Bytes3 { a: 1, b: 2, c: 3 }),
            Bytes3 { a: 2, b: 4, c: 6 }
        );
        assert_eq!(struct9_sum(Struct9 { big: 1 << 40, tail: 7 }), (1u64 << 40) + 7);
        assert_eq!(
            struct16_swap(Struct16 { lo: 111, hi: 222 }),
            Struct16 { lo: 222, hi: 111 }
        );
        assert_eq!(struct17_sum(Struct17 { lo: 1, hi: 2, tail: 3 }), 6);
        // (1+2+3+4+6) as f64 + 0.25
        assert_eq!(
            struct17_and_args(Struct17 { lo: 1, hi: 2, tail: 3 }, 4, 0.25, 6),
            16.25
        );

        // 15. Big struct byval mixed with register args (10..=17 sums to 108)
        let big2 = return_big_struct(10);
        assert_eq!(consume_big_and_args(big2, 2, 0.5), 110.5);

        // sret combined with ordinary arguments: base = 1 + 2 + 3 = 6
        let built = big_struct_from_args(1, 2, 3.5);
        assert_eq!(built.data, [6, 7, 8, 9, 10, 11, 12, 13]);

        // A returned struct fed straight into the next call: 5 + 12
        assert_eq!(big_struct_roundtrip(5), 17);

        // 16. 128-bit integers
        assert_eq!(u128_add(1, 2), 3);
        assert_eq!(u128_add(u64::MAX as u128, 1), (u64::MAX as u128) + 1);
        assert_eq!(u128_add(u128::MAX, 1), 0); // wrapping
        assert_eq!(i128_mix(-5, 3, 10), 8);
        assert_eq!(i128_mix(i64::MIN as i128, -1, 0), (i64::MIN as i128) - 1);
        assert_eq!(u128_late(1, 2, 3, 4, 5, 10), 25);

        // 17. Narrow scalar arguments and returns
        // 1 + 200 - 5 + 60000 - 1000 + 65
        assert_eq!(narrow_args(true, 200, -5, 60000, -1000, 'A'), 59261);
        assert_eq!(narrow_args(false, 0, i8::MIN, u16::MAX, i16::MIN, '\0'), -128 + 65535 - 32768);
        assert_eq!(select_bool(true, 7, 9), 7);
        assert_eq!(select_bool(false, 7, 9), 9);
        assert_eq!(narrow_ret_u8(0xDEAD_BEEF), 0xEF);
        assert_eq!(narrow_ret_i16(0x1234_8001u32 as i64), 0x8001u16 as i16);

        // 18. Indirect calls through function pointers
        assert_eq!(apply_binary(host_sub, 10, 4), 6);
        assert_eq!(apply_binary(callback_target, 3, 4), 34);
        // host_sub(10,3)=7, host_sub(3,10)=-7, host_sub(7,-7)=14
        assert_eq!(apply_twice(host_sub, 10, 3), 14);
        let cb = get_callback();
        assert_eq!(cb(5, 6), 56);
        assert_eq!(apply_binary(cb, 1, 2), 12);
        // host_point_shift applied twice
        assert_eq!(
            apply_point(host_point_shift, Point3D { x: 1.0, y: 2.0, z: 3.0 }),
            Point3D { x: 3.0, y: 6.0, z: 9.0 }
        );

        // 19. Raw pointer arguments
        let data: [i64; 5] = [1, 2, 3, 4, 5];
        assert_eq!(sum_ptr(data.as_ptr(), 5), 15);
        assert_eq!(sum_ptr(data.as_ptr(), 0), 0);

        let mut out_a: i64 = 0;
        let mut out_b: f64 = 0.0;
        write_out_params(&mut out_a, &mut out_b, 21);
        assert_eq!(out_a, 42);
        assert_eq!(out_b, 10.5);

        assert_eq!(*nth_ptr(data.as_ptr(), 3), 4);

        // 20. Nested aggregates and arrays
        let outer = Outer {
            first: Inner { a: 1, b: 2 },
            second: Inner { a: 3, b: 4 },
            scale: 1.5,
        };
        assert_eq!(nested_struct_sum(outer), 15.0);
        assert_eq!(
            nested_struct_build(1, 2.0),
            Outer {
                first: Inner { a: 1, b: 2 },
                second: Inner { a: 3, b: 4 },
                scale: 2.0,
            }
        );
        assert_eq!(array4_sum(Array4 { data: [1, 2, 3, 4] }), 10);

        // All-float structs (SSE class)
        assert_eq!(vec2_dot(Vec2 { x: 1.0, y: 2.0 }, Vec2 { x: 3.0, y: 4.0 }), 11.0);
        assert_eq!(
            vec2_scale(Vec2 { x: 1.5, y: 2.5 }, 2.0),
            Vec2 { x: 3.0, y: 5.0 }
        );

        // 21. Wider tuple / unit returns
        assert_eq!(triple(1, 2.5, 10), (2, 5.0, 9));
        assert_eq!(nested_tuple(5, 1.5), ((5, 6), (1.5, 2.5)));
        returns_unit(1);
        assert_eq!(min_max_spread(10, 5), 5);
        assert_eq!(min_max_spread(-4, 4), 8);

        // 22. Calling out into the C ABI (caller-side stack/byval/sret setup)
        // 1+2+..+8 = 36, plus 9 and 10
        assert_eq!(call_external_many_args(), 55);
        // external_make_point(2.0) = (2,4,6); (2+4+6) * 2.0 = 24
        assert_eq!(call_external_chain(2.0), 24.0);
        // 3+4+..+10 = 52
        assert_eq!(call_external_big(3), 52);
    }
}