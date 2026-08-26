# `WorldLocationData`: remaining unidentified per-location bytes

Byte-density survey done across every real `.pak` (65 files, 1375 `exists != 0` location records, plus
989 with extras). One block turned out fully inert and was promoted to `constant_2_0x00` (was
`unknown_3`, 100 bytes, 0x00 in all 1375 samples) - see `validate()`/read/write in
`world_location_data.rs`. The rest are **not** filler; they carry real, still-undecoded content:

- `unknown_1` (91 bytes): only 43/91 bytes are constant. Real content: byte 0 (binary 0/1), byte 3
  (binary 0/0x0A), byte 5 (binary 0/4), byte 9 (26 distinct values, 0..=0x5F - a real count/id), byte 11
  (binary 0/1), byte 90 (binary 0/0x0C). Two suggestively paired 21-byte runs of all-binary-per-column
  flags at bytes 20..=40 and 56..=76 (same shape, same value domain `{0,1}` per column) - look like two
  parallel bit-flag arrays (e.g. two independent 21-item checklists), not yet correlated to anything.
- `unknown_2` (55 bytes): 45/55 constant. Real content confined to bytes 35..=44 (10 bytes): low distinct
  counts (2-5) per column, and from byte 38 onward the second-most-common value per column climbs by
  exactly 1 per byte (7, 8, 9, 10, 11, 12, 13) - looks like a template/default index sequence with some
  records overriding it, worth checking if it lines up with `slot_index` or trade-route indices.
- `unknown_4` (100 bytes): 98/100 constant. Byte 80 varies rarely (2/1375 samples, value 0x1C); byte 96
  is a real boolean-shaped flag (282/1375 samples are 1, rest 0).
- `unknown_5` (28-byte extras-only block): 27/28 constant. Byte 12 has real content: 22 distinct values,
  0..=0xC5(197).
- `leader_name: u32`'s text-table group (for non-extras files where `custom_leader_name` is empty) is
  still unresolved - not covered by this byte-density pass since it's a scalar `u32`, not a blob.

## Also corrected

`WorldMapElementData.data_a[0]`'s existing code comment ("likely indexes a sprite sheet... constant `2`
in every sample") does not hold up: across 2066 real `variant != 0` elements, byte 0 is `2` in only
75% of samples (1559/2066); the rest are 0 (199), 204 (70), 3 (42), 205 (28), and 15 other scattered
values. `data_a`/`unknown_1`/`unknown_2`/`data_d` on `WorldMapElementData` have **no** inert bytes at
all - every column carries real, varying content across the full corpus (see
`world_map_trade_route_unknown_fields.md`), so a byte-density pass won't narrow those further; they need
actual correlation work instead.

## Next steps

- `unknown_1`'s two paired 21-byte flag runs: check whether either run's popcount or specific bit
  correlates with `WorldLocationType`, `Relationship`, or the diplomacy cluster fields already decoded.
- `unknown_2` bytes 35..=44: check whether the climbing baseline value (7..13) is literally
  `byte_index - 31`, and if so what breaks the pattern in the records where it's overridden.
- `unknown_4` byte 96: strong boolean-flag candidate - cross-reference against something already decoded
  per-location (visibility, active state, trade-route-visible) to see if it duplicates or gates one of
  those.
- `leader_name`: find an adventure with a known, distinctive in-game leader name for a non-extras slot,
  dump `EngText`'s groups for that adventure, and search for the string at the index `leader_name`
  predicts.
