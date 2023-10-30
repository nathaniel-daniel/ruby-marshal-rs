use crate::Error;
use crate::ValueArena;
use crate::ValueHandle;
use std::collections::HashSet;

/// Implemented for any type that can be created from a Ruby Value.
pub trait FromValue: Sized {
    /// Create this type from the given value from the [`ValueArena`].
    ///
    /// # Arguments
    /// 1. `arena`: The arena where the value to convert from is stored.
    /// 2. `handle`: The handle that points to the value to convert.
    /// 3. `visited`: A set of already-visited values, to prevent cycles.
    fn from_value(
        arena: &ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, Error>;
}

/*
impl FromValue for NilValue {
    fn from_value(
        arena: &ValueArena,
        handle: ValueHandle,
        visited: &mut HashSet<ValueHandle>,
    ) -> Result<Self, Error> {
        todo!()
    }
}
*/

/// Implemented for any type that can be converted into a Ruby Value.
pub trait IntoValue: Sized {
    /// Turn this type into a Ruby Value.
    fn into_value(self, arena: &mut ValueArena) -> Result<ValueHandle, Error>;
}

#[cfg(test)]
mod test {
    // use super::*;

    #[test]
    fn sanity() {
        // let arena = ValueArena::new();
        // let mut visited = HashSet::new();

        //let nil = NilValue::from_value(&arena, arena.root(), &mut visited).expect("failed to extract nil");
        //dbg!(nil);
    }
}
