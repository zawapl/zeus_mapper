use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::validation::ValidationError;
use crate::utils::validation::ValidationResult;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct BasicEpisodeData {
    pub exists: u32,
    pub constant_1_0x00: u32,
    pub unknown_1: u32,
    pub episode_no: u32,
    // Always exactly 0 or exactly `u32::MAX` in every real sample observed so far, never any other
    // value - usually (but not always; see the old-format "The Voyages of Jason" counterexample,
    // `basic_episode_data[0..2]`) `u32::MAX` when `exists != 0`. Meaning otherwise undeciphered.
    pub unknown_2: u32,
    pub unknown_3: u32,
    pub unknown_4: BoxedArray<u8, 60>,
    pub next_episode: u32,
    pub episode_type: u32,
    pub name_padding: u8,
    pub name: String,
}

impl BasicEpisodeData {
    pub fn validate(&self) -> ValidationResult {
        if self.constant_1_0x00 != 0 {
            return Err(ValidationError::expected_exactly("constant_1_0x00", self.constant_1_0x00, 0));
        }

        if self.unknown_2 != 0 && self.unknown_2 != u32::MAX {
            return Err(ValidationError::expected_one_of("unknown_2", self.unknown_2, &[0, u32::MAX]));
        }

        return Ok(());
    }
}

impl ReadFrom for BasicEpisodeData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(BasicEpisodeData {
            exists: ReadFrom::read_from(reader)?,
            constant_1_0x00: ReadFrom::read_from(reader)?,
            unknown_1: ReadFrom::read_from(reader)?,
            episode_no: ReadFrom::read_from(reader)?,
            unknown_2: ReadFrom::read_from(reader)?,
            unknown_3: ReadFrom::read_from(reader)?,
            unknown_4: ReadFrom::read_from(reader)?,
            next_episode: ReadFrom::read_from(reader)?,
            episode_type: ReadFrom::read_from(reader)?,
            name_padding: ReadFrom::read_from(reader)?,
            name: read_string_from(reader, 263)?,
        });
    }
}

impl WriteTo for BasicEpisodeData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.exists, writer)?;
        bytes += WriteTo::write_to(&self.constant_1_0x00, writer)?;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.episode_no, writer)?;
        bytes += WriteTo::write_to(&self.unknown_2, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
        bytes += WriteTo::write_to(&self.unknown_4, writer)?;
        bytes += WriteTo::write_to(&self.next_episode, writer)?;
        bytes += WriteTo::write_to(&self.episode_type, writer)?;
        bytes += WriteTo::write_to(&self.name_padding, writer)?;
        bytes += write_string_to(&self.name, writer, 263)?;

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
        let original = BasicEpisodeData {
            exists: 1,
            constant_1_0x00: 2,
            unknown_1: 3,
            episode_no: 4,
            unknown_2: 5,
            unknown_3: 6,
            unknown_4: BoxedArray::from_vec((0..60).collect()),
            next_episode: 7,
            episode_type: 8,
            name_padding: 9,
            name: "Test Episode".to_owned(),
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = BasicEpisodeData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }
}
