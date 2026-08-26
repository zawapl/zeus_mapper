# `MapData`/`RealEpisodeData`: remaining small unidentified fields

After the naming-cleanup pass that promoted every confirmed-constant byte range to `constant_N_0x??`
and every genuinely-undeciphered one to `unknown_N`, what's left on `MapData`
(`zeus_mapper/src/file_data/map_data.rs`):

- `unknown_3: BoxedArray<u8, 18628>` - a single large opaque blob, unsurveyed.
- `unknown_5: Vec<Vec<u8>>` - manifest-driven (14 rows x up to 52 bytes each by default), unsurveyed.
- `unknown_6: [BoxedArray<u8, 76>; 6]`, `unknown_7: u32`, `unknown_8: u32` - unsurveyed.

And on the embedded `scenario_data: RealEpisodeData` (`zeus_mapper/src/file_data/real_episode_data.rs`)
- note this is per-map data, separate from `SettingsData.real_episode_data[]`, and a past investigation
mistakenly promoted these as constant based on checking only the settings-level copy (see
`[[feedback_zeus_mapper_struct_fields]]` memory) - any future survey must check both:

- `unknown_1: [u8; 8]`, `unknown_3: BoxedArray<u8, 76>`, `unknown_4: BoxedArray<u8, 40>`
  (previously noted to decode as something like a `u32` around 100,000 plus a couple of small counts,
  not independently re-verified against the current field boundaries), `unknown_5: [u8; 2]`.

Not a todo, but worth remembering so it isn't re-flagged: `MapData`'s own `mythology` field is a
map-level duplicate of `SettingsData.mythology[i]`, confirmed unused by the game (zeroing it changes
nothing in-game) - present in the file for round-trip fidelity only, not a decoding target.

## Next steps

Byte-density/constant-fill survey first (same method used throughout this crate - see
`MythologyData.constant_1_0xff`/`SettingsData`'s tab-visibility split for examples) across every real
`.pak`, to separate inert filler from real content before investing in per-field hypotheses. `unknown_6`
being an array of 6 identically-sized 76-byte blocks is suggestive of one-per-episode-slot data (6
colonies?) worth checking first.
