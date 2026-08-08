use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::read_utils::read_string_nul_from;
use crate::utils::read_utils::read_vec_from;
use std::io;
use std::io::Read;
use std::io::Seek;

#[derive(Clone, Debug)]
pub struct EngText {
    pub header: Header,
    pub groups: Vec<StringGroupData>,
    pub string_tables: Vec<StringTable>,
}

impl EngText {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let header = Header::read_from(reader)?;
        let mut groups = read_vec_from::<StringGroupData>(reader, 1000)?;

        groups.truncate(header.group_count as usize);

        let mut string_tables = Vec::with_capacity(groups.len());

        for group in groups.iter() {
            string_tables.push(StringTable::read_from(reader, group.count as usize)?);
        }

        return Ok(EngText {
            header,
            groups,
            string_tables,
        });
    }
}

#[derive(Clone, Debug, Default)]
pub struct Header {
    pub signature: String,
    pub group_count: i32,
    pub string_count: i32,
    pub word_count: i32,
}

impl ReadFrom for Header {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(Header {
            signature: read_string_from(reader, 16)?,
            group_count: ReadFrom::read_from(reader)?,
            string_count: ReadFrom::read_from(reader)?,
            word_count: ReadFrom::read_from(reader)?,
        });
    }
}

#[derive(Clone, Debug, Default)]
pub struct StringGroupData {
    pub data_offset: i32,
    pub count: i32,
}

impl ReadFrom for StringGroupData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(StringGroupData {
            data_offset: ReadFrom::read_from(reader)?,
            count: ReadFrom::read_from(reader)?,
        });
    }
}

#[derive(Clone, Debug, Default)]
pub struct StringTable(pub Vec<String>);

impl StringTable {
    fn read_from<R: Read + Seek>(reader: &mut R, count: usize) -> io::Result<Self> {
        let mut strings = Vec::with_capacity(count);

        for _ in 0..count {
            strings.push(read_string_nul_from(reader)?);
        }

        return Ok(StringTable(strings));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;

    #[test]
    fn test_zeus_editor_text() -> Result<()> {
        let _file = test_file("Zeus_Editor_Text.eng")?;
        return Ok(());
    }

    #[ignore] // todo investigate why the performance is so slow and remove the ignored
    #[test]
    fn test_zeus_mm() -> Result<()> {
        let _file = test_file("Zeus_MM.eng")?;
        return Ok(());
    }

    #[test]
    fn test_zeus_text() -> Result<()> {
        let _file = test_file("Zeus_Text.eng")?;
        return Ok(());
    }

    fn test_file(file_name: &str) -> Result<EngText> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

        let mut reader = File::open(format!("{game_root}/{file_name}")).map(BufReader::new)?;
        let eng_text = EngText::read_from(&mut reader)?;

        assert_eq!(eng_text.header.signature, "Zeus textfile.");

        // Ensure all data was read
        let mut tail = String::new();

        reader.read_to_string(&mut tail)?;

        assert_eq!(tail.trim_end_matches(char::from(0)).to_owned(), "");

        return Ok(eng_text);
    }
}
