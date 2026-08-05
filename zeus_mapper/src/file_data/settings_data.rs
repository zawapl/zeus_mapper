use crate::file_data::basic_episode_data::BasicEpisodeData;
use crate::file_data::event_data::EventData;
use crate::file_data::map_data::MapData;
use crate::file_data::mythology_data::MythologyData;
use crate::file_data::real_episode_data::RealEpisodeData;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_bytes_to_end;
use crate::utils::read_utils::read_vec_from;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

#[derive(Debug, LogDifferences)]
pub struct SettingsData {
    pub version_1: u32,
    pub version_2: u32,
    pub parent_episodes: u32,
    pub colony_episodes_used: u32,
    pub colony_episodes_available: u32,
    pub basic_episode_data: [BasicEpisodeData; 20],
    pub real_episode_data: [RealEpisodeData; 14],
    pub field_4: [u32; 37],
    // Odyssey changes:
    //  set_file.field_4[010]: difference detected: 5 vs 6 = when bitmap changed from 7 to 2
    pub mythology: [MythologyData; 14], // 300 or 224
    pub events: Vec<[EventData; 150]>,
    pub field_8: [u8; 5],
    pub data_length: u32,
    pub field_9: [u32; 4],
    pub map_data: MapData,
    pub padding: Vec<u8>,
    pub events_per_episode: [u8; 43],
    pub unused_blocks: [u8; 31_693], // "10 different blocks of data, all apparently never used" per notes.md
    pub parent_city_favor: [u32; 10],
    pub colony_goals: [u8; 4394],
    pub parent_city_goals: [u8; 1840],
    pub field_10: Vec<u8>,
    // Odyssey changes:
    // set_file.field_11[431281]: difference detected: 7 vs 2 = bitmap id
    // set_file.field_11[431519]: difference detected: 1 vs 0 = Atlantean vs Greek
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

        let end_position = if new_file_ver { 842_931 } else { 841_863 };

        let events_per_episode_offset = if new_file_ver { 800_361 } else { 799_297 };

        // limited length readers for different sections, the -8 accounts for the versions which were already read
        let mut limited_reader = reader.take(end_position - 8);
        let mut map_data_reader = (&mut limited_reader).take(events_per_episode_offset - 8);

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
            events: read_vec_from(&mut map_data_reader, 14)?,
            field_8: ReadFrom::read_from(&mut map_data_reader)?,
            data_length: ReadFrom::read_from(&mut map_data_reader)?,
            field_9: ReadFrom::read_from(&mut map_data_reader)?,
            map_data: MapData::read_from(&mut map_data_reader)?,
            padding: read_bytes_to_end(&mut map_data_reader)?,
            events_per_episode: ReadFrom::read_from(&mut limited_reader)?,
            unused_blocks: ReadFrom::read_from(&mut limited_reader)?,
            parent_city_favor: ReadFrom::read_from(&mut limited_reader)?,
            colony_goals: ReadFrom::read_from(&mut limited_reader)?,
            parent_city_goals: ReadFrom::read_from(&mut limited_reader)?,
            field_10: read_bytes_to_end(&mut limited_reader)?,
        });
    }
}

impl WriteTo for SettingsData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let new_file_ver = self.version_2 == 26;
        let end_position: usize = if new_file_ver { 842_931 } else { 841_863 };
        let events_per_episode_offset: usize = if new_file_ver { 800_361 } else { 799_297 };

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

        if bytes > events_per_episode_offset {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "SettingsData: map_data overflowing to events_per_episode",
            ));
        }

        bytes += WriteTo::write_to(&vec![0u8; events_per_episode_offset - bytes], writer)?;
        bytes += WriteTo::write_to(&self.events_per_episode, writer)?;
        bytes += WriteTo::write_to(&self.unused_blocks, writer)?;
        bytes += WriteTo::write_to(&self.parent_city_favor, writer)?;
        bytes += WriteTo::write_to(&self.colony_goals, writer)?;
        bytes += WriteTo::write_to(&self.parent_city_goals, writer)?;
        bytes += WriteTo::write_to(&self.field_10, writer)?;

        // ignore content of the `padding` field, instead write the right amount of bytes
        if bytes < end_position {
            let filler = vec![0u8; end_position - bytes];
            writer.write_all(&filler)?;
            bytes += filler.len();
        }

        return Ok(bytes);
    }
}

impl Default for SettingsData {
    fn default() -> Self {
        return SettingsData {
            version_1: 0,
            version_2: 0,
            parent_episodes: 0,
            colony_episodes_used: 0,
            colony_episodes_available: 0,
            basic_episode_data: Default::default(),
            real_episode_data: Default::default(),
            field_4: [0; 37],
            mythology: Default::default(),
            events: vec![[EventData::default(); 150]; 14],
            field_8: [0; 5],
            data_length: 0,
            field_9: [0; 4],
            map_data: Default::default(),
            padding: vec![],
            events_per_episode: [0; 43],
            unused_blocks: [0; 31_693],
            parent_city_favor: [0; 10],
            colony_goals: [0; 4394],
            parent_city_goals: [0; 1840],
            field_10: vec![],
        };
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
