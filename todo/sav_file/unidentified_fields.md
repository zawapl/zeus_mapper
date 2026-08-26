# `SavData`: remaining unidentified `field_N`s

Most of `SavData` (`zeus_mapper/src/file_data/sav_data.rs`) now reads/round-trips cleanly (see
`SavData::validate` and the `validate_sav_files` test - 62/62 real `.sav` files), and every field has
a *confirmed shape* (scalar, or a `BoxedArray`/`Vec` of a known element type and, where fixed, a known
length). What's still missing for most of them is *meaning* - what the field actually represents.

Confirmed via cross-referencing against `.pak`'s `MapData` (byte-identical on fresh saves):
`root_offset`, `terrain`, `random`, `meadow`, `scrub`, `tile_size`. Confirmed inert filler (always `0`
on every real sample): `field_13`, `field_19`, `field_21`. Everything else below is open.

## 228x228-tile grids (one value per tile, shape confirmed via `manifest`)

`field_11` (`u16`), `field_14` (`u16`), `field_16` (`u8`), `field_18` (`u8`), `field_20` (`u16`),
`field_22` (`u8`), `field_40` (`u32`, a second full grid alongside `terrain`), `field_42` (`u8`). All
show real variation across tiles/saves (not constant), with no exact byte-match against any `.pak`
grid field or against each other. `field_40` is speculated (via an old external minimap tool's own
comment, not independently verified here) to relate to maintenance-officer coverage/pathing.

## Fixed-size blobs (shape confirmed via `pkware::explode` decompressed length / `manifest`)

`field_2` (19,184 bytes), `field_23` (2,000 bytes), `field_24` (500,000 bytes), `field_25` (15,600
bytes), `field_26` (4 bytes), `field_28` (60,000 bytes), `field_30`/`field_31` (`u16`, 500 each),
`field_32` (`u16`, 4,000), `field_36` (75,168 bytes - `manifest`'s `size=324,count=232` for this entry
is an unconfirmed lead toward `TradeRouteData`'s 232-slot array, same element count), `field_39` (36
bytes). `field_4`/`field_8` are the same category but vary by vanilla-vs-Poseidon (10,472/12,584 and
8,000/14,400 bytes respectively) rather than one fixed size.

## Never even shape-surveyed

`field_3` (8 bytes), `field_5` (188 bytes), `field_6` (600 bytes), `field_7` (1,383 bytes), `field_9`
(18,609 bytes), `field_27` (352 bytes), `field_29` (17,974 bytes - note 3 small scalars, matched to
`manifest` entries elsewhere, already live inside this range: `field_26b`'s width and the run that
became `field_30`/`31`/`32`, so this remaining size is what's left after those), `field_33` (53,783
bytes), `field_35` (16 bytes), `field_38` (32 bytes), `field_41` (39 bytes). These haven't had the
`manifest`-cross-reference or per-save content-variation treatment the fields above got - that's the
obvious first step (same method as everything above: match against a `manifest` entry by position +
byte length, then survey distinct-value count across `Save Tests/`/`Theocritos/` saves to tell "live
data" from "inert filler").

## Next steps

- For the grids and blobs with confirmed shape: correlate against gameplay (diff two `.sav`s of the
  same city saved minutes/months apart, the way `terrain`'s road/forest-deletion diff and `field_37`'s
  played-vs-fresh-save diff worked - see `sav_file/field_37_live_tile_tracker.md`).
- For the never-surveyed fields: run the `manifest`-position-and-length match first: `manifest` has
  ~139 populated entries and about a third are still unaccounted for.
- `field_1` (the single trailing byte right after `manifest`, distinct from the large `field_1`-turned-
  many-fields region above) is `0xFF` on most samples but `0x01` on a few
  (`Theocritos/Youngest Twins.sav`, `Theocritos/autosave_history.sav`, `tutorial15.sav`) - a real
  varying value, not constant padding, cause unknown.
