
pub struct User {
    pub id: u64,
    pub name: &'static str
}

extern "Rust" {
    fn give_me_str() -> &'static str;
    fn more_string() -> &'static str;
    fn my_user() -> &'static User;
}

fn main() {
    let str = unsafe { give_me_str() };
    assert_eq!(str, "Hello world!");

    let str2 = unsafe { more_string() };
    assert_eq!(str2, "Hallo Welt!");

    let user = unsafe { my_user() };
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "John");
}