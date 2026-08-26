# Everything past `field_44` (and most content categories generally) has zero lead

`SavData`'s current skeleton was derived from studying a small, independent GPL-licensed minimap
extractor's own read logic (see the module doc in `sav_data.rs` / the "Where this skeleton's structure
came from" background if it still exists in `DATA_MAPPING.md`/module docs). That external tool only
reads far enough to render a minimap - terrain, a few resource-ish grids, `units`, `buildings` - so its
insight runs out well before the end of the file. `field_44` (raw bytes to EOF, minus the trailing 4-byte
`poseidon_marker`) is a single unparsed tail with no structure identified at all.

More broadly, several whole content categories a save file must contain have **no lead from any source
used so far**, not just an unparsed byte range:

- City finances (treasury, income/expenses).
- Resource stockpiles and warehouse contents.
- Individual citizen/unit AI or pathing state beyond bare position (`UnitData` has `type_id`/`x`/`y`
  confirmed; the rest of its 388-byte record is unidentified `field_N` placeholders).
- Building interiors/workers (`BuildingData` has `size`/`x`/`y`/`type_id`/`rotation` confirmed; the rest
  of its 280-byte record is unidentified).
- Diplomacy/trade state as actually played (as opposed to the `.pak`-authored starting state already
  modeled for `WorldLocationData`).
- Episode/goal progress - `SettingsData` models episode goals for `.pak` (partially - see
  `adventure_model/episode_goals_decoding.md`), but `.sav` has no confirmed equivalent for *progress*
  toward them.

## Next steps

This needs the same from-scratch approach `.pak`/`.map` originally got, since there's no shortcut
source to lean on here: hex-edit real save files, diff two saves taken minutes/months apart in the same
city to isolate what changed, and correlate against an in-game action (spend money, stockpile a good,
complete a goal) the way `terrain`'s road/forest-deletion diff or `field_37`'s played-vs-fresh diff did
elsewhere in this format.
