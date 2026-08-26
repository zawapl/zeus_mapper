# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`zeus_mapper` is a Rust library (plus `my_macros`, a proc-macro helper crate) for reading and writing the binary
save/scenario file formats used by the 2000 city-builder game **Zeus: Master of Olympus** (and its expansion
Poseidon): `.pak`, `.set`, `.map`, `.sav` files. The formats were reverse-engineered by hex-editing game files.
Fields whose purpose is still unknown are named `field_N` or `unknown`.

## Build / test / lint

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt          # nightly rustfmt; edition/style set to 2024 in rustfmt.toml, max_width = 140
```

Run a single test: `cargo test -p zeus_mapper <test_name>` (e.g. `cargo test -p zeus_mapper test_map_files`).

The toolchain is pinned to `nightly` via `rust-toolchain.toml`.

### `ZEUS_HOME` env var

Most tests (in `file_data/`, `l10n/`, `adventure/`, `model/`, `map_helper/`) and all `examples/` binaries read real
game files and require `ZEUS_HOME` to point at a Zeus: Master of Olympus installation directory (expects a
`ZEUS_HOME/Adventures/...` layout). Without it set, those tests panic with `"ZEUS_HOME env var is not set"` — this is
expected in an environment without the game installed, not a bug to fix. A handful of tests are also marked
`#[ignore]` (e.g. `compress_decompress`, `test_maps`, `test_zeus_mm`) because they're slow/exploratory; run them
explicitly with `cargo test -- --ignored` when needed.

Some tests/examples reference a `crates/zeus_mapper/...` relative path for output — that's stale from a prior repo
layout (this repo root is `zeus_mapper/`, not `crates/zeus_mapper/`); check the actual cwd before relying on it.

## Architecture

The codebase is a layered pipeline, low-level binary parsing at the bottom, a friendlier domain model at the top:

1. **`utils/read_utils.rs` / `utils/write_utils.rs`** — the `ReadFrom` / `WriteTo` traits are the foundation. Every
   struct that maps onto file bytes implements both, reading/writing little-endian primitives, fixed-size arrays,
   Windows-1252 strings, and PKWare-"exploded"/"imploded" compressed blocks (`explode` / `pklib` crates). Structs
   generally implement `read_from` by delegating field-by-field to `ReadFrom::read_from`, so field declaration order
   in the struct **is** the file layout — reordering fields breaks parsing.

2. **`file_data/`** — one struct per binary record type (`PakData`, `SettingsData`, `MapData`, `RealEpisodeData`,
   `MythologyData`, `EventData`, `ManifestData`, `WorldLocationData`, `WorldMapElementData`, `TradeRouteData`,
   `PyramidData`, `BasicEpisodeData`). `PakData::read_from` composes `SettingsData` (the `.set` portion) followed by
   N `MapData` blocks (the `.map` portions, delimited by a `"MAPS"` header) — this is how a `.pak` file is actually a
   concatenation of a `.set` file and one-or-more `.map` files. Several formats are manifest-driven: a `ManifestData`
   entry's `size`/`count` determines whether optional trailing fields are present (e.g. `include_custom_names`,
   `include_pyramids` in `map_data.rs`) — read and write logic must stay in sync on these conditionals.

3. **`constants/`** — `data_constants!` (in `data_constant.rs`) is a macro that generates a C-like enum plus a
   `DataConstant` trait impl (`try_resolve` from a primitive, `value()` back to primitive, `values()`). Used for
   fields whose primitive integer values are known but only some game constants have been identified (e.g.
   `Unresolved` types, or `Civilization`/`God`/`Monster`/`NextEpisode` in `adventure/mod.rs`). `utils/primitive_enum.rs`
   has a similar but simpler `primitive_enum!` macro for closed sets without the resolve/value trait ceremony.

4. **`adventure/`** and **`model/`** — a higher-level domain model built *from* the raw `file_data` structs (e.g.
   `Adventure::from_pak`, `CityMap::from_map_data`) that reshapes raw fields into friendlier types (tuples for
   x/y pairs, resolved enums, etc.) for downstream consumers. This layer is one-way (built from file data, not
   round-tripped back to it).

5. **`map_helper/`** — pure logic over the parsed tile grid: `MapPosition`/`MapAccessor` for isometric coordinate
   math, `position_iterator` for walking tiles, `texture_calculator` for converting raw terrain/sprite data into
   renderable texture IDs (`Direction` in `utils/mod.rs` documents the N/S/E/W-vs-NW/NE/SW/SE isometric-vs-tile-coord
   distinction).

6. **`differ/`** — the `LogDifferences` trait (implemented via `#[derive(LogDifferences)]` from the `my_macros`
   crate, which is a `#[proc_macro_derive]` that walks struct fields and recursively logs mismatches) is used to diff
   two parsed structures field-by-field and print what changed — primarily for round-trip verification (read → write
   → read again → diff) and comparing save files. See `examples/log_deltas.rs` and `examples/log_pak_diffs.rs`.

7. **`l10n/`** — text/localization file parsing (`EngText`, `AdventureText`), separate from the binary save formats.

8. **`prelude.rs`** — the public API surface; almost everything else in `zeus_mapper` is module-private
   (`pub(crate)`), so new public functionality must be re-exported here to be usable from outside the crate
   (examples import via `zeus_mapper::prelude::*`).

`examples/` are small standalone binaries exercising the library against real game data (`dump_pak`, `parse_maps`,
`log_deltas`, `log_pak_diffs`, `scan_l10n`) — useful references for how to drive the API end-to-end, and all require
`ZEUS_HOME`.

## Key conventions

- **Explicit `return`**: every function ends its control-flow paths with `return X;`, including the final expression
  of a function (not relying on implicit tail-expression return). This is consistent throughout `zeus_mapper` and
  `my_macros`.
- **No `let`-`else` guard clauses**: prefer `if let Ok(x) = ... { ... }` / `if let Some(x) = ... { ... }` over
  `let Some(x) = expr else { return/continue; }; ...use x...`. This applies to any pattern (`Some(x)`, `Ok(x)`,
  etc.) — the thing being avoided is the early-return/continue `else` branch, not any specific binding. When
  multiple such conditions apply, chain them in one `if` with `&&` rather than nesting one `if let` inside another:
  `if let Some(x) = a() && let Some(y) = b(x) { ... }`, not `if let Some(x) = a() { if let Some(y) = b(x) { ... } }`.
- **Avoid `unwrap()`/`expect()` in library code** (the `file_data`/`adventure`/`model`/`map_helper`/`l10n` parsing
  and writing paths) — propagate `io::Result` instead. Tests and `examples/` binaries are the exception: they read
  real fixture files and are expected to fail loudly (`.unwrap()`/`.expect("...")`) on bad input.
- **Imports, one item per line**: `use` statements bind a single item each (e.g. two separate
  `use crate::file_data::manifest_data::ManifestData;` and `use crate::file_data::manifest_data::read_segment;`
  lines rather than a brace-grouped `use crate::file_data::manifest_data::{ManifestData, read_segment};`). This is
  enforced by `rustfmt.toml` (`imports_granularity = "Item"`, `group_imports = "One"`) — run `cargo fmt` rather than
  hand-formatting imports.
- **Rustdoc**: `///` doc comments, one sentence per line, and each line must fit within rustfmt's `max_width`
  (`rustfmt.toml`) on its own. A sentence never spans multiple lines: if it doesn't fit on one line, split it into
  two shorter sentences (each still on its own line) instead of wrapping it. Structure:
  ```
  /// One-sentence summary of what the function/type does.
  ///
  /// Optional additional paragraph(s) with more detail, same one-sentence-per-line rule.
  ///
  /// **Assumptions**: (only if the function relies on invariants established elsewhere)
  /// - each assumption as its own bullet
  ```
  Don't name other functions or call sites by name — those names rot as the code changes. State the invariant or
  expectation instead of the mechanism that provides it, e.g. write "assumes the reader is positioned at the start
  of a `MAPS` block" rather than "assumes it's called right after `PakData::read_from` seeks past the header". Keep
  rustdoc short — a few lines, not an essay.
- **Comments are ASCII only** — no non-ASCII symbols (arrows, math symbols, etc.); write `->` and `<=` instead of
  their Unicode equivalents.
- **Tests live in the same file** as the code they cover, in a trailing `#[cfg(test)] mod tests { ... }` block, not
  in a separate `tests/` file — follow this placement for new tests rather than introducing a parallel test tree.

## Working with these formats

- Struct field order is load-bearing: it defines binary layout for both reading and writing. When adding a field,
  place it in the correct byte-offset position, not just appended.
- When a field's meaning is unknown, keep the `field_N`/`unknown` naming convention rather than guessing a name.
- Read and write paths for a struct must stay symmetric (same manifest-driven conditionals, same compression calls)
  — a round-trip (`read_from` → `write_to` → `read_from`) should reproduce the original data, which is exactly what
  the `LogDifferences` derive is for when validating changes.
- When new understanding of a `field_N`/`unknown` blob's content is confirmed, promote the relevant bytes to a
  properly named field on the struct itself (splitting the blob around it, e.g. `unknown_240`/`old_format_favour`/
  `unknown_341` in `world_location_data.rs`) rather than reading them out with a raw index/offset in `adventure`/
  `model` conversion code. The `file_data` structs should stay the one place that reflects current understanding of
  the layout — call sites should read named fields, not compute into blobs.
- If applying new understanding surfaces a byte-alignment mismatch (data landing a few bytes early/late, a field
  short/long by a constant), don't work around it in the read/decode logic — investigate until the exact offset is
  known (diff raw bytes directly between two files expected to hold the same content in each format generation if
  such a pair exists; ask for more real-file examples if none are available yet; cross-check any computed offset
  against an already-independently-confirmed value before trusting it further), then
  fix the struct's field boundaries themselves rather than patching around them.

## Open work

`todo/` at the repo root (not under `docs/`) is a generic punch list of open project items, one file per
item, grouped into subfolders by area — mostly still-open reverse-engineering questions about these file
formats today, but not limited to that. It has its own `todo/CLAUDE.md` with the folder's conventions
(grouped by area, deleted once resolved, never referenced by path from outside the folder). Check there
before assuming a `field_N`/`unknown_N` you're investigating hasn't already got documented partial
progress.
