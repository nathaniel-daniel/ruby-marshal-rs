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
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

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

    /// A hash key was provided twice.
    DuplicateHashKey {
        /// The key that was provided twice.
        ///
        /// This does not need to be a symbol.
        key: ValueHandle,
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
            Self::DuplicateHashKey { .. } => {
                write!(f, "duplicate hash key")
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

/// A context to manage extracting values.
pub struct FromValueContext<'a> {
    arena: &'a ValueArena,
    stack: RefCell<Vec<ValueHandle>>,
}

impl<'a> FromValueContext<'a> {
    /// Create a new context from an arena.
    pub fn new(arena: &'a ValueArena) -> Self {
        Self {
            arena,
            stack: RefCell::new(Vec::new()),
        }
    }

    fn begin_handle(&self, handle: ValueHandle) -> Result<(), FromValueError> {
        let mut stack = self.stack.borrow_mut();

        if stack.contains(&handle) {
            return Err(FromValueError::Cycle { handle });
        }

        stack.push(handle);

        Ok(())
    }

    fn end_handle(&self, handle: ValueHandle) {
        let stack_handle = self.stack.borrow_mut().pop();

        // This should always be Some.
        let stack_handle = stack_handle.unwrap();

        assert!(handle == stack_handle);
    }

    // The "value" here is a represented by the value handle.
    #[allow(clippy::wrong_self_convention)]
    /// Extract a type from a value.
    pub fn from_value<T>(&self, handle: ValueHandle) -> Result<T, FromValueError>
    where
        T: FromValue<'a>,
    {
        let guard = FromValueGuard::new(self, handle)?;
        let value = self
            .arena
            .get(handle)
            .ok_or(FromValueError::InvalidValueHandle { handle })?;
        let value = T::from_value(self, value)?;
        drop(guard);

        Ok(value)
    }
}

/// A guard for a handle.
///
/// Do NOT drop this before you are done using the handle and its children.
pub struct FromValueGuard<'a, 'b> {
    ctx: &'a FromValueContext<'b>,
    handle: ValueHandle,
}

impl<'a, 'b> FromValueGuard<'a, 'b> {
    fn new(ctx: &'a FromValueContext<'b>, handle: ValueHandle) -> Result<Self, FromValueError> {
        ctx.begin_handle(handle)?;

        Ok(Self { ctx, handle })
    }
}

impl<'a, 'b> Drop for FromValueGuard<'a, 'b> {
    fn drop(&mut self) {
        self.ctx.end_handle(self.handle);
    }
}

/// Implemented for any type that can be created from a Ruby Value.
pub trait FromValue<'a>: Sized {
    /// Create this type from the given value from the [`ValueArena`].
    ///
    /// # Arguments
    /// 1. `ctx`: The value extraction context.
    /// 2. `handle`: The handle that points to the value to convert.
    fn from_value(ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError>;
}

impl<'a> FromValue<'a> for &'a Value {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        Ok(value)
    }
}

impl<'a> FromValue<'a> for &'a NilValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Nil(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a BoolValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Bool(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a FixnumValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Fixnum(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a SymbolValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Symbol(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a ArrayValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Array(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a HashValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Hash(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a ObjectValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Object(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a StringValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::String(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for &'a UserDefinedValue {
    fn from_value(_ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::UserDefined(value) => Ok(value),
            value => Err(FromValueError::UnexpectedValueKind { kind: value.kind() }),
        }
    }
}

impl<'a> FromValue<'a> for bool {
    fn from_value(ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        let value: &BoolValue = FromValue::from_value(ctx, value)?;
        Ok(value.value())
    }
}

impl<'a> FromValue<'a> for i32 {
    fn from_value(ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        let value: &FixnumValue = FromValue::from_value(ctx, value)?;
        Ok(value.value())
    }
}

impl<'a, T> FromValue<'a> for Option<T>
where
    T: FromValue<'a>,
{
    fn from_value(ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        match value {
            Value::Nil(_) => Ok(None),
            _ => T::from_value(ctx, value).map(Some),
        }
    }
}

impl<'a, T> FromValue<'a> for Vec<T>
where
    T: FromValue<'a>,
{
    fn from_value(ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        let array: &ArrayValue = FromValue::from_value(ctx, value)?;
        let array = array.value();

        let mut vec = Vec::with_capacity(array.len());
        for handle in array.iter().copied() {
            let value = ctx.from_value(handle)?;
            vec.push(value);
        }

        Ok(vec)
    }
}

impl<'a, K, V> FromValue<'a> for HashMap<K, V>
where
    K: FromValue<'a> + Hash + Eq,
    V: FromValue<'a>,
{
    fn from_value(ctx: &FromValueContext<'a>, value: &'a Value) -> Result<Self, FromValueError> {
        let value: &HashValue = FromValue::from_value(ctx, value)?;
        let value = value.value();

        let mut map = HashMap::with_capacity(value.len());
        for (key_handle, value_handle) in value.iter().copied() {
            let key = ctx.from_value(key_handle)?;
            let value = ctx.from_value(value_handle)?;

            let old_value = map.insert(key, value);

            if old_value.is_some() {
                return Err(FromValueError::DuplicateHashKey { key: key_handle });
            }
        }

        Ok(map)
    }
}
