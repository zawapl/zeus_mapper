# zeus_mapper

A Rust library for reading and writing the binary save/scenario file formats used by the 2000 city-builder
game **Zeus: Master of Olympus** (and its expansion, Poseidon): `.pak`, `.set`, `.map`, and `.sav` files.

## Status

Many fields in these formats remain unidentified, and the higher-level domain model
([`Adventure`](zeus_mapper/src/adventure/adventure.rs)) only reshapes a subset of the raw data so far. This
is a reverse-engineering project built incrementally against real game files, not a complete or stable API.

## Usage

Add the crate as a path/git dependency (it is not published to crates.io) and import from its prelude:

```rust
use std::fs::File;
use std::io::BufReader;
use zeus_mapper::prelude::PakData;

let mut reader = BufReader::new(File::open("MyAdventure.pak") ? );
let pak_data = PakData::read_from( & mut reader) ?;

println!("{:?}", pak_data.settings_data.real_episode_data[0]);
```

`zeus_mapper::prelude` is the crate's entire public API surface — everything usable from outside the crate
is re-exported there.

## Building and testing

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt          # nightly rustfmt; see rustfmt.toml (edition/style 2024, max_width = 140)
```

The toolchain is pinned to `nightly` via `rust-toolchain.toml`.

Run a single test with `cargo test -p zeus_mapper <test_name>` (e.g. `cargo test -p zeus_mapper
test_map_files`).

### `ZEUS_HOME`

Most tests and all `examples/` binaries read real game files and require the `ZEUS_HOME` environment
variable to point at a Zeus: Master of Olympus installation directory (expects a `ZEUS_HOME/Adventures/...`
layout). Without it set, those tests panic with `"ZEUS_HOME env var is not set"` — this is expected without
the game installed, not a bug. A handful of tests are also marked `#[ignore]` because they are
slow/exploratory; run them explicitly with `cargo test -- --ignored` when needed.

## Examples

`zeus_mapper/examples/` contains small standalone binaries exercising the library against real game data —
useful references for driving the API end-to-end (all require `ZEUS_HOME`):

- `dump_pak` — reads a `.pak` file and dumps its parsed contents.
- `parse_maps` — parses `.map` files.
- `log_deltas` — round-trips a file (read -> write -> read again) and logs any differences, for verifying
  read/write symmetry.
- `log_pak_diffs` — compares two `.pak` files field-by-field.
- `scan_l10n` — scans localization (`.eng`) files.

Run one with `cargo run --example dump_pak`.

## Architecture

The codebase is a layered pipeline, low-level binary parsing at the bottom, a friendlier domain model at
the top:

1. **`utils/read_utils.rs` / `utils/write_utils.rs`** — the `ReadFrom` / `WriteTo` traits are the
   foundation. Every struct that maps onto file bytes implements both, reading/writing little-endian
   primitives, fixed-size arrays, Windows-1252 strings, and PKWare-"exploded"/"imploded" compressed blocks.
   Struct field declaration order **is** the file layout — reordering fields breaks parsing.

2. **`file_data/`** — one struct per binary record type, mirroring the file layout almost 1:1 (`PakData`,
   `SettingsData`, `MapData`, `RealEpisodeData`, `MythologyData`, `EventData`, `ManifestData`,
   `WorldLocationData`, `WorldMapElementData`, `TradeRouteData`, `PyramidData`, `BasicEpisodeData`).
   `PakData` composes a `SettingsData` (the `.set` portion) followed by one or more `MapData` blocks (the
   `.map` portions) — a `.pak` file is a concatenation of a `.set` file and one-or-more `.map` files.
   Several formats are manifest-driven: a `ManifestData` entry's `size`/`count` determines whether optional
   trailing fields are present.

3. **`constants/`** — the `data_constants!` macro generates a C-like enum plus a `DataConstant` trait impl
   for fields whose primitive integer values are known but only some game constants have been identified
   (`Unresolved` wraps a raw value that may or may not resolve to a known variant).

4. **`adventure/`** and **`model/`** — a higher-level domain model built *from* the raw `file_data` structs
   (`Adventure::from_pak`, `CityMap::from_map_data`) that reshapes raw fields into friendlier types (tuples
   for x/y pairs, resolved enums, and so on) for downstream consumers. This layer is one-way — built from
   file data, not round-tripped back to it.

5. **`map_helper/`** — pure logic over the parsed tile grid: `MapPosition`/`MapAccessor` for isometric
   coordinate math, `position_iterator` for walking tiles, `texture_calculator` for converting raw
   terrain/sprite data into renderable texture IDs.

6. **`differ/`** — the `LogDifferences` trait (usually derived via `#[derive(LogDifferences)]` from
   `my_macros`) diffs two parsed structures field-by-field and prints what changed, primarily for
   round-trip verification and comparing save files.

7. **`l10n/`** — text/localization file parsing (`EngText`, `AdventureText`), separate from the binary save
   formats.

8. **`prelude.rs`** — the public API surface; everything else in `zeus_mapper` is module-private, so new
   public functionality must be re-exported here to be usable from outside the crate.

## License

MIT — see [LICENSE](LICENSE).
