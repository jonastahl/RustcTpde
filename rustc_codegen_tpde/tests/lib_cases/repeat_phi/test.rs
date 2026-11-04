#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Big { pub a: u64, pub b: u64, pub c: u64, pub d: u32 }

extern "Rust" {
  fn repeat_after_call(x: u32) -> [u32; 5];
  fn repeat_between_calls(x: u32) -> u32;
  fn repeat_pair(a: u64, b: u64) -> [(u64, u64); 4];
  fn repeat_u128(a: u128) -> [u128; 3];
  fn repeat_f64(a: f64) -> [f64; 6];
  fn repeat_big(x: u64) -> [Big; 3];
  fn inlined_loops(a: u32, b: u32) -> u32;
  fn repeat_in_loop(n: u32) -> u32;
}

fn main() {
  unsafe {
    assert_eq!(repeat_after_call(7), [21; 5]);
    assert_eq!(repeat_between_calls(2), 18 + 18);
    assert_eq!(repeat_pair(1, 2), [(1, 2); 4]);
    let v = 0x1111_2222_3333_4444_5555_6666_7777_8888;
    assert_eq!(repeat_u128(v), [v; 3]);
    assert_eq!(repeat_f64(1.5), [1.5; 6]);
    assert_eq!(repeat_big(10), [Big { a: 10, b: 11, c: 12, d: 13 }; 3]);
    assert_eq!(inlined_loops(1, 2), 12 + 24);
    assert_eq!(repeat_in_loop(0), 0);
    assert_eq!(repeat_in_loop(4), 18);
  }
}
