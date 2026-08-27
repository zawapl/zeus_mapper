# `EventData`/`Event`: remaining decoding gaps

Most of what this file used to track is resolved: `Quest` (`event_type == 4`) is now fully modeled both
ways (previously dropped), `MilitaryRequestSubtype`'s three variants (`CityUnderAttack`/
`CityAttacksRival`/`GreekCityTerrorized`) are all modeled (previously two were dropped for lacking a
confirmed second `CityId` source), the fixed-or-range quantity pattern keeps both `min`/`max` bounds on
every `Event` variant except one (see below), and `parent_event_counts`/`colony_event_counts` are now
confirmed as the sole reliable per-episode/colony event count (see `DATA_MAPPING.md`'s "Events" section)
rather than "inconsistently matching". What's actually left:

- **`unknown_row: [UnconfirmedSign<u16>; 9]`** - still genuinely unmapped, no lead.
- **Still-`// unverified` fields** (`file_data/event_data.rs`): `item`, `time_counter`, `event_status`,
  `needs_message_response`, `progress_state`, `ally_troop_count`, `own_troop_strength`,
  `quest_high_byte`. `on_failure`/`on_late`/`on_lost`/`triggering_event_id` are narrower than before -
  confirmed always `-1`/unset in every real event surveyed (only `on_success` is ever populated, 25
  occurrences across 60 `.pak` files, exclusively on `Quest`/`MonsterInvasion`/`GodInvasion`) - so what's
  left for those four is only "what they'd mean if populated", plausibly a `.sav`-runtime-only concern
  rather than something `.pak`/`.set` data ever exercises.
- **`WageDecrease.amount`** is now the *only* remaining fixed-or-range field collapsed to a single
  representative value (`resolve_range(fixed, min)`, dropping `max`) - every other `Event` variant keeps
  a `_min`/`_max` pair or `RangeInclusive`. Accepted as-is until there's a reason to give it the same
  treatment.
- **Map-editor template leftovers on fields that shouldn't apply**: `first_item`, `fixed_target`,
  `source_min`/`source_max` still show up with real, non-default values on events with no item/marker/
  source concept (e.g. recurring pairs like `(1, 8)` or `(0, 231)`). Documented as an accepted, unmodeled
  quirk in `DATA_MAPPING.md`; still not surveyed further.
- **`RivalArmyChange`**: has a `to_data` but no `from_data` dispatch at all - its raw `event_type`/
  `subtype` was never observed in any real adventure surveyed, so there's nothing to decode yet.
- **`event_type == 19`, `subtype == 22`**: one previously-unseen, well-formed real occurrence found in
  `^The Peloponnesian War.pak` (proper `flags`/time-range fields, no sign of corruption) - not yet named
  or added to `CityStatusChangeSubtype`. Needs its own investigation (ideally more real samples) before
  `EventData::validate`'s accepted subtype list can be safely widened.
- **`Quest`'s `MonumentReward`/`QuestType` raw encoding** is only weakly confirmed - one low-confidence
  real sample for the payload fields not already covered by `episode_goals::resolve_quest`'s stronger
  confirmation.
- **`GodInvasion`** doesn't read/write `on_success`/`trig_reason` at all, even though it's one of only
  three event types observed using `on_success` for real - that link is silently dropped on round-trip.

## Next steps

Survey each item the same way the rest of the struct was cross-referenced against real adventures - more
samples are the actual blocker for `RivalArmyChange`, `event_type == 19`/`subtype == 22`, and
`MonumentReward`/`QuestType`, not analysis of what's already available.
