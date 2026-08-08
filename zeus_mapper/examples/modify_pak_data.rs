use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::ErrorKind;
use zeus_mapper::prelude::AdventureText;
use zeus_mapper::prelude::DataConstant;
use zeus_mapper::prelude::God;
use zeus_mapper::prelude::PakData;

fn main() -> io::Result<()> {
    let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

    let source_name = "Open Play Economic 2";
    let source_dir = format!("{game_root}/Adventures/{source_name}");

    let mut pak_reader = File::open(format!("{source_dir}/{source_name}.pak")).map(BufReader::new)?;
    let mut pak_data = PakData::read_from(&mut pak_reader)?;
    println!(
        "Read '{source_name}', starting cash: {}",
        pak_data.settings_data.real_episode_data[0].starting_cash
    );

    let mut text_reader = File::open(format!("{source_dir}/{source_name}.txt")).map(BufReader::new)?;
    let adventure_text = AdventureText::read_from(&mut text_reader)?;

    // Modify the adventure:
    pak_data.settings_data.mythology[0].opponent_gods[3] = u32::MAX; // dissapears from opponent gods in 1st episode, but comes back after 1st episode
    pak_data.settings_data.mythology[1].proponent_gods[6] = God::Hephaestus.value(); // doesn't seem to take effect

    let dest_name = format!("{source_name} - Modified");
    let dest_dir = format!("{game_root}/Adventures/{dest_name}");
    if let Err(err) = fs::remove_dir_all(&dest_dir)
        && err.kind() != ErrorKind::NotFound
    {
        return Err(err);
    }
    fs::create_dir_all(&dest_dir)?;

    let pak_path = format!("{dest_dir}/{dest_name}.pak");
    println!("Writing {pak_path}");
    let mut pak_writer = File::create(pak_path)?;
    pak_data.write_to(&mut pak_writer)?;

    let set_path = format!("{dest_dir}/{dest_name}.set");
    println!("Writing {set_path}");
    let mut set_writer = File::create(set_path)?;
    pak_data.settings_data.write_to(&mut set_writer)?;

    for (i, map_data) in pak_data.map_data.iter().enumerate() {
        let suffix = if i == 0 { "P".to_string() } else { format!("C{i}") };
        let map_path = format!("{dest_dir}/{dest_name}{suffix}.map");
        println!("Writing {map_path}");
        let mut map_writer = File::create(map_path)?;
        map_data.write_to(&mut map_writer)?;
    }

    let text_path = format!("{dest_dir}/{dest_name}.txt");
    println!("Writing {text_path}");
    let mut text_writer = File::create(text_path)?;
    adventure_text.write_to(&mut text_writer)?;

    return Ok(());
}
