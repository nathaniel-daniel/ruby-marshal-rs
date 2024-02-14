mod from_value;

pub use self::from_value::FromValue;
pub use self::from_value::FromValueError;
use crate::ValueArena;
use crate::ValueHandle;

#[derive(Debug)]
struct DisplayByteString<'a>(&'a [u8]);

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
    use std::collections::HashSet;

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

        let mut visited = HashSet::new();

        let _value: &Value = <&Value>::from_value(&arena, nil_handle, &mut visited)
            .expect("failed to exec &Value::from_value");

        visited.clear();
        let _nil_value: &NilValue = <&NilValue>::from_value(&arena, nil_handle, &mut visited)
            .expect("failed exec &NilValue::from_value");

        visited.clear();
        let _bool_value: &BoolValue = <&BoolValue>::from_value(&arena, bool_handle, &mut visited)
            .expect("failed exec &BoolValue::from_value");

        visited.clear();
        let _fixnum_value: &FixnumValue =
            <&FixnumValue>::from_value(&arena, fixnum_handle, &mut visited)
                .expect("failed exec &FixnumValue::from_value");

        visited.clear();
        let _symbol_value: &SymbolValue =
            <&SymbolValue>::from_value(&arena, symbol_handle, &mut visited)
                .expect("failed exec &SymbolValue::from_value");

        visited.clear();
        let _array_value: &ArrayValue =
            <&ArrayValue>::from_value(&arena, array_handle, &mut visited)
                .expect("failed exec &ArrayValue::from_value");

        visited.clear();
        let _hash_value: &HashValue = <&HashValue>::from_value(&arena, hash_handle, &mut visited)
            .expect("failed exec &HashValue::from_value");

        visited.clear();
        let _string_value: &ObjectValue =
            <&ObjectValue>::from_value(&arena, object_handle, &mut visited)
                .expect("failed exec &ObjectValue::from_value");

        visited.clear();
        let _string_value: &StringValue =
            <&StringValue>::from_value(&arena, string_handle, &mut visited)
                .expect("failed exec &StringValue::from_value");

        visited.clear();
        let _user_defined_value: &UserDefinedValue =
            <&UserDefinedValue>::from_value(&arena, user_defined_handle, &mut visited)
                .expect("failed exec &UserDefinedValue::from_value");

        visited.clear();
        let _bool_value: bool = <bool>::from_value(&arena, bool_handle, &mut visited)
            .expect("failed exec bool::from_value");

        visited.clear();
        let _i32_value: i32 = <i32>::from_value(&arena, fixnum_handle, &mut visited)
            .expect("failed exec i32::from_value");

        visited.clear();
        let _some_symbol_value: Option<&SymbolValue> =
            <Option<&SymbolValue>>::from_value(&arena, symbol_handle, &mut visited)
                .expect("failed exec Option<&SymbolValue>::from_value");
        let _none_symbol_value: Option<&SymbolValue> =
            <Option<&SymbolValue>>::from_value(&arena, nil_handle, &mut visited)
                .expect("failed exec Option<&SymbolValue>::from_value");

        let _vec_value: Vec<i32> = <Vec<i32>>::from_value(&arena, array_handle, &mut visited)
            .expect("failed exec <Vec<i32>>::from_value");

        true.into_value(&mut arena)
            .expect("failed to exec bool::into_value");

        0_i32
            .into_value(&mut arena)
            .expect("failed to exec i32::into_value");
        vec![0, 1, 2]
            .into_value(&mut arena)
            .expect("failed to exec Vec::<i32>::into_value");
    }
}
