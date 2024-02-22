use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;

/// A handle around a Ruby Value.
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ValueHandle {
    /// The arena index
    pub(super) index: slotmap::DefaultKey,
    // TODO: Should we an a counter to the arena itself to prevent accidental accesses to other arenas?
}

impl ValueHandle {
    /// Create a new [`ValueHandle`] from an Index
    pub(super) fn new(index: slotmap::DefaultKey) -> Self {
        Self { index }
    }
}

/// A typed version of a [`ValueHandle`].
#[derive(Debug)]
pub struct TypedValueHandle<T> {
    handle: ValueHandle,
    _data: PhantomData<T>,
}

impl<T> TypedValueHandle<T> {
    /// Create a new [`TypedValueHandle`] from a [`ValueHandle`] without type checking.
    pub(crate) fn new_unchecked(handle: ValueHandle) -> Self {
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

impl<T> Clone for TypedValueHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TypedValueHandle<T> {}

impl<T> PartialEq<Self> for TypedValueHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.handle.eq(&other.handle)
    }
}

impl<T> Eq for TypedValueHandle<T> {}

impl<T> Hash for TypedValueHandle<T> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.handle.hash(state)
    }
}

impl<T> From<TypedValueHandle<T>> for ValueHandle {
    fn from(handle: TypedValueHandle<T>) -> Self {
        handle.into_raw()
    }
}
