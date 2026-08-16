use crate::file_data::basic_episode_data::BasicEpisodeData;
use crate::file_data::episode_goal_data::EpisodeGoalData;
use crate::file_data::event_data::EventData;
use crate::file_data::map_data::MapData;
use crate::file_data::mythology_data::MythologyData;
use crate::file_data::real_episode_data::RealEpisodeData;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_bytes_to_end;
use crate::utils::validation::ValidationError;
use crate::utils::validation::ValidationResult;
use crate::utils::validation::validate_expected_constant;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, LogDifferences)]
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

impl Default for SettingsData {
    fn default() -> Self {
        return SettingsData {
            version_1: 8871,
            version_2: 26,
            parent_episodes: 1,
            colony_episodes_used: 0,
            colony_episodes_available: 0,
            basic_episode_data: Default::default(),
            real_episode_data: Default::default(),
            unknown_1: Default::default(),
            colony_location_names: Default::default(),
            unknown_2: Default::default(),
            mythology: Default::default(),
            events: Default::default(),
            adventure_type: 0,
            constant_1_0x00: Default::default(),
            data_length: data_length(true),
            unknown_3: Default::default(),
            map_data: Default::default(),
            padding: Vec::new(),
            parent_event_counts: Default::default(),
            colony_event_counts: Default::default(),
            unused_blocks: Default::default(),
            parent_city_favor: Default::default(),
            constant_2_0x00: Default::default(),
            bitmap: 0,
            tab_visibility: Default::default(),
            constant_3_0x01: [1; 7],
            unknown_4: 1,
            constant_4_0x01: BoxedArray::from_vec(vec![1; 93]),
            unknown_5: 1,
            constant_5_0x01: [1; 3],
            unknown_6: 1,
            unknown_7: 1,
            constant_6_0x01: [1; 2],
            unknown_8: 1,
            constant_7_0x01: BoxedArray::from_vec(vec![1; 33]),
            constant_8_0x00: Default::default(),
            world_map_enabled: 0,
            constant_9_0x01: [1; 9],
            unknown_9: 0,
            unknown_10: 0,
            colony_episode_goal_counts: Default::default(),
            colony_episode_goals: Default::default(),
            parent_episode_goal_counts: Default::default(),
            parent_episode_goals: Default::default(),
        };
    }
}

impl SettingsData {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return ReadFrom::read_from(reader);
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        return WriteTo::write_to(self, writer);
    }

    pub fn validate(&self) -> ValidationResult {
        if self.version_1 != 8871 {
            return Err(ValidationError::expected_exactly("version_1", self.version_1, 8871));
        }

        if !(1..=10).contains(&self.parent_episodes) {
            return Err(ValidationError::expected_range("parent_episodes", self.parent_episodes, 1, 10));
        }

        if self.colony_episodes_used > 4 {
            return Err(ValidationError::expected_range(
                "colony_episodes_used",
                self.colony_episodes_used,
                0,
                4,
            ));
        }

        if self.colony_episodes_available > 4 {
            return Err(ValidationError::expected_range(
                "colony_episodes_available",
                self.colony_episodes_available,
                0,
                4,
            ));
        }

        if self.adventure_type > 4 {
            return Err(ValidationError::expected_range("adventure_type", self.adventure_type, 0, 4));
        }

        if let Some((i, count)) = self.parent_event_counts.iter().enumerate().find(|(_, c)| **c > 150) {
            return Err(ValidationError::expected_range(format!("parent_event_counts[{i}]"), *count, 0, 150));
        }

        if let Some((i, count)) = self.colony_event_counts.iter().enumerate().find(|(_, c)| **c > 150) {
            return Err(ValidationError::expected_range(format!("colony_event_counts[{i}]"), *count, 0, 150));
        }

        if let Some((i, count)) = self.colony_episode_goal_counts.iter().enumerate().find(|(_, c)| **c > 6) {
            return Err(ValidationError::expected_range(
                format!("colony_episode_goal_counts[{i}]"),
                *count,
                0,
                6,
            ));
        }

        if let Some((i, count)) = self.parent_episode_goal_counts.iter().enumerate().find(|(_, c)| **c > 6) {
            return Err(ValidationError::expected_range(
                format!("parent_episode_goal_counts[{i}]"),
                *count,
                0,
                6,
            ));
        }

        for (i, mythology) in self.mythology.iter().enumerate() {
            mythology
                .validate()
                .map_err(ValidationError::add_parent(format!("mythology[{i}]")))?;
        }

        validate_expected_constant("constant_3_0x01", &self.constant_3_0x01, 1)?;
        validate_expected_constant("constant_4_0x01", self.constant_4_0x01.as_ref(), 1)?;
        validate_expected_constant("constant_5_0x01", &self.constant_5_0x01, 1)?;
        validate_expected_constant("constant_6_0x01", &self.constant_6_0x01, 1)?;
        validate_expected_constant("constant_7_0x01", self.constant_7_0x01.as_ref(), 1)?;
        validate_expected_constant("constant_8_0x00", self.constant_8_0x00.as_ref(), 0)?;
        validate_expected_constant("constant_9_0x01", &self.constant_9_0x01, 1)?;
        validate_expected_constant("constant_1_0x00", &self.constant_1_0x00, 0)?;
        validate_expected_constant("constant_2_0x00", self.constant_2_0x00.as_ref(), 0)?;

        for (i, basic_episode_data) in self.basic_episode_data.iter().enumerate() {
            basic_episode_data
                .validate()
                .map_err(ValidationError::add_parent(format!("basic_episode_data[{i}]")))?;
        }

        for (i, real_episode_data) in self.real_episode_data.iter().enumerate() {
            real_episode_data
                .validate()
                .map_err(ValidationError::add_parent(format!("real_episode_data[{i}]")))?;
        }

        for (i, row) in self.events.iter().enumerate() {
            for (j, event) in row.iter().enumerate() {
                event.validate().map_err(ValidationError::add_parent(format!("events[{i}][{j}]")))?;
            }
        }

        self.map_data.validate().map_err(ValidationError::add_parent("map_data"))?;

        return Ok(());
    }
}

impl ReadFrom for SettingsData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let version_1 = ReadFrom::read_from(reader)?;
        let version_2 = ReadFrom::read_from(reader)?;

        let new_file_ver = version_2 == 26;

        let data_length = data_length(new_file_ver) as u64; // should match data_length field
        let padded_map_data_end = padded_map_data_end(new_file_ver) as u64;

        // limited length readers for different sections, the -8 accounts for the versions which were already read
        let mut limited_reader = reader.take(data_length - 8);
        let mut map_data_reader = (&mut limited_reader).take(padded_map_data_end - 8);

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

        let padded_map_data_end = padded_map_data_end(new_file_ver) as usize;
        if bytes > padded_map_data_end {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "SettingsData: map_data overflowing to parent_event_counts",
            ));
        }

        // Write the real `padding` content back  truncating/zero-extending it to fit the space actually available.
        // In the future should be possible to just write 0s (we want the original for now for byte checking tests)
        let padding_len = padded_map_data_end - bytes;
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

fn padded_map_data_end(new_file_ver: bool) -> u32 {
    return if new_file_ver { 800_361 } else { 799_297 };
}

fn data_length(new_file_ver: bool) -> u32 {
    return if new_file_ver { 842_931 } else { 841_863 };
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
