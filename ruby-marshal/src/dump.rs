use crate::Error;
use crate::SymbolValue;
use crate::TypedValueHandle;
use crate::Value;
use crate::ValueArena;
use crate::ValueHandle;
use crate::MAJOR_VERSION;
use crate::MINOR_VERSION;
use crate::VALUE_KIND_ARRAY;
use crate::VALUE_KIND_FALSE;
use crate::VALUE_KIND_FIXNUM;
use crate::VALUE_KIND_NIL;
use crate::VALUE_KIND_OBJECT_LINK;
use crate::VALUE_KIND_STRING;
use crate::VALUE_KIND_SYMBOL;
use crate::VALUE_KIND_SYMBOL_LINK;
use crate::VALUE_KIND_TRUE;
use indexmap::IndexSet;
use std::io::Write;

/// A dumper for ruby data
pub struct Dumper<'a, W> {
    writer: W,
    arena: &'a ValueArena,

    symbol_links: IndexSet<TypedValueHandle<SymbolValue>>,
    object_links: IndexSet<ValueHandle>,
}

impl<'a, W> Dumper<'a, W> {
    /// Create a new [`Dumper`] from a writer and entry arena.
    fn new(writer: W, arena: &'a ValueArena) -> Self {
        Self {
            writer,
            arena,
            symbol_links: IndexSet::new(),
            object_links: IndexSet::new(),
        }
    }
}

impl<'a, W> Dumper<'a, W>
where
    W: Write,
{
    /// Write the header
    fn write_header(&mut self) -> Result<(), Error> {
        self.writer.write_all(&[MAJOR_VERSION, MINOR_VERSION])?;
        Ok(())
    }

    /// Write a byte
    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        self.writer.write_all(std::slice::from_ref(&byte))?;
        Ok(())
    }

    /// Write a byte string.
    ///
    /// This is a fixnum, followed by that many bytes.
    fn write_byte_string(&mut self, value: &[u8]) -> Result<(), Error> {
        let len =
            i32::try_from(value.len()).map_err(|error| Error::USizeInvalidFixnum { error })?;

        self.write_fixnum(len)?;
        self.writer.write_all(value)?;

        Ok(())
    }

    /// Write a Fixnum
    fn write_fixnum(&mut self, mut value: i32) -> Result<(), Error> {
        if value == 0 {
            self.writer.write_all(&[0])?;
            return Ok(());
        }

        if value > 0 && value < 123 {
            let value = u8::try_from(value).unwrap();
            self.writer.write_all(&[value + 5])?;
            return Ok(());
        }

        if value < 0 && value > -124 {
            let value = u8::try_from((value - 5) & 0xFF).unwrap();
            self.writer.write_all(&[value])?;
            return Ok(());
        }

        let mut buffer = [0; std::mem::size_of::<i32>() + 1];
        let mut buffer_size = 0;
        for i in 1..(std::mem::size_of::<i32>() + 1) {
            buffer[i] = u8::try_from(value & 0xFF).unwrap();
            buffer_size = i + 1;

            value >>= 8;
            if value == 0 {
                buffer[0] = u8::try_from(i).unwrap();
                break;
            }
            if value == -1 {
                buffer[0] = (-i8::try_from(i).unwrap()) as u8;
                break;
            }
        }
        self.writer.write_all(&buffer[..buffer_size])?;

        Ok(())
    }

    /// Try to write a value object reference, if possible.
    /// If not successful, this entry is recorded and will be used for future resolutions.
    ///
    /// # Returns
    /// Returns true if successful.
    fn try_write_value_object_link(&mut self, handle: ValueHandle) -> Result<bool, Error> {
        let (index, inserted) = self.object_links.insert_full(handle);
        if !inserted {
            self.write_value_object_link(index)?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Write an object link, as a value.
    fn write_value_object_link(&mut self, index: usize) -> Result<(), Error> {
        let index = i32::try_from(index).map_err(|error| Error::USizeInvalidFixnum { error })?;

        self.write_byte(VALUE_KIND_OBJECT_LINK)?;
        self.write_fixnum(index)?;

        Ok(())
    }

    /// Write a value
    fn write_value(&mut self, handle: ValueHandle) -> Result<(), Error> {
        let value = self
            .arena
            .get(handle)
            .ok_or(Error::InvalidValueHandle { handle })?;

        match value {
            Value::Nil(_) => {
                self.write_byte(VALUE_KIND_NIL)?;
            }
            Value::True(_) => {
                self.write_byte(VALUE_KIND_TRUE)?;
            }
            Value::False(_) => {
                self.write_byte(VALUE_KIND_FALSE)?;
            }
            Value::Fixnum(value) => {
                self.write_byte(VALUE_KIND_FIXNUM)?;
                self.write_fixnum(value.value())?;
            }
            Value::Symbol(value) => {
                let handle = TypedValueHandle::new_unchecked(handle);

                match self.symbol_links.get_index_of(&handle) {
                    Some(index) => {
                        let index = i32::try_from(index)
                            .map_err(|error| Error::USizeInvalidFixnum { error })?;

                        self.write_byte(VALUE_KIND_SYMBOL_LINK)?;
                        self.write_fixnum(index)?;
                    }
                    None => {
                        self.symbol_links.insert(handle);

                        self.write_byte(VALUE_KIND_SYMBOL)?;
                        self.write_byte_string(value.value())?;
                    }
                }
            }
            Value::Array(value) => {
                if self.try_write_value_object_link(handle)? {
                    return Ok(());
                }

                let len = i32::try_from(value.len())
                    .map_err(|error| Error::USizeInvalidFixnum { error })?;

                self.write_byte(VALUE_KIND_ARRAY)?;
                self.write_fixnum(len)?;
                for value in value.value().iter() {
                    self.write_value(*value)?;
                }
            }
            Value::String(value) => {
                if self.try_write_value_object_link(handle)? {
                    return Ok(());
                }

                self.write_byte(VALUE_KIND_STRING)?;
                self.write_byte_string(value.value())?;
            }
        }

        Ok(())
    }

    /// Dump the root node to the writer.
    fn dump(&mut self) -> Result<(), Error> {
        self.write_header()?;
        self.write_value(self.arena.root())?;

        Ok(())
    }
}

/// Dump to a writer.
pub fn dump<W>(writer: W, value_arena: &ValueArena) -> Result<(), Error>
where
    W: Write,
{
    let mut dumper = Dumper::new(writer, value_arena);
    dumper.dump()?;
    Ok(())
}
