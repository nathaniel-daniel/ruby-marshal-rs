use ruby_marshal::FromValue;
use ruby_marshal::FromValueContext;
use ruby_marshal::FromValueError;
use ruby_marshal::IntoValue;
use ruby_marshal::StringValue;
use ruby_marshal::Value;
use ruby_marshal::ValueArena;
use ruby_marshal::ValueHandle;

#[derive(Debug, ruby_marshal_derive::FromValue, ruby_marshal_derive::IntoValue)]
#[ruby_marshal(object = b"MyObject")]
pub struct MyObject {
    field1: i32,

    #[ruby_marshal(name = b"renamed_field2")]
    field2: Vec<i32>,

    #[ruby_marshal(from_value = "ruby_string2string", into_value = "string2ruby_string")]
    field3: String,
}

fn ruby_string2string<'a>(
    ctx: &FromValueContext,
    value: &'a Value,
) -> Result<String, FromValueError> {
    let value: &StringValue = FromValue::from_value(ctx, value)?;
    let value = value.value();
    let value = std::str::from_utf8(value)
        .map_err(FromValueError::new_other)?
        .to_string();

    Ok(value)
}

fn string2ruby_string(
    s: String,
    arena: &mut ValueArena,
) -> Result<ValueHandle, ruby_marshal::IntoValueError> {
    Ok(arena.create_string(s.into()).into())
}

fn main() {
    let mut arena = ruby_marshal::ValueArena::new();

    {
        let object_name = arena.create_symbol("MyObject".into());

        let field1_name = arena.create_symbol("@field1".into());
        let field2_name = arena.create_symbol("renamed_field2".into());
        let field3_name = arena.create_symbol("@field3".into());

        let field1_value = arena.create_fixnum(21).into();
        let field2_value = arena.create_array(vec![field1_value, field1_value]);
        let field3_value = arena.create_string(b"hello world!".into());

        let object = arena.create_object(
            object_name,
            vec![
                (field1_name, field1_value),
                (field2_name, field2_value.into()),
                (field3_name, field3_value.into()),
            ],
        );
        arena.replace_root(object);
    }

    let ctx = ruby_marshal::FromValueContext::new(&arena);
    let object: MyObject = ctx.from_value(arena.root()).unwrap();
    dbg!(&object.field1);
    dbg!(&object.field2);
    dbg!(&object.field3);

    let encoded = object.into_value(&mut arena).unwrap();
    let ctx = ruby_marshal::FromValueContext::new(&arena);
    let decoded: MyObject = ctx.from_value(encoded).unwrap();

    dbg!(&decoded.field1);
    dbg!(&decoded.field2);
    dbg!(&decoded.field3);
}
