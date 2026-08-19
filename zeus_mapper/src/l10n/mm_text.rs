use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::read_utils::read_vec_from;
use encoding_rs::WINDOWS_1252;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;

#[derive(Clone, Debug)]
pub struct MmText {
    pub header: Header,
    pub entries: Vec<MmEntry>,
}

impl MmText {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let header = Header::read_from(reader)?;

        // Header.total_slots seems to be ignored and should read a 1000 instead
        let raw_entries = read_vec_from::<RawMmEntry>(reader, 1000)?;

        let mut string_blob = Vec::new();
        reader.read_to_end(&mut string_blob)?;

        let mut entries = Vec::with_capacity(raw_entries.len());
        for raw in raw_entries.iter() {
            entries.push(raw.resolve(&string_blob)?);
        }

        return Ok(MmText { header, entries });
    }
}

#[derive(Clone, Debug, Default)]
pub struct Header {
    pub signature: String,
    pub total_slots: i32,
    pub used_slots: i32,
}

impl ReadFrom for Header {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(Header {
            signature: read_string_from(reader, 16)?,
            total_slots: ReadFrom::read_from(reader)?,
            used_slots: ReadFrom::read_from(reader)?,
        });
    }
}

/// One help-topic entry, with its strings already resolved from the trailing string blob.
#[derive(Clone, Debug, Default)]
pub struct MmEntry {
    pub unknown_0: BoxedArray<u8, 60>,
    pub link: Option<String>,
    pub unknown_64: [u8; 4],
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub body: Option<String>,
}

/// The on-disk shape of an `MmEntry`: string fields are byte offsets into the trailing string
/// blob (or `0` when absent) rather than resolved strings.
#[derive(Clone, Debug, Default)]
struct RawMmEntry {
    unknown_0: BoxedArray<u8, 60>,
    link_offset: i32,
    unknown_64: [u8; 4],
    title_offset: i32,
    subtitle_offset: i32,
    body_offset: i32,
}

impl ReadFrom for RawMmEntry {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(RawMmEntry {
            unknown_0: ReadFrom::read_from(reader)?,
            link_offset: ReadFrom::read_from(reader)?,
            unknown_64: ReadFrom::read_from(reader)?,
            title_offset: ReadFrom::read_from(reader)?,
            subtitle_offset: ReadFrom::read_from(reader)?,
            body_offset: ReadFrom::read_from(reader)?,
        });
    }
}

impl RawMmEntry {
    fn resolve(&self, string_blob: &[u8]) -> io::Result<MmEntry> {
        return Ok(MmEntry {
            unknown_0: self.unknown_0.clone(),
            link: resolve_offset(string_blob, self.link_offset)?,
            unknown_64: self.unknown_64,
            title: resolve_offset(string_blob, self.title_offset)?,
            subtitle: resolve_offset(string_blob, self.subtitle_offset)?,
            body: resolve_offset(string_blob, self.body_offset)?,
        });
    }
}

fn resolve_offset(string_blob: &[u8], offset: i32) -> io::Result<Option<String>> {
    if offset == 0 {
        return Ok(None);
    }

    let (string, _next) = read_string_nul_from_offset(string_blob, offset as usize)?;

    return Ok(Some(string));
}

/// Reads a nul-terminated string starting at `offset` within an in-memory buffer.
///
/// Returns the decoded string together with the offset of the byte immediately after the
/// terminating nul run. `MmEntry`'s fields each resolve independently via their own offset and
/// ignore this, but it falls out for free from finding the end of the run.
///
/// **Assumptions**: `offset` points at the start of a string, and the buffer contains a nul byte
/// somewhere at or after `offset` to terminate it.
fn read_string_nul_from_offset(data: &[u8], offset: usize) -> io::Result<(String, usize)> {
    let relative_end = data[offset..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, "missing nul terminator"))?;
    let end = offset + relative_end;

    // todo this may need to vary depending on language, PL seems to be WINDOWS_1250 for example
    let (cow, _) = WINDOWS_1252.decode_with_bom_removal(&data[offset..end]);

    // A string may be terminated with multiple nul chars, consume them all.
    let mut next = end;
    while next < data.len() && data[next] == 0 {
        next += 1;
    }

    return Ok((cow.into_owned(), next));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;

    #[test]
    fn test_zeus_mm() -> Result<()> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

        let mut reader = File::open(format!("{game_root}/Zeus_MM.eng")).map(BufReader::new)?;
        let mm_text = MmText::read_from(&mut reader)?;

        assert_eq!(mm_text.header.signature, "Zeus MM file.");
        assert_eq!(mm_text.header.total_slots, 1000);
        assert_eq!(mm_text.entries.len(), 1000);

        let about = mm_text.entries[0].clone();
        assert_eq!(about.title.as_deref(), Some("About Zeus and Poseidon"));
        assert_eq!(about.body.as_deref(), Some("Copyright Sierra On-line 2001. @LVersion 2.1"));

        let toc = mm_text.entries[1].clone();
        assert_eq!(toc.title.as_deref(), Some("Table of Contents"));
        assert_eq!(toc.subtitle.as_deref(), Some("Click on a topic to view help"));

        return Ok(());
    }
}
