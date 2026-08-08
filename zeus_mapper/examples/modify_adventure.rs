use std::io;
use zeus_mapper::prelude::Adventure;

fn main() -> io::Result<()> {
    let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

    let source_name = "Open Play Economic 2";
    let source_dir = format!("{game_root}/Adventures/{source_name}");

    let mut adventure = Adventure::read_from(&source_dir)?;
    println!("Read '{}', initial funds: {}", adventure.title, adventure.initial_funds);

    // adventure.initial_funds *= 2;
    // println!("Doubled initial funds to {}", adventure.initial_funds);
    //
    // adventure.title = format!("{} - Modified", adventure.title);
    // println!("Renamed to '{}'", adventure.title);

    let dest_dir = format!("{game_root}/Adventures/{source_name} - Modified");
    println!("Writing {dest_dir}");
    adventure.write_to(&dest_dir)?;

    return Ok(());
}
