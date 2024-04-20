#[derive(Debug, ruby_marshal_derive::FromValue)]
#[ruby_marshal(object = b"MyObject")]
pub struct MyObject {
    field1: i32,

    #[ruby_marshal(name = b"@renamed_field2")]
    field2: Vec<i32>,
}

fn main() {
    let mut arena = ruby_marshal::ValueArena::new();

    let object_name = arena.create_symbol("MyObject".into());

    let object_field1_name = arena.create_symbol("field1".into());
    let object_field2_name = arena.create_symbol("@renamed_field2".into());

    let field1_value = arena.create_fixnum(21).into();
    let field2_value = arena.create_array(vec![field1_value, field1_value]);

    let object = arena.create_object(
        object_name,
        vec![
            (object_field1_name, field1_value),
            (object_field2_name, field2_value.into()),
        ],
    );
    arena.replace_root(object);

    let ctx = ruby_marshal::FromValueContext::new(&arena);
    let object: MyObject = ctx.from_value(arena.root()).unwrap();
    dbg!(object.field1);
    dbg!(object.field2);
}
