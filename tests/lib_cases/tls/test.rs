// Every thread must see its own copy of each thread-local static.

use std::thread;

extern "Rust" {
  fn counter_get() -> u64;
  fn counter_add(n: u64) -> u64;
  fn byte_get() -> u8;
  fn byte_set(v: u8);
  fn zeroed_bump() -> u32;
}

fn main() {
  unsafe {
    // Initial values
    assert_eq!(counter_get(), 10);
    assert_eq!(byte_get(), 1);

    assert_eq!(counter_add(5), 15);
    assert_eq!(counter_add(7), 22);
    byte_set(200);
    assert_eq!(byte_get(), 200);
    assert_eq!(zeroed_bump(), 1);
    assert_eq!(zeroed_bump(), 2);
  }

  // New threads start from the initial value and do not affect this thread
  let handles: Vec<_> = (0..4u64).map(|i| thread::spawn(move || unsafe {
    assert_eq!(counter_get(), 10);
    assert_eq!(byte_get(), 1);
    assert_eq!(zeroed_bump(), 1);
    assert_eq!(counter_add(i), 10 + i);
    byte_set(i as u8);
    assert_eq!(counter_add(i), 10 + 2 * i);
    assert_eq!(byte_get(), i as u8);
    assert_eq!(zeroed_bump(), 2);
    10 + 2 * i
  })).collect();

  for (i, h) in handles.into_iter().enumerate() {
    assert_eq!(h.join().unwrap(), 10 + 2 * i as u64);
  }

  unsafe {
    assert_eq!(counter_get(), 22);
    assert_eq!(byte_get(), 200);
    assert_eq!(zeroed_bump(), 3);
  }
}
