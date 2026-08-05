use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Write;
use zeus_mapper::prelude::PakData;

fn main() -> io::Result<()> {
    let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

    let file_path = format!("{game_root}/Adventures/Open Play Economic 2/Open Play Economic 2.pak");

    let mut reader = BufReader::new(File::open(file_path)?);
    let pak_data = PakData::read_from(&mut reader)?;

    let output_file = "crates/zeus_mapper/test_data/adventures/Open Play Economic 2.json";

    println!("Preparing to write to {output_file}");

    let mut writer = File::create(output_file)?;
    writer.write_all(format!("{pak_data:?}").as_bytes())?;

    return Ok(());
}
