# Confirm `BuildingData`/`UnitData` `x`/`y` are local (not absolute) tile coordinates

**Single-sample finding, not yet promoted to a real fix or documented invariant.** On
`Adventures/tutorial02.sav` (`map_size = 140`), looking up `terrain[MapPosition::from_x_y(b.x, b.y)]`
for real `SavData.buildings` entries consistently landed on `OutOfBounds`-flagged tiles - impossible
for an actually-placed building. Scanning the whole `terrain` grid for the real playable area found it
sitting at absolute `x=44..183, y=44..183`, i.e. offset by `(228 - map_size) / 2 = 44` on each axis from
the grid's own frame. Adding that offset to a building's `x`/`y` before building a `MapPosition` landed
exactly on the expected tile (confirmed on an agora, a maintenance office, a common house).

**Working hypothesis**: `BuildingData.x`/`.y` (and presumably `UnitData.x`/`.y`, not separately checked
at all) are local tile coordinates relative to the map's own `0..map_size` frame, while the big grid
fields (`terrain`, `root_offset`, `tile_size`, `random`, `meadow`, `scrub`, etc.) are always addressed
in the fixed 228x228 canvas's absolute frame, centered for maps smaller than the full 228 size. The two
frames only coincide when `map_size == 228`, which is presumably why this wasn't caught during the
earlier `.pak`/`.sav` grid cross-referencing (those samples all happened to be full-size maps).

## Next steps

- Verify against a second, differently-sized map (only one `map_size = 140` sample checked so far).
- Check `UnitData.x`/`.y` under the same offset - not yet tested at all.
- Reconcile against the existing isometric bitmap-coordinate formula this crate already uses
  (`mapsize/2 + x - y - 1` / `1 + x + y - mapsize/2` in `map_helper::map_position`) - it's not yet clear
  whether that formula already implicitly assumes this local-vs-absolute split or needs adjusting too.
- Once confirmed on a second sample, promote this to a real helper (e.g. a `MapPosition`-returning
  method on `BuildingData`/`UnitData` that takes `map_size`) rather than leaving it as inline math at
  call sites.
