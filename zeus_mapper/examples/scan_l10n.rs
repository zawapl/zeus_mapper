use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use zeus_mapper::prelude::EngText;
use zeus_mapper::prelude::MmText;

fn main() -> io::Result<()> {
    let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

    print!("Enter search phrase: ");
    io::Write::flush(&mut io::stdout())?;

    let stdin = io::stdin();
    let mut phrase = String::new();
    stdin.lock().read_line(&mut phrase)?;
    let phrase = phrase.trim().to_lowercase();

    // Scan end files
    {
        let eng_files = [("Zeus_Text.eng", "ZEUS_TEXT"), ("Zeus_Editor_Text.eng", "ZEUS_EDITOR_TEXT")];
        for (file_name, prefix) in eng_files {
            let path = format!("{game_root}/{file_name}");
            let mut reader = File::open(&path).map(BufReader::new)?;
            let eng_text = EngText::read_from(&mut reader)?;

            for (group_idx, table) in eng_text.string_tables.iter().enumerate() {
                for (str_idx, s) in table.0.iter().enumerate() {
                    print_match(&format!("{prefix}[{group_idx}][{str_idx}]"), s, &phrase);
                }
            }
        }
    }

    // Scan the MM file
    {
        let mm_path = format!("{game_root}/Zeus_MM.eng");
        let mut mm_reader = File::open(&mm_path).map(BufReader::new)?;
        let mm_text = MmText::read_from(&mut mm_reader)?;

        for (entry_idx, entry) in mm_text.entries.iter().enumerate() {
            if let Some(title) = &entry.title {
                print_match(&format!("ZEUS_MM[{entry_idx}].title"), title, &phrase);
            }

            if let Some(subtitle) = &entry.subtitle {
                print_match(&format!("ZEUS_MM[{entry_idx}].subtitle"), subtitle, &phrase);
            }

            if let Some(body) = &entry.body {
                print_match(&format!("ZEUS_MM[{entry_idx}].body"), body, &phrase);
            }
        }
    }

    return Ok(());
}

/// Prints `location` and a snippet of `text` around the first case-insensitive match of `phrase`.
///
/// `phrase` must already be lowercased by the caller. Matching and slicing both operate on chars
/// (not bytes) so multi-byte Windows-1252-decoded text can't produce a mid-character split.
fn print_match(location: &str, text: &str, phrase: &str) {
    if phrase.is_empty() {
        println!("{location} = {text:?}");
        return;
    }

    let chars: Vec<char> = text.chars().collect();
    let lower_chars: Vec<char> = text.to_lowercase().chars().collect();
    let phrase_chars: Vec<char> = phrase.chars().collect();

    if let Some(match_start) = lower_chars
        .windows(phrase_chars.len())
        .position(|window| window == phrase_chars.as_slice())
    {
        let context = 40;
        let snippet_start = match_start.saturating_sub(context);
        let snippet_end = (match_start + phrase_chars.len() + context).min(chars.len());

        let prefix_ellipsis = if snippet_start > 0 { "..." } else { "" };
        let suffix_ellipsis = if snippet_end < chars.len() { "..." } else { "" };
        let snippet: String = chars[snippet_start..snippet_end].iter().collect();

        println!("{location} = {prefix_ellipsis}{snippet}{suffix_ellipsis}");
    }
}
