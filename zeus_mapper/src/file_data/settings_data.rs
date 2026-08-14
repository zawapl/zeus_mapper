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
    pub unknown_1: [u32; 2],             // still otherwise-undeciphered
    pub colony_location_names: [u32; 4], // `world_locations[].name` for up to 4 Colony locations, in order; see DATA_MAPPING.md
    pub unknown_2: [u32; 31],            // still otherwise-undeciphered
    pub mythology: [MythologyData; 14],  // 300 or 224
    pub events: BoxedArray<BoxedArray<EventData, 150>, 14>,
    pub adventure_type: u8,
    pub constant_1_0x00: [u8; 4],
    pub data_length: u32,
    pub unknown_3: [u32; 4],
    pub map_data: MapData,
    pub padding: Vec<u8>, // Should not contain useful data, we should be able to remove this in the future
    pub parent_event_counts: [u32; 10],
    pub colony_event_counts: [u8; 3],
    pub unused_blocks: BoxedArray<u8, 31_693>, // "10 different blocks of data, all apparently never used" per notes.md
    pub parent_city_favor: [u32; 10],
    pub constant_2_0x00: BoxedArray<u8, 4112>,
    pub bitmap: u32,
    pub tab_visibility: [u8; 11],
    pub constant_3_0x01: [u8; 7],
    // Varies across the game's tutorial/demo `.pak`s specifically (0 on colony/palace/peacewar/
    // warrior, 1 everywhere else including bigcity and every real adventure); meaning otherwise
    // undeciphered. See "Extending the correlation sweep" in docs/adventure_model_todo.md.
    pub unknown_4: u8,
    pub constant_4_0x01: BoxedArray<u8, 93>,
    // Varies on bigcity/peacewar specifically (0 there, 1 everywhere else); meaning otherwise
    // undeciphered.
    pub unknown_5: u8,
    pub constant_5_0x01: [u8; 3],
    // Varies on bigcity/peacewar specifically (0 there, 1 everywhere else); meaning otherwise
    // undeciphered.
    pub unknown_6: u8,
    // Varies on bigcity/colony/peacewar specifically (0 there, 1 everywhere else); meaning
    // otherwise undeciphered.
    pub unknown_7: u8,
    pub constant_6_0x01: [u8; 2],
    // Varies on bigcity/peacewar specifically (0 there, 1 everywhere else); meaning otherwise
    // undeciphered.
    pub unknown_8: u8,
    pub constant_7_0x01: BoxedArray<u8, 33>,
    pub constant_8_0x00: BoxedArray<u8, 66>,
    pub world_map_enabled: u8,
    pub constant_9_0x01: [u8; 9],
    // Varies on colony specifically (1 there, 0 everywhere else including every other tutorial/
    // demo `.pak`); meaning otherwise undeciphered. Top 3 bytes confirmed always 0 across every
    // real `.set` file sampled, so this is a single little-endian u32 (matching a C `BOOL`/`int`
    // field's natural width) rather than a flag byte plus 3 bytes of padding.
    pub unknown_9: u32,
    pub unknown_10: u32, // only present for new file format; see DATA_MAPPING.md
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

    pub fn validate(&self) -> Result<(), String> {
        if self.version_1 != 8871 {
            return Err(format!("version_1 is {}, expected 8871", self.version_1));
        }

        for mythology in &self.mythology {
            mythology.validate()?;
        }

        if let Some(offset) = self.constant_3_0x01.iter().position(|b| *b != 1) {
            return Err(format!("constant_3_0x01[{offset}] is not 1"));
        }

        if let Some(offset) = self.constant_4_0x01.iter().position(|b| *b != 1) {
            return Err(format!("constant_4_0x01[{offset}] is not 1"));
        }

        if let Some(offset) = self.constant_5_0x01.iter().position(|b| *b != 1) {
            return Err(format!("constant_5_0x01[{offset}] is not 1"));
        }

        if let Some(offset) = self.constant_6_0x01.iter().position(|b| *b != 1) {
            return Err(format!("constant_6_0x01[{offset}] is not 1"));
        }

        if let Some(offset) = self.constant_7_0x01.iter().position(|b| *b != 1) {
            return Err(format!("constant_7_0x01[{offset}] is not 1"));
        }

        if let Some(offset) = self.constant_8_0x00.iter().position(|b| *b != 0) {
            return Err(format!("constant_8_0x00[{offset}] is non-zero"));
        }

        if let Some(offset) = self.constant_9_0x01.iter().position(|b| *b != 1) {
            return Err(format!("constant_9_0x01[{offset}] is not 1"));
        }

        if self.constant_1_0x00.iter().any(|b| *b != 0) {
            return Err("constant_1_0x00 is non-zero".to_owned());
        }

        if let Some(offset) = self.constant_2_0x00.as_ref().iter().position(|b| *b != 0) {
            return Err(format!("constant_2_0x00[{offset}] is non-zero"));
        }

        for (i, basic_episode_data) in self.basic_episode_data.iter().enumerate() {
            basic_episode_data.validate().map_err(|e| format!("basic_episode_data[{i}]: {e}"))?;
        }

        for (i, real_episode_data) in self.real_episode_data.iter().enumerate() {
            real_episode_data.validate().map_err(|e| format!("real_episode_data[{i}]: {e}"))?;
        }

        for (i, row) in self.events.iter().enumerate() {
            for (j, event) in row.iter().enumerate() {
                event.validate().map_err(|e| format!("events[{i}][{j}]: {e}"))?;
            }
        }

        self.map_data.validate().map_err(|e| format!("map_data: {e}"))?;

        return Ok(());
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
            unknown_1: ReadFrom::read_from(&mut map_data_reader)?,
            colony_location_names: ReadFrom::read_from(&mut map_data_reader)?,
            unknown_2: ReadFrom::read_from(&mut map_data_reader)?,
            mythology: MythologyData::read_arr_from(&mut map_data_reader, new_file_ver)?,
            events: ReadFrom::read_from(&mut map_data_reader)?,
            adventure_type: ReadFrom::read_from(&mut map_data_reader)?,
            constant_1_0x00: ReadFrom::read_from(&mut map_data_reader)?,
            data_length: ReadFrom::read_from(&mut map_data_reader)?,
            unknown_3: ReadFrom::read_from(&mut map_data_reader)?,
            map_data: MapData::read_from(&mut map_data_reader)?,
            padding: read_bytes_to_end(&mut map_data_reader)?,
            parent_event_counts: ReadFrom::read_from(&mut limited_reader)?,
            colony_event_counts: ReadFrom::read_from(&mut limited_reader)?,
            unused_blocks: ReadFrom::read_from(&mut limited_reader)?,
            parent_city_favor: ReadFrom::read_from(&mut limited_reader)?,
            constant_2_0x00: ReadFrom::read_from(&mut limited_reader)?,
            bitmap: ReadFrom::read_from(&mut limited_reader)?,
            tab_visibility: ReadFrom::read_from(&mut limited_reader)?,
            constant_3_0x01: ReadFrom::read_from(&mut limited_reader)?,
            unknown_4: ReadFrom::read_from(&mut limited_reader)?,
            constant_4_0x01: ReadFrom::read_from(&mut limited_reader)?,
            unknown_5: ReadFrom::read_from(&mut limited_reader)?,
            constant_5_0x01: ReadFrom::read_from(&mut limited_reader)?,
            unknown_6: ReadFrom::read_from(&mut limited_reader)?,
            unknown_7: ReadFrom::read_from(&mut limited_reader)?,
            constant_6_0x01: ReadFrom::read_from(&mut limited_reader)?,
            unknown_8: ReadFrom::read_from(&mut limited_reader)?,
            constant_7_0x01: ReadFrom::read_from(&mut limited_reader)?,
            constant_8_0x00: ReadFrom::read_from(&mut limited_reader)?,
            world_map_enabled: ReadFrom::read_from(&mut limited_reader)?,
            constant_9_0x01: ReadFrom::read_from(&mut limited_reader)?,
            unknown_9: ReadFrom::read_from(&mut limited_reader)?,
            unknown_10: if new_file_ver {
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
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.colony_location_names, writer)?;
        bytes += WriteTo::write_to(&self.unknown_2, writer)?;
        bytes += MythologyData::write_arr_to(&self.mythology, writer, new_file_ver)?;
        bytes += WriteTo::write_to(&self.events, writer)?;
        bytes += WriteTo::write_to(&self.adventure_type, writer)?;
        bytes += WriteTo::write_to(&self.constant_1_0x00, writer)?;
        bytes += WriteTo::write_to(&self.data_length, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
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
        bytes += WriteTo::write_to(&self.constant_2_0x00, writer)?;
        bytes += WriteTo::write_to(&self.bitmap, writer)?;
        bytes += WriteTo::write_to(&self.tab_visibility, writer)?;
        bytes += WriteTo::write_to(&self.constant_3_0x01, writer)?;
        bytes += WriteTo::write_to(&self.unknown_4, writer)?;
        bytes += WriteTo::write_to(&self.constant_4_0x01, writer)?;
        bytes += WriteTo::write_to(&self.unknown_5, writer)?;
        bytes += WriteTo::write_to(&self.constant_5_0x01, writer)?;
        bytes += WriteTo::write_to(&self.unknown_6, writer)?;
        bytes += WriteTo::write_to(&self.unknown_7, writer)?;
        bytes += WriteTo::write_to(&self.constant_6_0x01, writer)?;
        bytes += WriteTo::write_to(&self.unknown_8, writer)?;
        bytes += WriteTo::write_to(&self.constant_7_0x01, writer)?;
        bytes += WriteTo::write_to(&self.constant_8_0x00, writer)?;
        bytes += WriteTo::write_to(&self.world_map_enabled, writer)?;
        bytes += WriteTo::write_to(&self.constant_9_0x01, writer)?;
        bytes += WriteTo::write_to(&self.unknown_9, writer)?;
        if new_file_ver {
            bytes += WriteTo::write_to(&self.unknown_10, writer)?;
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
    use std::io;
    use std::io::BufReader;
    use std::io::Seek;
    use std::io::SeekFrom;

    #[test]
    fn validate_set_files() -> io::Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventures_folder = fs::read_dir(format!("{}/Adventures", game_root))?;

            for adventure_folder in adventures_folder {
                let folder_path = adventure_folder?.path();

                if folder_path.is_dir() {
                    for adventure_file in fs::read_dir(folder_path)? {
                        let path = adventure_file?.path();
                        let extension = path.extension().map_or("", |x| x.to_str().unwrap_or(""));
                        if extension == "set" {
                            let mut reader = File::open(&path).map(BufReader::new)?;
                            let settings_data = SettingsData::read_from(&mut reader)?;

                            let current_pos = reader.stream_position()?;
                            reader.seek(SeekFrom::End(0))?;
                            let end_pos = reader.stream_position()?;
                            let unread = end_pos - current_pos;

                            assert_eq!(unread, 0, "{path:?}: {} unread bytes", end_pos - current_pos);

                            if let Err(e) = settings_data.validate() {
                                panic!("{path:?}: {e}");
                            }
                        }
                    }
                }
            }
        }

        return Ok(());
    }
}
