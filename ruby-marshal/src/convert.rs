mod from_value;

pub use self::from_value::BTreeMapFromValueError;
pub use self::from_value::FromValue;
pub use self::from_value::FromValueContext;
pub use self::from_value::FromValueError;
pub use self::from_value::HashMapFromValueError;
use crate::ValueArena;
use crate::ValueHandle;
use std::collections::BTreeMap;
use std::collections::HashMap;

/// A utility to display a byte sequence as a string if it is UTF8 or a slice otherwise.
#[derive(Debug)]
pub struct DisplayByteString<'a>(pub &'a [u8]);

impl<'a> std::fmt::Display for DisplayByteString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.0;
        match std::str::from_utf8(string) {
            Ok(string) => write!(f, "{string}"),
            Err(_error) => write!(f, "{string:?}"),
        }
    }
}

/// An error that may occur while transforming types into Ruby Values.
#[derive(Debug)]
pub enum IntoValueError {
    /// Another user-provided kind of error occured.
    Other {
        error: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

impl IntoValueError {
    /// Shorthand for creating a new `Other` error variant.
    pub fn new_other<E>(error: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    {
        Self::Other {
            error: error.into(),
        }
    }
}

impl std::fmt::Display for IntoValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other { .. } => write!(f, "a user-provided error was encountered"),
        }
    }
}

impl std::error::Error for IntoValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Other { error } => Some(&**error),
            // _ => None,
        }
    }
}

/// Implemented for any type that can be converted into a Ruby Value.
pub trait IntoValue: Sized {
    /// Turn this type into a Ruby Value.
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError>;
}

impl IntoValue for bool {
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError> {
        Ok(arena.create_bool(self).into())
    }
}

impl IntoValue for i32 {
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError> {
        Ok(arena.create_fixnum(self).into())
    }
}

impl<T> IntoValue for Vec<T>
where
    T: IntoValue,
{
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError> {
        let mut array = Vec::with_capacity(self.len());
        for item in self.into_iter() {
            array.push(item.into_value(arena)?);
        }
        Ok(arena.create_array(array).into())
    }
}

impl<K, V> IntoValue for HashMap<K, V>
where
    K: IntoValue,
    V: IntoValue,
{
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError> {
        let mut items = Vec::new();

        for (key, value) in self.into_iter() {
            let key_handle = key.into_value(arena)?;
            let value_handle = value.into_value(arena)?;

            items.push((key_handle, value_handle));
        }

        Ok(arena.create_hash(items, None).into())
    }
}

impl<K, V> IntoValue for BTreeMap<K, V>
where
    K: IntoValue,
    V: IntoValue,
{
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError> {
        let mut items = Vec::new();

        for (key, value) in self.into_iter() {
            let key_handle = key.into_value(arena)?;
            let value_handle = value.into_value(arena)?;

            items.push((key_handle, value_handle));
        }

        Ok(arena.create_hash(items, None).into())
    }
}

impl<T> IntoValue for Option<T>
where
    T: IntoValue,
{
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError> {
        match self {
            Some(value) => value.into_value(arena),
            None => Ok(arena.create_nil().into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ArrayValue;
    use crate::BoolValue;
    use crate::FixnumValue;
    use crate::HashValue;
    use crate::NilValue;
    use crate::ObjectValue;
    use crate::StringValue;
    use crate::SymbolValue;
    use crate::UserDefinedValue;
    use crate::Value;

    #[test]
    fn sanity() {
        let mut arena = ValueArena::new();

        let nil_handle = arena.create_nil().into_raw();
        let bool_handle = arena.create_bool(true).into_raw();
        let fixnum_handle = arena.create_fixnum(23).into_raw();
        let symbol_handle = arena.create_symbol("symbol".into());
        let array_handle = arena.create_array(vec![fixnum_handle]).into_raw();
        let hash_handle = arena.create_hash(Vec::new(), None).into_raw();
        let object_handle = arena.create_object(symbol_handle, Vec::new()).into_raw();
        let string_handle = arena.create_string("string".into()).into_raw();
        let user_defined_handle = arena
            .create_user_defined(symbol_handle, Vec::new())
            .into_raw();

        let symbol_handle = symbol_handle.into_raw();

        let ctx = FromValueContext::new(&arena);

        let _value: &Value = ctx
            .from_value(nil_handle)
            .expect("failed to exec &Value::from_value");

        let _nil_value: &NilValue = ctx
            .from_value(nil_handle)
            .expect("failed exec &NilValue::from_value");

        let _bool_value: &BoolValue = ctx
            .from_value(bool_handle)
            .expect("failed exec &BoolValue::from_value");

        let _fixnum_value: &FixnumValue = ctx
            .from_value(fixnum_handle)
            .expect("failed exec &FixnumValue::from_value");

        let _symbol_value: &SymbolValue = ctx
            .from_value(symbol_handle)
            .expect("failed exec &SymbolValue::from_value");

        let _array_value: &ArrayValue = ctx
            .from_value(array_handle)
            .expect("failed exec &ArrayValue::from_value");

        let _hash_value: &HashValue = ctx
            .from_value(hash_handle)
            .expect("failed exec &HashValue::from_value");

        let _string_value: &ObjectValue = ctx
            .from_value(object_handle)
            .expect("failed exec &ObjectValue::from_value");

        let _string_value: &StringValue = ctx
            .from_value(string_handle)
            .expect("failed exec &StringValue::from_value");

        let _user_defined_value: &UserDefinedValue = ctx
            .from_value(user_defined_handle)
            .expect("failed exec &UserDefinedValue::from_value");

        let _bool_value: bool = ctx
            .from_value(bool_handle)
            .expect("failed exec bool::from_value");

        let _i32_value: i32 = ctx
            .from_value(fixnum_handle)
            .expect("failed exec i32::from_value");

        let _some_symbol_value: Option<&SymbolValue> = ctx
            .from_value(symbol_handle)
            .expect("failed exec Option<&SymbolValue>::from_value");

        let _none_symbol_value: Option<&SymbolValue> = ctx
            .from_value(nil_handle)
            .expect("failed exec Option<&SymbolValue>::from_value");

        let _vec_value: Vec<i32> = ctx
            .from_value(array_handle)
            .expect("failed exec <Vec<i32>>::from_value");

        let _hash_map_value: HashMap<i32, i32> = ctx
            .from_value(hash_handle)
            .expect("failed exec <HashMap<i32, i32>>::from_value");

        let _btree_map_value: BTreeMap<i32, i32> = ctx
            .from_value(hash_handle)
            .expect("failed exec <BTreeMap<i32, i32>>::from_value");

        true.into_value(&mut arena)
            .expect("failed to exec bool::into_value");

        0_i32
            .into_value(&mut arena)
            .expect("failed to exec i32::into_value");

        vec![0, 1, 2]
            .into_value(&mut arena)
            .expect("failed to exec Vec::<i32>::into_value");

        HashMap::<i32, i32>::new()
            .into_value(&mut arena)
            .expect("failed to exec HashMap::<i32, i32>::into_value");

        BTreeMap::<i32, i32>::new()
            .into_value(&mut arena)
            .expect("failed to exec BTreeMap::<i32, i32>::into_value");

        Some(2_i32)
            .into_value(&mut arena)
            .expect("failed to exec Option::<i32>::Some::into_value");

        None::<i32>
            .into_value(&mut arena)
            .expect("failed to exec Option::<i32>::None::into_value");
    }
}
