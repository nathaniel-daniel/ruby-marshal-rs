use crate::TypedValueHandle;
use crate::ValueHandle;

/// A Ruby Value
#[derive(Debug)]
pub enum Value {
    /// Nil
    Nil(NilValue),

    /// A Bool
    Bool(BoolValue),

    /// A Fixnum
    Fixnum(FixnumValue),

    /// A Symbol
    Symbol(SymbolValue),

    /// An Array
    Array(ArrayValue),

    /// A hash value
    Hash(HashValue),

    /// An Object
    Object(ObjectValue),

    /// A String
    String(StringValue),

    /// A User Defined Value
    UserDefined(UserDefinedValue),
}

impl Value {
    /// Get a ref to the [`SymbolValue`], if it is a symbol.
    pub fn as_symbol(&self) -> Option<&SymbolValue> {
        match self {
            Self::Symbol(value) => Some(value),
            _ => None,
        }
    }

    /// Get a ref to the [`ObjectValue`], if it is an object.
    pub fn as_object(&self) -> Option<&ObjectValue> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    /// Get a ref to the [`StringValue`], if it is a string.
    pub fn as_string(&self) -> Option<&StringValue> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Get the kind of value.
    pub fn kind(&self) -> ValueKind {
        match self {
            Self::Nil(_) => ValueKind::Nil,
            Self::Bool(_) => ValueKind::Bool,
            Self::Fixnum(_) => ValueKind::Fixnum,
            Self::Symbol(_) => ValueKind::Symbol,
            Self::Array(_) => ValueKind::Array,
            Self::Hash(_) => ValueKind::Hash,
            Self::Object(_) => ValueKind::Object,
            Self::String(_) => ValueKind::String,
            Self::UserDefined(_) => ValueKind::UserDefined,
        }
    }
}

impl From<NilValue> for Value {
    fn from(value: NilValue) -> Self {
        Self::Nil(value)
    }
}

impl From<BoolValue> for Value {
    fn from(value: BoolValue) -> Self {
        Self::Bool(value)
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

impl From<HashValue> for Value {
    fn from(value: HashValue) -> Self {
        Self::Hash(value)
    }
}

impl From<ObjectValue> for Value {
    fn from(value: ObjectValue) -> Self {
        Self::Object(value)
    }
}

impl From<StringValue> for Value {
    fn from(value: StringValue) -> Self {
        Self::String(value)
    }
}

impl From<UserDefinedValue> for Value {
    fn from(value: UserDefinedValue) -> Self {
        Self::UserDefined(value)
    }
}

/// A Nil value.
#[derive(Debug)]
pub struct NilValue;

/// A bool value.
#[derive(Debug, Copy, Clone)]
pub struct BoolValue {
    value: bool,
}

impl BoolValue {
    /// Create a new [`BoolValue`].
    pub(super) fn new(value: bool) -> Self {
        Self { value }
    }

    /// Get the inner value
    pub fn value(self) -> bool {
        self.value
    }
}

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

/// A Hash
#[derive(Debug)]
pub struct HashValue {
    value: Vec<(ValueHandle, ValueHandle)>,
    default_value: Option<ValueHandle>,
}

impl HashValue {
    /// Create a new [`HashValue`].
    pub(crate) fn new(
        value: Vec<(ValueHandle, ValueHandle)>,
        default_value: Option<ValueHandle>,
    ) -> Self {
        Self {
            value,
            default_value,
        }
    }

    /// Get the inner value.
    pub fn value(&self) -> &[(ValueHandle, ValueHandle)] {
        &self.value
    }

    /// Get the default value.
    pub fn default_value(&self) -> Option<ValueHandle> {
        self.default_value
    }
}

/// An object
#[derive(Debug)]
pub struct ObjectValue {
    name: TypedValueHandle<SymbolValue>,
    instance_variables: Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>,
}

impl ObjectValue {
    /// Create a new [`ObjectValue`].
    pub(crate) fn new(
        name: TypedValueHandle<SymbolValue>,
        instance_variables: Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>,
    ) -> Self {
        Self {
            name,
            instance_variables,
        }
    }

    /// Get the name.
    pub fn name(&self) -> TypedValueHandle<SymbolValue> {
        self.name
    }

    /// Get the instance variables
    pub fn instance_variables(&self) -> &[(TypedValueHandle<SymbolValue>, ValueHandle)] {
        &self.instance_variables
    }
}

/// A String
#[derive(Debug)]
pub struct StringValue {
    value: Vec<u8>,
    instance_variables: Option<Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>>,
}

impl StringValue {
    /// Create a new [`String`].
    pub(crate) fn new(value: Vec<u8>) -> Self {
        Self {
            value,
            instance_variables: None,
        }
    }

    /// Get the inner value.
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Get the instance variables
    pub fn instance_variables(&self) -> Option<&[(TypedValueHandle<SymbolValue>, ValueHandle)]> {
        self.instance_variables.as_deref()
    }

    /// Set the instance variables.
    ///
    /// # Returns
    /// Returns the old instance variables
    pub(crate) fn set_instance_variables(
        &mut self,
        mut instance_variables: Option<Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>>,
    ) -> Option<Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>> {
        std::mem::swap(&mut self.instance_variables, &mut instance_variables);
        instance_variables
    }
}

/// A User Defined value
#[derive(Debug)]
pub struct UserDefinedValue {
    name: TypedValueHandle<SymbolValue>,
    value: Vec<u8>,
    instance_variables: Option<Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>>,
}

impl UserDefinedValue {
    /// Create a new [`UserDefinedValue`].
    pub(crate) fn new(name: TypedValueHandle<SymbolValue>, value: Vec<u8>) -> Self {
        Self {
            name,
            value,
            instance_variables: None,
        }
    }

    /// Get the name.
    pub fn name(&self) -> TypedValueHandle<SymbolValue> {
        self.name
    }

    /// Get the inner value.
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    /// Get the instance variables
    pub fn instance_variables(&self) -> Option<&[(TypedValueHandle<SymbolValue>, ValueHandle)]> {
        self.instance_variables.as_deref()
    }

    /// Set the instance variables.
    ///
    /// # Returns
    /// Returns the old instance variables
    pub(crate) fn set_instance_variables(
        &mut self,
        mut instance_variables: Option<Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>>,
    ) -> Option<Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>> {
        std::mem::swap(&mut self.instance_variables, &mut instance_variables);
        instance_variables
    }
}

/// The kind of value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ValueKind {
    Nil,
    Bool,
    Fixnum,
    Symbol,
    Array,
    Hash,
    Object,
    String,
    UserDefined,
}
