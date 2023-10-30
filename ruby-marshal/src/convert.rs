use crate::Error;
use crate::NilValue;
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
}

impl std::fmt::Display for FromValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle { .. } => write!(f, "attempted to extract recursively"),
            Self::InvalidValueHandle { .. } => write!(f, "a handle was invalid"),
            Self::UnexpectedValueKind { kind } => write!(f, "unexpected value kind {kind:?}"),
        }
    }
}

impl std::error::Error for FromValueError {}

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

/// Implemented for any type that can be converted into a Ruby Value.
pub trait IntoValue: Sized {
    /// Turn this type into a Ruby Value.
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, Error>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity() {
        let mut arena = ValueArena::new();
        let symbol_handle = arena.create_symbol("symbol".into());
        let mut visited = HashSet::new();

        let _value: &Value = <&Value>::from_value(&arena, arena.root(), &mut visited)
            .expect("failed to exec &Value::from_value");

        visited.clear();
        let _nil_value: &NilValue = <&NilValue>::from_value(&arena, arena.root(), &mut visited)
            .expect("failed exec &NilValue::from_value");

        visited.clear();
        let _symbol_value: &SymbolValue =
            <&SymbolValue>::from_value(&arena, symbol_handle.into(), &mut visited)
                .expect("failed exec &NilValue::from_value");
    }
}
