use std::fs::File;
use std::io::BufReader;
use zeus_mapper::prelude::PakData;

const SCENARIO: &str = "The Odyssey";

fn main() {
    let game_root = std::env::var("ZEUS_HOME").expect("Game root in ZEUS_HOME is not valid");
    let file =
        File::open(format!("{}/Adventures/{}/{}.file_data", game_root, SCENARIO, SCENARIO)).expect("Could not open file for reading");
    let mut reader = BufReader::new(file);
    let pak_data = PakData::read_from(&mut reader).expect("Could not read ZeusPak data");
    println!("No of parent episodes: {}", pak_data.settings_data.parent_episodes)
}
