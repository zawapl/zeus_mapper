# `WorldLocationData`: remaining unidentified per-location bytes

`WorldLocationData` (`zeus_mapper/src/file_data/world_location_data.rs`) has the diplomacy cluster,
`buying`/`selling`, names, and label position all wired up now (see `DATA_MAPPING.md`). What's left:

- `unknown_1: BoxedArray<u8, 91>`, `unknown_2: BoxedArray<u8, 55>`, `unknown_3: BoxedArray<u8, 100>`,
  `unknown_4: BoxedArray<u8, 100>` - four blocks, ~346 bytes total, no content survey done yet.
- `unknown_5: Vec<u8>` (extras-only, 28 bytes when present) and `tail: Vec<u8>` - trailing,
  manifest-driven, also unsurveyed.
- **`leader_name: u32`'s text-table group** (for older/non-extras files, where `custom_leader_name` is
  empty and this numeric id is used instead - small/double-digit values seen). Presumably an index into
  a group in `l10n::eng_text::EngText`'s group/string-table structure, but which group hasn't been
  cross-referenced against a real adventure's known leader names yet.

## Next steps

- Byte-density/constant-fill survey first (the same method that resolved `MythologyData.constant_1_0xff`
  and `SettingsData`'s tab-visibility split): flag which of these blocks are 100%-constant across every
  real sample (likely inert filler, name `constant_N_0x??` and move on) vs. genuinely varying (worth
  deeper investigation).
- For `leader_name`'s text-table group: find an adventure with a known, distinctive in-game leader name
  for a non-extras `WorldLocationData` slot, dump `EngText`'s groups for that adventure, and search for
  the string at the index `leader_name` predicts.
