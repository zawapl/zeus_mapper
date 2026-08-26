# `MapData`/`RealEpisodeData`: remaining small unidentified fields

Byte-density survey done across every real `.pak` (65 files, 126 `MapData` records). One field turned
out fully inert and was promoted: `RealEpisodeData.unknown_5` ([u8; 2], the per-map `scenario_data` copy,
**not** `SettingsData.real_episode_data[]` - see the "Not a todo" note below and
`[[feedback_zeus_mapper_struct_fields]]`) is `0x00` in all 126 samples and is now `constant_8_0x00` in
`real_episode_data.rs`. Everything else surveyed carries real content or a structural lead worth
recording:

## `MapData`

- `unknown_3: BoxedArray<u8, 18628>` - **not** inert (11778/18628 bytes constant, but that leaves ~6850
  bytes of real content). Contains a repeating structure: a ~124-byte period starting somewhere before
  offset 17446, with an embedded byte that increments by exactly 1 every period (observed climbing
  `0x8D, 0x8E, 0x8F, ..., 0x95` across 9 consecutive stanzas near the end of the blob). Reads as a real
  record array (something indexed/counted, one record per ~124 bytes), not a single opaque blob - worth
  finding the record's start offset and total count next.
- `unknown_5: Vec<Vec<u8>>` - manifest-driven row length is not fixed at "52 bytes" as previously
  assumed; the real corpus has two distinct row lengths depending on the manifest: 40 bytes (276 rows
  surveyed, 36/40 constant) and 52 bytes (1382 rows surveyed, 43/52 constant). Both lengths carry real,
  non-filler content in the varying bytes.
- `unknown_6: [BoxedArray<u8, 76>; 6]` - 71/76 bytes constant per slot (flattened across all 6 slots and
  756 samples). Real content at byte 0 (3 distinct values), bytes 4-5 (2-4 distinct), byte 8 (4 distinct,
  up to 0x40), byte 12 (binary 0/8). Consistent with the existing "one-per-episode-slot small stats"
  hypothesis - small per-slot counters/flags, not junk.
- `unknown_7: u32` - mostly 0 (119/126 samples), otherwise 3 (6 samples) or 1 (1 sample). Small enum,
  not surveyed further.
- `unknown_8: u32` - mostly 0 or 1 (72 and 40 of 126 samples), otherwise 2, 3, 4, 5, or 10 (rare). Small
  counter, not surveyed further.

## `MapData.scenario_data` (`RealEpisodeData`, the per-map copy)

- `unknown_1: [u8; 8]` - only byte 0 varies (binary 0/4), the other 7 bytes are constant 0x00 in all 126
  samples. Narrowed from "8 unsurveyed bytes" to "1 real byte, likely boolean-ish".
- `unknown_3: BoxedArray<u8, 76>` - 69/76 constant. Real content: byte 16 (binary 0/0x0A), byte 20
  (binary 0/0x0A), byte 28 (binary 0/0x0A), bytes 40..=43 (each independently binary 0/1). Narrowed to 7
  meaningful bytes out of 76.
- `unknown_4: BoxedArray<u8, 40>` - 34/40 constant. Bytes 0..=2 carry real content (2-3 distinct values
  per column, up to 0xF4/244 - possibly overlapping a wider field rather than three independent bytes).
  **Notable lead**: byte 4 is binary 0/0x0A(10), byte 8 is binary 0/0x14(20), byte 12 is binary 0/0x1E(30)
  - three flags at a consistent 4-byte stride whose "on" values are exactly 10, 20, 30 respectively. Reads
  like three related thresholds/tiers (e.g. difficulty levels) rather than independent unrelated bytes.

Not a todo, but restating since it's easy to re-trip on: this file's `unknown_1`/`unknown_3`/`unknown_4`
figures are all from `MapData.scenario_data` (the per-map copy), not `SettingsData.real_episode_data[]`
(the settings-level copy) - a past investigation conflated the two and mistakenly called some of these
bytes constant based on checking only the settings-level copy.

## Next steps

1. `unknown_3`'s repeating ~124-byte record: find where the pattern actually starts (before offset
   17446) and confirm the period holds for the whole blob, then figure out what's counted/indexed by the
   incrementing byte.
2. `unknown_4` bytes 4/8/12 (10/20/30 stride): check correlation against difficulty settings or other
   already-decoded tier-like fields on the same episode.
3. `unknown_5` row-length split (40 vs 52 bytes): check which manifest slot/flag actually gates the
   length, and whether the 12-byte difference is a fixed trailing suffix or spread through the row.
