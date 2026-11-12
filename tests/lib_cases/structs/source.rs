
struct User {
    id: u64,
    age: u8,
    height: u8,
    birthday: i16,
    salary: u64,
}

#[no_mangle]
fn increase_age(user: User) -> User {
    User {
        id: user.id,
        age: user.age + 1,
        height: user.height + 2,
        birthday: user.birthday - 20,
        salary: user.salary + 11111,
    }
}

#[no_mangle]
fn find_user(age: u32) -> User {
    let u: User;
    if age > 100 {
        u = User {
            id: 0xF0F0F0F0F0F0F0F0,
            age: 0xF1,
            height: 0xF2,
            birthday: 0x73F3,
            salary: 0xF3F3F3F3F3F3F3F3,
        }
    } else {
        u = User {
            id: 0x0F0F0F0F0F0F0F0F,
            age: 0x1F,
            height: 0x2F,
            birthday: 0x3F3F,
            salary: 0x3F3F3F3F3F3F3F3F,
        }
    }
    return u;
}

#[no_mangle]
fn find_user_by_age(age: u32) -> User {
    if age > 100 {
        User {
            id: 0xF0F0F0F0F0F0F0F0,
            age: 0xF1,
            height: 0xF2,
            birthday: 0x73F3,
            salary: 0xF3F3F3F3F3F3F3F3,
        }
    } else {
        User {
            id: 0x0F0F0F0F0F0F0F0F,
            age: 0x1F,
            height: 0x2F,
            birthday: 0x3F3F,
            salary: 0x3F3F3F3F3F3F3F3F,
        }
    }
}