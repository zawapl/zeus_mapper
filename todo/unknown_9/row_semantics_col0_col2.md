# `MapData.unknown_9`: what do the 9 `u32` columns per row mean?

`unknown_9: [BoxedArray<u8, 36>; 10]` (`zeus_mapper/src/file_data/map_data.rs`) is the struct's last
field: 10 rows of 36 bytes, read uncompressed. Reinterpreted as `[u32; 9]` per row, the 9 columns behave
very differently from each other (a packed record, not a homogeneous array) - see the table below.
Row count (how many of the 10 rows are non-zero) correlates with how "developed"/edited the map is.

| Col | Shape | Status |
|---|---|---|
| col0 | large, varies per row (thousands-tens of thousands) | **Untested.** Candidate theories (aggregate cost, instance count, elapsed-time-since-built) differ in whether it should reset when a building is newly built vs. already present - worth revisiting once col2 is settled. |
| col1 | tiny, always even, `{0,2,4,6}` | Looks like an enum/direction/state flag. Untested. |
| col2 | `{2,3,4,5}` on most maps | **Strong lead**: on a handful of old-format adventures (`Hercules Labors`, `The Peloponnesian War`, `Perseus and Medusa`, `The Trojan War` map[1], `The Voyages of Jason`), col2 instead takes values from `{12,21,30,34,36,40,45,46,54,66,84}` - which matches, 11/11, the `BUILD_*` command-id table in `$ZEUS_HOME/Model/Zeus_Model_Normal.txt` (identical across difficulty variants, confirmed by md5). This is a stronger, better-attested match than this crate's own `BuildingType` enum, which is a *different, unrelated* id numbering (only 9/11 hit, and diverges outright on shared low ids). |
| col3/col4, col6/col7 | mid-range (tens-low hundreds) | Originally hypothesized as two `(x,y)` tile-coordinate pairs - **refuted for the general case** (28% terrain-flag agreement, below the ~70-78% baseline two random points would coincidentally hit). A rescoped test restricted to `BUILD_*`-tagged rows was attempted but came back uninformative (0/36 hits vs 0/9984 baseline) - not a refutation, just uninformative, because `unknown_9` lives in `.pak` scenario files where `terrain`'s `Building` flag is essentially never set (editor-authored maps rarely pre-place buildings); a real test needs `.sav` data or in-editor placement + resave, not `.pak` alone. |
| col5 | mid-range, similar scale to col3/4/6/7 | Untested independently. |
| col8 | `0`, or huge (~32-36M, occasionally ~92-120M) | Originally hypothesized as a live heap pointer - **refuted** (byte-for-byte identical across a plain in-game resave; a real pointer would differ across sessions). |

**Row-shape correlate**: on rows where col2 is one of the `BUILD_*`-matching outlier values, the huge/
pointer-like value shifts from col8 to col7 instead (with col8 forced to `0`), confirmed on every one of
19 rows checked - i.e. col2's value changes which of the 9 slots holds the trailing huge value, as if
the record is a tagged/variant layout keyed by col2.

**Complication**: within a single map, the *same* col2 value repeats across multiple rows (e.g.
`Hercules Labors` has all 10 rows at `col2=45`), which argues against "one row per distinct building
type" and toward "one row per build-related instance/event" instead - but 2 of the 11 matched ids (`30`,
`34`) are `(SUBMENU)` category headers in the model-file table, not placeable buildings, which a pure
per-instance reading struggles to explain.

## Also confirmed not the cause

- Not resource/price-indexed (offsets don't shift old-vs-new format the way `ResourceType` ids do; ~1.5%
  byte match against `prices`, noise level).
- Not touched by terrain edits in the editor (placed forest/sand/water tiles, `unknown_9` unchanged).
- Row count does not track `SettingsData.parent_episodes` or its sibling event-count arrays - an
  initially-promising 32% exact-match rate turned out to be a tutorial-stub artifact (all matches came
  from near-empty tutorial cities where both quantities happen to be 1-2; real content adventures
  matched 0/25).

## Next steps

1. Rerun the col3/4/6/7 coordinate test properly scoped to `BUILD_*`-tagged rows using `.sav` data
   (real placed-building records) instead of `.pak` scenario data, or via in-editor placement + resave.
2. With the in-game editor: place a building of a specific, currently-absent type in a colony whose
   `unknown_9` is already captured, resave, and check whether a *new* row appears with col2 equal to
   that building's `BUILD_*` id. Also test placing something from a `(SUBMENU)`-matching category (ids
   `30`/`34`) - if no row appears, that argues those two matches are coincidental.
3. Once col2 is settled, revisit col0 with the row-instance-vs-aggregate question in mind.
