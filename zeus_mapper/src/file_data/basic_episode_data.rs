use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, LogDifferences)]
pub struct BasicEpisodeData {
    pub exists: u32,
    pub field_2: u32,
    pub field_3: u32,
    pub episode_no: u32,
    pub field_5: u32,
    pub field_6: u32,
    pub field_7: [u8; 60],
    pub next_episode: u32,
    pub episode_type: u32,
    pub name_padding: u8,
    pub name: String,
}

impl ReadFrom for BasicEpisodeData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(BasicEpisodeData {
            exists: ReadFrom::read_from(reader)?,
            field_2: ReadFrom::read_from(reader)?,
            field_3: ReadFrom::read_from(reader)?,
            episode_no: ReadFrom::read_from(reader)?,
            field_5: ReadFrom::read_from(reader)?,
            field_6: ReadFrom::read_from(reader)?,
            field_7: ReadFrom::read_from(reader)?,
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
        bytes += WriteTo::write_to(&self.field_2, writer)?;
        bytes += WriteTo::write_to(&self.field_3, writer)?;
        bytes += WriteTo::write_to(&self.episode_no, writer)?;
        bytes += WriteTo::write_to(&self.field_5, writer)?;
        bytes += WriteTo::write_to(&self.field_6, writer)?;
        bytes += WriteTo::write_to(&self.field_7, writer)?;
        bytes += WriteTo::write_to(&self.next_episode, writer)?;
        bytes += WriteTo::write_to(&self.episode_type, writer)?;
        bytes += WriteTo::write_to(&self.name_padding, writer)?;
        bytes += write_string_to(&self.name, writer, 263)?;

        return Ok(bytes);
    }
}

impl Default for BasicEpisodeData {
    fn default() -> Self {
        return BasicEpisodeData {
            exists: 0,
            field_2: 0,
            field_3: 0,
            episode_no: 0,
            field_5: 0,
            field_6: 0,
            field_7: [0; 60],
            next_episode: 0,
            episode_type: 0,
            name_padding: 0,
            name: "".to_string(),
        };
    }
}
