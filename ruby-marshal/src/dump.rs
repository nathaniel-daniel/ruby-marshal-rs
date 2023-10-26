use crate::Error;
use crate::Value;
use crate::ValueArena;
use crate::ValueHandle;
use crate::MAJOR_VERSION;
use crate::MINOR_VERSION;
use crate::VALUE_KIND_FALSE;
use crate::VALUE_KIND_NIL;
use crate::VALUE_KIND_TRUE;
use std::io::Write;

/// A dumper for ruby data
pub struct Dumper<'a, W> {
    writer: W,
    arena: &'a ValueArena,
}

impl<'a, W> Dumper<'a, W> {
    /// Create a new [`Dumper`] from a writer and entry arena.
    fn new(writer: W, arena: &'a ValueArena) -> Self {
        Self { writer, arena }
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
