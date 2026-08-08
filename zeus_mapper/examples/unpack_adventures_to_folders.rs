// Walks the top-level `.pak` files directly under `$ZEUS_HOME/Adventures` - the built-in
// campaign missions etc. that ship as a single `.pak` with no companion `.set`/`.map`/`.txt`, and
// so aren't openable in the in-game Adventure Editor - and writes each one out as a sibling
// folder named "<name> - Recreated", containing `<name> - Recreated.pak`,
// `<name> - Recreated.set`, and one `<name> - Recreated{P,C1,C2,...}.map` per map: the same
// layout `Adventure::write_to` (adventure/adventure.rs) produces for a single adventure.
// Adventures that already have a folder are left untouched.

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::ErrorKind;
use std::path::PathBuf;
use zeus_mapper::prelude::PakData;

fn main() -> io::Result<()> {
    let game_root = env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");
    let adventures_dir = PathBuf::from(&game_root).join("Adventures");

    println!("writing recreated adventures to {}", adventures_dir.display());

    for entry in fs::read_dir(&adventures_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|ext| ext == "pak").unwrap_or(false) {
            continue;
        }

        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let recreated_name = format!("{name} - Recreated");

        let mut reader = BufReader::new(File::open(&path)?);
        let pak_data = match PakData::read_from(&mut reader) {
            Ok(pak_data) => pak_data,
            Err(err) => {
                println!("FAILED to read {name}: {err}");
                continue;
            }
        };

        let folder = adventures_dir.join(&recreated_name);
        if let Err(err) = fs::remove_dir_all(&folder)
            && err.kind() != ErrorKind::NotFound
        {
            return Err(err);
        }
        fs::create_dir_all(&folder)?;

        let mut pak_writer = File::create(folder.join(format!("{recreated_name}.pak")))?;
        pak_data.write_to(&mut pak_writer)?;

        let mut set_writer = File::create(folder.join(format!("{recreated_name}.set")))?;
        pak_data.settings_data.write_to(&mut set_writer)?;

        for (i, map_data) in pak_data.map_data.iter().enumerate() {
            let suffix = if i == 0 { "P".to_string() } else { format!("C{i}") };
            let mut map_writer = File::create(folder.join(format!("{recreated_name}{suffix}.map")))?;
            map_data.write_to(&mut map_writer)?;
        }

        println!(
            "wrote {recreated_name} (version_1={}, version_2={})",
            pak_data.settings_data.version_1, pak_data.settings_data.version_2
        );
    }

    return Ok(());
}
