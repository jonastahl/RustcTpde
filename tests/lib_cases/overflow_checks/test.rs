// Drives the checked add/sub/mul functions in `source.rs`. Every call is run
// through `catch_unwind`: an operation that overflows must panic, one that
// does not must return the same value the corresponding `checked_*` method
// computes.

use std::fmt::Debug;
use std::panic::{catch_unwind, UnwindSafe};

extern "Rust" {
  fn add_i8(a: i8, b: i8) -> i8;
  fn sub_i8(a: i8, b: i8) -> i8;
  fn mul_i8(a: i8, b: i8) -> i8;
  fn add_i16(a: i16, b: i16) -> i16;
  fn sub_i16(a: i16, b: i16) -> i16;
  fn mul_i16(a: i16, b: i16) -> i16;
  fn add_i32(a: i32, b: i32) -> i32;
  fn sub_i32(a: i32, b: i32) -> i32;
  fn mul_i32(a: i32, b: i32) -> i32;
  fn add_i64(a: i64, b: i64) -> i64;
  fn sub_i64(a: i64, b: i64) -> i64;
  fn mul_i64(a: i64, b: i64) -> i64;
  fn add_u8(a: u8, b: u8) -> u8;
  fn sub_u8(a: u8, b: u8) -> u8;
  fn mul_u8(a: u8, b: u8) -> u8;
  fn add_u16(a: u16, b: u16) -> u16;
  fn sub_u16(a: u16, b: u16) -> u16;
  fn mul_u16(a: u16, b: u16) -> u16;
  fn add_u32(a: u32, b: u32) -> u32;
  fn sub_u32(a: u32, b: u32) -> u32;
  fn mul_u32(a: u32, b: u32) -> u32;
  fn add_u64(a: u64, b: u64) -> u64;
  fn sub_u64(a: u64, b: u64) -> u64;
  fn mul_u64(a: u64, b: u64) -> u64;
  fn add_usize(a: usize, b: usize) -> usize;
  fn sub_usize(a: usize, b: usize) -> usize;
  fn mul_isize(a: isize, b: isize) -> isize;

  fn add_then_sub_u32(a: u32, b: u32, c: u32) -> u32;
  fn add_then_sub_u8(a: u8, b: u8, c: u8) -> u8;
  fn mul_add_i32(a: i32, b: i32, c: i32) -> i32;
  fn accumulate_u8(a: u8, b: u8) -> u8;
  fn add8_args_u32(a: u32, b: u32, c: u32, d: u32, e: u32, f: u32, g: u32, h: u32) -> u32;
  fn sum_to_u8(n: u8) -> u8;
  fn branchy_i32(a: i32, b: i32, take_mul: bool) -> i32;
  fn add_twice_u32(a: u32, b: u32) -> u32;
}

/// Runs `f` and compares against `expected`: `Some(v)` means it must return
/// `v`, `None` means it must panic. Failures are collected rather than
/// asserted so that one broken operation does not hide the rest.
fn check<T: PartialEq + Debug>(
  failures: &mut Vec<String>,
  label: String,
  expected: Option<T>,
  f: impl FnOnce() -> T + UnwindSafe,
) {
  match (catch_unwind(f), expected) {
    (Ok(got), Some(want)) if got == want => {}
    (Ok(got), Some(want)) => failures.push(format!("{label}: returned {got:?}, expected {want:?}")),
    (Ok(got), None) => failures.push(format!("{label}: returned {got:?}, expected a panic")),
    (Err(_), Some(want)) => failures.push(format!("{label}: panicked, expected {want:?}")),
    (Err(_), None) => {}
  }
}

/// Checks `add`, `sub` and `mul` of `$ty` over every ordered pair drawn from
/// `$vals`, against `checked_add`/`checked_sub`/`checked_mul`.
macro_rules! check_arith {
  ($fail:expr, $ty:ty, $add:ident, $sub:ident, $mul:ident, $vals:expr) => {{
    let vals: &[$ty] = &$vals;
    for &a in vals {
      for &b in vals {
        check($fail, format!("{}({a}, {b})", stringify!($add)), a.checked_add(b), || unsafe { $add(a, b) });
        check($fail, format!("{}({a}, {b})", stringify!($sub)), a.checked_sub(b), || unsafe { $sub(a, b) });
        check($fail, format!("{}({a}, {b})", stringify!($mul)), a.checked_mul(b), || unsafe { $mul(a, b) });
      }
    }
  }};
}

fn main() {
  // Panics are expected here, so keep the messages out of the test output.
  std::panic::set_hook(Box::new(|_| {}));

  let f = &mut Vec::new();

  check_arith!(f, i8, add_i8, sub_i8, mul_i8, [i8::MIN, i8::MIN + 1, -16, -2, -1, 0, 1, 2, 16, i8::MAX - 1, i8::MAX]);
  check_arith!(f, i16, add_i16, sub_i16, mul_i16, [i16::MIN, i16::MIN + 1, -256, -2, -1, 0, 1, 2, 256, i16::MAX - 1, i16::MAX]);
  check_arith!(f, i32, add_i32, sub_i32, mul_i32, [i32::MIN, i32::MIN + 1, -65536, -2, -1, 0, 1, 2, 65536, i32::MAX - 1, i32::MAX]);
  check_arith!(f, i64, add_i64, sub_i64, mul_i64, [i64::MIN, i64::MIN + 1, -(1 << 32), -2, -1, 0, 1, 2, 1 << 32, i64::MAX - 1, i64::MAX]);
  check_arith!(f, u8, add_u8, sub_u8, mul_u8, [0, 1, 2, 16, 0x7F, 0x80, 0x81, 0xFE, 0xFF]);
  check_arith!(f, u16, add_u16, sub_u16, mul_u16, [0, 1, 2, 256, 0x7FFF, 0x8000, 0x8001, 0xFFFE, 0xFFFF]);
  check_arith!(f, u32, add_u32, sub_u32, mul_u32, [0, 1, 2, 65536, 0x7FFF_FFFF, 0x8000_0000, 0x8000_0001, 0xFFFF_FFFE, 0xFFFF_FFFF]);
  check_arith!(f, u64, add_u64, sub_u64, mul_u64, [0, 1, 2, 1 << 32, 0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0x8000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFE, u64::MAX]);

  // Pointer-width types.
  for &a in &[0usize, 1, 2, 1 << 32, usize::MAX - 1, usize::MAX] {
    for &b in &[0usize, 1, 2, 1 << 32, usize::MAX - 1, usize::MAX] {
      check(f, format!("add_usize({a}, {b})"), a.checked_add(b), || unsafe { add_usize(a, b) });
      check(f, format!("sub_usize({a}, {b})"), a.checked_sub(b), || unsafe { sub_usize(a, b) });
    }
  }
  for &a in &[isize::MIN, -1, 0, 1, 1 << 32, isize::MAX] {
    for &b in &[isize::MIN, -1, 0, 1, 1 << 32, isize::MAX] {
      check(f, format!("mul_isize({a}, {b})"), a.checked_mul(b), || unsafe { mul_isize(a, b) });
    }
  }

  // Chained arithmetic: the first operation's check must fire even when the
  // final result would be in range after wrapping.
  for &(a, b, c) in &[
    (1u32, 2, 3),
    (u32::MAX, 0, 0),
    (u32::MAX, 1, 1),       // `a + b` overflows; wrapping would give 0 - 1.
    (0x8000_0000, 0x8000_0000, 1), // `a + b` overflows to 0 under wrapping.
    (5, 5, 11),             // `a + b` fits, but the subtraction underflows.
    (5, 5, 10),
  ] {
    let want = a.checked_add(b).and_then(|s| s.checked_sub(c));
    check(f, format!("add_then_sub_u32({a}, {b}, {c})"), want, || unsafe { add_then_sub_u32(a, b, c) });
  }
  for a in [0u8, 1, 0x7F, 0x80, 0xFF] {
    for b in [0u8, 1, 0x7F, 0x80, 0xFF] {
      for c in [0u8, 1, 0xFF] {
        let want = a.checked_add(b).and_then(|s| s.checked_sub(c));
        check(f, format!("add_then_sub_u8({a}, {b}, {c})"), want, || unsafe { add_then_sub_u8(a, b, c) });
      }
    }
  }
  for &(a, b, c) in &[
    (2i32, 3, 4),
    (i32::MAX, 1, 0),
    (i32::MAX, 2, 0),       // the multiply overflows.
    (i32::MIN, -1, 0),      // so does this one.
    (i32::MAX, 1, 1),       // the multiply fits, the add overflows.
    (-1, i32::MIN, 0),
  ] {
    let want = a.checked_mul(b).and_then(|p| p.checked_add(c));
    check(f, format!("mul_add_i32({a}, {b}, {c})"), want, || unsafe { mul_add_i32(a, b, c) });
  }

  // Compound assignment: `((a + b) * 2) - 1`.
  for a in [0u8, 1, 2, 0x3F, 0x40, 0x7F, 0x80, 0xFF] {
    for b in [0u8, 1, 2, 0x40, 0x7F, 0xFF] {
      let want = a
        .checked_add(b)
        .and_then(|s| s.checked_mul(2))
        .and_then(|p| p.checked_sub(1));
      check(f, format!("accumulate_u8({a}, {b})"), want, || unsafe { accumulate_u8(a, b) });
    }
  }

  // Stack-passed arguments still reach the checked add.
  check(f, "add8_args_u32(1..8)".into(), Some(36), || unsafe { add8_args_u32(1, 2, 3, 4, 5, 6, 7, 8) });
  check(f, "add8_args_u32(max in 8th)".into(), None, || unsafe { add8_args_u32(1, 0, 0, 0, 0, 0, 0, u32::MAX) });
  check(f, "add8_args_u32(max in 7th)".into(), None, || unsafe { add8_args_u32(0, 0, 0, 0, 0, 0, u32::MAX, 1) });
  check(f, "add8_args_u32(max in 1st)".into(), Some(u32::MAX), || unsafe { add8_args_u32(u32::MAX, 0, 0, 0, 0, 0, 0, 0) });
  check(f, "add8_args_u32(overflow early)".into(), None, || unsafe { add8_args_u32(u32::MAX, 1, 0, 0, 0, 0, 0, 0) });

  // A check inside a loop: 1 + ... + n, which leaves u8 range past n == 22.
  for n in 0u8..=40 {
    let want = (1..=n as u32).sum::<u32>();
    let want = if want <= u8::MAX as u32 { Some(want as u8) } else { None };
    check(f, format!("sum_to_u8({n})"), want, || unsafe { sum_to_u8(n) });
  }

  // Only the taken branch is checked.
  check(f, "branchy_i32 mul ok".into(), Some(6), || unsafe { branchy_i32(2, 3, true) });
  check(f, "branchy_i32 mul overflow".into(), None, || unsafe { branchy_i32(i32::MAX, 2, true) });
  check(f, "branchy_i32 sub ok".into(), Some(-1), || unsafe { branchy_i32(2, 3, false) });
  check(f, "branchy_i32 sub overflow".into(), None, || unsafe { branchy_i32(i32::MIN, 1, false) });
  // The untaken arm would overflow, the taken one must not panic.
  check(f, "branchy_i32 untaken mul".into(), Some(0), || unsafe { branchy_i32(i32::MAX, i32::MAX, false) });
  check(f, "branchy_i32 untaken sub".into(), Some(0), || unsafe { branchy_i32(0, 0, true) });

  // The check survives across a call boundary.
  check(f, "add_twice_u32(1, 2)".into(), Some(5), || unsafe { add_twice_u32(1, 2) });
  check(f, "add_twice_u32(0, MAX)".into(), None, || unsafe { add_twice_u32(0, u32::MAX) });
  check(f, "add_twice_u32(MAX, 0)".into(), Some(u32::MAX), || unsafe { add_twice_u32(u32::MAX, 0) });

  let _ = std::panic::take_hook();
  if !f.is_empty() {
    for line in f.iter() {
      eprintln!("{line}");
    }
    panic!("{} overflow-check failures", f.len());
  }
}
