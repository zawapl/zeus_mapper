# `EventData`/`Event`: remaining decoding gaps

The bulk of `EventData` (124 bytes/record, `zeus_mapper/src/file_data/event_data.rs`) is decoded and
wired up via `adventure/event.rs`, sourced from a forum post's byte-level reverse-engineering
(`zeus_mapper/notes.md`). What's left:

- **`unknown_row: [UnconfirmedSign<u16>; 9]`** - genuinely unmapped, no lead.
- **A batch of fields tagged `// unverified`** in the struct itself (best-guess names/types from the
  notes, not independently confirmed against real data): `item`, `on_failure`, `time_counter`,
  `event_status`, `needs_message_response`, `triggering_event_id`, `on_late`, `on_lost`,
  `progress_state`, `ally_troop_count`, `own_troop_strength`, `quest_high_byte`. Worth a real-adventure
  survey to confirm or correct each, the same way the rest of the struct was validated.
- **RANGE/VC min/max collapse**: several fields represent a fixed-or-min/max quantity (see
  `resolve_range` in `adventure/event.rs`); `Event` only keeps a single representative value, losing the
  `max` bound on range-based events. Accepted as a deliberate simplification (round-trips fine since
  `Event::to_data` reproduces whatever it read) - not a bug, but worth noting if `Event` ever needs to
  round-trip losslessly through edits rather than just pass through.
- **Map-editor template leftovers on fields that shouldn't apply**: `first_item`, `fixed_target`,
  `source_min`/`source_max` show up with real, non-default values even on event types/subtypes that
  don't semantically use them (e.g. `RivalBecomesAlly` carrying `first_item == 1` and
  `source_min`/`source_max` at recurring pairs like `(1, 8)` or `(0, 231)`). Likely the same
  "map-editor template constant" phenomenon documented for `MapData.scenario_data`'s
  `start_date`/`starting_cash`/etc. in `DATA_MAPPING.md`, but with more than one recurring shape and not
  yet fully surveyed across event types.
- **`SettingsData.parent_event_counts`/`colony_event_counts`** (`[u32; 10]`/`[u32; 4]`, stride-indexed
  per episode): read/written but their per-slot counts don't consistently match the real dense-event
  count for the corresponding episode (small, inconsistently-directioned off-by-ones). Not needed for
  `Event::vec_from_data`/`vec_to_data` (slot occupancy is derived from the dense `id` sequence directly
  instead), so this is a bookkeeping-role question, not a blocker.
- **Deliberately dropped events** (returns `None` in `Event::from_data`, not misclassified): `event_type
  == 4` (Quest - only one real sample, several fields including a monument-reward source too uncertain),
  and a couple of "goods request" subtypes lacking a confirmed second `CityId` source
  (`DefensiveMilitaryRequest`/`OffensiveMilitaryRequest`).

## Next steps

Survey each unverified/dropped item across more real adventures the same way the rest of the struct was
originally cross-referenced against `notes.md` - look for more `Quest` samples in particular, since a
single sample was the blocker there.
