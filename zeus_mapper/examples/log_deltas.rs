use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use zeus_mapper::prelude::LogDifferences;
use zeus_mapper::prelude::MapData;
use zeus_mapper::prelude::SettingsData;

fn main() -> io::Result<()> {
    let mut paths: Vec<String> = fs::read_dir("crates/zeus_mapper/test_data/adventures")?
        .map(|path| path.unwrap().path().display().to_string())
        .collect();

    paths.sort();

    let files: Vec<(String, SettingsData, MapData)> = paths
        .iter()
        .map(|path| {
            let name = path.split("\\").last().unwrap_or("").to_owned();
            println!("Reading {}", name);

            let mut set_file_reader = BufReader::new(File::open(format!("{}/The Odyssey 2.set", path)).unwrap());
            let set_data = SettingsData::read_from(&mut set_file_reader).unwrap();

            let mut map_file_reader = BufReader::new(File::open(format!("{}/The Odyssey 2P.map", path)).unwrap());
            let map_data = MapData::read_from(&mut map_file_reader).unwrap();

            return (name, set_data, map_data);
        })
        .collect();

    println!();

    files[..].windows(2).for_each(|window| {
        let (file_a, set_a, map_a) = &window[0];
        let (file_b, set_b, map_b) = &window[1];

        println!("Comparing [{file_a}] and [{file_b}]");

        LogDifferences::log_differences(set_a, set_b, "set_file".to_owned());
        LogDifferences::log_differences(map_a, map_b, "map_file".to_owned());

        println!();
    });

    return Ok(());
}
