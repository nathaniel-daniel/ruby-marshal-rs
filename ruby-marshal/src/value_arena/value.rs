use crate::ValueHandle;

/// A Ruby Value
#[derive(Debug)]
pub enum Value {
    /// Nil
    Nil(NilValue),

    /// True
    True(TrueValue),

    /// False
    False(FalseValue),

    /// A Fixnum
    Fixnum(FixnumValue),

    /// A Symbol
    Symbol(SymbolValue),

    /// An Array
    Array(ArrayValue),

    /// A String
    String(StringValue),
}

impl From<NilValue> for Value {
    fn from(value: NilValue) -> Self {
        Self::Nil(value)
    }
}

impl From<TrueValue> for Value {
    fn from(value: TrueValue) -> Self {
        Self::True(value)
    }
}

impl From<FalseValue> for Value {
    fn from(value: FalseValue) -> Self {
        Self::False(value)
    }
}

impl From<FixnumValue> for Value {
    fn from(value: FixnumValue) -> Self {
        Self::Fixnum(value)
    }
}

impl From<SymbolValue> for Value {
    fn from(value: SymbolValue) -> Self {
        Self::Symbol(value)
    }
}

impl From<ArrayValue> for Value {
    fn from(value: ArrayValue) -> Self {
        Self::Array(value)
    }
}

impl From<StringValue> for Value {
    fn from(value: StringValue) -> Self {
        Self::String(value)
    }
}

/// A Nil value.
#[derive(Debug)]
pub struct NilValue;

/// A true value.
#[derive(Debug)]
pub struct TrueValue;

/// A false value.
#[derive(Debug)]
pub struct FalseValue;

/// A Fixnum Value
#[derive(Debug, Copy, Clone)]
pub struct FixnumValue {
    value: i32,
}

impl FixnumValue {
    /// Create a new [`FixnumValue`].
    pub(super) fn new(value: i32) -> Self {
        Self { value }
    }

    /// Get the inner value
    pub fn value(self) -> i32 {
        self.value
    }
}

/// A Symbol
#[derive(Debug)]
pub struct SymbolValue {
    value: Vec<u8>,
}

impl SymbolValue {
    /// Create a new [`SymbolValue`].
    pub(super) fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// Get the inner value.
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}

/// An Array
#[derive(Debug)]
pub struct ArrayValue {
    value: Vec<ValueHandle>,
}

impl ArrayValue {
    /// Create a new [`Array`].
    pub(crate) fn new(value: Vec<ValueHandle>) -> Self {
        Self { value }
    }

    /// Get the inner value.
    pub fn value(&self) -> &[ValueHandle] {
        &self.value
    }

    /// Get the number of elements in the array
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Check if this is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

/// A String
#[derive(Debug)]
pub struct StringValue {
    value: Vec<u8>,
}

impl StringValue {
    /// Create a new [`String`].
    pub(crate) fn new(value: Vec<u8>) -> Self {
        Self { value }
    }

    /// Get the inner value.
    pub fn value(&self) -> &[u8] {
        &self.value
    }
}
