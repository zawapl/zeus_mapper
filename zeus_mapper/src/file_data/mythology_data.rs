use crate::file_data::pyramid_data::PyramidData;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_vec_from;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct MythologyData {
    pub opponent_gods: [u32; 12],
    pub proponent_gods: [u32; 12],
    pub monster: u32,
    pub buffer_0xff: BoxedArray<u8, 96>,
    pub buffer_0x00: [u8; 12],
    pub sanctuaries_allowed: [u8; 12],
    pub max_sanctuaries: u32,
    pub max_pyramids: u32,
    pub pyramids: Vec<PyramidData>,
}

impl MythologyData {
    pub(crate) fn read_arr_from<const N: usize>(reader: &mut impl Read, include_pyramids: bool) -> io::Result<[Self; N]> {
        let mut result = Vec::with_capacity(N);

        for _ in 0..N {
            result.push(Self::read_from(reader, include_pyramids)?);
        }

        return result.try_into().map_err(|_| Error::from(ErrorKind::InvalidData));
    }

    pub(crate) fn read_from(reader: &mut impl Read, include_pyramids: bool) -> io::Result<Self> {
        let pyramids_count = if include_pyramids { 6 } else { 0 };

        return Ok(MythologyData {
            opponent_gods: ReadFrom::read_from(reader)?,
            proponent_gods: ReadFrom::read_from(reader)?,
            monster: ReadFrom::read_from(reader)?,
            buffer_0xff: ReadFrom::read_from(reader)?,
            buffer_0x00: ReadFrom::read_from(reader)?,
            sanctuaries_allowed: ReadFrom::read_from(reader)?,
            max_sanctuaries: ReadFrom::read_from(reader)?,
            max_pyramids: if include_pyramids { ReadFrom::read_from(reader)? } else { 0 },
            pyramids: read_vec_from(reader, pyramids_count)?,
        });
    }

    #[allow(dead_code)]
    pub(crate) fn write_arr_to<W: Write, const N: usize>(
        data: &[MythologyData; N],
        writer: &mut W,
        include_pyramids: bool,
    ) -> io::Result<usize> {
        let mut bytes = 0;

        for mythology_data in data {
            bytes += Self::write_to(mythology_data, writer, include_pyramids)?;
        }

        return Ok(bytes);
    }
    pub(crate) fn write_to<W: Write>(&self, writer: &mut W, include_pyramids: bool) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.opponent_gods, writer)?;
        bytes += WriteTo::write_to(&self.proponent_gods, writer)?;
        bytes += WriteTo::write_to(&self.monster, writer)?;
        bytes += WriteTo::write_to(&self.buffer_0xff, writer)?;
        bytes += WriteTo::write_to(&self.buffer_0x00, writer)?;
        bytes += WriteTo::write_to(&self.sanctuaries_allowed, writer)?;
        bytes += WriteTo::write_to(&self.max_sanctuaries, writer)?;

        if include_pyramids {
            bytes += WriteTo::write_to(&self.max_pyramids, writer)?;
            bytes += WriteTo::write_to(&self.pyramids, writer)?;
        }

        return Ok(bytes);
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(offset) = self.buffer_0xff.as_ref().iter().position(|b| *b != 0xFF) {
            return Err(format!("buffer_0xff[{offset}] is not 0xFF"));
        }

        if let Some(offset) = self.buffer_0x00.iter().position(|b| *b != 0) {
            return Err(format!("buffer_0x00[{offset}] is non-zero"));
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write_without_pyramids() -> io::Result<()> {
        let original = MythologyData {
            opponent_gods: [1; 12],
            proponent_gods: [2; 12],
            monster: 3,
            buffer_0xff: BoxedArray::from_vec(vec![4; 96]),
            buffer_0x00: [5; 12],
            sanctuaries_allowed: [6; 12],
            max_sanctuaries: 7,
            max_pyramids: 0,
            pyramids: vec![],
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer, false)?;

        let deserialized = MythologyData::read_from(&mut Cursor::new(buffer), false)?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    #[test]
    fn read_write_with_pyramids() -> io::Result<()> {
        let original = MythologyData {
            opponent_gods: [1; 12],
            proponent_gods: [2; 12],
            monster: 3,
            buffer_0xff: BoxedArray::from_vec(vec![4; 96]),
            buffer_0x00: [5; 12],
            sanctuaries_allowed: [6; 12],
            max_sanctuaries: 7,
            max_pyramids: 6,
            pyramids: (7..13).map(pyramid).collect(),
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer, true)?;

        let deserialized = MythologyData::read_from(&mut Cursor::new(buffer), true)?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    fn pyramid(pyramid_type: u32) -> PyramidData {
        return PyramidData {
            pyramid_type,
            deity: pyramid_type + 1,
            coloration: pyramid_type + 2,
        };
    }
}
