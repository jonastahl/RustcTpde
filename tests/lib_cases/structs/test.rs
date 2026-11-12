struct User {
  id: u64,
  age: u8,
  height: u8,
  birthday: i16,
  salary: u64,
}

extern "Rust" {
  fn increase_age(user: User) -> User;
  fn find_user(age: u32) -> User;
  fn find_user_by_age(age: u32) -> User;
}

fn check_inc(
  id: u64,
  age: u8, expected_age: u8,
  height: u8, exp_height: u8,
  birthday: i16, exp_birthday: i16,
  salary: u64, exp_salary: u64
) {
  let u = unsafe { increase_age(User { id, age, height, birthday, salary }) };
  assert_eq!(u.id, id);
  assert_eq!(u.age, expected_age);
  assert_eq!(u.height, exp_height);
  assert_eq!(u.birthday, exp_birthday);
  assert_eq!(u.salary, exp_salary);
}

fn check_find(
  age: u32,
  expected_id: u64,
  expected_age: u8,
  expected_height: u8,
  expected_birthday: i16,
  expected_salary: u64
) {
  let u = unsafe { find_user(age) };
  assert_eq!(u.id, expected_id, "id for age {}", age);
  assert_eq!(u.age, expected_age, "age for age {}", age);
  assert_eq!(u.height, expected_height, "height for age {}", age);
  assert_eq!(u.birthday, expected_birthday, "birthday for age {}", age);
  assert_eq!(u.salary, expected_salary, "salary for age {}", age);
}

fn check_find_age(
  age: u32,
  expected_id: u64,
  expected_age: u8,
  expected_height: u8,
  expected_birthday: i16,
  expected_salary: u64
) {
  let u = unsafe { find_user_by_age(age) };
  assert_eq!(u.id, expected_id, "id for age {}", age);
  assert_eq!(u.age, expected_age, "age for age {}", age);
  assert_eq!(u.height, expected_height, "height for age {}", age);
  assert_eq!(u.birthday, expected_birthday, "birthday for age {}", age);
  assert_eq!(u.salary, expected_salary, "salary for age {}", age);
}

fn main() {
  // --- `increase_age` logic tests (age + 1, height + 2, birthday - 20, salary + 11111) ---
  check_inc(0, 0, 1, 2, 4, -10, -30, 100000, 111111);
  check_inc(1, 41, 42, 10, 12, -15, -35, 200000, 211111);

  // Extremes
  check_inc(u64::MAX, 0, 1, 0, 2, 0, -20, 0, 11111);
  check_inc(0, u8::MAX - 1, u8::MAX, u8::MAX - 2, u8::MAX, i16::MAX, i16::MAX - 20, u64::MAX - 11111, u64::MAX);

  // Bit patterns that would show up as a mix-up of the struct slots
  check_inc(
    0xF0F0_F0F0_F0F0_F0F0,
    0x0F, 0x10,
    0x0F, 0x11,
    0x0F0F, 0x0EFB,
    0xF0F0_F0F0_F0F0_F0F0, 0xF0F0_F0F0_F0F1_1C57
  );
  check_inc(
    0x0F0F_0F0F_0F0F_0F0F,
    0xF0, 0xF1,
    0xF0, 0xF2,
    -0x0F0F, -0x0F23,
    0x0F0F_0F0F_0F0F_0F0F, 0x0F0F_0F0F_0F0F_3A76
  );

  // --- `find_user` tests ---

  // Taken branch (age > 100)
  check_find(101, 0xF0F0_F0F0_F0F0_F0F0, 0xF1, 0xF2, 0x73F3, 0xF3F3_F3F3_F3F3_F3F3);

  // Not taken branch (age <= 100)
  check_find(0, 0x0F0F_0F0F_0F0F_0F0F, 0x1F, 0x2F, 0x3F3F, 0x3F3F_3F3F_3F3F_3F3F);
  check_find(99, 0x0F0F_0F0F_0F0F_0F0F, 0x1F, 0x2F, 0x3F3F, 0x3F3F_3F3F_3F3F_3F3F);

  // Exactly on the boundary: `age > 100` must be false for 100
  check_find(100, 0x0F0F_0F0F_0F0F_0F0F, 0x1F, 0x2F, 0x3F3F, 0x3F3F_3F3F_3F3F_3F3F);

  // `age` is a u32, so `age > 100` must be an *unsigned* comparison.
  check_find(0x8000_0000, 0xF0F0_F0F0_F0F0_F0F0, 0xF1, 0xF2, 0x73F3, 0xF3F3_F3F3_F3F3_F3F3);
  check_find(u32::MAX, 0xF0F0_F0F0_F0F0_F0F0, 0xF1, 0xF2, 0x73F3, 0xF3F3_F3F3_F3F3_F3F3);

  // --- `find_user_by_age` tests ---

  // Taken branch
  check_find_age(101, 0xF0F0_F0F0_F0F0_F0F0, 0xF1, 0xF2, 0x73F3, 0xF3F3_F3F3_F3F3_F3F3);

  // Not taken branch
  check_find_age(0, 0x0F0F_0F0F_0F0F_0F0F, 0x1F, 0x2F, 0x3F3F, 0x3F3F_3F3F_3F3F_3F3F);
  check_find_age(99, 0x0F0F_0F0F_0F0F_0F0F, 0x1F, 0x2F, 0x3F3F, 0x3F3F_3F3F_3F3F_3F3F);

  // Exactly on the boundary
  check_find_age(100, 0x0F0F_0F0F_0F0F_0F0F, 0x1F, 0x2F, 0x3F3F, 0x3F3F_3F3F_3F3F_3F3F);

  // Unsigned limits
  check_find_age(0x8000_0000, 0xF0F0_F0F0_F0F0_F0F0, 0xF1, 0xF2, 0x73F3, 0xF3F3_F3F3_F3F3_F3F3);
  check_find_age(u32::MAX, 0xF0F0_F0F0_F0F0_F0F0, 0xF1, 0xF2, 0x73F3, 0xF3F3_F3F3_F3F3_F3F3);
}