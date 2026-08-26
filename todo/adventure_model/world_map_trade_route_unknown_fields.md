# `WorldMapElementData`/`TradeRoutePointData`: remaining unidentified bytes

`WorldMapElementData` (`zeus_mapper/src/file_data/world_map_element_data.rs`) has `variant`,
`region_name` (self-referential slot index), and `label_position` decoded. Still open:

- `sprite_id: u8` - likely indexes a sprite sheet (`Zeus_Interface.sg3`/`Zeus_empire_bits.BMP` per an
  existing code comment), not cross-referenced.
- `data_a: [u8; 3]` - `data_a[0]` is a constant `2` in every sample; `data_a[1]` varies `0..=5` with no
  confirmed correlation to anything else yet; `data_a[2]` not separately characterized.
- `data_d: [u8; 14]` - unsurveyed.
- `unknown_1: [u8; 6]` - byte 0 is known to be set to `13` specifically for the Atlantean rival (per an
  existing code comment); the rest is unsurveyed.
- `unknown_2: [u8; 4]` - unsurveyed.
- A resave-artifact worth remembering so it isn't mistaken for real data later: `sprite_id`/`unknown_1`
  shifted consistently across an editor resave in one observed case (`sprite_id` dropped by exactly 23,
  `unknown_1` flipped from `[67, 0]` to `[0, 13]`) - almost certainly an asset/texture-table version
  mismatch between the saving and loading editor builds, not live per-map content.

`TradeRoutePointData.unknown_1: u16` (one per point, 50 points per route) - not investigated at all; the
rest of `TradeRouteData` (`header`/`distance`/`route_type`/`points_count`/`exists`) is fully decoded.

## Next steps

- `sprite_id`: cross-reference against the sprite-sheet asset files named in the code comment if they're
  available, or against `data_a[1]`'s variation to see if the two correlate.
- `data_a`/`data_d`/`unknown_1`/`unknown_2`: byte-density survey first (same method as
  `world_location_unknown_fields.md`) to separate constant-fill from real content before investing in
  per-field hypotheses.
- `TradeRoutePointData.unknown_1`: check whether it varies per-point within a route (path-segment
  metadata) or is constant per-route (route-level flag duplicated across points).
