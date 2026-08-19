use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use std::io;
use std::io::Read;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;

/// A raw value read from a file whose real signedness has not been confirmed against the real
/// game/editor yet - e.g. `UnconfirmedSign<u16>` for a field that might actually be `i16`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct UnconfirmedSign<T>(pub T);

impl<T> Deref for UnconfirmedSign<T> {
    type Target = T;

    fn deref(&self) -> &T {
        return &self.0;
    }
}

impl<T> DerefMut for UnconfirmedSign<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl<T: ReadFrom> ReadFrom for UnconfirmedSign<T> {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(UnconfirmedSign(ReadFrom::read_from(reader)?));
    }
}

impl<T: WriteTo> WriteTo for UnconfirmedSign<T> {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        return WriteTo::write_to(&self.0, writer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write() -> io::Result<()> {
        let original = UnconfirmedSign(40000u16);

        let mut buffer = vec![];
        original.write_to(&mut buffer)?;

        let deserialized = UnconfirmedSign::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    #[test]
    fn derefs_to_the_wrapped_value() {
        let value = UnconfirmedSign(40000u16);

        assert_eq!(*value, 40000);
        assert_eq!(*value as i16, 40000u16 as i16);
    }
}
