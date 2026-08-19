use crate::pkware::explode;
use crate::utils::boxed_array::BoxedArray;
use encoding_rs::WINDOWS_1252;
use std::io;
use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;

pub trait ReadFrom: Sized {
    fn read_from(reader: &mut impl Read) -> io::Result<Self>;
}

impl ReadFrom for u8 {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut tmp = [0; 1];
        reader.read_exact(&mut tmp)?;
        return Ok(tmp[0]);
    }
}

impl ReadFrom for i8 {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut tmp = [0; 1];
        reader.read_exact(&mut tmp)?;
        return Ok(tmp[0] as i8);
    }
}

impl ReadFrom for u16 {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut tmp = [0; 2];
        reader.read_exact(&mut tmp)?;
        return Ok(u16::from_le_bytes(tmp));
    }
}

impl ReadFrom for i16 {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut tmp = [0; 2];
        reader.read_exact(&mut tmp)?;
        return Ok(i16::from_le_bytes(tmp));
    }
}

impl ReadFrom for u32 {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut tmp = [0; 4];
        reader.read_exact(&mut tmp)?;
        return Ok(u32::from_le_bytes(tmp));
    }
}

impl ReadFrom for i32 {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut tmp = [0; 4];
        reader.read_exact(&mut tmp)?;
        return Ok(i32::from_le_bytes(tmp));
    }
}

impl ReadFrom for u64 {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut tmp = [0; 8];
        reader.read_exact(&mut tmp)?;
        return Ok(u64::from_le_bytes(tmp));
    }
}

impl<T: ReadFrom, const N: usize> ReadFrom for [T; N] {
    // todo Use MaybeUninit with array transposing once stable: https://github.com/rust-lang/rust/issues/96097
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let vec: Vec<T> = read_vec_from(reader, N)?;
        return vec.try_into().map_err(|_| Error::from(ErrorKind::InvalidData));
    }
}

/// Reads a fixed-width string, trimming only trailing NUL padding.
///
/// **Assumptions**: callers that need a clean C-style string (stopping at the first NUL rather than
/// just the trailing run) must trim it themselves - this function deliberately preserves any bytes
/// between an embedded NUL and the buffer's own trailing padding, since a shorter string written into
/// a buffer that previously held a longer one can leave stray non-NUL bytes behind the true
/// terminator, and several callers round-trip that content byte-for-byte (see
/// `byte_identical_adventures_round_trip_exactly` in `file_data/pak_data.rs`).
pub fn read_string_from(reader: &mut impl Read, bytes: usize) -> io::Result<String> {
    let mut buffer = vec![0; bytes];

    reader.read_exact(&mut buffer)?;

    // todo this may need to vary depending on language, PL seems to be WINDOWS_1250 for example, or maybe all of them are?
    let (cow, _) = WINDOWS_1252.decode_with_bom_removal(buffer.as_slice());

    let result = cow.trim_end_matches(char::from(0)).to_owned();

    return Ok(result);
}

pub fn read_vec_from<T: ReadFrom>(reader: &mut impl Read, count: usize) -> io::Result<Vec<T>> {
    let mut result = Vec::with_capacity(count);

    for _ in 0..count {
        result.push(T::read_from(reader)?);
    }

    return Ok(result);
}

pub(crate) fn read_compressed_boxed_array_from<T: ReadFrom + Default, const N: usize>(
    reader: &mut impl Read,
) -> io::Result<BoxedArray<T, N>> {
    let compressed_size = i32::read_from(reader)?;

    if compressed_size < 0 {
        return BoxedArray::read_from(reader);
    } else {
        let mut compressed = vec![0; to_usize(compressed_size)?];
        reader.read_exact(&mut compressed)?;
        let mut decompressed_reader = Cursor::new(explode(&compressed)?);
        return BoxedArray::read_from(&mut decompressed_reader);
    };
}

pub fn read_bytes_to_end(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut buf = vec![];
    reader.read_to_end(&mut buf)?;
    return Ok(buf);
}

/// Same shape as `read_compressed_boxed_array_from`, but for a `Vec<u8>` whose length is only known
/// at read time (not a compile-time constant), e.g. a field whose decompressed size depends on a
/// runtime condition such as vanilla-vs-Poseidon.
pub(crate) fn read_compressed_vec_from(reader: &mut impl Read, expected_len: usize) -> io::Result<Vec<u8>> {
    let compressed_size = i32::read_from(reader)?;

    if compressed_size < 0 {
        return read_vec_from(reader, expected_len);
    } else {
        let mut compressed = vec![0; to_usize(compressed_size)?];
        reader.read_exact(&mut compressed)?;
        let mut decompressed_reader = Cursor::new(explode(&compressed)?);
        return read_vec_from(&mut decompressed_reader, expected_len);
    };
}
pub fn to_usize<T>(value: T) -> io::Result<usize>
where
    usize: TryFrom<T>,
{
    usize::try_from(value).map_err(|_| Error::new(ErrorKind::InvalidInput, "Failed to convert value to usize."))
}
