# `SavData.field_37`: what per-tile value is this?

`field_37` (`zeus_mapper/src/file_data/sav_data.rs`) is a raw, uncompressed 228x228 byte grid (one
value per tile, same framing as `random`/`meadow`/`scrub`).

**Confirmed live gameplay data, not filler.** Every `Save Tests/` quick-save (seconds into a fresh
game) reads uniformly `0xFF` across all 51,984 tiles - this originally looked like constant filler.
But every `Theocritos/` save (a real, played-for-a-while city) is *not* uniform: 4 to 69 distinct
values per save, clustered/smooth-looking (nearby values grouping together, not noise) rather than
random. So this starts at `0xFF` and changes with play - a real slowly-changing per-tile counter.

Candidates, none tested: soil fertility/depletion, appeal, or a similar accumulating per-tile stat.

## Next steps

- Correlate against tile properties that plausibly affect fertility/appeal: elevation, distance from
  water, adjacency to specific building types (needs `BuildingData.x`/`.y` -> tile mapping first, see
  `sav_file/building_unit_local_coordinates.md`).
- Watch it change: diff `field_37` between two saves of the same city a few in-game months apart
  (`Theocritos/` has several same-city save pairs already) and see which tiles moved, by how much, and
  whether the direction/rate is consistent with any of the candidate mechanics.
