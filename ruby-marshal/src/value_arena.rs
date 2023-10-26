mod value;
mod value_handle;

pub use self::value::FalseValue;
pub use self::value::NilValue;
pub use self::value::TrueValue;
pub use self::value::Value;
pub use self::value_handle::ValueHandle;
use std::marker::PhantomData;

/// A typed version of a [`ValueHandle`].
#[derive(Debug)]
pub struct TypedValueHandle<T> {
    handle: ValueHandle,
    _data: PhantomData<T>,
}

impl<T> TypedValueHandle<T> {
    /// Create a new [`TypedValueHandle`] from a [`ValueHandle`] without type checking.
    fn new_unchecked(handle: ValueHandle) -> Self {
        Self {
            handle,
            _data: PhantomData,
        }
    }

    /// Get the raw untyped handle.
    pub fn into_raw(self) -> ValueHandle {
        self.handle
    }
}

impl<T> From<TypedValueHandle<T>> for ValueHandle {
    fn from(handle: TypedValueHandle<T>) -> Self {
        handle.into_raw()
    }
}

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
}

impl Default for ValueArena {
    fn default() -> Self {
        Self::new()
    }
}
