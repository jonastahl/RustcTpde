extern "Rust" {
  fn outer(y: u32) -> u32;
  fn no_overflow() -> u16;
}

fn main() {
  for (y, expected) in [(0, 123), (5, 118), (23, 100), (40, 117), (1000, 7)] {
    assert_eq!(unsafe { outer(y) }, expected);
  }
  assert_eq!(unsafe { no_overflow() }, 1);
}
