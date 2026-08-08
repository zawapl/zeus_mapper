//! Bit-level reader/writer matching the exact bit-packing order the real game's compressor and
//! decompressor use.
//!
//! Both pack/unpack bits LSB-first within each byte, and a call for more than 8 bits recurses,
//! writing/reading the low 8 bits first.

/// Accumulates bits LSB-first into a growable byte buffer.
///
/// **Assumptions**: never called with more than 16 bits in a single `write_bits` call (the real
/// compressor never does either).
pub(crate) struct BitWriter {
    bytes: Vec<u8>,
    bits_used: u32,
}

impl BitWriter {
    pub(crate) fn new() -> Self {
        return BitWriter {
            bytes: vec![0u8],
            bits_used: 0,
        };
    }

    pub(crate) fn write_bits(&mut self, num_bits: u32, value: u32) {
        if num_bits > 8 {
            self.write_bits(8, value);
            self.write_bits(num_bits - 8, value >> 8);
            return;
        }

        let current_bits_used = self.bits_used;
        let last = self.bytes.len() - 1;
        let shifted_value = ((value << self.bits_used) & 0xFF) as u8;
        self.bytes[last] |= shifted_value;
        self.bits_used += num_bits;

        if self.bits_used == 8 {
            self.bytes.push(0);
            self.bits_used = 0;
        } else if self.bits_used > 8 {
            self.bytes.push((value >> (8 - current_bits_used)) as u8);
            self.bits_used -= 8;
        }

        return;
    }

    /// Finishes the stream, dropping the trailing byte if it was never written into.
    pub(crate) fn finish(mut self) -> Vec<u8> {
        if self.bits_used == 0 && self.bytes.len() > 1 {
            self.bytes.pop();
        }
        return self.bytes;
    }
}

/// Reads bits LSB-first from a byte slice via a 16-bit sliding window, matching the real
/// decompressor's exact refill semantics.
pub(crate) struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    current_bits: u16,
    bits_available: i32,
}

impl<'a> BitReader<'a> {
    /// Creates a reader whose sliding window is pre-loaded with `first_byte` and no bits
    /// "available" beyond it, matching how the real decompressor primes its window before reading
    /// the first token. `data` is everything in the compressed stream after that priming byte.
    pub(crate) fn new(data: &'a [u8], first_byte: u8) -> Self {
        return BitReader {
            data,
            pos: 0,
            current_bits: first_byte as u16,
            bits_available: 0,
        };
    }

    /// Peek at the current 16-bit sliding window, consulted before consuming bits, matching the
    /// real decompressor.
    pub(crate) fn peek(&self) -> u16 {
        return self.current_bits;
    }

    /// Consumes `num_bits` (<= 8) from the sliding window, refilling one byte from `data` if
    /// needed.
    ///
    /// Returns `false` (matching the real decompressor's failure return) if a refill was needed
    /// but no more input bytes remain.
    pub(crate) fn set_bits_used(&mut self, num_bits: u32) -> bool {
        if self.bits_available >= num_bits as i32 {
            self.bits_available -= num_bits as i32;
            self.current_bits >>= num_bits;
            return true;
        }

        self.current_bits >>= self.bits_available as u32;
        if self.pos >= self.data.len() {
            return false;
        }
        let next_byte = self.data[self.pos];
        self.pos += 1;
        self.current_bits |= (next_byte as u16) << 8;
        self.current_bits >>= num_bits - self.bits_available as u32;
        self.bits_available += 8 - num_bits as i32;
        return true;
    }
}

// Round-trip correctness for BitWriter+BitReader together (including the exact refill/priming
// convention the real decompressor uses) is covered by the much more representative tests in
// `pkware::tests` (encode real multi-KB data with `implode`, decode it back with `explode`,
// assert the result matches byte-for-byte). A synthetic low-level test here would need to
// fabricate a plausible header + trailing-slack shape by hand to be representative, which the
// higher-level tests already give for free by construction.
