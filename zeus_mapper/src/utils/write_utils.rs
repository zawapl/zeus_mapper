use crate::pkware::DictionarySize;
use crate::pkware::implode;
use encoding_rs::WINDOWS_1252;
use std::io;
use std::io::Cursor;
use std::io::Write;

pub(crate) const DICTIONARY_SIZE: DictionarySize = DictionarySize::Size4K;

pub trait WriteTo: Sized {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize>;
}

impl WriteTo for u8 {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let bytes = [*self];
        writer.write_all(&bytes)?;
        return Ok(bytes.len());
    }
}

impl WriteTo for u16 {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let bytes = u16::to_le_bytes(*self);
        writer.write_all(&bytes)?;
        return Ok(bytes.len());
    }
}

impl WriteTo for i16 {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let bytes = i16::to_le_bytes(*self);
        writer.write_all(&bytes)?;
        return Ok(bytes.len());
    }
}

impl WriteTo for u32 {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let bytes = u32::to_le_bytes(*self);
        writer.write_all(&bytes)?;
        return Ok(bytes.len());
    }
}

impl WriteTo for i32 {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let bytes = i32::to_le_bytes(*self);
        writer.write_all(&bytes)?;
        return Ok(bytes.len());
    }
}

impl<T: WriteTo, const N: usize> WriteTo for [T; N] {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut written_bytes = 0;
        for element in self {
            written_bytes += element.write_to(writer)?;
        }
        return Ok(written_bytes);
    }
}

pub(crate) fn write_string_to<W: Write>(str: &str, writer: &mut W, bytes: usize) -> io::Result<usize> {
    let mut buf = vec![0; bytes];
    let (data, _encoding, _errs) = WINDOWS_1252.encode(str);
    if data.len() > bytes {
        buf.copy_from_slice(&data[..bytes]);
    } else {
        buf[..data.len()].copy_from_slice(&data);
    }
    writer.write_all(&buf)?;
    return Ok(buf.len());
}

impl<T: WriteTo> WriteTo for Vec<T> {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut written_bytes = 0;
        for element in self {
            written_bytes += element.write_to(writer)?;
        }
        return Ok(written_bytes);
    }
}

pub(crate) fn write_maybe_compressed<W: Write>(uncompressed: &[u8], writer: &mut W) -> io::Result<usize> {
    let compressed = implode(uncompressed, DICTIONARY_SIZE);

    let mut bytes = 0;
    if compressed.len() < uncompressed.len() {
        bytes += WriteTo::write_to(&(compressed.len() as i32), writer)?;
        bytes += WriteTo::write_to(&compressed, writer)?;
    } else {
        bytes += WriteTo::write_to(&i32::MIN, writer)?;
        writer.write_all(uncompressed)?;
        bytes += uncompressed.len();
    }
    return Ok(bytes);
}

pub(crate) fn write_compressed<T: WriteTo, W: Write>(data: &T, writer: &mut W) -> io::Result<usize> {
    let mut buf_writer = Cursor::new(vec![]);
    WriteTo::write_to(data, &mut buf_writer)?;
    return write_maybe_compressed(buf_writer.get_ref(), writer);
}

#[cfg(test)]
mod tests {
    use crate::utils::boxed_array::BoxedArray;
    use crate::utils::read_utils::read_compressed_boxed_array_from;
    use crate::utils::write_utils::write_compressed;
    use std::io::Cursor;

    #[test]
    fn compress_decompress() {
        let data: BoxedArray<u16, 51984> = BoxedArray::from_vec(vec![1; 51984]);

        let compressed = vec![];
        let mut data_writer = Cursor::new(compressed);
        write_compressed(&data, &mut data_writer).unwrap();

        let mut data_reader = Cursor::new(data_writer.into_inner());
        let reconstructed: BoxedArray<u16, 51984> = read_compressed_boxed_array_from(&mut data_reader).unwrap();

        assert_eq!(*data, *reconstructed);
    }
}
