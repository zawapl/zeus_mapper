use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Default, Debug, LogDifferences)]
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
