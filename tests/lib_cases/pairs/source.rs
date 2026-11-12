
struct User {
    id: u64,
    age: u8,
}

#[no_mangle]
fn increase_age(user: User) -> User {
    User {
        id: user.id,
        age: user.age + 1,
    }
}

#[no_mangle]
fn find_user(age: u32) -> User {
    let u: User;
    if age > 100 {
        u = User {
            id: 0xF0F0F0F0F0F0F0F0,
            age: 0xF1,
        }
    } else {
        u = User {
            id: 0x0F0F0F0F0F0F0F0F,
            age: 0xF2,
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
        }
    } else {
        User {
            id: 0x0F0F0F0F0F0F0F0F,
            age: 0xF2,
        }
    }
}