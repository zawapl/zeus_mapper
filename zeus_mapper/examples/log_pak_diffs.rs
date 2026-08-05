use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use zeus_mapper::prelude::LogDifferences;
use zeus_mapper::prelude::PakData;
use zeus_mapper::prelude::SettingsData;

fn main() -> io::Result<()> {
    let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

    let adventures_folder = fs::read_dir(format!("{}/Adventures", game_root))?;

    for adventure in adventures_folder {
        let adventure_path = adventure?.path();
        if adventure_path.is_dir() {
            let adventure_name = adventure_path.file_name().map(OsStr::to_str).flatten().unwrap();

            let mut pak_path = adventure_path.clone();
            pak_path.push(format!("{adventure_name}.pak"));

            let mut set_path = adventure_path.clone();
            set_path.push(format!("{adventure_name}.set"));

            if fs::exists(&pak_path)? && fs::exists(&set_path)? {
                println!("Comparing {adventure_name}");

                let mut pak_file_reader = BufReader::new(File::open(pak_path).unwrap());
                let pak_data = PakData::read_from(&mut pak_file_reader).unwrap();

                let mut set_file_reader = BufReader::new(File::open(set_path).unwrap());
                let settings_data = SettingsData::read_from(&mut set_file_reader).unwrap();

                LogDifferences::log_differences(&settings_data, &pak_data.settings_data, "set_file".to_owned());

                println!();
            }
        }
    }

    return Ok(());
}
