// Walks every flat, single-file adventure directly under `$ZEUS_HOME/Adventures` - the built-in
// campaign missions etc. that ship as a lone `.pak` with no companion folder - parses each one
// through the higher-level `Adventure` model, flips its `adventure_type` to `PoseidonCustom`, and
// writes the result as a folder-based adventure under `<repo>/.tmp`, so each becomes openable in
// the in-game Adventure Editor as a custom Poseidon adventure.
//
// This goes through the `Adventure` model on purpose (parse -> mutate -> `Adventure::write_to`),
// which is one-way: any raw `PakData` field the model doesn't represent is dropped/defaulted on the
// way back out. Flat adventures usually have no `.txt` sidecar (their text lives in the game's
// shared localization files), so their title/intro/completion text default to empty here.
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use zeus_mapper::prelude::Adventure;
use zeus_mapper::prelude::AdventureText;
use zeus_mapper::prelude::AdventureType;
use zeus_mapper::prelude::PakData;

fn main() -> io::Result<()> {
    let game_root = env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");
    let adventures_dir = PathBuf::from(&game_root).join("Adventures");

    let dest_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    fs::create_dir_all(&dest_root)?;
    println!("writing PoseidonCustom copies to {}", dest_root.display());

    for entry in fs::read_dir(&adventures_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("pak") {
            continue;
        }

        // Real adventure filenames sometimes carry a single leading non-alphanumeric sort-order
        // character (`#`, `&`, `@`, `[`, `]`, `^`, `{`, `}`) not part of the adventure's real name.
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let name = stem.trim_start_matches(|c: char| !c.is_alphanumeric()).to_string();

        let mut pak_reader = BufReader::new(File::open(&path)?);
        let pak_data = match PakData::read_from(&mut pak_reader) {
            Ok(pak_data) => pak_data,
            Err(err) => {
                println!("FAILED to read {name}: {err}");
                continue;
            }
        };

        // Flat adventures rarely ship a `.txt` sidecar - use it when present, otherwise default to
        // empty mission text rather than skipping the adventure.
        let text_path = path.with_file_name(format!("{stem}.txt"));
        let adventure_text = if text_path.exists() {
            AdventureText::read_from(&mut BufReader::new(File::open(&text_path)?))?
        } else {
            AdventureText::default()
        };

        let mut adventure = Adventure::from_pak(&pak_data, &adventure_text);
        adventure.adventure_type = AdventureType::PoseidonCustom;

        let dest_dir = dest_root.join(&name);
        if let Err(err) = adventure.write_to(&dest_dir) {
            println!("FAILED to write {name}: {err}");
            continue;
        }

        println!("wrote {name} as PoseidonCustom");
    }

    return Ok(());
}
