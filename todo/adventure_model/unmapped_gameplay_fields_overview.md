# `Adventure`/`CityMap` doesn't reflect several fields with real gameplay meaning

`Adventure`/`CityMap` is deliberately a *lossy*, friendlier reshaping of `PakData`/`MapData` (see
`CLAUDE.md` - "one-way, not round-tripped back"). `adventure.rs`'s `all_real_adventures_round_trip` test
already passes today across every real adventure - but only because it round-trips through the model
symmetrically (`Adventure::from_pak(adventure.to_pak()) == adventure`), which trivially holds for any
field the model doesn't read in the first place (both sides default it the same way). So a green test
here does **not** mean the model is complete - it just means nothing *modeled* has regressed. Passing
this test is not evidence that a given field is safe to ignore.

Fields with real gameplay meaning that are entirely unmodeled and not exercised by that test at all:

1. The `world_location`/`world_map_element` unknown-field todos in this folder. (`episode_goals` and
   `world_settings.traded_resources` are now fully modeled both ways and no longer belong on this list -
   see `episode_goals_decoding.md` for the one remaining question there, an in-game causal test rather
   than an unmodeled field.)
2. Small per-map fields whose meaning isn't known yet - see `map_data_small_unknown_fields.md`.
3. "Inert but populated" padding slots (e.g. episode-array slots beyond an episode's real index, kept
   in the original file with real-looking-but-unused data) that `Adventure::to_pak` doesn't currently
   reproduce since it only writes the slots it models - harmless for gameplay (confirmed unused by the
   game) but means a `.pak` written by this crate isn't byte-identical to a game-authored one.

Byte-level `file_data`-layer fidelity (does `PakData::read_from` -> `write_to` -> `read_from` reproduce
the original bytes) is a separate, already-tracked concern -
`pak_data.rs::byte_identical_adventures_round_trip_exactly` (15 real adventures confirmed
byte-identical as of last check, out of a larger real corpus not all of which round-trips exactly) - not
what this file is about.

## Next steps

Each item above needs to actually be decoded into a first-class, named model field (there's no shortcut
that makes `all_real_adventures_round_trip` a useful signal for these) - work through the linked todos
in this folder one at a time.
