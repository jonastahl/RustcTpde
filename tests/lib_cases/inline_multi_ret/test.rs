extern "Rust" {
  fn two_sites(a: u8, b: u8) -> i32;
  fn pick_u32(first: bool, a: u32, b: u32) -> u32;
  fn pick_const(first: bool) -> u32;
  fn pick_folded(a: u32, b: u32) -> u32;
  fn pick_u128(first: bool, a: u128, b: u128) -> u128;
  fn pick_f64(first: bool, a: f64, b: f64) -> f64;
  fn pick_global(first: bool) -> u32;
  fn pick_ignored(first: bool, a: u32, b: u32) -> u32;
  fn ordered_diff(a: u64, b: u64) -> u64;
  fn nested_twice(a: u8, b: u8) -> i32;
  fn sum_signs(n: u8) -> i32;
}

fn main() {
  unsafe {
    for (a, b, expected) in [(0, 0, 0), (0, 1, 1), (1, 0, 10), (1, 1, 11), (2, 2, -11), (1, 5, 9), (7, 1, -9)] {
      assert_eq!(two_sites(a, b), expected);
    }

    assert_eq!(pick_u32(true, 3, 4), 3);
    assert_eq!(pick_u32(false, 3, 4), 4);
    assert_eq!(pick_const(true), 7);
    assert_eq!(pick_const(false), 9);
    assert_eq!(pick_folded(5, 6), 6);

    let (a, b) = (0x1111_2222_3333_4444_5555_6666_7777_8888, 0x9999_aaaa_bbbb_cccc_dddd_eeee_ffff_0001);
    assert_eq!(pick_u128(true, a, b), a);
    assert_eq!(pick_u128(false, a, b), b);

    assert_eq!(pick_f64(true, 1.5, 2.5), 1.5);
    assert_eq!(pick_f64(false, 1.5, 2.5), 2.5);

    assert_eq!(pick_global(true), 11);
    assert_eq!(pick_global(false), 22);

    assert_eq!(pick_ignored(true, 3, 4), 7);
    assert_eq!(pick_ignored(false, 3, 4), 7);

    assert_eq!(ordered_diff(3, 10), 7);
    assert_eq!(ordered_diff(10, 3), 7);
    assert_eq!(ordered_diff(5, 5), 0);

    assert_eq!(nested_twice(0, 0), 100_100);
    assert_eq!(nested_twice(0, 1), 100_000);
    assert_eq!(nested_twice(1, 0), 100);
    assert_eq!(nested_twice(1, 1), 1_001);
    assert_eq!(nested_twice(2, 2), -1_001);
    assert_eq!(nested_twice(2, 1), 999);

    assert_eq!(sum_signs(0), 0);
    assert_eq!(sum_signs(1), 0);
    assert_eq!(sum_signs(2), 1);
    assert_eq!(sum_signs(5), -2);
  }
}
