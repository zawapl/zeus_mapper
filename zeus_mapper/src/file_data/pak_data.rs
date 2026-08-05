use crate::file_data::map_data::MapData;
use crate::file_data::settings_data::SettingsData;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

#[derive(Debug, LogDifferences)]
pub struct PakData {
    pub settings_data: SettingsData,
    pub map_data: Vec<MapData>,
}

impl PakData {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let settings_data = SettingsData::read_from(reader)?;
        let map_data = MapData::read_maps(reader, 1 + settings_data.colony_episodes_available as usize)?;
        return Ok(PakData { settings_data, map_data });
    }
}

impl WriteTo for PakData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.settings_data, writer)?;
        bytes += WriteTo::write_to(&self.map_data, writer)?;

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use crate::file_data::pak_data::PakData;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::path::PathBuf;

    #[test]
    fn test_pak_files() -> Result<()> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

        let adventures_folder = fs::read_dir(format!("{}/Adventures", game_root))?;

        let mut files_tested = 0;

        for adventure in adventures_folder {
            let adventure_path = adventure?.path();
            if adventure_path.is_dir() {
                let files = fs::read_dir(adventure_path)?;
                for file in files {
                    let file_path = file?.path();
                    let extension = file_path.extension().map_or("", |x| x.to_str().unwrap_or(""));
                    if extension == "pak" {
                        check_pak_file(&file_path)?;
                        files_tested += 1;
                    }
                }
            } else {
                let extension = adventure_path.extension().map_or("", |x| x.to_str().unwrap_or(""));
                if extension == "pak" {
                    check_pak_file(&adventure_path)?;
                    files_tested += 1;
                }
            }
        }

        assert!(files_tested >= 40);

        return Ok(());
    }

    fn check_pak_file(path: &PathBuf) -> Result<()> {
        let mut reader = File::open(&path).map(BufReader::new)?;
        let pak_data = PakData::read_from(&mut reader)?;

        let current_pos = reader.stream_position()?;
        reader.seek(SeekFrom::End(0))?;
        let end_pos = reader.stream_position()?;
        let unread = end_pos - current_pos;

        assert_eq!(unread, 0);

        assert_eq!(pak_data.settings_data.version_1, 8871);

        for map_data in pak_data.map_data {
            assert_eq!(map_data.version_2, 33);
        }

        return Ok(());
    }
}
