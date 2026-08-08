//! PKWARE DCL "implode" (compression). Raw pointer arithmetic into a fixed-size input buffer
//! becomes plain `usize` indices into a `Vec`, and manual output-buffer flush bookkeeping is
//! replaced by a growable `BitWriter` - the bit-level encoding itself matches exactly.
//!
//! **Assumptions**: only binary mode (`has_literal_encoding == 0`) is implemented, since the real
//! compressor always writes binary mode and the matching decompressor rejects ASCII mode outright
//! - real save files therefore never use it.

use crate::pkware::DictionarySize;
use crate::pkware::bit_io::BitWriter;
use crate::pkware::tables::CODEWORD_COUNT;
use crate::pkware::tables::COPY_LENGTH_BASE_BITS;
use crate::pkware::tables::COPY_LENGTH_BASE_CODE;
use crate::pkware::tables::COPY_LENGTH_EXTRA_BITS;
use crate::pkware::tables::COPY_OFFSET_BITS;
use crate::pkware::tables::COPY_OFFSET_CODE;
use crate::pkware::tables::EOF_CODE_INDEX;
use crate::pkware::tables::MAX_REP_LENGTH;

const LOOKAHEAD: usize = 516;
const CHUNK: usize = 4096;
/// Fixed input window size (dictionary + lookahead + one chunk), sized for the largest dictionary
/// (4K) regardless of the one actually in use, matching the real compressor's fixed-size input
/// buffer.
const INPUT_BUF_SIZE: usize = 4096 + LOOKAHEAD + CHUNK;
/// Extra padding beyond `INPUT_BUF_SIZE`. On the final (EOF) chunk, the read position can reach
/// close to the end of that buffer, and match-extension (both the initial scan and the
/// KMP-assisted phase that looks for an even longer match one position further in) can walk up to
/// `MAX_REP_LENGTH` bytes past that looking for a run to extend - a real reach for low-entropy
/// data (e.g. mostly-zero tile arrays). The real compressor's fixed-size buffer tolerates this by
/// silently reading adjacent memory; this pads generously instead so the same reads stay
/// in-bounds and deterministically read zero.
const INPUT_DATA_SIZE: usize = INPUT_BUF_SIZE + MAX_REP_LENGTH + 8;
const ANALYZE_TABLE_SIZE: usize = 2304;

#[derive(Clone, Copy, Default)]
struct CopyLengthOffset {
    length: usize,
    offset: u32,
}

struct ImplodeState<'a> {
    input: &'a [u8],
    input_read_pos: usize,

    window_size: u32,
    dictionary_size: usize,
    copy_offset_extra_mask: u32,

    input_data: Vec<u8>,
    analyze_offset_table: Vec<u16>,
    analyze_index: Vec<u16>,
    long_matcher: Vec<i16>,

    codeword_values: [u16; CODEWORD_COUNT],
    codeword_bits: [u8; CODEWORD_COUNT],

    writer: BitWriter,
}

/// Implodes `data`, matching the real game's own compressor output byte-for-byte.
pub(crate) fn implode(data: &[u8], dictionary_size: DictionarySize) -> Vec<u8> {
    let mut state = ImplodeState::new(data, dictionary_size);
    state.run();
    return state.writer.finish();
}

impl<'a> ImplodeState<'a> {
    fn new(input: &'a [u8], dictionary_size: DictionarySize) -> Self {
        let mut codeword_bits = [0u8; CODEWORD_COUNT];
        let mut codeword_values = [0u16; CODEWORD_COUNT];

        for i in 0..256usize {
            codeword_bits[i] = 9;
            codeword_values[i] = (i << 1) as u16;
        }

        let mut code_index = 256usize;
        for copy in 0..16usize {
            let base_bits = COPY_LENGTH_BASE_BITS[copy] as u32;
            let extra_bits = COPY_LENGTH_EXTRA_BITS[copy] as u32;
            let base_code = COPY_LENGTH_BASE_CODE[copy] as u32;
            let max = 1u32 << extra_bits;
            for i in 0..max {
                codeword_bits[code_index] = (1 + base_bits + extra_bits) as u8;
                codeword_values[code_index] = (1 | (base_code << 1) | (i << (base_bits + 1))) as u16;
                code_index += 1;
            }
        }
        debug_assert_eq!(code_index, CODEWORD_COUNT);

        let mut writer = BitWriter::new();
        writer.write_bits(8, 0); // no literal encoding
        writer.write_bits(8, dictionary_size.window_size());

        return ImplodeState {
            input,
            input_read_pos: 0,
            window_size: dictionary_size.window_size(),
            dictionary_size: dictionary_size.bytes(),
            copy_offset_extra_mask: (1u32 << dictionary_size.window_size()) - 1,
            input_data: vec![0u8; INPUT_DATA_SIZE],
            analyze_offset_table: vec![0u16; ANALYZE_TABLE_SIZE],
            analyze_index: vec![0u16; INPUT_BUF_SIZE],
            long_matcher: vec![0i16; 518],
            codeword_values,
            codeword_bits,
            writer,
        };
    }

    /// Copies up to `bytes_to_read` bytes from `self.input` into the input buffer. The real
    /// compressor does this via a caller-supplied input callback invoked in a retry loop (it reads
    /// from a file); our in-memory input never short-reads, so this is a single slice copy instead.
    fn fill_input_buffer(&mut self, bytes_to_read: usize) -> usize {
        let available = self.input.len() - self.input_read_pos;
        let used = available.min(bytes_to_read);
        let dest_start = self.dictionary_size + LOOKAHEAD;
        self.input_data[dest_start..dest_start + used].copy_from_slice(&self.input[self.input_read_pos..self.input_read_pos + used]);
        self.input_read_pos += used;
        return used;
    }

    /// Builds a hash-bucket sorted index over the given input range (byte-pair hashed), matching
    /// the real compressor's analysis pass.
    fn analyze_input(&mut self, input_start: usize, input_end: usize) {
        self.analyze_offset_table.fill(0);
        for index in input_start..input_end {
            let hash = 4 * self.input_data[index] as usize + 5 * self.input_data[index + 1] as usize;
            self.analyze_offset_table[hash] += 1;
        }

        let mut running_total = 0u32;
        for i in 0..ANALYZE_TABLE_SIZE {
            running_total += self.analyze_offset_table[i] as u32;
            self.analyze_offset_table[i] = running_total as u16;
        }

        for index in (input_start..input_end).rev() {
            let hash = 4 * self.input_data[index] as usize + 5 * self.input_data[index + 1] as usize;
            self.analyze_offset_table[hash] -= 1;
            let value = self.analyze_offset_table[hash] as usize;
            self.analyze_index[value] = index as u16;
        }

        return;
    }

    /// Finds the best copy (length, offset) match at `input_index`, matching the real compressor's
    /// match search exactly - including its second phase that uses a KMP failure function
    /// (`long_matcher`) to look for an even longer match once an initial match longer than 10
    /// bytes has been found.
    fn determine_copy(&mut self, input_index: usize) -> CopyLengthOffset {
        let hash_value = 4 * self.input_data[input_index] as usize + 5 * self.input_data[input_index + 1] as usize;
        let mut hash_analyze_index = self.analyze_offset_table[hash_value] as usize;
        let min_match_index = input_index + 1 - self.dictionary_size;

        if (self.analyze_index[hash_analyze_index] as usize) < min_match_index {
            loop {
                hash_analyze_index += 1;
                if (self.analyze_index[hash_analyze_index] as usize) >= min_match_index {
                    break;
                }
            }
            self.analyze_offset_table[hash_value] = hash_analyze_index as u16;
        }

        let mut max_matched_bytes: usize = 1;
        let prev_input_index = input_index - 1;
        let mut start_match = self.analyze_index[hash_analyze_index] as usize;
        if prev_input_index <= start_match {
            return CopyLengthOffset { length: 0, offset: 0 };
        }

        let mut copy = CopyLengthOffset { length: 0, offset: 0 };

        loop {
            if self.input_data[start_match + max_matched_bytes - 1] == self.input_data[input_index + max_matched_bytes - 1]
                && self.input_data[start_match] == self.input_data[input_index]
            {
                let mut start_match_plus = start_match + 1;
                let mut input_ptr_plus = input_index + 1;
                let mut matched_bytes: usize = 2;
                loop {
                    start_match_plus += 1;
                    input_ptr_plus += 1;
                    if self.input_data[start_match_plus] != self.input_data[input_ptr_plus] {
                        break;
                    }
                    matched_bytes += 1;
                    if matched_bytes >= MAX_REP_LENGTH {
                        break;
                    }
                }
                if matched_bytes >= max_matched_bytes {
                    // signed: start_match_plus can transiently exceed input_index here even
                    // though the final distance is always positive (the reference uses `int`)
                    copy.offset = (input_index as i64 - start_match_plus as i64 - 1 + matched_bytes as i64) as u32;
                    max_matched_bytes = matched_bytes;
                    if matched_bytes > 10 {
                        break;
                    }
                }
            }
            hash_analyze_index += 1;
            start_match = self.analyze_index[hash_analyze_index] as usize;
            if prev_input_index <= start_match {
                copy.length = if max_matched_bytes < 2 { 0 } else { max_matched_bytes };
                return copy;
            }
        }

        if max_matched_bytes == MAX_REP_LENGTH {
            copy.length = max_matched_bytes;
            copy.offset -= 1;
            return copy;
        }
        if (self.analyze_index[hash_analyze_index + 1] as usize) >= prev_input_index {
            copy.length = max_matched_bytes;
            return copy;
        }

        // Complex algorithm for finding an even longer match, using a KMP failure function built
        // over input_data[input_index..input_index+max_matched_bytes].
        let mut long_offset: i32 = 0;
        let mut long_index: usize = 1;
        self.long_matcher[0] = -1;
        self.long_matcher[1] = 0;
        loop {
            if long_index >= max_matched_bytes {
                break;
            }
            if self.input_data[input_index + long_index] != self.input_data[input_index + long_offset as usize] {
                long_offset = self.long_matcher[long_offset as usize] as i32;
                if long_offset != -1 {
                    continue;
                }
            }
            long_index += 1;
            long_offset += 1;
            self.long_matcher[long_index] = long_offset as i16;
        }

        let mut matched_bytes: i32 = max_matched_bytes as i32;
        let mut match_pos: usize = max_matched_bytes + self.analyze_index[hash_analyze_index] as usize;

        loop {
            matched_bytes = self.long_matcher[matched_bytes as usize] as i32;
            if matched_bytes == -1 {
                matched_bytes = 0;
            }

            let mut better_match_pos;
            loop {
                hash_analyze_index += 1;
                better_match_pos = self.analyze_index[hash_analyze_index] as usize;
                if better_match_pos >= prev_input_index {
                    copy.length = max_matched_bytes;
                    return copy;
                }
                if better_match_pos + matched_bytes as usize >= match_pos {
                    break;
                }
            }

            if self.input_data[input_index + max_matched_bytes - 2] != self.input_data[better_match_pos + max_matched_bytes - 2] {
                loop {
                    hash_analyze_index += 1;
                    better_match_pos = self.analyze_index[hash_analyze_index] as usize;
                    if better_match_pos >= prev_input_index {
                        copy.length = max_matched_bytes;
                        return copy;
                    }
                    if self.input_data[better_match_pos + max_matched_bytes - 2] == self.input_data[input_index + max_matched_bytes - 2]
                        && self.input_data[better_match_pos] == self.input_data[input_index]
                    {
                        matched_bytes = 2;
                        match_pos = better_match_pos + 2;
                        break;
                    }
                }
            } else if better_match_pos + matched_bytes as usize != match_pos {
                matched_bytes = 0;
                match_pos = better_match_pos;
            }

            while self.input_data[input_index + matched_bytes as usize] == self.input_data[match_pos] {
                matched_bytes += 1;
                if matched_bytes as usize >= MAX_REP_LENGTH {
                    break;
                }
                match_pos += 1;
            }

            if matched_bytes as usize >= max_matched_bytes {
                copy.offset = (input_index - better_match_pos - 1) as u32;
                if matched_bytes as usize > max_matched_bytes {
                    max_matched_bytes = matched_bytes as usize;
                    if matched_bytes as usize == MAX_REP_LENGTH {
                        copy.length = MAX_REP_LENGTH;
                        return copy;
                    }
                    loop {
                        if long_index >= matched_bytes as usize {
                            break;
                        }
                        if self.input_data[input_index + long_index] != self.input_data[input_index + long_offset as usize] {
                            long_offset = self.long_matcher[long_offset as usize] as i32;
                            if long_offset != -1 {
                                continue;
                            }
                        }
                        long_index += 1;
                        long_offset += 1;
                        self.long_matcher[long_index] = long_offset as i16;
                    }
                }
            }
        }
    }

    /// Returns whether the match one position ahead is strictly better - used to decide whether to
    /// defer a short match in favor of a literal + a better match next. This mirrors logic that's
    /// inlined directly into the real compressor's main loop, pulled out into its own function
    /// here for readability.
    fn next_copy_is_better(&mut self, offset: usize, current_copy: &CopyLengthOffset) -> bool {
        let next_copy = self.determine_copy(offset + 1);
        if current_copy.length >= next_copy.length {
            return false;
        }
        if current_copy.length + 1 == next_copy.length && current_copy.offset <= 128 {
            return false;
        }
        return true;
    }

    fn write_copy_length_offset(&mut self, copy: CopyLengthOffset) {
        let code_index = copy.length + 254;
        self.writer
            .write_bits(self.codeword_bits[code_index] as u32, self.codeword_values[code_index] as u32);

        if copy.length == 2 {
            let idx = (copy.offset >> 2) as usize;
            self.writer.write_bits(COPY_OFFSET_BITS[idx] as u32, COPY_OFFSET_CODE[idx] as u32);
            self.writer.write_bits(2, copy.offset & 3);
        } else {
            let idx = (copy.offset >> self.window_size) as usize;
            self.writer.write_bits(COPY_OFFSET_BITS[idx] as u32, COPY_OFFSET_CODE[idx] as u32);
            self.writer.write_bits(self.window_size, copy.offset & self.copy_offset_extra_mask);
        }

        return;
    }

    fn run(&mut self) {
        let mut eof = false;
        let mut has_leftover_data = 0u32;
        let mut input_ptr = self.dictionary_size + LOOKAHEAD;

        while !eof {
            let bytes_read = self.fill_input_buffer(CHUNK);
            if bytes_read != CHUNK {
                eof = true;
                if bytes_read == 0 && has_leftover_data == 0 {
                    break;
                }
            }

            let mut input_end = self.dictionary_size + bytes_read;
            if eof {
                input_end += LOOKAHEAD;
            }

            if has_leftover_data == 0 {
                self.analyze_input(input_ptr, input_end + 1);
                has_leftover_data += 1;
                if self.dictionary_size != 4096 {
                    has_leftover_data += 1;
                }
            } else if has_leftover_data == 1 {
                self.analyze_input(input_ptr - self.dictionary_size + LOOKAHEAD, input_end + 1);
                has_leftover_data += 1;
            } else if has_leftover_data == 2 {
                self.analyze_input(input_ptr - self.dictionary_size, input_end + 1);
            }

            while input_ptr < input_end {
                let mut copy = self.determine_copy(input_ptr);

                // a length-2 match with too large an offset can't be encoded at all (see
                // encode_match's bounds check in write_copy_length_offset), so it must fall back
                // to a literal
                let too_short_to_encode = copy.length == 2 && copy.offset >= 256;

                let write_copy = if copy.length == 0 || too_short_to_encode {
                    false
                } else if eof && input_ptr + copy.length > input_end {
                    copy.length = input_end - input_ptr;
                    copy.length > 2 || (copy.length == 2 && copy.offset < 256)
                } else if copy.length >= 8 || input_ptr + 1 >= input_end {
                    true
                } else {
                    !self.next_copy_is_better(input_ptr, &copy)
                };

                if write_copy {
                    self.write_copy_length_offset(copy);
                    input_ptr += copy.length;
                } else {
                    let byte = self.input_data[input_ptr];
                    self.writer
                        .write_bits(self.codeword_bits[byte as usize] as u32, self.codeword_values[byte as usize] as u32);
                    input_ptr += 1;
                }
            }

            if !eof {
                input_ptr -= CHUNK;
                let src_start = CHUNK;
                let len = self.dictionary_size + LOOKAHEAD;
                self.input_data.copy_within(src_start..src_start + len, 0);
            }
        }

        self.writer.write_bits(
            self.codeword_bits[EOF_CODE_INDEX] as u32,
            self.codeword_values[EOF_CODE_INDEX] as u32,
        );

        return;
    }
}
