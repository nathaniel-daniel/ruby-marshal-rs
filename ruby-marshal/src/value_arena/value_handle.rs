/// A handle around a Ruby Value.
#[derive(Debug, Copy, Clone)]
pub struct ValueHandle {
    /// The arena index
    pub(super) index: generational_arena::Index,
    // TODO: Should we an a counter to the arena itself to prevent accidental accesses to other arenas?
}

impl ValueHandle {
    /// Create a new [`ValueHandle`] from an Index
    pub(super) fn new(index: generational_arena::Index) -> Self {
        Self { index }
    }
}
