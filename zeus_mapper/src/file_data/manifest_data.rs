use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::to_usize;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, Default, LogDifferences)]
pub struct ManifestData {
    pub compressed: u32,
    pub address: u32,
    pub size: u32,
    pub count: u32,
    pub unknown: u32,
}

impl ReadFrom for ManifestData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(ManifestData {
            compressed: ReadFrom::read_from(reader)?,
            address: ReadFrom::read_from(reader)?,
            size: ReadFrom::read_from(reader)?,
            count: ReadFrom::read_from(reader)?,
            unknown: ReadFrom::read_from(reader)?,
        });
    }
}

pub fn read_segment(reader: &mut impl Read, manifest_element: &ManifestData) -> io::Result<Vec<Vec<u8>>> {
    let mut result = Vec::with_capacity(to_usize(manifest_element.count)?);

    for _ in 0..manifest_element.count {
        let mut row = vec![0; to_usize(manifest_element.size)?];
        reader.read_exact(row.as_mut_slice())?;
        result.push(row);
    }

    return Ok(result);
}

impl WriteTo for ManifestData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.compressed, writer)?;
        bytes += WriteTo::write_to(&self.address, writer)?;
        bytes += WriteTo::write_to(&self.size, writer)?;
        bytes += WriteTo::write_to(&self.count, writer)?;
        bytes += WriteTo::write_to(&self.unknown, writer)?;
        return Ok(bytes);
    }
}
