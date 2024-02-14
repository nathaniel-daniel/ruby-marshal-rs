use super::DisplayByteString;
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

    /// An object name was unexpected.
    UnexpectedObjectName {
        /// The object name.
        ///
        /// This may or may not be UTF-8.
        name: Vec<u8>,
    },

    /// A user defined value name was unexpected.
    UnexpectedUserDefinedName {
        /// The user defined name.
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

impl std::fmt::Display for FromValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle { .. } => write!(f, "attempted to extract recursively"),
            Self::InvalidValueHandle { .. } => write!(f, "a handle was invalid"),
            Self::UnexpectedValueKind { kind } => write!(f, "unexpected value kind {kind:?}"),
            Self::UnexpectedObjectName { name } => {
                write!(f, "unexpected object name \"{}\"", DisplayByteString(name))
            }
            Self::UnexpectedUserDefinedName { name } => {
                write!(
                    f,
                    "unexpected user defined name \"{}\"",
                    DisplayByteString(name)
                )
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
            Self::Other { .. } => write!(f, "a user-provided error was encountered"),
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

impl<'a> FromValue<'a> for &'a HashValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::Hash(value) => Ok(value),
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
            Value::String(value) => {
                // Remove the string from the visited set.
                // Strings can't be a part of reference cycles since they have no children.
                visited.remove(&handle);

                Ok(value)
            }
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a UserDefinedValue {
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let value: &Value = FromValue::from_value(arena, handle, visited)?;
        match value {
            Value::UserDefined(value) => Ok(value),
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

impl<'a, T> FromValue<'a> for Vec<T>
where
    T: FromValue<'a>,
{
    fn from_value(
        arena: &'a ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, FromValueError> {
        let array: &ArrayValue = FromValue::from_value(arena, handle, visited)?;
        let array = array.value();

        let mut vec = Vec::with_capacity(array.len());
        for handle in array.iter().copied() {
            vec.push(FromValue::from_value(arena, handle, visited)?);
        }

        Ok(vec)
    }
}
