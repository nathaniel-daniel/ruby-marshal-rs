mod load;
mod value_arena;

pub use self::load::load;
pub use self::value_arena::Value;
pub use self::value_arena::ValueArena;
pub use self::value_arena::ValueHandle;

const MAJOR_VERSION: u8 = 4;
const MINOR_VERSION: u8 = 8;

const VALUE_KIND_NIL: u8 = b'0';

/// The library error type
#[derive(Debug)]
pub enum Error {
    /// Invalid version
    InvalidVersion {
        /// The major version
        major: u8,

        /// The minor version
        minor: u8,
    },

    /// An I/O Error
    Io(std::io::Error),

    /// An invalid value kind was encountered
    InvalidValueKind { kind: u8 },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidVersion { major, minor } => write!(f, "invalid version {major}.{minor}"),
            Self::Io(_error) => write!(f, "I/O error"),
            Self::InvalidValueKind { kind } => write!(f, "invalid value kind {}", kind),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn kitchen_sink() {
        for entry in std::fs::read_dir("test_data").expect("failed to read \"test_data\"") {
            let entry = entry.expect("failed to read entry");
            let data = std::fs::read(entry.path()).expect("failed to read entry");

            let value_arena = load(std::io::Cursor::new(&data)).expect("failed to load");
            dbg!(value_arena.get(value_arena.root()));
        }
    }
}
