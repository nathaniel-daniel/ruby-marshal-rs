/// A Ruby Value
#[derive(Debug)]
pub enum Value {
    /// Nil
    Nil(NilValue),

    /// True
    True(TrueValue),

    /// False
    False(FalseValue),
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
