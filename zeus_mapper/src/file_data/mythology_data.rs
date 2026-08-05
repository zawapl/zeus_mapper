use crate::file_data::pyramid_data::PyramidData;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_vec_from;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

#[derive(Debug, LogDifferences)]
pub struct MythologyData {
    pub opponent_gods: [u32; 12],
    pub proponent_gods: [u32; 12],
    pub monster: u32,
    pub field_4: [u8; 96],
    pub field_5: [u8; 12],
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
            field_4: ReadFrom::read_from(reader)?,
            field_5: ReadFrom::read_from(reader)?,
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
        bytes += WriteTo::write_to(&self.field_4, writer)?;
        bytes += WriteTo::write_to(&self.field_5, writer)?;
        bytes += WriteTo::write_to(&self.sanctuaries_allowed, writer)?;
        bytes += WriteTo::write_to(&self.max_sanctuaries, writer)?;

        if include_pyramids {
            bytes += WriteTo::write_to(&self.max_pyramids, writer)?;
            bytes += WriteTo::write_to(&self.pyramids, writer)?;
        }

        return Ok(bytes);
    }
}

impl Default for MythologyData {
    fn default() -> Self {
        return MythologyData {
            opponent_gods: [0; 12],
            proponent_gods: [0; 12],
            monster: 0,
            field_4: [0; 96],
            field_5: [0; 12],
            sanctuaries_allowed: [0; 12],
            max_sanctuaries: 0,
            max_pyramids: 0,
            pyramids: vec![],
        };
    }
}
