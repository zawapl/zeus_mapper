use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct PyramidData {
    pub pyramid_type: u32,
    pub deity: u32,
    pub coloration: u32,
}

impl ReadFrom for PyramidData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(PyramidData {
            pyramid_type: ReadFrom::read_from(reader)?,
            deity: ReadFrom::read_from(reader)?,
            coloration: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for PyramidData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.pyramid_type, writer)?;
        bytes += WriteTo::write_to(&self.deity, writer)?;
        bytes += WriteTo::write_to(&self.coloration, writer)?;
        return Ok(bytes);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write() -> io::Result<()> {
        let original = PyramidData {
            pyramid_type: 1,
            deity: 2,
            coloration: 3,
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = PyramidData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }
}
