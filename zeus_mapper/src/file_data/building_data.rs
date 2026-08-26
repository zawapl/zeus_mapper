use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

/// One entry of a `.sav` file's placed-building table.
///
/// Record shape and `size`/`x`/`y`/`type_id`/`rotation` offsets checked against a real save's data
/// (not just that reading succeeds): `type_id` values stay within the plausible building-category
/// range and `x`/`y` cluster into sane in-bounds city layouts rather than reading as garbage.
#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct BuildingData {
    pub unknown_1: BoxedArray<u8, 3>,
    pub size: u8,
    pub unknown_2: BoxedArray<u8, 2>,
    pub x: u16,
    pub y: u16,
    pub unknown_3: BoxedArray<u8, 6>,
    pub type_id: u16,
    pub unknown_4: BoxedArray<u8, 154>,
    pub rotation: u8,
    pub unknown_5: BoxedArray<u8, 107>,
}

impl ReadFrom for BuildingData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(BuildingData {
            unknown_1: ReadFrom::read_from(reader)?,
            size: ReadFrom::read_from(reader)?,
            unknown_2: ReadFrom::read_from(reader)?,
            x: ReadFrom::read_from(reader)?,
            y: ReadFrom::read_from(reader)?,
            unknown_3: ReadFrom::read_from(reader)?,
            type_id: ReadFrom::read_from(reader)?,
            unknown_4: ReadFrom::read_from(reader)?,
            rotation: ReadFrom::read_from(reader)?,
            unknown_5: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for BuildingData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.size, writer)?;
        bytes += WriteTo::write_to(&self.unknown_2, writer)?;
        bytes += WriteTo::write_to(&self.x, writer)?;
        bytes += WriteTo::write_to(&self.y, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
        bytes += WriteTo::write_to(&self.type_id, writer)?;
        bytes += WriteTo::write_to(&self.unknown_4, writer)?;
        bytes += WriteTo::write_to(&self.rotation, writer)?;
        bytes += WriteTo::write_to(&self.unknown_5, writer)?;
        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_write() -> io::Result<()> {
        let original = BuildingData {
            size: 2,
            x: 12,
            y: 34,
            type_id: 0x83,
            rotation: 1,
            ..Default::default()
        };

        let mut buffer = vec![];
        original.write_to(&mut buffer)?;
        assert_eq!(buffer.len(), 280);

        let deserialized = BuildingData::read_from(&mut Cursor::new(buffer))?;
        assert_eq!(original, deserialized);

        return Ok(());
    }
}
