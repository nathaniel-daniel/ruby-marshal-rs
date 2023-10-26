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
