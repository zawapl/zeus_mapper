use crate::adventure::Civilization;
use crate::file_data::building_data::BuildingData;
use crate::file_data::manifest_data::ManifestData;
use crate::file_data::unit_data::UnitData;
use crate::utils::MAX_MAP_SIZE;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_bytes_to_end;
use crate::utils::read_utils::read_compressed_boxed_array_from;
use crate::utils::read_utils::read_compressed_vec_from;
use crate::utils::read_utils::read_string_from;
use crate::utils::validation::ValidationError;
use crate::utils::validation::ValidationResult;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_compressed;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

/// A live saved-game (`.sav`) file.
///
/// Confirmed to read every byte with no error, and round-trip (`read` -> `write` -> `read`) to an
/// equal `SavData`, against all 62/62 real `.sav` files sampled (real player saves and auto-generated
/// tutorial saves alike) - see `docs/sav_investigation.md` for the full verification results and what
/// to do next. Still treat any `field_N` not called out in that doc as an unconfirmed hypothesis, not
/// a known fact.
#[derive(Debug, Clone, PartialEq, LogDifferences)]
pub struct SavData {
    /// Format version marker, the same role as every other format's leading `version_1`/`version_2`
    /// pair in this crate. Confirmed `238`/`235`/`241` (paired with `version_2 == 138`) on vanilla
    /// tutorial saves and a constant `321` (paired with `version_2 == 139`) on every Poseidon-owning
    /// player save sampled (`Save/Save Tests/`, `Save/Theocritos/`) - unlike `MapData.version_1`,
    /// this doesn't vary per adventure, so it plausibly tracks the game engine build rather than the
    /// loaded map's own format. See `docs/sav_investigation.md`.
    pub version_1: u32,
    pub version_2: u32,
    /// A live in-memory analog of `MapData.manifest`: 300 `ManifestData`-shaped entries, one per
    /// dynamic array/buffer the game engine's save routine walks. `address` is a live process memory
    /// pointer at save time (not a file offset, and not reproducible across saves), but `size`/
    /// `count` are meaningful and independently cross-checked - e.g. entry 2 is self-referential
    /// (`size == 20`, `count == 300`, describing this very table), and several other entries'
    /// `size`/`count` match already-confirmed array shapes elsewhere in this crate (36 `u32` prices,
    /// 22 `WorldLocationData`-with-extras at 572 bytes, 200 `WorldMapElementData`-with-custom-names
    /// at 72 bytes, 150 `EventData` at 124 bytes, several 51,984-tile grids). 139 of 300 slots are
    /// populated on Poseidon saves, 138 on vanilla tutorial saves. See `docs/sav_investigation.md`.
    pub manifest: BoxedArray<ManifestData, 300>,
    /// Single trailing byte right after `manifest`, still unidentified; observed as `0xFF` on most
    /// samples but `0x01` on a few (`Theocritos/Youngest Twins.sav`, `Theocritos/autosave_history.sav`,
    /// `tutorial15.sav`) - a real varying flag, not constant padding.
    pub field_1: u8,
    /// Not identified yet, but `manifest`'s entry for this field consistently decompresses to exactly
    /// 19,184 bytes on every save sampled (matching `docs/sav_investigation.md`'s pre-existing
    /// estimate for this field, from an independent source) - promoted from an opaque `Vec<u8>` to a
    /// fixed-size, decompressed array. See `docs/sav_investigation.md`.
    pub field_2: BoxedArray<u8, 19_184>,
    pub field_3: BoxedArray<u8, 8>,
    /// Not identified yet. Decompressed size depends on vanilla-vs-Poseidon (10,472 bytes on vanilla
    /// tutorial saves, 12,584 on every Poseidon-owning player save sampled) but is constant *within*
    /// each category - `version_2` (already read by this point) selects which, so this holds actual
    /// decompressed content rather than opaque captured-compressed bytes. Kept as `Vec<u8>` rather
    /// than a `BoxedArray` since its length isn't a single crate-wide constant. See
    /// `docs/sav_investigation.md`.
    pub field_4: Vec<u8>,
    pub field_5: BoxedArray<u8, 188>,
    /// Presumed map width/height in tiles (max observed elsewhere in this crate: 228).
    pub map_size: u32,
    pub field_6: BoxedArray<u8, 600>,
    /// Presumed Poseidon/Atlantean flag. Not civilization on its own - it's `1` for every real save
    /// sampled except `Proteus and Bellerophon.sav`/`Two Worlds Collide.sav` (the same 2 Poseidon-
    /// published-but-Greek episodes `poseidon_marker` doesn't disambiguate either). Combined with
    /// `poseidon_marker` it does resolve civilization correctly on all 28/28 samples checked - see
    /// `civilization` and `docs/sav_investigation.md`.
    pub poseidon_flag: u8,
    pub field_7: BoxedArray<u8, 1_383>,
    /// Not identified yet. Same vanilla-vs-Poseidon split as `field_4` (8,000 bytes vanilla, 14,400
    /// Poseidon), holding actual decompressed content the same way. See `docs/sav_investigation.md`.
    pub field_8: Vec<u8>,
    pub field_9: BoxedArray<u8, 18_609>,
    /// 228x228 byte grid; confirmed byte-identical to `MapData.root_offset` on 6 real adventures
    /// covering vanilla and Poseidon-expansion, story and Open Play content (see
    /// `docs/sav_investigation.md`).
    pub root_offset: BoxedArray<u8, 51_984>,
    /// 228x228 `u16` grid; not identified yet, but `manifest`'s entry for this field consistently
    /// shows `size == 2, count == 51,984` across every save sampled (see `docs/sav_investigation.md`),
    /// so it's promoted from an opaque `Vec<u8>` to a fixed-size, decompressed array like every other
    /// grid field here.
    pub field_11: BoxedArray<u16, 51_984>,
    /// 228x228 `u32` grid; confirmed to be the same `TerrainTypeFlag` bitfield as `MapData.terrain`
    /// (99.9%+ exact match on 6 real adventures, and the only field that changes when
    /// road/forest tiles are removed - see `docs/sav_investigation.md`).
    pub terrain: BoxedArray<u32, 51_984>,
    /// 228x228 byte grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_13: BoxedArray<u8, 51_984>,
    /// 228x228 `u16` grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_14: BoxedArray<u16, 51_984>,
    /// 228x228 byte grid; confirmed byte-identical to `MapData.tile_size` on 6 real adventures
    /// covering vanilla and Poseidon-expansion, story and Open Play content (see
    /// `docs/sav_investigation.md`).
    pub tile_size: BoxedArray<u8, 51_984>,
    /// 228x228 byte grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_16: BoxedArray<u8, 51_984>,
    /// 228x228 byte grid, stored uncompressed rather than length-prefixed; confirmed byte-identical
    /// to `MapData.random` on 6 real adventures (see `docs/sav_investigation.md`).
    pub random: BoxedArray<u8, 51_984>,
    /// 228x228 byte grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_18: BoxedArray<u8, 51_984>,
    /// 228x228 byte grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_19: BoxedArray<u8, 51_984>,
    /// 228x228 `u16` grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_20: BoxedArray<u16, 51_984>,
    /// 228x228 byte grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_21: BoxedArray<u8, 51_984>,
    /// 228x228 byte grid; not identified yet, promoted per `manifest` the same way as `field_11` (see
    /// `docs/sav_investigation.md`).
    pub field_22: BoxedArray<u8, 51_984>,
    pub units: BoxedArray<UnitData, 2_000>,
    /// Not identified yet. Consistently decompresses to exactly 2,000 bytes on every save sampled
    /// (matching `manifest`'s `size=2, count=1000` entry at this position) - promoted from `Vec<u8>`.
    /// See `docs/sav_investigation.md`.
    pub field_23: BoxedArray<u8, 2_000>,
    /// Not identified yet. Consistently decompresses to exactly 500,000 bytes on every save sampled
    /// (matching `manifest`'s `size=1, count=500000` entry at this position) - promoted from
    /// `Vec<u8>`. See `docs/sav_investigation.md`.
    pub field_24: BoxedArray<u8, 500_000>,
    /// Not identified yet. Consistently decompresses to exactly 15,600 bytes on every save sampled
    /// (matching `manifest`'s `size=156, count=100` entry at this position) - promoted from
    /// `Vec<u8>`. See `docs/sav_investigation.md`.
    pub field_25: BoxedArray<u8, 15_600>,
    /// First part of what used to be a single fixed 69-byte `field_26`; still unidentified.
    /// Consistently a 4-byte value on every save sampled. See `docs/sav_investigation.md`.
    pub field_26: BoxedArray<u8, 4>,
    /// Second part of what used to be `field_26`; still unidentified, but its *width* is the
    /// previously-unexplained root cause of the tutorial-save `buildings` misalignment bug: 1 byte on
    /// every currently-passing save, 4 bytes on the 13 vanilla tutorial saves that otherwise fail to
    /// parse. `manifest[33].size` (already read by this point) reliably predicts which - reading (and
    /// writing) this field's width accordingly is what fixes those 13 files. Stored widened to `u32`
    /// regardless of the on-disk width, losslessly (narrowing back to 1 byte on write, when
    /// applicable, can't lose data since the value was never wider than a byte to begin with). See
    /// `docs/sav_investigation.md`.
    pub field_26b: u32,
    /// Third part of what used to be `field_26`: a null-terminated ASCII string. **Confirmed** on all
    /// 46 real (non-tutorial) `.sav` files sampled: exactly matches the name of the folder the save
    /// lives under (e.g. `"Theocritos"`, `"Save Tests"`) - the game's save dialog asks for a player
    /// name and groups saves under a folder of that name, so this is that same name recorded inside
    /// the file. On the 16 auto-generated tutorial saves (not normal player saves) it instead holds
    /// developer/tester free-text notes (`"redo tutorials"`, `"fix me"`, `"new tutorial"`), consistent
    /// with the same free-text field being used for scratch notes during content creation rather than
    /// a real player name. The 64-byte buffer is right-aligned (leading NUL padding, then the name,
    /// then a NUL terminator, then occasional stray bytes) rather than a plain leading C-string, so
    /// this is trimmed on read past `read_string_from`'s usual trailing-NUL-only trim: leading NULs are
    /// stripped, then the result is cut at the first remaining NUL - handles a shorter name written
    /// into a buffer that previously held a longer one leaving stray bytes behind the true terminator
    /// (observed on `Theocritos/Youngest Twins.sav`). Safe to trim more aggressively than
    /// `read_string_from` normally would because, unlike `.pak`/`.map`, `SavData` never promises
    /// byte-identical round trips (see `docs/sav_investigation.md`) - written back zero-padded, so a
    /// discarded tail is simply not reproduced on write, not silently corrupted.
    pub player_name: String,
    pub buildings: BoxedArray<BuildingData, 4_000>,
    pub field_27: BoxedArray<u8, 352>,
    /// Not identified yet. Consistently decompresses to exactly 60,000 bytes on every save sampled
    /// (matching `manifest`'s `size=60, count=1000` entry at this position) - promoted from
    /// `Vec<u8>`. See `docs/sav_investigation.md`.
    pub field_28: BoxedArray<u8, 60_000>,
    pub field_29: BoxedArray<u8, 17_974>,
    /// Not identified yet, but `manifest`'s entry for this field (index 71) consistently shows
    /// `size == 2, count == 500` across every save sampled - the true element type is `u16`, not the
    /// `u8` this was originally promoted as. See `docs/sav_investigation.md`.
    pub field_30: BoxedArray<u16, 500>,
    /// Not identified yet, but `manifest`'s entry for this field (index 72) consistently shows
    /// `size == 2, count == 500` across every save sampled - the true element type is `u16`, not the
    /// `u8` this was originally promoted as. See `docs/sav_investigation.md`.
    pub field_31: BoxedArray<u16, 500>,
    /// Not identified yet, but `manifest`'s entry for this field (index 73) consistently shows
    /// `size == 2, count == 4000` across every save sampled - the true element type is `u16`, not the
    /// `u8` this was originally promoted as. See `docs/sav_investigation.md`.
    pub field_32: BoxedArray<u16, 4_000>,
    pub field_33: BoxedArray<u8, 53_783>,
    /// 228x228 byte grid, stored uncompressed rather than length-prefixed; confirmed byte-identical
    /// to `MapData.meadow` on 6 real adventures (see `docs/sav_investigation.md`).
    pub meadow: BoxedArray<u8, 51_984>,
    pub field_35: BoxedArray<u8, 16>,
    /// Not identified yet. Consistently decompresses to exactly 75,168 bytes on every save sampled
    /// (matching `manifest`'s `size=324, count=232` entry at this position - `232` is the same count
    /// as `MapData.trade_routes`, a plausible but unconfirmed lead) - promoted from `Vec<u8>`. See
    /// `docs/sav_investigation.md`.
    pub field_36: BoxedArray<u8, 75_168>,
    /// 228x228 byte grid, stored uncompressed rather than length-prefixed; refutes the prior
    /// hypothesis that this held a per-tile marble/livestock/timber resource level (see
    /// `docs/sav_investigation.md`). Confirmed live, not filler: uniform `0xFF` on every fresh
    /// "quick save", but real per-tile variation (tens of distinct values, smooth-looking) on every
    /// actually-played save sampled - purpose still unidentified.
    pub field_37: BoxedArray<u8, 51_984>,
    pub field_38: BoxedArray<u8, 32>,
    /// Not identified yet. Consistently decompresses to exactly 36 bytes on every save sampled
    /// (matching `manifest`'s `size=36, count=1` entry at this position) - promoted from `Vec<u8>`.
    /// See `docs/sav_investigation.md`.
    pub field_39: BoxedArray<u8, 36>,
    /// 228x228 `u32` grid; not identified yet, but consistently decompresses to exactly 207,936 bytes
    /// (`4 * 51,984`) on every save sampled, matching `manifest`'s `size=4, count=51984` entry at this
    /// position - the same per-tile shape as `terrain`, but a distinct field. Promoted from `Vec<u8>`.
    /// See `docs/sav_investigation.md`.
    pub field_40: BoxedArray<u32, 51_984>,
    pub field_41: BoxedArray<u8, 39>,
    /// 228x228 byte grid; not identified yet, but consistently decompresses to exactly 51,984 bytes
    /// on every save sampled, matching `manifest`'s `size=1, count=51984` entry at this position -
    /// promoted from `Vec<u8>`. See `docs/sav_investigation.md`.
    pub field_42: BoxedArray<u8, 51_984>,
    /// 228x228 byte grid; confirmed byte-identical to `MapData.scrub` on 6 real adventures (see
    /// `docs/sav_investigation.md`).
    pub scrub: BoxedArray<u8, 51_984>,
    /// Everything from here to the trailing 4-byte `poseidon_marker`, unidentified.
    pub field_44: Vec<u8>,
    /// Last 4 bytes of the file. Always `0` or `1`. **Confirmed**: `1` iff the loaded episode's
    /// `.pak`-side `adventure_type` is Poseidon-published (`PoseidonCampaign`/`PoseidonCustom`, not
    /// `ZeusCampaign`/`ZeusCustom`/`Tutorial`) - 28/28 exact matches on a `Save Tests` sample, zero
    /// exceptions. Despite the name, this is **not** civilization - it only correlates with it because
    /// most Poseidon-published episodes happen to be Atlantean; `Proteus and Bellerophon.sav`/
    /// `Two Worlds Collide.sav` are Poseidon-published but Greek, and this field still reads `1` for
    /// them, correctly reflecting "Poseidon episode" rather than "Atlantean". See `civilization` and
    /// `docs/sav_investigation.md`.
    pub poseidon_marker: u32,
}

impl Default for SavData {
    fn default() -> Self {
        return SavData {
            version_1: 321,
            version_2: 139,
            manifest: ManifestData::default_sav_manifest(),
            field_1: Default::default(),
            field_2: Default::default(),
            field_3: Default::default(),
            field_4: Default::default(),
            field_5: Default::default(),
            map_size: Default::default(),
            field_6: Default::default(),
            poseidon_flag: Default::default(),
            field_7: Default::default(),
            field_8: Default::default(),
            field_9: Default::default(),
            root_offset: Default::default(),
            field_11: Default::default(),
            terrain: Default::default(),
            field_13: Default::default(),
            field_14: Default::default(),
            tile_size: Default::default(),
            field_16: Default::default(),
            random: Default::default(),
            field_18: Default::default(),
            field_19: Default::default(),
            field_20: Default::default(),
            field_21: Default::default(),
            field_22: Default::default(),
            units: Default::default(),
            field_23: Default::default(),
            field_24: Default::default(),
            field_25: Default::default(),
            field_26: Default::default(),
            field_26b: Default::default(),
            player_name: Default::default(),
            buildings: Default::default(),
            field_27: Default::default(),
            field_28: Default::default(),
            field_29: Default::default(),
            field_30: Default::default(),
            field_31: Default::default(),
            field_32: Default::default(),
            field_33: Default::default(),
            meadow: Default::default(),
            field_35: Default::default(),
            field_36: Default::default(),
            field_37: Default::default(),
            field_38: Default::default(),
            field_39: Default::default(),
            field_40: Default::default(),
            field_41: Default::default(),
            field_42: Default::default(),
            scrub: Default::default(),
            field_44: Default::default(),
            poseidon_marker: Default::default(),
        };
    }
}

impl SavData {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return ReadFrom::read_from(reader);
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        return WriteTo::write_to(self, writer);
    }

    pub fn validate(&self) -> ValidationResult {
        if self.version_2 < 138 || self.version_2 > 139 {
            return Err(ValidationError::expected_range("version_2", self.version_2, 138, 139));
        }

        ManifestData::validate_sav_manifest(&self.manifest, self.version_1, self.version_2)
            .map_err(ValidationError::add_parent("manifest"))?;

        if self.map_size > MAX_MAP_SIZE {
            // todo this probably also has a lower bound - check examples
            return Err(ValidationError::expected_range("map_size", self.map_size, 0, MAX_MAP_SIZE));
        }

        if self.poseidon_flag > 1 {
            return Err(ValidationError::expected_range("poseidon_flag", self.poseidon_flag, 0, 1));
        }

        if self.poseidon_marker > 1 {
            return Err(ValidationError::expected_range("poseidon_marker", self.poseidon_marker, 0, 1));
        }

        return Ok(());
    }

    /// This save's civilization, derived from `poseidon_marker` combined with `poseidon_flag`.
    ///
    /// Neither field is civilization on its own: `poseidon_marker` tracks whether the loaded episode
    /// is Poseidon-published at all (Atlantean is Poseidon-only, so a non-Poseidon episode can only be
    /// Greek), and `poseidon_flag` is what actually distinguishes Atlantean from Greek within
    /// Poseidon-published episodes. Confirmed against `PakData::civilization` with zero exceptions
    /// across 28 real `Save Tests` files, including both cases where the episode is Poseidon-published
    /// but Greek. See `docs/sav_investigation.md`.
    pub fn civilization(&self) -> Civilization {
        if self.poseidon_marker == 1 && self.poseidon_flag == 1 {
            return Civilization::Atlantean;
        }

        return Civilization::Greek;
    }
}

impl ReadFrom for SavData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let version_1 = ReadFrom::read_from(reader)?;
        let version_2 = ReadFrom::read_from(reader)?;
        let manifest: BoxedArray<ManifestData, 300> = ReadFrom::read_from(reader)?;
        let field_26b_is_narrow = manifest[33].size == 1;
        let is_vanilla = version_2 == 138;

        let poseidon_marker;

        return Ok(SavData {
            version_1,
            version_2,
            manifest,
            field_1: ReadFrom::read_from(reader)?,
            field_2: read_compressed_boxed_array_from(reader)?,
            field_3: ReadFrom::read_from(reader)?,
            field_4: read_compressed_vec_from(reader, if is_vanilla { 10_472 } else { 12_584 })?,
            field_5: ReadFrom::read_from(reader)?,
            map_size: ReadFrom::read_from(reader)?,
            field_6: ReadFrom::read_from(reader)?,
            poseidon_flag: ReadFrom::read_from(reader)?,
            field_7: ReadFrom::read_from(reader)?,
            field_8: read_compressed_vec_from(reader, if is_vanilla { 8_000 } else { 14_400 })?,
            field_9: ReadFrom::read_from(reader)?,
            root_offset: read_compressed_boxed_array_from(reader)?,
            field_11: read_compressed_boxed_array_from(reader)?,
            terrain: read_compressed_boxed_array_from(reader)?,
            field_13: read_compressed_boxed_array_from(reader)?,
            field_14: read_compressed_boxed_array_from(reader)?,
            tile_size: read_compressed_boxed_array_from(reader)?,
            field_16: read_compressed_boxed_array_from(reader)?,
            random: ReadFrom::read_from(reader)?,
            field_18: read_compressed_boxed_array_from(reader)?,
            field_19: read_compressed_boxed_array_from(reader)?,
            field_20: read_compressed_boxed_array_from(reader)?,
            field_21: read_compressed_boxed_array_from(reader)?,
            field_22: read_compressed_boxed_array_from(reader)?,
            units: read_compressed_boxed_array_from(reader)?,
            field_23: read_compressed_boxed_array_from(reader)?,
            field_24: read_compressed_boxed_array_from(reader)?,
            field_25: read_compressed_boxed_array_from(reader)?,
            field_26: ReadFrom::read_from(reader)?,
            field_26b: if field_26b_is_narrow {
                let narrow: u8 = ReadFrom::read_from(reader)?;
                narrow as u32
            } else {
                ReadFrom::read_from(reader)?
            },
            player_name: {
                let raw = read_string_from(reader, 64)?;
                let after_leading_nuls = raw.trim_start_matches(char::from(0));
                let end = after_leading_nuls.find(char::from(0)).unwrap_or(after_leading_nuls.len());
                after_leading_nuls[..end].to_owned()
            },
            buildings: read_compressed_boxed_array_from(reader)?,
            field_27: ReadFrom::read_from(reader)?,
            field_28: read_compressed_boxed_array_from(reader)?,
            field_29: ReadFrom::read_from(reader)?,
            field_30: read_compressed_boxed_array_from(reader)?,
            field_31: read_compressed_boxed_array_from(reader)?,
            field_32: read_compressed_boxed_array_from(reader)?,
            field_33: ReadFrom::read_from(reader)?,
            meadow: ReadFrom::read_from(reader)?,
            field_35: ReadFrom::read_from(reader)?,
            field_36: read_compressed_boxed_array_from(reader)?,
            field_37: ReadFrom::read_from(reader)?,
            field_38: ReadFrom::read_from(reader)?,
            field_39: read_compressed_boxed_array_from(reader)?,
            field_40: read_compressed_boxed_array_from(reader)?,
            field_41: ReadFrom::read_from(reader)?,
            field_42: read_compressed_boxed_array_from(reader)?,
            scrub: read_compressed_boxed_array_from(reader)?,
            field_44: {
                let mut tail = read_bytes_to_end(reader)?;

                if tail.len() < 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "field_44 tail is shorter than 4 bytes"));
                }

                let marker_bytes = tail.split_off(tail.len() - 4);
                poseidon_marker = u32::from_le_bytes([marker_bytes[0], marker_bytes[1], marker_bytes[2], marker_bytes[3]]);
                tail
            },
            poseidon_marker,
        });
    }
}

impl WriteTo for SavData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.version_1, writer)?;
        bytes += WriteTo::write_to(&self.version_2, writer)?;
        bytes += WriteTo::write_to(&self.manifest, writer)?;
        bytes += WriteTo::write_to(&self.field_1, writer)?;
        bytes += write_compressed(&self.field_2, writer)?;
        bytes += WriteTo::write_to(&self.field_3, writer)?;
        bytes += write_compressed(&self.field_4, writer)?;
        bytes += WriteTo::write_to(&self.field_5, writer)?;
        bytes += WriteTo::write_to(&self.map_size, writer)?;
        bytes += WriteTo::write_to(&self.field_6, writer)?;
        bytes += WriteTo::write_to(&self.poseidon_flag, writer)?;
        bytes += WriteTo::write_to(&self.field_7, writer)?;
        bytes += write_compressed(&self.field_8, writer)?;
        bytes += WriteTo::write_to(&self.field_9, writer)?;
        bytes += write_compressed(&self.root_offset, writer)?;
        bytes += write_compressed(&self.field_11, writer)?;
        bytes += write_compressed(&self.terrain, writer)?;
        bytes += write_compressed(&self.field_13, writer)?;
        bytes += write_compressed(&self.field_14, writer)?;
        bytes += write_compressed(&self.tile_size, writer)?;
        bytes += write_compressed(&self.field_16, writer)?;
        bytes += WriteTo::write_to(&self.random, writer)?;
        bytes += write_compressed(&self.field_18, writer)?;
        bytes += write_compressed(&self.field_19, writer)?;
        bytes += write_compressed(&self.field_20, writer)?;
        bytes += write_compressed(&self.field_21, writer)?;
        bytes += write_compressed(&self.field_22, writer)?;
        bytes += write_compressed(&self.units, writer)?;
        bytes += write_compressed(&self.field_23, writer)?;
        bytes += write_compressed(&self.field_24, writer)?;
        bytes += write_compressed(&self.field_25, writer)?;
        bytes += WriteTo::write_to(&self.field_26, writer)?;
        if self.manifest[33].size == 1 {
            bytes += WriteTo::write_to(&(self.field_26b as u8), writer)?;
        } else {
            bytes += WriteTo::write_to(&self.field_26b, writer)?;
        }
        bytes += write_string_to(&self.player_name, writer, 64)?;
        bytes += write_compressed(&self.buildings, writer)?;
        bytes += WriteTo::write_to(&self.field_27, writer)?;
        bytes += write_compressed(&self.field_28, writer)?;
        bytes += WriteTo::write_to(&self.field_29, writer)?;
        bytes += write_compressed(&self.field_30, writer)?;
        bytes += write_compressed(&self.field_31, writer)?;
        bytes += write_compressed(&self.field_32, writer)?;
        bytes += WriteTo::write_to(&self.field_33, writer)?;
        bytes += WriteTo::write_to(&self.meadow, writer)?;
        bytes += WriteTo::write_to(&self.field_35, writer)?;
        bytes += write_compressed(&self.field_36, writer)?;
        bytes += WriteTo::write_to(&self.field_37, writer)?;
        bytes += WriteTo::write_to(&self.field_38, writer)?;
        bytes += write_compressed(&self.field_39, writer)?;
        bytes += write_compressed(&self.field_40, writer)?;
        bytes += WriteTo::write_to(&self.field_41, writer)?;
        bytes += write_compressed(&self.field_42, writer)?;
        bytes += write_compressed(&self.scrub, writer)?;
        bytes += WriteTo::write_to(&self.field_44, writer)?;
        bytes += WriteTo::write_to(&self.poseidon_marker, writer)?;

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use crate::file_data::sav_data::SavData;
    use std::fs;
    use std::fs::File;
    use std::io;
    use std::io::BufReader;

    #[test]
    #[ignore] // a bit slow, will remove ignored after looking into this more
    fn validate_sav_files() -> io::Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let mut sav_files = vec![];
            find_sav_files(std::path::Path::new(&format!("{game_root}/Adventures")), &mut sav_files)?;
            find_sav_files(std::path::Path::new(&format!("{game_root}/Save")), &mut sav_files)?;

            for path in sav_files {
                let mut reader = File::open(&path).map(BufReader::new)?;
                let original = SavData::read_from(&mut reader)?;

                if let Err(e) = original.validate() {
                    panic!("{path:?}: {e}");
                }

                let mut buffer = vec![];
                original.write_to(&mut buffer)?;
                let reread = SavData::read_from(&mut io::Cursor::new(&buffer))?;
                assert_eq!(original, reread, "{path:?} did not round-trip to an equal SavData");
            }
        }

        return Ok(());
    }

    fn find_sav_files(dir: &std::path::Path, result: &mut Vec<std::path::PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                find_sav_files(&path, result)?;
            } else if path.extension().map_or("", |x| x.to_str().unwrap_or("")) == "sav" {
                result.push(path);
            }
        }

        return Ok(());
    }
}
