use crate::Error;
use crate::Value;
use crate::ValueArena;
use crate::ValueHandle;
use crate::MAJOR_VERSION;
use crate::MINOR_VERSION;
use crate::VALUE_KIND_FALSE;
use crate::VALUE_KIND_FIXNUM;
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

    fn write_fixnum(&mut self, value: i32) -> Result<(), Error> {
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

        todo!("write fixnum {value}");
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
