mod value;
mod value_handle;

pub use self::value::ArrayValue;
pub use self::value::FalseValue;
pub use self::value::FixnumValue;
pub use self::value::NilValue;
pub use self::value::StringValue;
pub use self::value::SymbolValue;
pub use self::value::TrueValue;
pub use self::value::Value;
pub use self::value_handle::TypedValueHandle;
pub use self::value_handle::ValueHandle;

/// An arena of Ruby values.
#[derive(Debug)]
pub struct ValueArena {
    arena: generational_arena::Arena<Value>,
    root: ValueHandle,
}

impl ValueArena {
    /// Make a new empty [`ValueArena`].
    ///
    /// The root node is nil.
    pub fn new() -> Self {
        let mut arena = generational_arena::Arena::new();
        let root = ValueHandle::new(arena.insert(Value::Nil(NilValue)));

        Self { arena, root }
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

    /// Create an orphan `Nil` value and return the handle.
    pub fn create_nil(&mut self) -> TypedValueHandle<NilValue> {
        let index = self.arena.insert(Value::Nil(NilValue));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `True` value and return the handle.
    pub fn create_true(&mut self) -> TypedValueHandle<TrueValue> {
        let index = self.arena.insert(Value::True(TrueValue));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `True` value and return the handle.
    pub fn create_false(&mut self) -> TypedValueHandle<FalseValue> {
        let index = self.arena.insert(Value::False(FalseValue));
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
    pub fn create_symbol(&mut self, value: Vec<u8>) -> TypedValueHandle<SymbolValue> {
        let index = self.arena.insert(Value::Symbol(SymbolValue::new(value)));
        let handle = ValueHandle::new(index);

        TypedValueHandle::new_unchecked(handle)
    }

    /// Create an orphan `String` value and return the handle.
    pub fn create_string(&mut self, value: Vec<u8>) -> TypedValueHandle<StringValue> {
        let index = self.arena.insert(Value::String(StringValue::new(value)));
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
