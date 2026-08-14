# `Adventure` model: PakData mapping

Current understanding of how the `adventure` module's friendlier types map onto the raw `file_data`
structs, and what's confirmed used vs. unused by the game. This is the up-to-date reference; for the
investigation history behind these conclusions (in-game test cases, dead ends, byte-level surveys) see
`docs/adventure_model_todo.md`.

Conventions used throughout the raw formats, referenced below rather than repeated per field:

- **`u16::MAX`/`u32::MAX` as "unset"** - position pairs, single points, and several per-event
  quantities use the max value of their integer type to mean "nothing here", not `0` (which is itself
  a valid coordinate/quantity). Confirmed for every position field in `RealEpisodeData` and several
  `EventData` fields.
- **Settings-level vs. map-level duplicates** - several concepts (`building_flags`, mythology gods,
  `civilization`) are stored on both `SettingsData` and the per-map `RealEpisodeData`/`MythologyData`.
  Where confirmed in-game, the settings-level copy is authoritative when populated, with the map-level
  copy as a fallback for "structured" episodes (`exists == 255`) where the settings-level copy is
  normally empty. `Adventure::to_pak` writes both.
- **Map-editor template leftovers** - some fields hold a fixed constant on every real map/event
  regardless of content, not real per-item data (e.g. `MapData.scenario_data`'s narrative/economy
  fields, several `EventData` fields on events that don't semantically use them). These are treated as
  free/hardcoded on write rather than modeled.

## `Adventure`

| Field                                       | Source                                                         |
|---------------------------------------------|----------------------------------------------------------------|
| `title`/`introduction_text`/`complete_text` | `AdventureText` (the `.txt` sidecar file, not `PakData`)       |
| `civilization`                              | `SettingsData.real_episode_data[0].civilization`               |
| `adventure_type`                            | `SettingsData.adventure_type`                                  |
| `initial_funds`/`start_year`                | `SettingsData.real_episode_data[0].starting_cash`/`start_date` |
| `parent_city`                               | `CityMap::from_map_data(map_data[0])`                          |
| `parent_episodes`/`colony_episodes`         | see below                                                      |
| `world_locations`/`trade_routes`            | see "World map" below                                          |
| `prices`                                    | `map_data[0].prices` (raw, index-preserving copy - see below)  |

`available_colonies` is `SettingsData.colony_episodes_available`. `bitmap` - the adventure-selection-
screen icon id - is `SettingsData.bitmap`, confirmed by diffing two otherwise-byte-identical `.pak`
files that only differ in this one `u32` (`The Youngest Twins`, `14` -> `8`). It sits between two
chunks of what a prior pass had assumed was one undifferentiated `SettingsData.colony_goals` blob -
that field is now split into `field_11` (4112 still-unidentified leading bytes), `bitmap`, then the
much smaller real `colony_goals` (278 trailing bytes, holding the actual per-colony goal record -
see `episode_goals` below). An earlier version of this doc claimed `bitmap` lived after
`parent_city_goals` instead, citing a `14` shared across every real, story-driven Atlantean campaign
surveyed as confirmation; that byte was actually the `goal_type` field of a parent episode's own
goal record (see `episode_goals` below), which happens to also use the value `14` - so that
"confirmation" wasn't actually reading `bitmap` at all, and whether those other adventures' *real*
`bitmap` values also cluster around `14` is unverified.

`Adventure.background_image` is `MapData.background_image` from `map_data[0]` (the parent map) - the
world-map screen's background art set id ("world bitmap") - confirmed by diffing a copy of
`The Youngest Twins` (made via the in-game Adventure Editor) against the original after changing only
its world-map background in the editor: `map_data[0].background_image` (the parent map) went from `11`
to `13`, exactly matching `The Mayan Revolution`'s own value (`13`) - the target the edit was copying.
Unlike `world_locations`/`trade_routes`/`prices`, this is **not** a world-scoped field duplicated
identically across every map: the colony map's `background_image` and the `SettingsData.map_data`
duplicate both stayed at `11` in the edited copy, so only the parent map's copy is what the editor (and
presumably the game, on the world-map screen reached from that map) actually reads - `Adventure::to_pak`
only writes it back to `map_data[0]`, leaving colony maps' own copies at their (currently unmodeled,
always-`0`) `CityMap::to_map_data` default. The same editor session also perturbed a number of unrelated
rendering-derived bytes (`meadow` grass-texture variant ids, `elevation`, a few `sprite` entries,
`seed_1`/`seed_2`, `manifest[*].address` - the latter is a captured in-process pointer, not real file
data) - these are regenerated by the editor on every save and aren't signal.

`SettingsData.colony_episodes_available` (not `colony_episodes_used`) is what gates the colony list on
the episode-select screen - confirmed in-game (case 2 in `docs/adventure_model_todo.md`). `to_pak`
leaves `colony_episodes_used` at `0`; no evidence it needs to be written.

`adventure_type` (formerly the first byte of a `[u8; 5]` `field_8`, now `u8`, with the remaining 4
still-unidentified bytes renamed `field_10` - always `0` in every sample seen) determines which menu
(Zeus/Poseidon/tutorial) an adventure is listed under in-game. This was confirmed causally, not just
by correlation: byte 0 alone is `0`/`1` for every Zeus-menu adventure (`0` for the original shipped
`.pak`s, `1` for folder/editor-saved ones like `Test`), `2` for every tutorial/building-demo pak
(`agora`, `bigcity`, `colony`, ...), and `3`/`4` for every Poseidon-menu adventure (`3` shipped, `4`
folder/editor-saved) - zero exceptions across every `.pak` under `Adventures/`, not just the "official"
adventure list. Two other fields (`MapData.field_17`'s repeated template byte, and the mythology
roster's use of `Hera`/`Atlas`/their exclusive monsters) correlated just as cleanly across the same
corpus but were both later proven non-causal: copying a whole adventure's content into a fresh file
while leaving those fields at their original values still followed the original file's menu placement.
`adventure_type` is the one field where a minimal, isolated edit (a bisection chain that copied one
`SettingsData` field at a time from a Poseidon-origin adventure into a copy of a Zeus-origin one)
reproduced the actual in-game menu change at exactly the step this field was copied, and no earlier
step moved it. The variant names/set (`ZeusCampaign`/`ZeusCustom`/`Tutorial`/`PoseidonCampaign`/
`PoseidonCustom`) are provisional - only the raw values and the menu-placement behavior are confirmed.

`colony_location_names` (formerly part of `field_4: BoxedArray<u32, 37>`, now split into three:
`field_15: [u32; 2]` still-unidentified, this 4-element array, then `field_16: [u32; 31]` still-
unidentified) is a `SettingsData`-level cache of `MapData.world_locations[].name` for whichever
locations are `WorldLocationType::Colony` (`location_type == 1`), in `world_locations` scan order -
not one slot per colony *episode index*, and not one slot per `world_locations` index either.
Confirmed via two independent live in-game edits on a real adventure (`Test`, which has exactly the
two colonies `world_locations[1]`/`[2]`): changing colony 1's name to a standard entry and colony 2's
to a custom string moved `field_4[2]`/`field_4[3]` in lockstep with `world_locations[1].name`/
`world_locations[2].name` - an exact value match on two separately-made edits, not a fitted
correlation. `field_4[1]` was checked and does *not* mirror `world_locations[0]` (the parent city) -
there's no reserved "slot 0"; the array is just the 4 colony slots, unset ones holding this format's
standard `u32::MAX` sentinel (observed directly: `Test`'s two undefined colony slots were exactly
`0xFFFFFFFF`). `Adventure` does not store this separately - `to_pak` derives it by scanning its own
`world_locations` for `Colony` entries in order and taking each one's `name`, so it can never drift
from the `WorldLocationData` it's built from. This also retroactively explains an earlier byte-level
probe result that looked like it needed a "sentinel vs. fallback-to-entry-0" theory: zeroing bytes of
a `0xFFFFFFFF` slot one at a time cumulatively is zeroing the *whole* `u32` by the 4th byte, at which
point it's simply a valid (if arbitrary) `TextId(0)` - no special-case fallback logic required.
`custom_name`'s text itself (as opposed to the `name` id/sentinel) was confirmed absent from every
other field, known or unidentified, via a full `LogDifferences` comparison - it isn't duplicated
anywhere else the way the id is.

## `ParentEpisode` / `ColonyEpisode`

One entry per populated slot in `SettingsData.basic_episode_data`/`real_episode_data`/`mythology`
(indices `0..parent_episodes` for parents, `COLONY_SETTINGS_OFFSET + i` = `10 + i` for colony `i`;
colony `i`'s map is `map_data[1 + i]`). Slots beyond `parent_episodes` are populated with real-looking
data in every real adventure inspected but are confirmed inert in-game (raising or clearing them
changed nothing) - bounding strictly by the count field is correct.

| Field                                                          | Source                                                                                                                                                                                                                                                                                                                                                                                                                                      |
|----------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `title`/`introduction_text`/`complete_text`(/`selection_text`) | `AdventureText.episodes_text`/`colony_text`                                                                                                                                                                                                                                                                                                                                                                                                 |
| `next_episode` (`ParentEpisode` only)                          | `basic_episode_data[i].next_episode` - `ColonyEpisode` has no field for this, since a colony's next episode is always the parent adventure; `Adventure::to_pak` hardcodes `NextEpisode::Parent` for colony slots                                                                                                                                                                                                                            |
| `panhellenic_games`                                            | `real_episode_data[i].panhellenic_games == 0`                                                                                                                                                                                                                                                                                                                                                                                               |
| `mythology`                                                    | `Mythology::from_data(mythology[i])`                                                                                                                                                                                                                                                                                                                                                                                                        |
| `building_allowed`                                             | settings-level `real_episode_data[i].building_flags` if any bit is set, else the map's `scenario_data.building_flags` (see "Settings-level vs. map-level duplicates" above)                                                                                                                                                                                                                                                                 |
| `city_resources`                                               | `real_episode_data[i].city_resources: [u8; 20]`, a plain resource-id list (`0` = unused slot)                                                                                                                                                                                                                                                                                                                                               |
| `events`                                                       | `Event::vec_from_data(events[i], count)` - see "Events" below                                                                                                                                                                                                                                                                                                                                                                               |
| `episode_goals`                                                | `EpisodeGoal::vec_from_episode_goal_data(colony_episode_goals[i])` for `ColonyEpisode`; for `ParentEpisode`, the same over `parent_episode_goals[i]`, truncated to `parent_episode_goal_counts[i]` slots - see below                                                                                                                                                                                                                        |
| `world_settings` (`ParentEpisode` only)                        | see below                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `world_settings.traded_resources`                              | `TradedGoods::vec_from_data`: sell entries from `real_episode_data[i].city_resources_sold` then buy entries from `city_resources_bought` (plain resource-id lists, `0` = unused slot); each entry's `amount` is `city_resources_quantity[resource_id]` directly (indexed *by resource id*, not by position in `sold`/`bought`) - the raw value is itself the tier (`12`/`24`/`36` = low/medium/high), not a units figure needing conversion |
| `world_settings.favour`                                        | `SettingsData.parent_city_favor[i]` (a `[u32; 10]`, one slot per parent episode index - confirmed by name and value against `The Youngest Twins`; the "own city" `WorldLocationData` entry's own `favour` field is always `0`, an undefined "favor toward self", so it can't be the source)                                                                                                                                                 |
| `pyramid_settings`                                             | `PyramidSetting::vec_from_data(mythology[i].pyramids)` - see "Pyramids" below                                                                                                                                                                                                                                                                                                                                                               |

A goal is a `[goal_type, resource_id, amount]` record (three consecutive `u32`s) followed by a
64-byte still-mostly-unidentified `field_4`; `EpisodeGoalData` models this as a typed 76-byte struct.
`goal_type == 14` (`SetAsideGoods`) was the first confirmed against real data, by diffing rounds (see
below); other confirmed `goal_type`s are listed further down, after that history.
`EpisodeGoal::vec_from_episode_goal_data`/`vec_to_episode_goal_data` convert between a row of these
and a `Vec<EpisodeGoal>`.

This was pinned down over four rounds of diffing otherwise-byte-identical `.pak` files that each
only differ in one goal's fields:

- Round 1 (colony amount `8` -> `12`) landed on the record's *third* field, confirming `amount`'s
  position and disproving the field order originally assumed here (`[resource_id, goal_type,
  amount]` with `goal_type == 17`) - that reading happened to produce a plausible-looking
  `[14 (Fleece), 17, amount]` match for the colony's own bytes too, but attributed to the wrong
  episode entirely (misattributed to `ParentEpisode`, since nothing was indexing that blob per
  colony at the time).
- Round 2 (parent episode 1's amount `8` -> `18`) landed inside what had been assumed to be one
  undifferentiated trailing `field_10` blob, confirming `[14, 14 (Fleece), 8]` as a second, separate
  goal record (not the same bytes the colony record was found in) and ruling out the 1840-byte blob
  then called `parent_city_goals` (all zero in every adventure surveyed, and since renamed
  `field_12`, then absorbed into `colony_episode_goals` - see below) as its source.
- Round 3 (adding a second goal to parent episode 1, and a first goal to parent episode 2) landed
  74 bytes later - a whole `EpisodeGoalData` slot's width away - confirming a fixed 76-byte slot
  size, and a second, real, independently-editable slot for a *different* parent episode 456 bytes
  (6 slots) after the first, confirming a fixed 6-goal-slots-per-parent-episode row. The same round
  also landed on two `u32`s inside that same then-`parent_city_goals` blob (at the very end of what
  was then the smaller `field_12`, immediately before `parent_episode_goal_counts`), each
  incrementing by exactly the number of goals added to the corresponding parent episode -
  `parent_episode_goal_counts`, mirroring `parent_event_counts`'s "how many of this row's slots are
  real, vs. unused editor-template leftovers" role (the same pattern documented for `events` below).
  This resolved an earlier false lead: without a count, a *real, unmodified* adventure's episode 2
  slot 0 already held the exact same `[14, 14 (Fleece), 8]` bytes as episode 1's slot 0 (evidently
  copied as a template default when the episode was created), which looked like a real shared goal
  until `parent_episode_goal_counts[1] == 0` on that adventure showed the slot isn't actually used.
  `parent_episode_goals` is `[[EpisodeGoalData; 6]; 10]` (10 parent episodes, matching
  `parent_event_counts`/`parent_city_favor`'s convention), a fixed `10 * 6 * 76 = 4560` bytes -
  the same for both file generations, including `new_file_ver == false` adventures (`version_2 ==
  5` - `Open Play Economic 2`, `Open Play Military 2`, `Open Play Sandbox 2`, `The Odyssey`, and
  their "- Modified"/copy variants). See "Old-format goal encoding" below for where old format's
  real 4-byte difference from new format lives (`field_13`, not here).
- The colony side of this was never confirmed to the same per-slot precision (only one colony, one
  goal, ever observed) - `colony_goals` (278 bytes) and the then-`field_12` (1800 bytes) were kept
  as two separate opaque blobs, with the single confirmed colony goal record sitting at an
  unexplained, non-slot-aligned offset (254) within `colony_goals`. That stopped being a mystery once
  the two blobs' *combined* length was compared against what "4 colonies, 6 goal slots each"
  (the same per-episode-row shape as the parent side, per max colony/goal counts observable in the
  in-game adventure editor) implies: `278 + 1800 = 2078` bytes, and `4 * 6 * 76 = 1824` bytes for
  the table itself, leaving exactly `2078 - 1824 = 254` bytes over - matching the confirmed record's
  offset exactly, i.e. that offset *is* the table's start. `colony_goals` was split into a
  still-unidentified header (further split into `field_13`/`field_14`/`colony_episode_goal_counts` -
  see below) and `colony_episode_goals: [[EpisodeGoalData; 6]; 4]` (one row per colony index,
  mirroring `parent_episode_goals`); `field_12` was absorbed entirely into `colony_episode_goals` and
  no longer exists as a separate field. The trailing 16 bytes of that header (formerly `field_15`)
  are `colony_episode_goal_counts: [u32; 4]`, the colony-side counterpart to
  `parent_episode_goal_counts` - confirmed by cross-checking its four values against each colony's
  real (non-leftover) goal count across every fixture with populated colony goals (`#Zeus and
  Europa.pak`, `&Perseus and Medusa.pak`, `@Athens through the Ages.pak`, `Open Play Economic 2`,
  `Open Play Military 2`, `The Odyssey`, `The Youngest Twins`) - e.g. `Open Play Military 2`'s colony
  0 has a real 2-goal `SetAsideGoods`/`SetAsideGoods` pair followed by a leftover-template
  `Army(Warship, 6)` in slot 2, and `colony_episode_goal_counts[0] == 2` is exactly what excludes it.
  `ColonyEpisode::vec_from_data` truncates each row to this count, the same as the parent side.

`world_settings`/`traded_resources` aren't written back by `Adventure::to_pak` yet (round trips as
empty/`0`) - the read side was reverse-engineered from one adventure's data without a known general
write-back layout.

`real_episode_data[settings_index].starting_cash` (a colony's raw starting-cash field - not to be
confused with `real_episode_data[0].starting_cash`, which backs `Adventure.initial_funds`) isn't
modeled on `ColonyEpisode`, since a separate per-colony starting-cash concept didn't make sense
against observed data; `ColonyEpisode::vec_from_data`/`Adventure::to_pak` neither read nor write it,
and it round-trips as whatever `Default::default()` leaves it (`0`) rather than the file's real value.

`goal_type` values beyond `14` were pinned down from a single new-format adventure, edited entirely
through the in-game editor with one goal of each kind set per episode/colony row, by comparing the
resulting raw `goal_type`/`resource_id`/`amount`/`field_4` against what was entered - weaker
evidence than the multi-round diffing above (a single sample, not a controlled one-field-at-a-time
change), but every value matched exactly for the kinds below.
`EpisodeGoal::from_raw_fields` dispatches on these same numbers (a "marker") for *both* file
generations - what differs by `new_file_ver` is only where the marker/`resource_id`/`amount` triple
actually comes from within a row; see "Old-format goal encoding" below for the old-format case.

| `goal_type` | `EpisodeGoal`                               | Raw encoding                                                                                                                                                                                                                                                                                                                                                                                   |
|-------------|---------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `0`         | `Population(u32)`                           | value in `resource_id`; `amount` unused (observed `16`, a leftover, not written back)                                                                                                                                                                                                                                                                                                          |
| `1`         | `Treasury(MonetaryAmount)`                  | value in `resource_id`; `amount` unused                                                                                                                                                                                                                                                                                                                                                        |
| `2`         | `Sanctuary(God)`                            | `resource_id` is the `God` id (`Zeus == 0`, same encoding as elsewhere) - new-format only, see below                                                                                                                                                                                                                                                                                           |
| `3`         | `Army(UnitType, u32)`                       | `resource_id` is a `UnitType` id (`Archers = 11`, `Spearmen = 12`, `Horsemen = 13`, `Frigate = 57` confirmed), `amount` is the count - this is the same concept the pre-existing `Army` variant was a placeholder for, so its old `ArmyType(Untrained/Elite/EliteUpgraded/Naval)` payload (never confirmed against real data) was replaced with `UnitType` rather than adding a second variant |
| `6`         | `YearlyProduction(ResourceType, u32)`       | `resource_id` is the `ResourceType` id, `amount` is the yearly quantity                                                                                                                                                                                                                                                                                                                        |
| `7`         | `Rule(u8)`                                  | value in `resource_id` (a world-location/civilization id - `12` confirmed against "Azium")                                                                                                                                                                                                                                                                                                     |
| `8`         | `YearlyProfit(MonetaryAmount)`              | value in `resource_id`; `amount` unused                                                                                                                                                                                                                                                                                                                                                        |
| `9`         | `Housing(HouseLevel, u32)`                  | `resource_id` is a `HouseLevel` id (`CommonHousing = 2`, `Shack = 3`, `Residence = 11` confirmed), `amount` is the count                                                                                                                                                                                                                                                                       |
| `10`        | `TradingPartners(u32)`                      | value in `resource_id`; `amount` unused                                                                                                                                                                                                                                                                                                                                                        |
| `14`        | `SetAsideGoods(ResourceType, u32)`          | see above - unchanged, and (unlike the rest of this table) not gated on `new_file_ver`, since round-tripping was confirmed against both old- and new-format real adventures                                                                                                                                                                                                                    |
| `15`        | `Pyramids(u8)` (when `amount == 0`)         | `resource_id` a constant `1`; count is the second `u32` of `field_4` (offset `4`), preceded by a `u32::MAX` of unknown purpose at offset `0`                                                                                                                                                                                                                                                   |
| `15`        | `Pyramid(PyramidType)` (when `amount != 0`) | `resource_id` a constant `1`; `amount` is the `PyramidType` id directly - the same `100..=114` numbering as `PyramidConfig::from_raw_fields` (see "Pyramids" below), confirmed by cross-checking `100` (`ModestPyramid`), `105` (`MonumentToTheSky`), `110` (`PyramidOfThePantheon`), `112` (`TempleOfOlympus`), `113` (`ObservatoryKosmika`), and `114` (`MuseumAtlantika`) against real goals in `#The Birth of Atlantis.pak`/`&Enlightenment in the West.pak`                                                                                                  |
| `16`        | `Hippodrome(u32)`                           | value in `resource_id`; `amount` unused                                                                                                                                                                                                                                                                                                                                                        |

`PyramidType` is deliberately a flat id-only enum (`data_constants!`) rather than reusing the
detailed `PyramidConfig` enum (renamed from the original `PyramidType`) that models a *placed*
pyramid's per-tile `PyramidColor` array and, for shrines, its `God` - a goal only names which
structure is required, and `amount` being a single raw `u32` couldn't carry that placement data
even if the goal wanted to. It reuses `PyramidConfig`'s own `100..=114` id numbering (see below)
rather than a separate scheme, including for the three shrine ids (`107`/`108`/`109`,
`MinorShrine`/`Shrine`/`MajorShrine`) despite `PyramidType` having no god payload to put a shrine's
target deity in - unconfirmed against real data, since no real goal using a shrine id has been
observed yet, but assumed to hold given the two enums otherwise sharing this numbering exactly.

### Pyramids

`MythologyData.pyramids` is always a fixed 6-slot array (padded with zeroed `PyramidData` past
`max_pyramids` real entries); `PyramidSetting::vec_from_data`/`vec_to_data` in `pyramid.rs` convert
between it and `Vec<PyramidSetting>`, dropping/re-padding unused (`pyramid_type == 0`) slots.

Confirmed by cross-referencing every `pyramid_type` observed across the full `Adventures` library
(including the on-disk `.pak`s outside the `Adventures/<name>/` folders) against the tile count
implied by `coloration`'s low bits:

- `pyramid_type` values `100..=114` correspond 1:1 and in enum-declaration order to `PyramidConfig`'s
  fifteen variants (`100 = ModestPyramid`, ..., `114 = MuseumAtlantika`); `111` (`AltarOfOlympus`) is
  the one id never observed in real data, but it fills the only gap in an otherwise unbroken sequence.
- `deity` is a `God` id, meaningful only for the three shrine variants (`107`/`108`/`109`); every
  other variant carries a constant `1` (`Poseidon`) with no apparent meaning; `to_raw_fields` writes
  that same constant back for non-shrine variants.
- `coloration`'s low `N` bits (bit `i` = tile `i`, `0` = white, `1` = black) are each variant's
  `[PyramidColor; N]` array; the remaining, always-higher bits vary independently of tile count and
  aren't understood yet (not round-tripped - `to_raw_fields` always writes them as `0`).
- `PyramidSetting.allowed` has no identified raw source yet and is always read as `true`
  (unwritten on the way back out) - every populated slot found so far represents an offered option,
  so this may simply be redundant with "slot is populated" rather than a distinct flag.

### Old-format goal encoding (`new_file_ver == false`)

Old-format goals use the same `goal_type` marker numbering as the new-format table above -
`EpisodeGoalData`'s field layout doesn't differ by format, only `SettingsData`'s surrounding byte
lengths do (see below). `Sanctuary`/`Sanctuaries` are the one addition specific to old format.

`SettingsData.tab_visibility`/`buffer_0x01_a`/`buffer_0x00_a`/`world_map_enabled`/`buffer_0x01_b`/
`buffer_0x00_b`/`field_14`/`colony_episode_goal_counts` hold the exact boundary: what's now those six
fields (see below for how the 234-byte span between `tab_visibility` and `field_14` was carved up) was
originally found and named as one undivided `field_13: BoxedArray<u8, 234>`, still-unidentified bytes,
byte-identical between file generations; `field_14: u32` (also still-unidentified) sits between it and
`colony_episode_goal_counts` and is present only for `new_file_ver`. The boundary itself was confirmed
by diffing raw bytes directly rather than computing the split from field-size totals alone, against a
new-format adventure found to hold the same underlying content as `The Odyssey.pak` (`version_2 == 5`):
every byte from `parent_event_counts` through the end of `bitmap` (35,892 bytes) was identical between
them, and they stayed identical for another 234 bytes past `bitmap`. Right at that point the new-format
file had 4 extra bytes; past them the two stayed aligned (`Odyssey[234 + n] == other[238 + n]`) for
172+ more bytes, well into `colony_episode_goals` itself.

The original 234-byte `field_13` was later surveyed in full across every real `.pak` under
`Adventures` (not just the one Odyssey/new-format-resave pair above): every byte is strictly `0` or `1`
(a byte-per-flag array, not arbitrary data), and the whole value is shared byte-for-byte by all 26
real, playable adventures - every story campaign, `Open Play`/`Test` file, and format resave -
regardless of episode/colony counts, `adventure_type`, or `bitmap`, generalizing the single-pair
"byte-identical between file generations" finding above to the full corpus. It does vary, in a
monotonically-growing pattern (each step's flags are mostly, but not strictly, a superset of the
previous step's), across the game's 15 single-building tutorial/demo `.pak`s (`housing` -> `agora`/
`food` -> `culture`/`hygiene` -> ... -> `bigcity`). For real adventures the whole thing is a fixed
per-adventure template constant, the same "map-editor template leftovers" pattern
`MapData.scenario_data`'s narrative/economy fields already use (see above) - `Adventure::to_pak` writes
the real-adventure constant rather than defaulting it to zero, which is what it did previously.

**The front 11 bytes (offsets 0-10) are now split off as `tab_visibility: [u8; 11]`** - confirmed as a
byte-per-building-tab flag array by cross-referencing the byte-level tutorial progression above against
an in-game-observed list of which of the game's 11 building tabs are visible on each tutorial file.
Offsets `2`/`7`/`10` uniquely matched tabs `3`/`8`/`11`; offsets `{1,3}`/`{5,8}`/`{6,9}` each matched a
*pair* of tabs (`2`/`4`, `6`/`9`, `7`/`10`) that always toggle together in every sample available, so
those three assignments rest on the simplest hypothesis (byte `i` = tab `i + 1`, matching where the
three uniquely-confirmed offsets land) rather than independent confirmation; offsets `0`/`4` (tabs
`1`/`5`) never varied at all in the tutorial corpus, consistent with those two tabs always being
visible. Disambiguating the three pairs is parked pending further `.sav` investigation - see
`docs/adventure_model_todo.md` for the full writeup, including the remaining 7 of the original 16
varying offsets that do *not* correlate with any tab.

**The trailing 223 bytes are split by constant-fill run, plus one more named field**, rather than
kept as one opaque blob: `buffer_0x01_a` (143 bytes), `buffer_0x00_a` (66 bytes),
`world_map_enabled` (1 byte), `buffer_0x01_b` (9 bytes), `buffer_0x00_b` (4 bytes), in that order -
matching `MapData`'s own `buffer_0x00`/`buffer_0xff` naming for the same "constant on every real
adventure" pattern, but named `buffer_0x01`/`buffer_0x00` here since the constant values are `1`/`0`
rather than `0`/`0xFF`. `world_map_enabled` is old byte offset `220`, promoted to a named field on a
user's separately-reported in-game observation that the world map becomes available starting with
the `trade` tutorial - matching exactly: `0` through `interface`, `1` from `trade` onward and for
every real adventure. The four `buffer_0x*` fields are not assumed to each be a single semantic
field just because they read as one uniform run today - see `docs/adventure_model_todo.md`'s
"Update" note for why that's treated as evidence about value, not about field boundaries.

`Sanctuary`/`Sanctuaries` (`new_file_ver == false`) were confirmed from six real, shipped goals
across two old-format adventures (`The Odyssey`, `Zeus and Europa`): sanctuary-to-a-specific-god
goals for `Hermes`, `Athena`, `Zeus`, `Hephaestus`, and `Ares`, plus one "2 sanctuaries" (any god)
goal. `Sanctuary` is the marker scheme's own `goal_type == 2` category (god id in `resource_id`);
`Sanctuaries` (an unspecified count of *any* god) has no new-format equivalent marker, so it's an
extra old-format-only arm ahead of the general `goal_type == 2` one: `resource_id == u32::MAX`
("any god" sentinel) with the count in `field_4[0]` rather than `amount` - confirmed against
`#Zeus and Europa.pak`'s real `Sanctuaries(2)` (`field_4[0] == 2`) and its own leftover
`Sanctuaries(1)` duplicate (`field_4[0] == 1`, see the leftover-slot gap below).

`from_raw_fields` was confirmed against every populated goal in `#Zeus and Europa.pak`, `&Perseus
and Medusa.pak`, `@Athens through the Ages.pak`, and `The Odyssey`.

`goal_type == 4` is `Quest(God, QuestType)`: unlike every other kind, its payload isn't in the goal
record at all. `resource_id` is the index of a `Quest` event (`event_type == 4`) in this same
episode/colony's own event list (the same "index into `events`, bypassing `parent_event_counts`/
`colony_event_counts`" trick `Slay` uses below); the linked event's `subtype` is the offering god's
raw id (the same numbering `God` resolves everywhere else), and `quest` (`0`/non-`0`) selects
`QuestType::Type0`/`Type1` - only `0` has been observed in real data so far, so `Type1` is
unconfirmed. Confirmed against `&Perseus and Medusa.pak` (two parent-episode quests offering
`Hades`/`Athena`, one colony quest offering `Hermes`) and `@Athens through the Ages.pak` (one
parent-episode quest offering `Demeter`) - every linked event's `subtype` matched the expected god's
`value()` exactly. This supersedes an earlier, mistaken reading of the same real `&Perseus and
Medusa.pak` quests as `goal_type == 1` records colliding with the `Sanctuaries` sentinel; that
reading was wrong about which goal slot held them.

Some rows have a genuine goal immediately followed by a leftover editor-template slot duplicating
it (same kind, same or a differing count) - e.g. `#Zeus and Europa.pak` episode 4's real
`Sanctuaries(2)` followed by a template `Sanctuaries(1)`, or `Open Play Military 2`'s colony 0
(a real `SetAsideGoods`/`SetAsideGoods` pair followed by a leftover, well-formed-looking
`Army(Warship, 6)`). Not distinguishable from a real extra goal of the same or a different kind by
anything in the goal record itself - `parent_episode_goal_counts[i]`/`colony_episode_goal_counts[i]`
(see above) are what exclude these slots, for both file generations.

`ParentEpisode::vec_from_data`/`ColonyEpisode::vec_from_data` both truncate their row to
`parent_episode_goal_counts[i]`/`colony_episode_goal_counts[i]` for both file generations. An
earlier version of this doc claimed old-format rows couldn't be truncated this way (citing a
`Sanctuary` observed past what the count considered used, and a whole row whose count read `0`
despite holding a real goal) - re-examined against `The Odyssey`'s actual counts once
`colony_episode_goal_counts` was found and cross-checked, no such counterexample reproduces: every
old-format episode/colony's expected goal list, across all six old-format fixtures, matches its
count field exactly, including a 6-goal parent-episode row (`The Odyssey` episode 2) truncating to a
count of `6` (not `5`) and a 1-real-goal row truncating to a count of `1` (not `0`). The original
claim was evidently based on a stale or miscomputed count reading, not real counterexample data.

`EpisodeGoal::to_raw_fields` only encodes the kinds in the table above, always in the new-format
shape (e.g. `Sanctuary`'s god id goes straight into `resource_id`, not the old-format sentinel
shape); `Sanctuaries`/`Quest`/`Slay` - not yet reverse-engineered on the write side at all - are
silently dropped by `Adventure::to_pak` if present in the model. This is consistent with
`Adventure::to_pak` itself, which currently always builds new-format-shaped (`version_2: 26`)
settings data regardless of the source adventure's own format (a pre-existing gap - `Adventure`
doesn't track `version_2` at all), so today a round trip through the model always upgrades an
old-format adventure's goals to new-format encoding, dropping `Sanctuaries`/`Quest`/`Slay`.
`parent_episode_goal_counts[i]`/`colony_episode_goal_counts[i]` on write are the number of goals
from that episode/colony's list that actually got encoded (i.e. can be smaller than
`episode_goals.len()` if some goals didn't encode).

`Adventure::to_pak` writes `basic_episode_data[i]` and `real_episode_data[i].basic_episode_data` as
byte-for-byte duplicates of each other, with `field_3`/`field_5`/`field_6` always `0xFFFFFFFF` (their
meaning is unidentified, but every real populated slot has them at this value).

## `CityMap`

Sourced from one `MapData` (`map_data[0]` for `parent_city`, `map_data[1 + i]` for colony `i`).

- `map_size`/`tropical`/`entry_point`/`exit_point` - direct fields on `RealEpisodeData` (`scenario_data`).
- `sprite`/`root_offset`/`terrain`/`tile_size`/`random`/`meadow`/`scrub`/`elevation` - the per-tile
  arrays `CityMap` models today; several others (`field_20`, `elevation_rotation`, `field_17`) remain
  entirely unmodeled (round-trip as empty).
- Spawn/hazard point arrays (`fishing_spots`, `wolf_spawn`, `urchin_spawn`, `invasion_points` - where
  invading forces land, `deer_spawn`, `disaster_points` - fire/plague/etc. locations, `boar_spawn`,
  `monster_spawn`, `disembark_points` - where boats put ashore, `landslide_spawn`) and the single
  points (`earthquake_area`, `river_entry`, `river_exit`) all use the `u16::MAX`-is-unset convention.
  `monster_x`/`monster_y` are the one pair stored as `u32` rather than
  `u16` in the raw format; every real value observed still fits `u16`. Each is `Vec<Option<(u16, u16)>>`
  (or a single `Option`) rather than a dense `Vec`, because real adventures leave gaps mid-array
  (confirmed on `The Youngest Twins`' `invasion_x/y`) - `None` preserves the gap; trailing `None`s are
  stripped since they're indistinguishable from the array just being shorter.
- `MapData.scenario_data`'s `start_date`/`starting_cash`/`panhellenic_games`/`text_buffer_1`/
  `text_buffer_2` are map-editor template constants, not real per-episode data (the real values live on
  `SettingsData.real_episode_data[i]`, written by `Adventure::to_pak`) - confirmed identical across
  every real map file surveyed: `-500`/`1000`/`65535`/`"Brief description"`/`"Brief description of
  this episode, for players. History, aims and tips etc."`.
- `scenario_data.civilization`/`building_flags` are written by `Adventure::to_pak` (not `CityMap`
  itself, which has no access to either) - `civilization` is the same value on every map in the
  adventure; `building_flags` is written both to the settings-level copy (primary) and the map (fallback).

## `Mythology`

From `MythologyData` (`SettingsData.mythology[i]`, the settings-level copy - confirmed authoritative,
the map-level copy is unused). `proponent_gods`/`opponent_gods` decode the `God`-id arrays;
`max_sanctuaries` and `monster` are direct/resolved. `max_pyramids`/`pyramids` are modeled separately,
on `ParentEpisode`/`ColonyEpisode` rather than `Mythology` itself - see "Pyramids" above.

Each `proponent_gods[j]`'s sanctuary-allowed flag is `sanctuaries_allowed[j]` - matched by *array
position*, not by god id (`sanctuaries_allowed` is a parallel `0`/`1` flag array, not a set of allowed
god ids to search) - confirmed against `The Youngest Twins` episode 1's 6 proponent gods, whose flags are
`true`/`true`/`false`/`true`/`false`/`true`.

## World map: `WorldLocation` / `TradeRoute`

`Adventure.world_locations`/`trade_routes` are shared by the whole adventure: confirmed byte-for-byte
identical across every map in the common case, sourced from `map_data[0]` and written identically to
every map's `world_locations`/`world_map_elements`/`trade_routes` (and the `SettingsData.map_data`
duplicate) by `Adventure::to_pak`.

**`WorldLocationData[i]` and `WorldMapElementData[i]` are NOT reliably paired by matching raw index** -
`WorldMapElementData.region_name` is the real back-link and must be used to find a location's element,
not positional equality. `region_name == i` only coincides with raw index `i` while every earlier
`WorldLocationData` slot is populated; once an earlier slot is unused (`exists == 0` - e.g. `The
Youngest Twins`' location slot `19`, which has no `WorldLocationData` entry at all), every later
location's real `WorldMapElementData` is packed one raw index earlier than its own slot instead
(location `20`'s element sits at raw index `19`, location `21`'s at raw index `20`, etc.) - real
adventures leave gaps in the 22-slot `world_locations` array (a location can sit at any slot, not
just a dense prefix), and `world_map_elements` is apparently packed densely around those gaps rather
than mirroring them. `WorldLocation::vec_from_data` was fixed to search `world_map_elements` for
`variant != 0 && region_name == slot` instead of indexing by position - confirmed against `The Youngest
Twins`: under the old positional pairing, location `21` ("Egyptium") read as a marker-less placeholder
(`position (0, 0)`, `marker_variant 0`) sourced from the genuinely-unused raw index `21`, when its real,
visible marker (`position (660, 476)`, `variant 1`, matching what's visible on the colony map's own
copy) was sitting one slot earlier at raw index `20` the whole time - caught by moving/toggling
visibility on that location in-game (via an editor copy of the adventure) and finding the in-game
change didn't show up anywhere in the positionally-paired byte diff, while the `region_name`-paired
read matched exactly. `WorldLocation.slot` is preserved (not repacked into a dense list) so this gap-driven shift
doesn't need to be exposed on the model.

`WorldLocation.location_type: WorldLocationType` is a nested enum (`ParentCity`/`Colony`/`ForeignCity`/
`DistantCity`/`EnchantedPlace`, one payload struct per variant), the same shape `Event` uses for its
own type-specific payloads - the discriminant *is* the enum variant, there's no separate stored
"type" field on the model. Several raw fields the format stores for every location regardless of type
(e.g. `civilization`/tribute/`favour`/`traded_goods` on the parent city's own entry; `name`/
`leader_name`/`civilization` on `EnchantedPlace`) aren't modeled on the variants that don't seem to use
them in-game, even where the raw bytes are real/non-zero - `WorldLocationType::to_data` writes
defaults back for whatever a variant doesn't carry. See each variant's field list in
`world_location.rs` for exactly which raw fields it keeps.

| Field                                                 | Source                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
|-------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `location_type` (variant choice)                      | `WorldLocationData.location_type`: `0` = `ParentCity`, `1` = `Colony`, `2` = `ForeignCity`, `4` = `DistantCity`, `5` = `EnchantedPlace`, matches every value observed                                                                                                                                                                                                                                                                                                                                                                                  |
| `position`/`marker_variant`                          | `WorldMapElementData.x`/`y`/`variant`(`1`=city marker, `2`=label)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `sprite_id`/`sprite_size`                             | Derived via `WorldLocationType::sprite` for every variant, including `ParentCity` - see the "`sprite_id`/`sprite_size` investigation" note below                                                                                                                                                                                                                                                                                                                                                                                                    |
| `traded_goods`                                        | `TradedGoods::vec_from_data(selling, buying, trade_quantities)` - same sold/bought-resource-id-list-plus-quantity-table shape as `RealEpisodeData.city_resources_sold`/`bought`/`quantity`, indexed directly by resource id like that field. `ParentCity`/`EnchantedPlace` don't carry this                                                                                                                                                                                                                                                          |
| `name`                                                | `WorldLocationData.custom_name`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `leader_name`                                         | `custom_leader_name` if non-empty (`EntityName::Custom`), else the numeric `leader_name` field as an unresolved id (`EntityName::TextId`) - on non-"extras" format files `custom_leader_name` is always empty and `leader_name` holds a small numeric id, presumably into one of the `.eng` text-table groups (not cross-referenced yet). Every variant but `EnchantedPlace` carries this                                                                                                                                                            |
| `label_position`                                      | `WorldMapElementData.label_position` (`0` = `Left`, `3` = `Bottom`) - confirmed against `The Youngest Twins`; other observed raw values (`1`/`2`) aren't cross-referenced yet and resolve to `Bottom`. A location with a genuinely marker-less element (`variant == 0`, no `world_map_elements` entry with a matching `region_name`) has no real byte to read and also defaults to `Bottom`                                                                                                                                                           |
| `civilization`                                        | `WorldLocationData.civilization` (formerly the unnamed `variant` field) - the world-map sprite set ("nation skin") a location's marker uses, a different encoding from the adventure-level `Civilization`. Confirmed: `0` = `Greek`, `2` = `Persian`, `3` = `Centaur`, `5` = `Egyptian`, `8` = `Oceanid`, `9` = `Atlantean`; other raw values (`1`/`4`/`6`/`7`) are seen in other real adventures but not yet cross-referenced. Only `ForeignCity` carries this - it's meaningless on the other types, which still carry whatever raw value the file happens to have |
| `relation`                                            | `WorldLocationData.attitude` (`Relationship`: `0` = `Ally`, `1` = `Rival`, `2` = `Vasal`) - only `ForeignCity` carries this. Raw value `3` ("Colony") is also observed, unexplained, on the parent city's own entry and on `EnchantedPlace`s, which have no `relation` field to receive it                                                                                                                                                                                                                                                            |
| `active`/`visible`                                    | `active` via `resolve_active(location, new_file_ver)` (see the "`sprite_id`/`sprite_size` investigation" note below); `visible` from `WorldLocationData.visible` directly, inverted (`0` = visible, `4` = not yet visible; only these two raw values observed). `ParentCity` doesn't carry `visible` (always visible in-game); `Colony` doesn't carry either (always active and visible)                                                                                                                                                            |
| `military_strength`/`economy_strength`/`favour`       | `WorldLocationData.military_strength`/`economical_strength`/`favour_new` directly, except old-format non-`Parent`/non-`EnchantedPlace` locations with a zero `favour_new`, which read `favour_old` instead (see `resolve_favour`). `ParentCity` carries none of these three; `EnchantedPlace` carries only `military_strength`                                                                                                                                                                                                                      |
| `paying_tribute`                                      | `WorldLocationData.tribute != 0` - only `Colony`/`ForeignCity` carry this                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `paid_tribute`/`received_tribute`                     | `(ResourceType, MonetaryAmount)` from `(tribute_pay_resource, tribute_pay_amount)`/`(tribute_rec_resource, tribute_rec_amount)` - the two resource fields are independent resource ids, resolved via `ResourceType`. Only `Colony`/`ForeignCity` carry these                                                                                                                                                                                                                                                                                          |

| `direction` (`DistantCity` only)                     | `WorldLocationData.civilization` - the same raw field `ForeignCity` reads as `LocationCivilization`, repurposed for `Distant` records to encode the marker's off-map compass direction instead of a nation skin (`WorldDirection`: `0` = `None`, `3` = `E`, `4` = `Se`, `5` = `S`, `6` = `Sw`, `7` = `W`, confirmed; `1`/`2`/`8` = `N`/`Ne`/`Nw` inferred from the sequence, not yet confirmed)                                                                                                                                                    |

An initial pass looking for `direction` found nothing: neither a consistent byte anywhere in the
still-unmodeled fields nor a geometric (angle-from-parent-city) formula held up against 11 real
cross-adventure samples with known directions. The actual encoding turned up by comparing a
controlled single-field in-game edit (one location's direction changed from `Se` to `S`, nothing
else about its identity): the *only* field that changed besides the marker's sprite was
`civilization`, `4` -> `5`. Cross-checking against all 12 known-direction samples (the original 11
plus this edit) showed `civilization`'s raw value predicts direction exactly, with zero exceptions -
`ForeignCity`/`DistantCity` simply read the same raw byte with different meanings, the same way many
`EventData` fields are reinterpreted per event type.

### `sprite_id`/`sprite_size` investigation

Surveyed `sprite_id`/`sprite_width`/`sprite_height` across every real (non-tutorial-mission)
adventure under `ZEUS_HOME/Adventures`, ultimately ~400-500 location records across several rounds
(including two controlled in-game edits and a purpose-built test adventure covering every direction
in both the active and inactive state). Findings, now fully implemented in `WorldLocationType::sprite`:

- **`sprite_size` is a pure function of `sprite_id`**: every `sprite_id` observed maps to exactly
  one `(width, height)`, no exceptions once tutorial/single-building mission `.pak` files are
  excluded (those hold unused template data, not real content - see "Map-editor template leftovers"
  above).
- **`WorldLocationData.active_old`** (a `u32` split out of what was a 15-byte `unknown_341`
  block - see below; formerly named `unknown_352`, then `legacy_active`) and `active_new` are
  format-version duplicates of one "visually active" concept, not two independent triggers:
  surveyed across 250 real `Colony`/`ForeignCity`/`DistantCity`/`EnchantedPlace` records,
  `active_old` is nonzero only when `MapData.version_1 < 300` (old-format), `active_new` only when
  `>= 300` (new-format) - zero exceptions either way. Crucially this is gated by the *map's* own
  version, not the adventure-level `SettingsData.version_2`: `The Odyssey V2` has a resaved
  (new-format) `SettingsData` but an untouched (old-format) `MapData`, and its 8 `active_old`
  records prove `active_old` is still live there - see `Adventure::from_pak`'s own note on this
  settings/map version split. So the two bytes collapse to one domain-level `active: bool` via
  `resolve_active` (mirroring `resolve_favour`'s shape, though - unlike `favour` - the two `active`
  bytes are strictly mutually exclusive, no "prefer the new field when nonzero" fallback needed). A
  live in-game edit (toggling `active_new` on three `Test` locations, a new-format adventure) was
  also captured byte-for-byte and confirmed only `active_new` moved; `active_old` stayed `0`
  throughout, consistent with this.
- **`ForeignCity`**: `active ? civ_sprite[civilization] : generic_sprite`. One fixed sprite per
  `LocationCivilization` (`0`-`9`) plus one shared "generic small settlement" sprite (`117`, 14x13)
  for the inactive case. Zero exceptions across 274 real records.
- **`DistantCity`**: `active ? active_sprite[direction] : inactive_sprite[direction]` - a full
  18-value table (9 directions x 2 states), confirmed directly by a purpose-built test adventure
  covering every direction in both states, plus zero exceptions across 48 real records.
- **`EnchantedPlace`**: `active ? 85 : 86`. Zero exceptions across 16 real records (this fully
  resolved what looked like a few stray exceptions before `active_old` was accounted for).
- **`Colony`**: on read, a new-format `active_new` marker and an old-format `active_old` one render
  as *different* sprites (`47` vs `70`) rather than collapsing into one state the way the other
  three variants do - confirmed by `}Open Play Sandbox`, the one real record with `active_old` set
  (old-format, inactive, Greek), which shows sprite `70`, not the Greek civilization default a
  naive `Civilization`/`active`-only formula would predict. But since the domain model only keeps
  the collapsed `active: bool` (see above) and `Adventure::to_pak` - the only caller of
  `WorldLocationType::sprite` - always emits new-format maps (see `CityMap::to_map_data`), the `70`
  sprite is unreachable from this library's write path; `sprite`'s formula is simply
  `active ? 47 : civilization_default` (`Atlantean` -> `48`, `Greek` -> `71`). This means a
  from-scratch write of `}Open Play Sandbox`-shaped data (old-format, `active_old` set) would now
  emit `47` rather than round-tripping the original `70` - a real, accepted gap, the same shape as
  `ParentCity`'s `Odyssey V2` gap below. Zero exceptions on the read side across 26 records
  otherwise; separately, one apparent read-side mismatch in `Odyssey V2` is consistent with that
  adventure's own `real_episode_data.civilization` byte reading differently across the old/new
  format boundary rather than a sprite-rule failure - not fully run down, but `Test`/`Odyssey V2`
  are already known to carry format-conversion artifacts.
- **`ParentCity`**: **RESOLVED** - not active/inactive at all (unlike the other four types; the
  player's own home city has no "not yet founded" state to toggle), just the adventure's overall
  `Civilization`: `Atlantean` -> `46` (35x39), `Greek` -> `69` (33x38). An `active`-keyed formula
  (`active ? 46 : 69`) was tried first and looked almost right (only 2 exceptions out of 28 real
  records, both unexplained), because `active` happens to correlate with `Civilization` almost
  everywhere in the corpus (every sampled Atlantean adventure's own parent city happens to be
  `active`, every sampled Greek one not) - the 2 exceptions (`]Proetus and Bellerophon` and
  `^Two Worlds Collide`, both real, non-tutorial, new-format adventures) are exactly the cases that
  break that correlation: both are Greek with `active == true`, and both still show sprite `69`
  (Greek), which only the `Civilization`-keyed formula predicts. Re-run against all 45 real
  non-tutorial records, the `Civilization` formula gets 44/45 right; the one remaining miss is
  `Odyssey V2` - already flagged above as a file with a known old/new-format civilization-byte
  artifact, and independently confirmed by the pre-existing `parse_the_odyssey_v2` test, which
  hardcoded this adventure's parent-city sprite as `69` despite its `Civilization` being
  `Atlantean`. Implemented: `ParentCity` no longer stores `sprite_id`/`sprite_size` at all (dropped
  from the struct, matching the other four variants); `WorldLocationType::sprite` derives it like
  everything else. This means a from-scratch write of `Odyssey V2`-shaped data (Atlantean civ, this
  specific parent city state) would now emit `46` rather than round-tripping the original `69` -
  accepted the same way `Colony`'s formula already accepts the identical gap on the same file,
  rather than keeping a stored-field special case for one known-quirky adventure.

`Colony`'s formula is the reason `WorldLocationType::sprite` takes the adventure's `Civilization` as
a parameter, threaded from `Adventure.civilization` through `WorldLocation::to_data`/`vec_to_data` -
the only place in this module that needs adventure-level (not just per-location) context.

Writing `custom_name`/`custom_leader_name` requires `MapData.manifest[25].size == 572` (the flag
`WorldLocationData`'s (de)serialization uses to include those extras-format fields, uniformly for the
whole 22-slot array) - `Adventure::to_pak` always sets it, and pads every slot (populated or not) at
the matching record length so the whole array stays self-consistent.

Still unmodeled (meaning unknown) on `WorldLocationData`: ~371 bytes across `unknown_*` blocks
(`unknown_240` and `unknown_341`/`unknown_353` flank the resolved `favour_old` byte between them -
see `favour` above). `active_old` (a `u32`, split out of what used to be one 15-byte
`unknown_341` block) sits in that same gap but is fully understood now - see the `sprite_id`/
`sprite_size` investigation above - so it's promoted out of the `unknown_*` naming, unlike its
still-unmodeled neighbors. Still unmodeled on `WorldMapElementData`: `data_a`/`data_d`, and
`unknown_a`/`unknown_b` (the bytes flanking `label_position`; see above and their own field
comments) - `sprite_id`'s own mapping (which raw id draws which icon) is no longer unmodeled, see
above.

`TradeRoute.destination`/`slot` index into `world_locations`/the raw 232-slot array the same
gap-preserving way. `TradeRouteData.distance: [u8; 12]` packs three sub-fields: bytes `0..4` a `u32`
distance, `4..8` always zero, `8..10` a `u16` `destination`, `10..12` a `u16` `route_number`
(distinguishing alternate routes to the same destination). `header` (constant `[5, 0, 0, 0, 0, 0, 0,
0]`) and the trailing `unknown: u8` (constant `0`) are free/hardcoded on write. `route_type` is kept as
a raw id (only `1`/`2` observed, consistent with but not confirming "sea/land route?").

## `prices`

`Adventure.prices: Vec<u32>` is a direct, index-preserving copy of `MapData.prices: BoxedArray<u32,
36>` (indexed by `ResourceType` id - e.g. index `15` is `Horse`'s price - with index `0` and indices
past `23`/`Drachmas` unused/zero). Confirmed against `The Youngest Twins`: filtering the raw array's
non-zero entries reproduces its known real-world market prices in order. Same "world-scoped, shared
across every map" duplication as `world_locations`/`trade_routes` above - sourced from `map_data[0]`
and written identically to every map (and the `SettingsData.map_data` duplicate) by `Adventure::to_pak`.
Deliberately kept as a raw indexed array rather than resolved into `(ResourceType, u32)` pairs so a
genuine `0` price (as opposed to an absent resource) isn't accidentally dropped.

## Events

`SettingsData.events` is `[[EventData; 150]; 14]`, one 150-slot block per episode index (same
convention as `real_episode_data`/`mythology`). Real events are densely packed from slot `0` - unlike
most other dense arrays in this format, slot `0` is a real event, not a sentinel/always-unused slot -
with `id` always equal to the slot index, for both real and padding slots alike (so `id` alone can't
tell them apart).

The real per-episode event count comes from `SettingsData.parent_event_counts: [u32; 10]` (one count
per parent episode index) and `colony_event_counts: [u8; 3]` (one count per colony index) - split out
of what the raw format stores as a single `[u8; 43]` block (`parent_event_counts` occupies its first
40 bytes, 4-byte-strided; the 3 padding bytes between each `u32` are confirmed always `0`). Confirmed
against 5 real parent episodes across two adventures (`The Youngest Twins`, `The Founding of Troy`)
and one colony. `colony_event_counts` is one byte short of the 4 colony slots `real_episode_data` has
room for (`COLONY_SETTINGS_OFFSET` reserves indices `10..14`) - unresolved whether that's a real
format limit or `SettingsData`'s fields are mis-sized by one byte somewhere earlier, silently
borrowing from what's modeled as `unused_blocks`. Slots at and beyond a real count hold real-looking
but confirmed-unused editor template data, the same "extra populated slots" pattern documented for
`basic_episode_data`/`real_episode_data` above - this supersedes an earlier (incorrect) conclusion
that the raw `[u8; 43]` block "doesn't consistently match" and is unusable, which was itself an
artifact of comparing it against a dense scan that both skipped the real slot `0` and over-ran into
this padding.

The format itself was reverse-engineered in a forum post archived at `zeus_mapper/notes.md`; this
mapping's offsets and event/subtype tables were verified against it (hand-computed every `EventData`
field's byte offset - all 124 bytes and every documented offset land on the expected field).

`Event::from_data` dispatches on `event_type`/`subtype`. `GoodsRequest` and `MilitaryRequest` share
`event_type == 1`; `TradeChange` and `CityStatusChange` share `event_type == 19` - both are split
further by `subtype`:

| `event_type` | Meaning                         | `subtype` dispatch                                                                                                                                                                                                                                                                                                                                                                     |
|-------------:|---------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|            1 | goods/troops request            | `0`=`GoodsRequestSubtype::GeneralRequest`, `3`=`GoodsRequestSubtype::Festival`, `4`=`GoodsRequestSubtype::Construction`, `5`=`GoodsRequestSubtype::Famine`, `6`=`GoodsRequestSubtype::FinancialWoes`, `7`=`MilitaryRequestSubtype::GreekCityTerrorized`. `1`/`2` (`MilitaryRequestSubtype::CityUnderAttack`/`CityAttacksRival`) have no confirmed second-`CityId` source, not produced |
|            2 | invasion                        | `Invasion`                                                                                                                                                                                                                                                                                                                                                                             |
|            3 | earthquake                      | `Disaster(Earthquake)`                                                                                                                                                                                                                                                                                                                                                                 |
|            5 | landslide                       | `Disaster(Landslide)`                                                                                                                                                                                                                                                                                                                                                                  |
|          8/9 | wage increase/decrease          | `WageIncrease`/`WageDecrease`                                                                                                                                                                                                                                                                                                                                                          |
|        13/14 | trade demand increase/decrease  | `TradeChangeSubtype::DemandIncrease`/`DemandDecrease`                                                                                                                                                                                                                                                                                                                                  |
|        15/16 | price increase/decrease         | `TradeChangeSubtype::PriceIncrease`/`PriceDecrease`                                                                                                                                                                                                                                                                                                                                    |
|           19 | city status / trade-open change | see below                                                                                                                                                                                                                                                                                                                                                                              |
|        21/22 | trade supply increase/decrease  | `TradeChangeSubtype::SupplyIncrease`/`SupplyDecrease`                                                                                                                                                                                                                                                                                                                                  |
|           23 | gift                            | `Gift`                                                                                                                                                                                                                                                                                                                                                                                 |
|           24 | lava flow                       | `Disaster(LavaFlow)`                                                                                                                                                                                                                                                                                                                                                                   |
|           25 | tidal wave                      | `Disaster(TidalWave)`                                                                                                                                                                                                                                                                                                                                                                  |
|           26 | monster unleashed               | `0`=`MonsterInvasionSubtype::MonsterInCity`, `1`=`MonsterInvasionSubtype::MonsterUnleashed`, `2`=`MonsterInvasionSubtype::MonsterInvades`                                                                                                                                                                                                                                              |
|           27 | god invasion                    | `GodInvasion`                                                                                                                                                                                                                                                                                                                                                                          |
|           28 | sink land                       | `Disaster(SinkLand)`                                                                                                                                                                                                                                                                                                                                                                   |
|        other | -                               | not observed in any real adventure surveyed, or documented as computer-generated-only/unused - not produced                                                                                                                                                                                                                                                                            |

`event_type == 19` subtypes: `2`/`3`=`TradeChangeSubtype::TradeShutsDown`/`TradeOpensUp`,
`9`=`CityStatusChangeSubtype::RivalBecomesAlly`, `10`=`CityStatusChangeSubtype::CityBecomesRival`,
`11`=`CityStatusChangeSubtype::CityBecomesVassal`, `13`=`CityStatusChangeSubtype::GodDisaster`,
`14`/`15`=`CityStatusChangeSubtype::MilitaryBuildup`/`MilitaryDecline`,
`16`/`17`=`CityStatusChangeSubtype::EconomicProsperity`/`EconomicDecline`,
`18`/`19`=`CityStatusChangeSubtype::CityBecomesActive`/`CityBecomesInactive`,
`20`/`21`=`CityStatusChangeSubtype::CityAppears`/`CityDisappears`, `23`=`CityStatusChangeSubtype::RebellionOver`,
`24`=`CityStatusChangeSubtype::CityConquered`. Other documented subtypes (tribute suspended/resumed,
rebellion start, "colony becomes rebellious", "god disaster over") weren't observed in any real
adventure surveyed and aren't produced.

Field notes:

- `CityStatusChange.city` is `fixed_target`/`min_target` (the fixed-or-range quantity pattern, resolved
  the same way as amounts) for every subtype except `CityConquered`, which instead uses `other_city`
  for the (constant, conquering) city and reuses `fixed_target`/`min_target` for the conquered city -
  confirmed against all 13 real events in `The Youngest Twins` episode 1's `MilitaryBuildup`/
  `CityConquered` events. `ally_city` is not used by any subtype observed; the earlier assumption that
  it held `CityConquered`'s second city, and that `other_city` held every other subtype's city, was
  wrong (`other_city` reads `0` for every non-`CityConquered` event actually observed).
- `resolve_range(fixed, min)` collapses `EventData`'s fixed-or-min/max-range quantity pattern (an event
  stores either a fixed value or a range, whichever half is unused is `u16::MAX`) to one representative
  value, since `Event` variants only have room for one `u16`/`u8` quantity - this drops the
  `max` bound on range-based events, an accepted simplification (`Event::to_data` still reproduces
  whatever it read, so it's not a round-trip bug).
- `CityAttackOutcome` is `eff_on_city`'s low byte; the high byte (a "warning stage" flag) isn't modeled.
- `MonsterAttack.monument` is `mtar1`'s high byte; `target` is built from `mtar1`'s low byte plus
  `mtar2`/`mtar3` (each a `MonsterTarget` id).
- `Disaster.permanent` is `unknown_5`'s high byte; `disaster_type` is encoded as `event_type` itself
  (each disaster kind is its own top-level type, not a shared type with a subtype), so
  `DisasterSubtype::value()` returns that raw `event_type` byte directly.
- `Occurrence` comes from `flags` bits `0`/`1` (`1`=triggered-only, `2`=recurring) plus a
  `fixed_time`/`min_time`/`max_time` range (same fixed-or-range pattern as above, but `BetweenYears`
  keeps both bounds since it has room for two `u16`s). Bit `0x20000` is `EpisodeComplete` (no
  month/`BetweenYears` payload of its own - the raw `month`/`fixed_time`/`min_time`/`max_time` bytes on
  a real `EpisodeComplete` event are a fixed sentinel, not meaningful data), confirmed against
  `The Youngest Twins`' colony events (`CityBecomesRival`/`CityBecomesInactive` events that trigger only
  once the episode is otherwise complete).
- `TriggerType` comes from `trig_reason` (one shared field, so every `EventToTrigger` built from a
  single event's `on_success`/`on_failure`/`trigger_on_1`/`trigger_on_2` shares the same type).
- Several fields look like map-editor template leftovers even on events that don't semantically use
  them (`first_item`, `fixed_target`, `source_min`/`source_max` show up with real, non-default values -
  e.g. recurring pairs like `(1, 8)` or `(0, 231)` - on events with no item/marker/source concept).
  Not yet modeled.

Not modeled at all (no `from_data` dispatch, `to_data` only): `Quest` (only one low-confidence real
sample; `MonumentReward`'s source for `Quest` specifically, and `QuestType`'s raw encoding, aren't
confirmed), `RivalArmyChange` (its raw subtype wasn't observed in any real adventure surveyed).

## Duplicate/dead raw fields (confirmed, not modeled)

- `SettingsData.map_data` duplicates `map_data.last()` byte-for-byte; confirmed dead in-game (case 4 in
  `docs/adventure_model_todo.md`) but `Adventure::to_pak` still reproduces it for round-trip fidelity.
- `basic_episode_data`/`real_episode_data`/`mythology` slots beyond an episode's real index are
  populated with real-looking, confirmed-unused data (case 3) - not reproduced.
- Map-level `mythology` gods/pyramids are confirmed unused (case 5); settings-level is authoritative
  and is what `Mythology` models.
