mod value;
mod value_handle;

pub use self::value::ArrayValue;
pub use self::value::BoolValue;
pub use self::value::FixnumValue;
pub use self::value::HashValue;
pub use self::value::NilValue;
pub use self::value::ObjectValue;
pub use self::value::StringValue;
pub use self::value::SymbolValue;
pub use self::value::UserDefinedValue;
pub use self::value::Value;
pub use self::value::ValueKind;
pub use self::value_handle::TypedValueHandle;
pub use self::value_handle::ValueHandle;
use slotmap::SlotMap;
use std::collections::HashMap;

/// An arena of Ruby values.
#[derive(Debug)]
pub struct ValueArena {
    arena: SlotMap<slotmap::DefaultKey, Value>,
    symbols: HashMap<Vec<u8>, TypedValueHandle<SymbolValue>>,
    root: ValueHandle,
}

impl ValueArena {
    /// Make a new empty [`ValueArena`].
    ///
    /// The root node is nil.
    pub fn new() -> Self {
        let mut arena = SlotMap::new();
        let symbols = HashMap::new();
        let root = ValueHandle::new(arena.insert(Value::Nil(NilValue)));

        Self {
            arena,
            symbols,
            root,
        }
    }

    /// Get the root [`ValueHandle`].
    pub fn root(&self) -> ValueHandle {
        self.root
    }

    /// Replace the current root, returning the old root.
    pub fn replace_root<H>(&mut self, new_root: H) -> ValueHandle
    where
        H: Into<ValueHandle>,
    {
        let mut new_root = new_root.into();

        std::mem::swap(&mut self.root, &mut new_root);
        new_root
    }

    /// Get a reference to the [`Value`] denoted by the given [`ValueHandle`].
    pub fn get<H>(&self, handle: H) -> Option<&Value>
    where
        H: Into<ValueHandle>,
    {
        self.arena.get(handle.into().index)
    }

    /// Get a mutable reference to the [`Value`] denoted by the given [`ValueHandle`].
    pub(crate) fn get_mut<H>(&mut self, handle: H) -> Option<&mut Value>
    where
        H: Into<ValueHandle>,
    {
        self.arena.get_mut(handle.into().index)
    }

    /// Get a reference to the [`SymbolValue`] denoted by the given [`TypedValueHandle`].
    ///
    /// # Panics
    /// Panics if the value is not a SymbolValue.
    pub fn get_symbol(&self, handle: TypedValueHandle<SymbolValue>) -> Option<&SymbolValue> {
        Some(self.get(handle)?.as_symbol().expect("not a symbol"))
    }

    /// Create an orphan `Nil` value and return the handle.
    pub fn create_nil(&mut self) -> TypedValueHandle<NilValue> {
        let index = self.arena.insert(Value::Nil(NilValue));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `Bool` value and return the handle.
    pub fn create_bool(&mut self, value: bool) -> TypedValueHandle<BoolValue> {
        let index = self.arena.insert(Value::Bool(BoolValue::new(value)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `Fixnum` value and return the handle.
    pub fn create_fixnum(&mut self, value: i32) -> TypedValueHandle<FixnumValue> {
        let index = self.arena.insert(Value::Fixnum(FixnumValue::new(value)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `Symbol` value and return the handle.
    ///
    /// If a symbol with this name already exists in this arena, it is returned instead of creating a new symbol.
    pub fn create_symbol(&mut self, value: Vec<u8>) -> TypedValueHandle<SymbolValue> {
        if let Some(handle) = self.symbols.get(&value) {
            return *handle;
        }

        self.create_new_symbol(value)
    }

    /// Create a new orphan `Symbol` value and return the handle.
    pub fn create_new_symbol(&mut self, value: Vec<u8>) -> TypedValueHandle<SymbolValue> {
        let index = self
            .arena
            .insert(Value::Symbol(SymbolValue::new(value.clone())));
        let handle = ValueHandle::new(index);
        let handle = TypedValueHandle::new_unchecked(handle);

        self.symbols.entry(value).or_insert(handle);

        handle
    }

    /// Create an orphan `Array` value and return the handle.
    pub fn create_array(&mut self, value: Vec<ValueHandle>) -> TypedValueHandle<ArrayValue> {
        let index = self.arena.insert(Value::Array(ArrayValue::new(value)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `Hash` value and return the handle.
    pub fn create_hash(
        &mut self,
        value: Vec<(ValueHandle, ValueHandle)>,
        default_value: Option<ValueHandle>,
    ) -> TypedValueHandle<HashValue> {
        let index = self
            .arena
            .insert(Value::Hash(HashValue::new(value, default_value)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `Object` value and return the handle.
    pub fn create_object(
        &mut self,
        name: TypedValueHandle<SymbolValue>,
        instance_variables: Vec<(TypedValueHandle<SymbolValue>, ValueHandle)>,
    ) -> TypedValueHandle<ObjectValue> {
        let index = self
            .arena
            .insert(Value::Object(ObjectValue::new(name, instance_variables)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `String` value and return the handle.
    pub fn create_string(&mut self, value: Vec<u8>) -> TypedValueHandle<StringValue> {
        let index = self.arena.insert(Value::String(StringValue::new(value)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `UserDefined` value and return the handle.
    pub fn create_user_defined(
        &mut self,
        name: TypedValueHandle<SymbolValue>,
        value: Vec<u8>,
    ) -> TypedValueHandle<UserDefinedValue> {
        let index = self
            .arena
            .insert(Value::UserDefined(UserDefinedValue::new(name, value)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }
}

impl Default for ValueArena {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<ValueHandle> for ValueArena {
    type Output = Value;

    fn index(&self, index: ValueHandle) -> &Self::Output {
        self.get(index).expect("missing value")
    }
}
