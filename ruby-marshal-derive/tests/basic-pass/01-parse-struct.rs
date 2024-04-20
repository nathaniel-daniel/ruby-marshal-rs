#[derive(ruby_marshal_derive::FromValue, ruby_marshal_derive::IntoValue)]
#[ruby_marshal(object = b"MyObject")]
pub struct MyObject {
    field: i32,
    
    #[ruby_marshal(name = b"@renamed_field2")]
    field2: Vec<i32>,
}

fn main() {}