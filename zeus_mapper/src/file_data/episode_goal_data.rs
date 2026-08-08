use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

/// One 76-byte "set aside goods" goal slot from `SettingsData.parent_episode_goals`/
/// `SettingsData.colony_goals`.
///
/// **Assumptions**: `goal_type == 14` is the only confirmed value (`SetAsideGoods`); other goal
/// kinds aren't decoded yet - see `DATA_MAPPING.md`.
#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct EpisodeGoalData {
    pub goal_type: u32,
    pub resource_id: u32,
    pub amount: u32,
    pub field_4: BoxedArray<u8, 64>,
}

impl ReadFrom for EpisodeGoalData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(EpisodeGoalData {
            goal_type: ReadFrom::read_from(reader)?,
            resource_id: ReadFrom::read_from(reader)?,
            amount: ReadFrom::read_from(reader)?,
            field_4: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for EpisodeGoalData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.goal_type, writer)?;
        bytes += WriteTo::write_to(&self.resource_id, writer)?;
        bytes += WriteTo::write_to(&self.amount, writer)?;
        bytes += WriteTo::write_to(&self.field_4, writer)?;

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
            field_4: BoxedArray::from_vec((0..64).collect()),
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = EpisodeGoalData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }
}
