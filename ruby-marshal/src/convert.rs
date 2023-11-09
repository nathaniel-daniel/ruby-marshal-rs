use crate::ArrayValue;
use crate::BoolValue;
use crate::FixnumValue;
use crate::NilValue;
use crate::ObjectValue;
use crate::StringValue;
use crate::SymbolValue;
use crate::Value;
use crate::ValueArena;
use crate::ValueHandle;
use crate::ValueKind;
use std::collections::HashSet;

/// An error that may occur while creating a type from a Ruby Value.
#[derive(Debug)]
pub enum FromValueError {
    /// An already visited node was visited.
    Cycle {
        /// The already-visited node.
        handle: ValueHandle,
    },

    /// A given [`ValueHandle`] was invalid.
    InvalidValueHandle {
        /// The invalid handle
        handle: ValueHandle,
    },

    /// An unexpected value kind was encountered.
    UnexpectedValueKind {
        /// The unexpected value kind
        kind: ValueKind,
    },

    /// An object name was unexpected
    UnexpectedObjectName {
        /// The object name.
        ///
        /// This may or may not be UTF-8.
        name: Vec<u8>,
    },

    /// An instance variable was duplicated
    DuplicateInstanceVariable {
        /// The instance variable name.
        ///
        /// This may or may not be UTF-8.
        name: Vec<u8>,
    },

    /// An unknown instance variable was encountered.
    UnknownInstanceVariable {
        /// The instance variable name.
        ///
        /// This may or may not be UTF-8.
        name: Vec<u8>,
    },

    /// Missing an instance variable with the given name.
    MissingInstanceVariable {
        /// The instance variable name.
        ///
        /// This may or may not be UTF-8.
        name: Vec<u8>,
    },

    /// Another user-provided kind of error occured.
    Other {
        error: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

impl FromValueError {
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

impl std::fmt::Display for FromValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle { .. } => write!(f, "attempted to extract recursively"),
            Self::InvalidValueHandle { .. } => write!(f, "a handle was invalid"),
            Self::UnexpectedValueKind { kind } => write!(f, "unexpected value kind {kind:?}"),
            Self::UnexpectedObjectName { name } => {
                write!(f, "unexpected object name \"{}\"", DisplayByteString(name))
            }
            Self::DuplicateInstanceVariable { name } => {
                write!(
                    f,
                    "instance variable \"{}\" was encountered twice",
                    DisplayByteString(name)
                )
            }
            Self::UnknownInstanceVariable { name } => {
                write!(
                    f,
                    "instance variable \"{}\" is not known",
                    DisplayByteString(name)
                )
            }
            Self::MissingInstanceVariable { name } => {
                write!(
                    f,
                    "instance variable \"{}\" is missing",
                    DisplayByteString(name)
                )
            }
            Self::Other { .. } => write!(f, "an user-provided error was encountered"),
        }
    }
}

impl std::error::Error for FromValueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Other { error } => Some(&**error),
            _ => None,
        }
    }
}

/// Implemented for any type that can be created from a Ruby Value.
pub trait FromValue<'a>: Sized {
    /// Create this type from the given value from the [`ValueArena`].
    ///
    /// # Arguments
    /// 1. `arena`: The arena where the value to convert from is stored.
    /// 2. `handle`: The handle that points to the value to convert.
    /// 3. `visited`: A set of already-visited values, to prevent cycles.
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError>;
}

impl<'a> FromValue<'a> for &'a Value {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        if !visited.insert(handle) {
            return Err(FromValueError::Cycle { handle });
        }

        arena
            .get(handle)
            .ok_or(FromValueError::InvalidValueHandle { handle })
    }
}

impl<'a> FromValue<'a> for &'a NilValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::Nil(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a BoolValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::Bool(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a FixnumValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::Fixnum(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a SymbolValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::Symbol(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a ArrayValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::Array(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a ObjectValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::Object(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a StringValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::String(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}
impl<'a> FromValue<'a> for bool {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &BoolValue = FromValue::from_value(arena, handle, visited)?;
        Ok(value.value())
    }
}

impl<'a> FromValue<'a> for i32 {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &FixnumValue = FromValue::from_value(arena, handle, visited)?;
        Ok(value.value())
    }
}

impl<'a, T> FromValue<'a> for Option<T>
where
    T: FromValue<'a>,
{
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        visited.remove(&handle);

        match value {
            Value::Nil(_) => Ok(None),
            _ => T::from_value(arena, handle, visited).map(Some),
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

impl std::fmt::Display for IntoValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other { .. } => write!(f, "an user-provided error was encountered"),
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

impl IntoValue for i32 {
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, IntoValueError> {
        Ok(arena.create_fixnum(self).into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity() {
        let mut arena = ValueArena::new();

        let nil_handle = arena.create_nil().into_raw();
        let bool_handle = arena.create_bool(true).into_raw();
        let fixnum_handle = arena.create_fixnum(23).into_raw();
        let symbol_handle = arena.create_symbol("symbol".into());
        let array_handle = arena.create_array(Vec::new()).into_raw();
        let object_handle = arena.create_object(symbol_handle, Vec::new()).into_raw();
        let string_handle = arena.create_string("string".into()).into_raw();

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
        let _string_value: &ObjectValue =
            <&ObjectValue>::from_value(&arena, object_handle, &mut visited)
                .expect("failed exec &ObjectValue::from_value");

        visited.clear();
        let _string_value: &StringValue =
            <&StringValue>::from_value(&arena, string_handle, &mut visited)
                .expect("failed exec &StringValue::from_value");

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

        0_i32
            .into_value(&mut arena)
            .expect("failed to exec i32::into_value");
    }
}
