use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::read_utils::read_vec_from;
use encoding_rs::WINDOWS_1252;
use std::io;
use std::io::Read;

#[derive(Clone, Debug)]
pub struct EngText {
    pub header: Header,
    pub groups: Vec<StringGroupData>,
    pub string_tables: Vec<StringTable>,
}

impl EngText {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
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

/// One string group's location in the file.
///
/// `data_offset` is not used when reading: it does not always point at a clean string boundary
/// (confirmed against real `Zeus_Text.eng` data, where one populated group's `data_offset` lands
/// mid-string), so groups are instead read back-to-back in array order, the same way the game
/// itself lays them out.
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
    fn read_from(reader: &mut impl Read, count: usize) -> io::Result<Self> {
        let mut strings = Vec::with_capacity(count);

        for _ in 0..count {
            strings.push(read_string_nul_from(reader)?);
        }

        return Ok(StringTable(strings));
    }
}

/// Reads a nul-terminated string from a stream, skipping any leading nul bytes first.
///
/// Strings can be separated by more than one nul byte, so leading nuls are skipped rather than
/// read as a value in their own right - each byte is consumed at most once (skip-mode, then
/// read-mode), so unlike a "read then look past the terminator for more nuls" approach, this
/// never needs to peek ahead or seek back.
///
/// **Assumptions**: the stream is positioned either at the start of a string, or on nul padding
/// that leads into one; it contains a nul byte after that string to terminate it.
fn read_string_nul_from(reader: &mut impl Read) -> io::Result<String> {
    let mut byte = [0; 1];

    loop {
        reader.read_exact(&mut byte)?;
        if byte[0] != 0 {
            break;
        }
    }

    let mut bytes = vec![byte[0]];

    loop {
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        bytes.push(byte[0]);
    }

    let (cow, _) = WINDOWS_1252.decode_with_bom_removal(bytes.as_slice());

    return Ok(cow.into_owned());
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

        return Ok(eng_text);
    }
}
