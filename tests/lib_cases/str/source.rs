
#[no_mangle]
static HELLO_WORLD: &str = "Hello world!";
#[no_mangle]
static USER: User = User { id: 1, name: "John" };

pub struct User {
    pub id: u64,
    pub name: &'static str
}

#[no_mangle]
fn give_me_str() -> &'static str {
    HELLO_WORLD
}

#[no_mangle]
fn more_string() -> &'static str {
    "Hallo Welt!"
}

#[no_mangle]
fn my_user() -> &'static User {
    &USER
}