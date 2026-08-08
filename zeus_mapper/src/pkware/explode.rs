//! PKWARE DCL "explode" (decompression). Used to validate `implode`'s output against the same
//! algorithm the real game uses to read it.

use crate::pkware::bit_io::BitReader;
use crate::pkware::tables::COPY_LENGTH_BASE_BITS;
use crate::pkware::tables::COPY_LENGTH_BASE_CODE;
use crate::pkware::tables::COPY_LENGTH_BASE_VALUE;
use crate::pkware::tables::COPY_LENGTH_EXTRA_BITS;
use crate::pkware::tables::COPY_OFFSET_BITS;
use crate::pkware::tables::COPY_OFFSET_CODE;
use std::io;
use std::io::Error;
use std::io::ErrorKind;

const PK_EOF: i32 = 773;
const PK_ERROR_VALUE: i32 = 774;

fn build_jump_table(bits: &[u8], codes: &[u8], jump: &mut [u8; 256]) {
    for i in (0..bits.len()).rev() {
        let bit = bits[i];
        let mut code = codes[i] as u32;
        loop {
            jump[code as usize] = i as u8;
            code += 1 << bit;
            if code >= 0x100 {
                break;
            }
        }
    }
    return;
}

struct ExplodeState<'a> {
    reader: BitReader<'a>,
    window_size: u32,
    dict_mask: u16,
    copy_length_jump_table: [u8; 256],
    copy_offset_jump_table: [u8; 256],
}

/// Explodes `compressed`, matching the real game's own decompressor output exactly.
///
/// Decodes until the end-of-stream marker is reached, returning everything produced.
pub(crate) fn explode(compressed: &[u8]) -> io::Result<Vec<u8>> {
    if compressed.len() <= 4 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "pkware explode: too few input bytes"));
    }

    let has_literal_encoding = compressed[0];
    let window_size = compressed[1] as u32;
    if !(4..=6).contains(&window_size) {
        return Err(Error::new(ErrorKind::InvalidData, "pkware explode: invalid window size"));
    }
    if has_literal_encoding != 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "pkware explode: ASCII literal encoding is unsupported",
        ));
    }

    let mut state = ExplodeState {
        reader: BitReader::new(&compressed[3..], compressed[2]),
        window_size,
        dict_mask: (0xFFFFu32 >> (16 - window_size)) as u16,
        copy_length_jump_table: [0; 256],
        copy_offset_jump_table: [0; 256],
    };

    build_jump_table(&COPY_LENGTH_BASE_BITS, &COPY_LENGTH_BASE_CODE, &mut state.copy_length_jump_table);
    build_jump_table(&COPY_OFFSET_BITS, &COPY_OFFSET_CODE, &mut state.copy_offset_jump_table);

    let mut output = Vec::new();
    let result = state.decode(&mut output)?;
    if result != PK_EOF {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "pkware explode: stream did not end with a valid EOF marker",
        ));
    }

    return Ok(output);
}

impl<'a> ExplodeState<'a> {
    fn decode_next_token(&mut self) -> i32 {
        if self.reader.peek() & 1 != 0 {
            if !self.reader.set_bits_used(1) {
                return PK_ERROR_VALUE;
            }
            let mut index = self.copy_length_jump_table[(self.reader.peek() & 0xff) as usize] as usize;
            if !self.reader.set_bits_used(COPY_LENGTH_BASE_BITS[index] as u32) {
                return PK_ERROR_VALUE;
            }
            let extra_bits = COPY_LENGTH_EXTRA_BITS[index] as u32;
            if extra_bits != 0 {
                let extra_bits_value = (self.reader.peek() as u32) & ((1u32 << extra_bits) - 1);
                let ok = self.reader.set_bits_used(extra_bits);
                if !ok && index as u32 + extra_bits_value != 270 {
                    return PK_ERROR_VALUE;
                }
                index = (COPY_LENGTH_BASE_VALUE[index] as u32 + extra_bits_value) as usize;
            }
            return (index + 256) as i32;
        } else {
            if !self.reader.set_bits_used(1) {
                return PK_ERROR_VALUE;
            }
            let result = (self.reader.peek() & 0xff) as i32;
            if !self.reader.set_bits_used(8) {
                return PK_ERROR_VALUE;
            }
            return result;
        }
    }

    fn get_copy_offset(&mut self, copy_length: i32) -> u32 {
        let index = self.copy_offset_jump_table[(self.reader.peek() & 0xff) as usize] as usize;
        if !self.reader.set_bits_used(COPY_OFFSET_BITS[index] as u32) {
            return 0;
        }
        let offset;
        if copy_length == 2 {
            offset = ((self.reader.peek() & 3) as u32) | ((index as u32) << 2);
            if !self.reader.set_bits_used(2) {
                return 0;
            }
        } else {
            offset = ((self.reader.peek() & self.dict_mask) as u32) | ((index as u32) << self.window_size);
            if !self.reader.set_bits_used(self.window_size) {
                return 0;
            }
        }
        return offset + 1;
    }

    fn decode(&mut self, output: &mut Vec<u8>) -> io::Result<i32> {
        let mut token;
        loop {
            token = self.decode_next_token();
            if token >= PK_ERROR_VALUE - 1 {
                break;
            }
            if token >= 256 {
                let length = token - 254;
                let offset = self.get_copy_offset(length);
                if offset == 0 {
                    token = PK_ERROR_VALUE;
                    break;
                }
                if offset as usize > output.len() {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "pkware explode: copy offset before start of output",
                    ));
                }
                for _ in 0..length {
                    let byte = output[output.len() - offset as usize];
                    output.push(byte);
                }
            } else {
                output.push(token as u8);
            }
        }
        return Ok(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ascii_mode() {
        let compressed = [1u8, 6, 0, 0, 0];
        let err = explode(&compressed).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_bad_window_size() {
        let compressed = [0u8, 9, 0, 0, 0];
        let err = explode(&compressed).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_too_few_bytes() {
        let compressed = [0u8, 6, 0, 0];
        let err = explode(&compressed).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::UnexpectedEof);
    }
}
