use crate::file_data::basic_episode_data::BasicEpisodeData;
use crate::file_data::episode_goal_data::EpisodeGoalData;
use crate::file_data::event_data::EventData;
use crate::file_data::map_data::MapData;
use crate::file_data::mythology_data::MythologyData;
use crate::file_data::real_episode_data::RealEpisodeData;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_bytes_to_end;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct SettingsData {
    pub version_1: u32,
    pub version_2: u32,
    pub parent_episodes: u32,
    pub colony_episodes_used: u32,
    pub colony_episodes_available: u32,
    pub basic_episode_data: [BasicEpisodeData; 20],
    pub real_episode_data: [RealEpisodeData; 14],
    pub field_4: BoxedArray<u32, 37>,
    pub mythology: [MythologyData; 14], // 300 or 224
    pub events: BoxedArray<BoxedArray<EventData, 150>, 14>,
    pub field_8: [u8; 5],
    pub data_length: u32,
    pub field_9: [u32; 4],
    pub map_data: MapData,
    pub padding: Vec<u8>, // Should not contain useful data, we should be able to remove this in the future
    pub parent_event_counts: [u32; 10],
    pub colony_event_counts: [u8; 3],
    pub unused_blocks: BoxedArray<u8, 31_693>, // "10 different blocks of data, all apparently never used" per notes.md
    pub parent_city_favor: [u32; 10],
    pub field_11: BoxedArray<u8, 4112>, // still otherwise-undeciphered; see DATA_MAPPING.md
    pub bitmap: u32,
    pub field_13: BoxedArray<u8, 234>, // still otherwise-undeciphered; see DATA_MAPPING.md
    pub field_14: u32,                 // only present for new file format; see DATA_MAPPING.md
    pub colony_episode_goal_counts: [u32; 4],
    pub colony_episode_goals: [[EpisodeGoalData; 6]; 4],
    pub parent_episode_goal_counts: [u32; 10],
    pub parent_episode_goals: [[EpisodeGoalData; 6]; 10],
}

impl SettingsData {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return ReadFrom::read_from(reader);
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        return WriteTo::write_to(self, writer);
    }
}

impl ReadFrom for SettingsData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let version_1 = ReadFrom::read_from(reader)?;
        let version_2 = ReadFrom::read_from(reader)?;

        let new_file_ver = version_2 == 26;

        let end_position = if new_file_ver { 842_931 } else { 841_863 }; // should match data_length field

        let event_counts_offset = if new_file_ver { 800_361 } else { 799_297 };

        // limited length readers for different sections, the -8 accounts for the versions which were already read
        let mut limited_reader = reader.take(end_position - 8);
        let mut map_data_reader = (&mut limited_reader).take(event_counts_offset - 8);

        return Ok(SettingsData {
            version_1,
            version_2,
            parent_episodes: ReadFrom::read_from(&mut map_data_reader)?,
            colony_episodes_used: ReadFrom::read_from(&mut map_data_reader)?,
            colony_episodes_available: ReadFrom::read_from(&mut map_data_reader)?,
            basic_episode_data: ReadFrom::read_from(&mut map_data_reader)?,
            real_episode_data: ReadFrom::read_from(&mut map_data_reader)?,
            field_4: ReadFrom::read_from(&mut map_data_reader)?,
            mythology: MythologyData::read_arr_from(&mut map_data_reader, new_file_ver)?,
            events: ReadFrom::read_from(&mut map_data_reader)?,
            field_8: ReadFrom::read_from(&mut map_data_reader)?,
            data_length: ReadFrom::read_from(&mut map_data_reader)?,
            field_9: ReadFrom::read_from(&mut map_data_reader)?,
            map_data: MapData::read_from(&mut map_data_reader)?,
            padding: read_bytes_to_end(&mut map_data_reader)?,
            parent_event_counts: ReadFrom::read_from(&mut limited_reader)?,
            colony_event_counts: ReadFrom::read_from(&mut limited_reader)?,
            unused_blocks: ReadFrom::read_from(&mut limited_reader)?,
            parent_city_favor: ReadFrom::read_from(&mut limited_reader)?,
            field_11: ReadFrom::read_from(&mut limited_reader)?,
            bitmap: ReadFrom::read_from(&mut limited_reader)?,
            field_13: ReadFrom::read_from(&mut limited_reader)?,
            field_14: if new_file_ver {
                ReadFrom::read_from(&mut limited_reader)?
            } else {
                0
            },
            colony_episode_goal_counts: ReadFrom::read_from(&mut limited_reader)?,
            colony_episode_goals: ReadFrom::read_from(&mut limited_reader)?,
            parent_episode_goal_counts: ReadFrom::read_from(&mut limited_reader)?,
            parent_episode_goals: ReadFrom::read_from(&mut limited_reader)?,
        });
    }
}

impl WriteTo for SettingsData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let new_file_ver = self.version_2 == 26;
        let event_counts_offset: usize = if new_file_ver { 800_361 } else { 799_297 };

        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.version_1, writer)?;
        bytes += WriteTo::write_to(&self.version_2, writer)?;
        bytes += WriteTo::write_to(&self.parent_episodes, writer)?;
        bytes += WriteTo::write_to(&self.colony_episodes_used, writer)?;
        bytes += WriteTo::write_to(&self.colony_episodes_available, writer)?;
        bytes += WriteTo::write_to(&self.basic_episode_data, writer)?;
        bytes += WriteTo::write_to(&self.real_episode_data, writer)?;
        bytes += WriteTo::write_to(&self.field_4, writer)?;
        bytes += MythologyData::write_arr_to(&self.mythology, writer, new_file_ver)?;
        bytes += WriteTo::write_to(&self.events, writer)?;
        bytes += WriteTo::write_to(&self.field_8, writer)?;
        bytes += WriteTo::write_to(&self.data_length, writer)?;
        bytes += WriteTo::write_to(&self.field_9, writer)?;
        bytes += WriteTo::write_to(&self.map_data, writer)?;

        if bytes > event_counts_offset {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "SettingsData: map_data overflowing to parent_event_counts",
            ));
        }

        // Write the real `padding` content back  truncating/zero-extending it to fit the space actually available.
        // In the future should be possible to just write 0s (we need the original for byte checking tests)
        let padding_len = event_counts_offset - bytes;
        let mut padding_buf = vec![0u8; padding_len];
        let copy_len = usize::min(padding_len, self.padding.len());
        padding_buf[..copy_len].copy_from_slice(&self.padding[..copy_len]);
        bytes += WriteTo::write_to(&padding_buf, writer)?;

        bytes += WriteTo::write_to(&self.parent_event_counts, writer)?;
        bytes += WriteTo::write_to(&self.colony_event_counts, writer)?;
        bytes += WriteTo::write_to(&self.unused_blocks, writer)?;
        bytes += WriteTo::write_to(&self.parent_city_favor, writer)?;
        bytes += WriteTo::write_to(&self.field_11, writer)?;
        bytes += WriteTo::write_to(&self.bitmap, writer)?;
        bytes += WriteTo::write_to(&self.field_13, writer)?;
        if new_file_ver {
            bytes += WriteTo::write_to(&self.field_14, writer)?;
        }
        bytes += WriteTo::write_to(&self.colony_episode_goal_counts, writer)?;
        bytes += WriteTo::write_to(&self.colony_episode_goals, writer)?;
        bytes += WriteTo::write_to(&self.parent_episode_goal_counts, writer)?;
        bytes += WriteTo::write_to(&self.parent_episode_goals, writer)?;

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use crate::file_data::settings_data::SettingsData;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;
    use std::io::Seek;
    use std::io::SeekFrom;

    #[test]
    fn test_set_files() -> Result<()> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

        let adventures_folder = fs::read_dir(format!("{}/Adventures", game_root))?;

        let mut files_tested = 0;

        for adventure in adventures_folder {
            let folder_path = adventure?.path();
            if folder_path.is_dir() {
                let files = fs::read_dir(folder_path)?;
                for file in files {
                    let file_path = file?.path();
                    let extension = file_path.extension().map_or("", |x| x.to_str().unwrap_or(""));
                    if extension == "set" {
                        let mut reader = File::open(&file_path).map(BufReader::new)?;
                        let settings_data = SettingsData::read_from(&mut reader)?;

                        let current_pos = reader.stream_position()?;
                        reader.seek(SeekFrom::End(0))?;
                        let end_pos = reader.stream_position()?;
                        let unread = end_pos - current_pos;

                        assert_eq!(unread, 0);

                        assert_eq!(settings_data.version_1, 8871);

                        files_tested += 1;
                    }
                }
            }
        }

        assert!(files_tested >= 8, "Files checked = {files_tested}");

        return Ok(());
    }
}
