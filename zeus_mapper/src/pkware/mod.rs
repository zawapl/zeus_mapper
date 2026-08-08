//! An implementation of the PKWARE Data Compression Library ("implode"/"explode") algorithm used
//! by the Impressions Games city-builder engine (Caesar 3, Zeus, Pharaoh all share it) for
//! compressed sections of `.pak`/`.set`/`.map` files.
//!
//! `implode` compresses and `explode` decompresses; together they're wired into this crate's write
//! and read paths for every compressed field. `implode`'s output always produces a
//! validly-decodable stream matching this module's own `explode` (see `tests` below), and matches
//! the real game's own compressor byte-for-byte for the large majority of real adventure data.
//!
//! See `NOTES.md` in this directory for where this implementation comes from, how it was
//! validated, and the one known, well-understood gap in that byte-for-byte matching.

mod bit_io;
mod explode;
mod implode;
mod tables;

use std::io;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DictionarySize {
    Size1K,
    Size2K,
    Size4K,
}

impl DictionarySize {
    fn bytes(self) -> usize {
        return match self {
            DictionarySize::Size1K => 1024,
            DictionarySize::Size2K => 2048,
            DictionarySize::Size4K => 4096,
        };
    }

    fn window_size(self) -> u32 {
        return match self {
            DictionarySize::Size1K => 4,
            DictionarySize::Size2K => 5,
            DictionarySize::Size4K => 6,
        };
    }
}

/// Compresses `data` (always binary mode - real save files never use ASCII/literal mode).
pub(crate) fn implode(data: &[u8], dictionary_size: DictionarySize) -> Vec<u8> {
    return implode::implode(data, dictionary_size);
}

/// Decompresses `compressed`. Decodes until the end-of-stream marker; returns an error if the
/// stream is truncated, uses an unsupported mode, or doesn't end with a valid marker.
pub(crate) fn explode(compressed: &[u8]) -> io::Result<Vec<u8>> {
    return explode::explode(compressed);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(data: &[u8]) {
        let compressed = implode(data, DictionarySize::Size4K);
        let decompressed = explode(&compressed).unwrap_or_else(|e| panic!("explode failed for {} byte input: {e}", data.len()));
        assert_eq!(decompressed, data, "round trip mismatch for {} byte input", data.len());
    }

    #[test]
    fn round_trips_small_literal_data() {
        round_trip(b"Hello, World! This is a test of the PKWARE compression.");
    }

    #[test]
    fn round_trips_single_repeated_byte() {
        round_trip(&[0u8; 51984]);
        round_trip(&[0xFFu8; 51984]);
    }

    #[test]
    fn round_trips_incompressible_random_data() {
        // deterministic pseudo-random bytes, no external RNG dependency
        let mut state: u32 = 0x2545F491;
        let data: Vec<u8> = (0..51984)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                (state & 0xFF) as u8
            })
            .collect();
        round_trip(&data);
    }

    #[test]
    fn round_trips_various_sizes_around_chunk_boundaries() {
        for size in [1, 2, 3, 516, 517, 4095, 4096, 4097, 8191, 8192, 8193, 8708, 8709, 20000] {
            let data: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
            round_trip(&data);
        }
    }

    #[test]
    fn round_trips_repeating_pattern() {
        let data: Vec<u8> = (0..51984).map(|i| (i % 7) as u8).collect();
        round_trip(&data);
    }
}
