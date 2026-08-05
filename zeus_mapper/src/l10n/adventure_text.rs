use encoding_rs::WINDOWS_1252;
use std::collections::HashMap;
use std::io;
use std::io::Read;

#[derive(Default)]
pub struct AdventureText {
    pub title: String,
    pub introduction: String,
    pub complete: String,
    pub episodes_text: Vec<EpisodeText>,
    pub colony_text: Vec<ColonyText>,
}

#[derive(Default)]
pub struct EpisodeText {
    pub title: String,
    pub introduction: String,
    pub complete: String,
}

#[derive(Default)]
pub struct ColonyText {
    pub title: String,
    pub introduction: String,
    pub complete: String,
    pub selection: String,
}

impl AdventureText {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut texts = to_map(reader)?;

        return Ok(AdventureText {
            title: texts.remove("Adventure_Title").unwrap_or_default(),
            introduction: texts.remove("Adventure_Introduction").unwrap_or_default(),
            complete: texts.remove("Adventure_Complete").unwrap_or_default(),
            episodes_text: extract_episodes_text(&mut texts),
            colony_text: extract_colony_text(&mut texts),
        });
    }
}

fn to_map(reader: &mut impl Read) -> io::Result<HashMap<String, String>> {
    let mut results = HashMap::new();

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    let (cow, _) = WINDOWS_1252.decode_with_bom_removal(buffer.as_slice());

    let mut data = cow.trim();

    while !data.is_empty() {
        data = trim_comments(data);
        let split = data.find('=').unwrap();

        let key = data[..split].trim();
        data = &data[split + 1..].trim_start();

        let string_start = data.find('"').take().unwrap();
        data = &data[string_start + 1..].trim_start();

        let string_end = data.find('"').take().unwrap();
        let value = &data[..string_end].replace("@L", "\n").replace("@P", "\n\t");

        results.insert(key.to_string(), value.to_string());
        data = &data[string_end + 1..].trim_start();
    }

    return Ok(results);
}

fn trim_comments(data: &str) -> &str {
    let mut result = data;
    while result.starts_with(";") {
        result = result.trim_start_matches(|c| c != '\n').trim_start_matches('\n').trim_start();
    }
    return result;
}

fn extract_episodes_text(texts: &mut HashMap<String, String>) -> Vec<EpisodeText> {
    let mut result = vec![];

    for i in 1..=10 {
        result.push(EpisodeText {
            title: texts.remove(&format!("Parent_Episode_{i}_Title")).unwrap_or_default(),
            introduction: texts.remove(&format!("Parent_Episode_{i}_Introduction")).unwrap_or_default(),
            complete: texts.remove(&format!("Parent_Episode_{i}_Complete")).unwrap_or_default(),
        })
    }

    return result;
}

fn extract_colony_text(texts: &mut HashMap<String, String>) -> Vec<ColonyText> {
    let mut result = vec![];

    for i in 1..=4 {
        result.push(ColonyText {
            title: texts.remove(&format!("Colony_{i}_Title")).unwrap_or_default(),
            introduction: texts.remove(&format!("Colony_{i}_Introduction")).unwrap_or_default(),
            complete: texts.remove(&format!("Colony_{i}_Complete")).unwrap_or_default(),
            selection: texts.remove(&format!("Colony_{i}_Selection")).unwrap_or_default(),
        })
    }

    return result;
}

#[cfg(test)]
mod tests {
    use crate::l10n::adventure_text::AdventureText;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;

    #[test]
    fn test_odyssey() -> Result<()> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

        let mut reader = File::open(format!("{}/Adventures/The Odyssey/The Odyssey.txt", game_root)).map(BufReader::new)?;
        let adventure_text = AdventureText::read_from(&mut reader)?;

        assert_eq!(adventure_text.title, "The Odyssey");
        assert_eq!(
            adventure_text.introduction,
            "The Trojan War is near its end, and Penelope eagerly awaits the return of her husband, Odysseus.  \
            Help her prepare the way for Odysseus to come home."
        );

        return Ok(());
    }
}
