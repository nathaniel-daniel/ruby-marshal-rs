/// A Ruby Value
#[derive(Debug)]
pub enum Value {
    /// Nil
    Nil(NilValue),
}

/// A Nil value.
#[derive(Debug)]
pub struct NilValue;
