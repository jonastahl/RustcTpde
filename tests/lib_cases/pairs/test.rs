
struct User {
    id: u64,
    age: u8,
}

extern "Rust" {
  fn increase_age(user: User) -> User;
  fn find_user(age: u32) -> User;
  fn find_user_by_age(age: u32) -> User;
}

fn check_inc(id: u64, age: u8, expected_age: u8) {
  let u = unsafe { increase_age(User { id, age }) };
  assert_eq!(u.id, id);
  assert_eq!(u.age, expected_age);
}

fn check_find(age: u32, expected_id: u64, expected_age: u8) {
  let u = unsafe { find_user(age) };
  assert_eq!(u.id, expected_id, "id for age {}", age);
  assert_eq!(u.age, expected_age, "age for age {}", age);
}

fn check_find_age(age: u32, expected_id: u64, expected_age: u8) {
  let u = unsafe { find_user_by_age(age) };
  assert_eq!(u.id, expected_id, "id for age {}", age);
  assert_eq!(u.age, expected_age, "age for age {}", age);
}

fn main() {
  check_inc(0, 0, 1);
  check_inc(1, 41, 42);

  // Both halves of the pair at their extremes
  check_inc(u64::MAX, 0, 1);
  check_inc(0, u8::MAX - 1, u8::MAX);

  // Bit patterns that would show up as a mix-up of the two pair slots
  check_inc(0xF0F0_F0F0_F0F0_F0F0, 0x0F, 0x10);
  check_inc(0x0F0F_0F0F_0F0F_0F0F, 0xF0, 0xF1);

  // Taken branch
  check_find(101, 0xF0F0_F0F0_F0F0_F0F0, 0xF1);

  // Not taken branch
  check_find(0, 0x0F0F_0F0F_0F0F_0F0F, 0xF2);
  check_find(99, 0x0F0F_0F0F_0F0F_0F0F, 0xF2);

  // Exactly on the boundary: `age > 100` must be false for 100
  check_find(100, 0x0F0F_0F0F_0F0F_0F0F, 0xF2);

  // `age` is a u32, so `age > 100` must be an *unsigned* comparison. These
  // fail today: InstructionKind has no unsigned compares, so builder.rs maps
  // IntUGT onto CMPgt, which RustCompilerX64 lowers to the signed `jg`.
  check_find(0x8000_0000, 0xF0F0_F0F0_F0F0_F0F0, 0xF1);
  check_find(u32::MAX, 0xF0F0_F0F0_F0F0_F0F0, 0xF1);

  // Taken branch
  check_find_age(101, 0xF0F0_F0F0_F0F0_F0F0, 0xF1);

  // Not taken branch
  check_find_age(0, 0x0F0F_0F0F_0F0F_0F0F, 0xF2);
  check_find_age(99, 0x0F0F_0F0F_0F0F_0F0F, 0xF2);

  // Exactly on the boundary: `age > 100` must be false for 100
  check_find_age(100, 0x0F0F_0F0F_0F0F_0F0F, 0xF2);

  check_find_age(0x8000_0000, 0xF0F0_F0F0_F0F0_F0F0, 0xF1);
  check_find_age(u32::MAX, 0xF0F0_F0F0_F0F0_F0F0, 0xF1);
}
