#[derive(ruby_marshal_derive::FromValue)]
pub enum MyObject {
    A {
        field1: i32,
    },
    B {
        field2: i32,
    },
}

fn main() {}