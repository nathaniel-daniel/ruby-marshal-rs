use crate::Error;
use crate::FixnumValue;
use crate::TypedValueHandle;
use crate::ValueArena;
use crate::ValueHandle;
use crate::MAJOR_VERSION;
use crate::MINOR_VERSION;
use crate::VALUE_KIND_FALSE;
use crate::VALUE_KIND_FIXNUM;
use crate::VALUE_KIND_NIL;
use crate::VALUE_KIND_TRUE;
use std::io::Read;

#[derive(Debug)]
struct Loader<R> {
    reader: R,

    arena: ValueArena,
}

impl<R> Loader<R> {
    /// Make a new [`Loader`] around a reader.
    fn new(reader: R) -> Self {
        let arena = ValueArena::new();

        Self { reader, arena }
    }
}

impl<R> Loader<R>
where
    R: Read,
{
    /// Read a byte
    fn read_byte(&mut self) -> Result<u8, Error> {
        let mut byte = 0;
        self.reader.read_exact(std::slice::from_mut(&mut byte))?;
        Ok(byte)
    }

    /// Read and validate the header.
    fn read_header(&mut self) -> Result<(), Error> {
        let major_version = self.read_byte()?;
        let minor_version = self.read_byte()?;

        if major_version != MAJOR_VERSION || minor_version > MINOR_VERSION {
            return Err(Error::InvalidVersion {
                major: major_version,
                minor: minor_version,
            });
        }

        Ok(())
    }

    fn read_fixnum_value(&mut self) -> Result<i32, Error> {
        let len = self.read_byte()?;
        if len == 0 {
            return Ok(0);
        }
        let positive = (len as i8) > 0;
        let byte = len;

        if positive {
            if byte > 4 {
                return Ok(i32::from(byte) - 5);
            }

            todo!("load fixnum {len}")
        } else {
            if (byte as i8) < -4 {
                return Ok(i32::from(byte as i8) + 5);
            }

            todo!("load negative fixnum")
        }
    }

    fn read_fixnum(&mut self) -> Result<TypedValueHandle<FixnumValue>, Error> {
        let value = self.read_fixnum_value()?;
        Ok(self.arena.create_fixnum(value))
    }

    /// Read the next value
    fn read_value(&mut self) -> Result<ValueHandle, Error> {
        let kind = self.read_byte()?;
        match kind {
            VALUE_KIND_NIL => Ok(self.arena.create_nil().into()),
            VALUE_KIND_TRUE => Ok(self.arena.create_true().into()),
            VALUE_KIND_FALSE => Ok(self.arena.create_false().into()),
            VALUE_KIND_FIXNUM => Ok(self.read_fixnum()?.into()),
            _ => Err(Error::InvalidValueKind { kind }),
        }
    }

    /// Load from the reader and get the value.
    fn load(mut self) -> Result<ValueArena, Error> {
        self.read_header()?;
        let root = self.read_value()?;
        let _old_root = self.arena.replace_root(root);

        // TODO: Delete old root.

        Ok(self.arena)
    }
}

/// Load from a reader.
pub fn load<R>(reader: R) -> Result<ValueArena, Error>
where
    R: Read,
{
    let loader = Loader::new(reader);
    let value_arena = loader.load()?;

    Ok(value_arena)
}
