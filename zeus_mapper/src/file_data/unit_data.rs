use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

/// One entry of a `.sav` file's unit table.
///
/// Record shape and `type_id`/`x`/`y` offsets checked against a real save's data (not just that
/// reading succeeds): `type_id` values are large/mostly-unique until masked with `& 0xff`, after
/// which they collapse into a small, repeating set - consistent with a real type enumeration; see
/// `docs/sav_investigation.md`. The masking itself, and the meaning of the unmasked high byte, are
/// not implemented/decoded here yet.
#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct UnitData {
    pub unknown_1: BoxedArray<u8, 10>,
    pub type_id: u16,
    pub unknown_2: BoxedArray<u8, 8>,
    pub x: u16,
    pub y: u16,
    pub unknown_3: BoxedArray<u8, 364>,
}

impl ReadFrom for UnitData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(UnitData {
            unknown_1: ReadFrom::read_from(reader)?,
            type_id: ReadFrom::read_from(reader)?,
            unknown_2: ReadFrom::read_from(reader)?,
            x: ReadFrom::read_from(reader)?,
            y: ReadFrom::read_from(reader)?,
            unknown_3: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for UnitData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.type_id, writer)?;
        bytes += WriteTo::write_to(&self.unknown_2, writer)?;
        bytes += WriteTo::write_to(&self.x, writer)?;
        bytes += WriteTo::write_to(&self.y, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_write() -> io::Result<()> {
        let original = UnitData {
            type_id: 0x43,
            x: 12,
            y: 34,
            ..Default::default()
        };

        let mut buffer = vec![];
        original.write_to(&mut buffer)?;
        assert_eq!(buffer.len(), 388);

        let deserialized = UnitData::read_from(&mut Cursor::new(buffer))?;
        assert_eq!(original, deserialized);

        return Ok(());
    }
}
