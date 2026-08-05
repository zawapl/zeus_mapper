use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use zeus_mapper::prelude::EngText;

fn main() -> io::Result<()> {
    let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

    print!("Enter search phrase: ");
    io::Write::flush(&mut io::stdout())?;

    let stdin = io::stdin();
    let mut phrase = String::new();
    stdin.lock().read_line(&mut phrase)?;
    let phrase = phrase.trim().to_lowercase();

    let files = [
        ("Zeus_Text.eng", "ZEUS_TEXT"),
        ("Zeus_Editor_Text.eng", "ZEUS_EDITOR_TEXT"),
        ("Zeus_MM.eng", "ZEUS_MM"),
    ];

    for (file_name, prefix) in files {
        let path = format!("{game_root}/{file_name}");
        let mut reader = File::open(&path).map(BufReader::new)?;
        let eng_text = EngText::read_from(&mut reader)?;

        for (group_idx, table) in eng_text.string_tables.iter().enumerate() {
            for (str_idx, s) in table.0.iter().enumerate() {
                if s.to_lowercase().contains(&phrase) {
                    println!("{prefix}[{group_idx}][{str_idx}] = {s:?}");
                }
            }
        }
    }

    return Ok(());
}
