use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct EpisodeGoalData {
    pub goal_type: u32,
    pub resource_id: i32,
    pub amount: u32,
    pub goal_extra: BoxedArray<u8, 64>, // todo extract the 2 used named fields from it
}

impl ReadFrom for EpisodeGoalData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(EpisodeGoalData {
            goal_type: ReadFrom::read_from(reader)?,
            resource_id: ReadFrom::read_from(reader)?,
            amount: ReadFrom::read_from(reader)?,
            goal_extra: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for EpisodeGoalData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.goal_type, writer)?;
        bytes += WriteTo::write_to(&self.resource_id, writer)?;
        bytes += WriteTo::write_to(&self.amount, writer)?;
        bytes += WriteTo::write_to(&self.goal_extra, writer)?;

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
        let original = EpisodeGoalData {
            goal_type: 14,
            resource_id: 17,
            amount: 8,
            goal_extra: BoxedArray::from_vec((0..64).collect()),
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = EpisodeGoalData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }
}
