# `Adventure.world_settings: OwnCitySettings` is always zeroed defaults

`military_strength`/`economy_strength`/`favour` (the player's own city's diplomacy-cluster equivalents)
have no identified source at all - there's no in-game diplomacy screen for the player's own city to
cross-check against, unlike foreign `WorldLocation`s.

`traded_resources: Vec<TradedGoods>` (resource id + amount + buying/selling) might be reconstructable
from `RealEpisodeData.city_resources_bought`/`city_resources_sold` (both `[i8; 4]`, confirmed to decode
to valid `ResourceType` ids the same way `city_resources: [i8; 20]` does) cross-referenced with
`city_resources_quantity: BoxedArray<u8, 40>` - one spot-checked adventure suggested the first ~22
entries of `city_resources_quantity` are plausibly indexed by `ResourceType`'s raw id (1-22, index 0
unused), which would give a per-resource quantity for `TradedGoods::amount`. This is a single data
point, not cross-checked across adventures, and doesn't explain what unit the quantity is in or how
buying vs. selling is distinguished (both bought and sold resource lists would presumably need to index
into the same 40-entry quantity table somehow).

## Next steps

- Cross-check the `city_resources_quantity`-indexed-by-`ResourceType`-id hypothesis against several more
  real adventures before trusting it.
- Figure out how buying vs. selling amounts map onto the single 40-entry quantity array (two disjoint
  index ranges? a sign bit? separate array halves?).
- `military_strength`/`economy_strength`/`favour` for the player's own city has no lead at all yet -
  would need either a hex-diff against a known in-game value shown somewhere else in the UI, or asking
  whether the game even tracks/displays these for the player's own city the same way it does for others.
