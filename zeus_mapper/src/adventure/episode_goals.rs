use crate::adventure::God;
use crate::adventure::resource::ResourceType;
use crate::constants::data_constant::DataConstant;
use crate::constants::data_constant::data_constants;
use crate::differ::default_differ_impl;
use crate::prelude::BoxedArray;
use crate::prelude::EpisodeGoalData;
use crate::prelude::EventData;

#[derive(PartialEq, Debug)]
pub enum EpisodeGoal {
    Population(u16),
    Treasury(u16),
    Sanctuary(God),
    Sanctuaries(u8),
    Army(UnitType, u32),
    Quest(u16),
    Slay(u16),
    YearlyProduction(ResourceType, u32),
    Rule(u8),
    YearlyProfit(u16),
    Housing(HouseLevel, u32),
    TradingPartners(u16),
    SetAsideGoods(ResourceType, u32),
    Pyramid(PyramidType),
    Pyramids(u8),
    Hippodrome(u16),
}

default_differ_impl!(EpisodeGoal);

impl EpisodeGoal {
    /// Reads goal slots from a `SettingsData.parent_episode_goals`/`colony_episode_goals`-shaped
    /// row: up to 6 `EpisodeGoalData` slots for one parent or colony episode, of which only
    /// populated (non-default) slots resolve to a goal.
    ///
    /// `new_file_ver` (`SettingsData.version_2 == 26`, the same flag `settings_data.rs` uses for
    /// other layout differences) selects which numbering scheme applies - see
    /// [`EpisodeGoal::from_raw_fields`]'s doc comment for the old-format one.
    ///
    /// **Assumptions**: new-format goal kinds were pinned down by diffing otherwise-byte-identical
    /// `.pak` files that only differ in one goal's fields (see `DATA_MAPPING.md` for
    /// `goal_type == 14`, `SetAsideGoods`), and by inspecting a single adventure with one goal of
    /// each other kind set via the in-game editor (weaker evidence than a diff round, but
    /// `resource_id`/`amount` matched the expected value exactly for every kind below). `Quest`'s
    /// payload comes from a linked event rather than the goal record itself - see
    /// [`resolve_quest`]'s doc comment.
    pub(crate) fn vec_from_episode_goal_data(
        slots: &[EpisodeGoalData],
        new_file_ver: bool,
        events: &BoxedArray<EventData, 150>,
    ) -> Vec<EpisodeGoal> {
        return slots
            .iter()
            .filter_map(|slot| EpisodeGoal::from_raw_fields(slot.goal_type, slot.resource_id, slot.amount, slot, new_file_ver, events))
            .collect();
    }

    /// Dispatches on `goal_type` - the same marker numbering for both file generations (see the
    /// table in DATA_MAPPING.md). Resource ids resolve through `ResourceType::try_resolve_for_format`,
    /// since old-format adventures use a narrower id table for real `ResourceType`s;
    /// `Pyramid`/`Pyramids`/`Hippodrome` (`goal_type`s `15`/`16`) have no confirmed old-format
    /// examples at all, so they're only attempted for new-format data.
    fn from_raw_fields(
        marker: u32,
        resource_id: i32,
        amount: u32,
        slot: &EpisodeGoalData,
        new_file_ver: bool,
        events: &BoxedArray<EventData, 150>,
    ) -> Option<EpisodeGoal> {
        return match marker {
            0 if resource_id > 0 => Some(EpisodeGoal::Population(resource_id as u16)),
            1 if resource_id > 0 => Some(EpisodeGoal::Treasury(resource_id as u16)),
            // Old-format only: an unspecified count of *any* god's sanctuary, with the count in
            // `unknown_1[0]` rather than `amount` - confirmed against `#Zeus and Europa.pak`, where
            // `unknown_1[0] == 2` matches a real, user-confirmed `Sanctuaries(2)`. No confirmed
            // new-format example exists, so this shape is only attempted for old-format data.
            2 if !new_file_ver && resource_id < 0 => Some(EpisodeGoal::Sanctuaries(u32_at(slot.unknown_1.as_slice(), 0) as u8)),
            2 => God::try_resolve(&(resource_id as u32)).map(EpisodeGoal::Sanctuary),
            3 => UnitType::try_resolve(&(resource_id as u32)).map(|unit| EpisodeGoal::Army(unit, amount)),
            4 => resolve_quest(resource_id, events), // todo this implies resource_id should be called something else
            5 => resolve_slay(resource_id, events),  // todo this implies resource_id should be called something else
            6 => ResourceType::try_resolve_for_format(&(resource_id as i8), new_file_ver)
                .map(|resource| EpisodeGoal::YearlyProduction(resource, amount)),
            7 if (0..=u8::MAX as i32).contains(&resource_id) => Some(EpisodeGoal::Rule(resource_id as u8)),
            8 if resource_id > 0 => Some(EpisodeGoal::YearlyProfit(resource_id as u16)),
            9 => HouseLevel::try_resolve(&(resource_id as u32)).map(|level| EpisodeGoal::Housing(level, amount)),
            10 if resource_id > 0 => Some(EpisodeGoal::TradingPartners(resource_id as u16)),
            14 if amount > 0 && amount < 256 => ResourceType::try_resolve_for_format(&(resource_id as i8), new_file_ver)
                .map(|resource| EpisodeGoal::SetAsideGoods(resource, amount)),
            15 if new_file_ver && amount == 0 => Some(EpisodeGoal::Pyramids(u32_at(slot.unknown_1.as_slice(), 4) as u8)),
            15 if new_file_ver => PyramidType::try_resolve(&amount).map(EpisodeGoal::Pyramid),
            16 if new_file_ver && resource_id > 0 => Some(EpisodeGoal::Hippodrome(resource_id as u16)),
            _ => None,
        };
    }

    /// Inverse of [`EpisodeGoal::vec_from_episode_goal_data`]: encodes up to 6 goals into a
    /// `parent_episode_goals`/`colony_episode_goals` row, dropping any beyond the 6th and any kind
    /// [`EpisodeGoal::to_raw_fields`] can't encode.
    ///
    /// Always writes the new file format - only reading needs to support the old format.
    ///
    /// Returns the row alongside how many slots were actually populated - the source for
    /// `parent_episode_goal_counts[i]` on the parent side (the colony side has no equivalent count
    /// field, see `DATA_MAPPING.md`).
    pub(crate) fn vec_to_episode_goal_data(goals: &[EpisodeGoal]) -> ([EpisodeGoalData; 6], u32) {
        let mut slots: [EpisodeGoalData; 6] = Default::default();
        let mut count = 0;

        for goal_data in goals.iter().filter_map(|goal| goal.to_raw_fields()) {
            if count >= slots.len() {
                break;
            }
            slots[count] = goal_data;
            count += 1;
        }

        return (slots, count as u32);
    }

    fn to_raw_fields(&self) -> Option<EpisodeGoalData> {
        let (goal_type, resource_id, amount, unknown_1) = match self {
            EpisodeGoal::Population(count) => (0, *count as i32, 0, Default::default()),
            EpisodeGoal::Treasury(amount) => (1, *amount as i32, 0, Default::default()),
            EpisodeGoal::Sanctuary(god) => (2, god.value() as i32, 0, Default::default()),
            EpisodeGoal::Army(unit, count) => (3, unit.value() as i32, *count, Default::default()),
            EpisodeGoal::Quest(event_id) => (4, *event_id as i32, 0, Default::default()),
            EpisodeGoal::Slay(event_id) => (5, *event_id as i32, 0, Default::default()),
            EpisodeGoal::YearlyProduction(resource, amount) => (6, resource.value() as i32, *amount, Default::default()),
            EpisodeGoal::Rule(location) => (7, *location as i32, 0, Default::default()),
            EpisodeGoal::YearlyProfit(amount) => (8, *amount as i32, 0, Default::default()),
            EpisodeGoal::Housing(level, count) => (9, level.value() as i32, *count, Default::default()),
            EpisodeGoal::TradingPartners(count) => (10, *count as i32, 0, Default::default()),
            EpisodeGoal::SetAsideGoods(resource, amount) => (14, resource.value() as i32, *amount, Default::default()),
            EpisodeGoal::Pyramids(count) => (15, 1, 0, pyramids_field_4(*count)),
            EpisodeGoal::Pyramid(kind) => (15, 1, kind.value(), Default::default()),
            EpisodeGoal::Hippodrome(count) => (16, *count as i32, 0, Default::default()),
            // No confirmed new-format encoding exists for `Sanctuaries` (see `from_raw_fields`) -
            // `Adventure::to_pak` always writes the new format, so there's nowhere to put it.
            EpisodeGoal::Sanctuaries(_) => return None,
        };

        return Some(EpisodeGoalData {
            goal_type,
            resource_id,
            amount,
            unknown_1,
        });
    }
}

/// `Quest`'s payload, like old-format `Slay`'s, isn't in the goal record at all: `resource_id` is
/// the index of a `Quest` event (`event_type == 4`) in this same episode/colony's own event list,
/// which `EpisodeGoal::Quest` carries as-is rather than resolving eagerly - the linked `Event::Quest`
/// already carries the resolved god/quest-type.
///
/// Confirmed against `&Perseus and Medusa.pak` (two parent-episode quests, one colony quest) and
/// `@Athens through the Ages.pak` (one parent-episode quest).
fn resolve_quest(goal_index: i32, events: &BoxedArray<EventData, 150>) -> Option<EpisodeGoal> {
    let event = events.get(goal_index as usize)?;
    if event.event_type != 4 {
        return None;
    }

    return Some(EpisodeGoal::Quest(goal_index as u16));
}

/// Old-format `Slay`'s payload isn't in the goal record at all: `goal_type` is the index of a
/// `MonsterInvasion` event (`event_type == 26`) in this same episode/colony's own event list -
/// looked up directly, bypassing `parent_event_counts`/`colony_event_counts` (real `Slay`-linked
/// events have been observed sitting past what that count considers "real") - which
/// `EpisodeGoal::Slay` carries as-is rather than resolving the linked event's monster eagerly.
///
/// Confirmed against `The Odyssey`: every parent episode and colony with a real `Slay` goal has
/// `goal_type` landing exactly on its own `MonsterInvasion` event.
fn resolve_slay(goal_type: i32, events: &BoxedArray<EventData, 150>) -> Option<EpisodeGoal> {
    let event = events.get(goal_type as usize)?;
    if event.event_type != 26 {
        return None;
    }

    return Some(EpisodeGoal::Slay(goal_type as u16));
}

// fixme this is a code smell - we should be using named fields, if we're using an unnamed field it means it should be promoted to named
fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    return u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
}

/// `Pyramids(count)`'s raw encoding: `resource_id`/`amount` are both otherwise-unused for this
/// goal (`resource_id` a constant `1`, `amount` `0`, matching the one real `Pyramids` goal seen so
/// far), and `count` itself lives at byte offset 4 (the second `u32`) of `unknown_1`, preceded by a
/// `u32::MAX` sentinel at offset 0 whose purpose isn't confirmed but is reproduced here to match
/// the one real record observed.
fn pyramids_field_4(count: u8) -> BoxedArray<u8, 64> {
    let mut bytes = u32::MAX.to_le_bytes().to_vec();
    bytes.extend((count as u32).to_le_bytes());
    return BoxedArray::from_vec(bytes);
}

#[derive(PartialEq, Debug)]
pub enum QuestType {
    Type0,
    Type1,
}

data_constants!(PyramidType<u32> {
    ModestPyramid = 100,
    Pyramid = 101,
    GreatPyramid = 102,
    MajesticPyramid = 103,
    SmallMonumentToTheSky = 104,
    MonumentToTheSky = 105,
    GrantMonumentToTheSky = 106,
    MinorShrine = 107,
    Shrine = 108,
    MajorShrine = 109,
    PyramidOfThePantheon = 110,
    AltarOfOlympus = 111,
    TempleOfOlympus = 112,
    ObservatoryKosmika = 113,
    MuseumAtlantika = 114,
});

data_constants!(UnitType<u32> {
    Untrained = 11,
    Infantry = 12,
    Cavalary = 13,
    Warship = 57,
});

data_constants!(HouseLevel<u32> {
    Hut = 2,
    Shack = 3,
    Hovel = 4,
    Homestead = 5,
    Tenement = 6,
    Apartment = 7,
    Townhouse = 8,
    Vacant = 9,
    Devolved = 10,
    Residence = 11,
    Mansion = 12,
    Manor = 13,
    Estate = 14,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::adventure::Adventure;
    use crate::prelude::AdventureText;
    use crate::prelude::PakData;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;
    use std::path::Path;

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.episode_goals, vec![EpisodeGoal::SetAsideGoods(ResourceType::Fleece, 8)]);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.episode_goals, vec![]);

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(
                colony_episode.episode_goals,
                vec![EpisodeGoal::SetAsideGoods(ResourceType::Orichalc, 8)]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.episode_goals, vec![EpisodeGoal::Housing(HouseLevel::Homestead, 800)]);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.episode_goals,
                vec![
                    EpisodeGoal::Rule(9), // Ismarus
                    EpisodeGoal::Population(1500),
                    EpisodeGoal::SetAsideGoods(ResourceType::Fish, 32),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 32),
                    EpisodeGoal::Treasury(5000),
                    EpisodeGoal::Sanctuary(God::Hermes)
                ]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.episode_goals,
                vec![
                    EpisodeGoal::Housing(HouseLevel::Manor, 80),
                    EpisodeGoal::YearlyProduction(ResourceType::Wine, 16),
                    EpisodeGoal::SetAsideGoods(ResourceType::Fish, 32),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 32),
                    EpisodeGoal::Treasury(5000),
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.episode_goals,
                vec![
                    EpisodeGoal::Sanctuary(God::Athena),
                    EpisodeGoal::Slay(9),
                    EpisodeGoal::Army(UnitType::Warship, 4)
                ],
            );

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(
                episode_5.episode_goals,
                vec![
                    EpisodeGoal::Slay(7),
                    EpisodeGoal::Rule(6), // Same
                    EpisodeGoal::Rule(7), // Olenus
                    EpisodeGoal::Sanctuary(God::Zeus)
                ]
            );

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.episode_goals,
                vec![
                    EpisodeGoal::Sanctuary(God::Hephaestus),
                    EpisodeGoal::Slay(2),
                    EpisodeGoal::YearlyProduction(ResourceType::Fleece, 24),
                    EpisodeGoal::YearlyProduction(ResourceType::Bronze, 10),
                    EpisodeGoal::Housing(HouseLevel::Townhouse, 1000)
                ]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.episode_goals,
                vec![
                    EpisodeGoal::Slay(6),
                    EpisodeGoal::Housing(HouseLevel::Tenement, 1000),
                    EpisodeGoal::Army(UnitType::Warship, 2)
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_open_play_economic_2() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/Open Play Economic 2"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.episode_goals, vec![EpisodeGoal::SetAsideGoods(ResourceType::Wheat, 16),]);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.episode_goals, vec![EpisodeGoal::SetAsideGoods(ResourceType::Wheat, 16),]);

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(episode_3.episode_goals, vec![]);

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.episode_goals,
                vec![EpisodeGoal::SetAsideGoods(ResourceType::Wood, 16)]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.episode_goals,
                vec![
                    EpisodeGoal::SetAsideGoods(ResourceType::Fleece, 16),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 16)
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_open_play_military_2() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/Open Play Military 2"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.episode_goals, vec![EpisodeGoal::SetAsideGoods(ResourceType::Fleece, 12),]);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.episode_goals, vec![]);

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(
                colony_episode.episode_goals,
                vec![
                    EpisodeGoal::SetAsideGoods(ResourceType::Wheat, 12),
                    EpisodeGoal::SetAsideGoods(ResourceType::Bronze, 12),
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_open_play_sandbox_2() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/Open Play Sandbox 2"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.episode_goals, vec![]);
        }

        return Ok(());
    }

    #[test]
    fn parse_zeus_and_europa() {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = adventure_from_pak(format!("{game_root}/Adventures/#Zeus and Europa.pak"));

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.episode_goals, vec![EpisodeGoal::Population(1000)]);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.episode_goals,
                vec![EpisodeGoal::YearlyProduction(ResourceType::Marble, 48)]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.episode_goals,
                vec![
                    EpisodeGoal::Sanctuary(God::Ares),
                    EpisodeGoal::Housing(HouseLevel::Residence, 50),
                    EpisodeGoal::SetAsideGoods(ResourceType::Wheat, 30),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 25),
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(episode_4.episode_goals, vec![EpisodeGoal::Slay(0), EpisodeGoal::Sanctuaries(2),]);

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(
                episode_5.episode_goals,
                vec![
                    EpisodeGoal::Rule(8), // Mt. Cithaeron
                    EpisodeGoal::Sanctuaries(3),
                ]
            );

            let episode_6 = adventure.parent_episodes.get(5).expect("Episode 6");
            assert_eq!(
                episode_6.episode_goals,
                vec![
                    EpisodeGoal::TradingPartners(11),
                    EpisodeGoal::Slay(17),
                    EpisodeGoal::YearlyProfit(2000),
                    EpisodeGoal::Population(4000)
                ]
            );

            let episode_7 = adventure.parent_episodes.get(6).expect("Episode 7");
            assert_eq!(
                episode_7.episode_goals,
                vec![
                    EpisodeGoal::Rule(7), // Calydon
                    EpisodeGoal::Rule(2), // Argos
                    EpisodeGoal::Rule(6), // Orchomenos
                    EpisodeGoal::Sanctuaries(4),
                    EpisodeGoal::Housing(HouseLevel::Estate, 100),
                ]
            );

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(
                colony_episode.episode_goals,
                vec![
                    EpisodeGoal::SetAsideGoods(ResourceType::Wood, 30),
                    EpisodeGoal::SetAsideGoods(ResourceType::Wine, 30),
                ]
            );
        }
    }

    #[test]
    fn parse_perseus_and_medusa() {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = adventure_from_pak(format!("{game_root}/Adventures/&Perseus and Medusa.pak"));

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.episode_goals,
                vec![
                    EpisodeGoal::Housing(HouseLevel::Homestead, 500),
                    EpisodeGoal::YearlyProduction(ResourceType::Fleece, 20),
                    EpisodeGoal::YearlyProfit(500),
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.episode_goals,
                vec![
                    EpisodeGoal::Housing(HouseLevel::Tenement, 640),
                    EpisodeGoal::SetAsideGoods(ResourceType::Marble, 48),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 12),
                ]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.episode_goals,
                vec![
                    EpisodeGoal::Quest(0),
                    EpisodeGoal::Quest(1),
                    EpisodeGoal::SetAsideGoods(ResourceType::Marble, 48),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 24),
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.episode_goals,
                vec![
                    EpisodeGoal::Sanctuaries(4),
                    EpisodeGoal::Population(4500),
                    EpisodeGoal::Treasury(10_000),
                    EpisodeGoal::Army(UnitType::Cavalary, 16),
                    EpisodeGoal::YearlyProduction(ResourceType::OliveOil, 32),
                    EpisodeGoal::Slay(0)
                ]
            );

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.episode_goals,
                vec![
                    EpisodeGoal::Slay(0),
                    EpisodeGoal::Sanctuary(God::Hermes),
                    EpisodeGoal::YearlyProfit(500),
                    EpisodeGoal::SetAsideGoods(ResourceType::Sculpture, 8),
                ]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.episode_goals,
                vec![
                    EpisodeGoal::Slay(0),
                    EpisodeGoal::Quest(1),
                    EpisodeGoal::Rule(7), // Ethiopia
                    EpisodeGoal::SetAsideGoods(ResourceType::Wheat, 16),
                    EpisodeGoal::SetAsideGoods(ResourceType::Wood, 24),
                    EpisodeGoal::SetAsideGoods(ResourceType::Bronze, 8),
                ]
            );
        }
    }

    #[test]
    fn parse_athens_through_the_ages() {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = adventure_from_pak(format!("{game_root}/Adventures/@Athens through the Ages.pak"));

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.episode_goals,
                vec![
                    EpisodeGoal::SetAsideGoods(ResourceType::Fleece, 12),
                    EpisodeGoal::SetAsideGoods(ResourceType::Fish, 12),
                    EpisodeGoal::Housing(HouseLevel::Homestead, 500),
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.episode_goals,
                vec![
                    EpisodeGoal::Rule(6), // Knossos
                    EpisodeGoal::Army(UnitType::Warship, 1),
                    EpisodeGoal::Army(UnitType::Infantry, 4),
                    EpisodeGoal::Housing(HouseLevel::Mansion, 10),
                ]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.episode_goals,
                vec![
                    EpisodeGoal::Rule(12), // Sardis
                    EpisodeGoal::Rule(7),  // Delos
                    EpisodeGoal::Army(UnitType::Warship, 3),
                    EpisodeGoal::Housing(HouseLevel::Homestead, 960),
                    EpisodeGoal::Housing(HouseLevel::Tenement, 480),
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.episode_goals,
                vec![
                    EpisodeGoal::Rule(4),  // Marathon
                    EpisodeGoal::Rule(21), // Hattusas
                    EpisodeGoal::Rule(10), // Eretria
                ]
            );

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(
                episode_5.episode_goals,
                vec![
                    EpisodeGoal::Rule(14), // Sparta
                    EpisodeGoal::Rule(10), // Eretria
                    EpisodeGoal::Rule(18), // Corinth
                    EpisodeGoal::Slay(0),
                    EpisodeGoal::Sanctuary(God::Apollo),
                    EpisodeGoal::Housing(HouseLevel::Tenement, 960),
                ]
            );

            let episode_6 = adventure.parent_episodes.get(5).expect("Episode 6");
            assert_eq!(
                episode_6.episode_goals,
                vec![
                    EpisodeGoal::Sanctuaries(3),
                    EpisodeGoal::Population(5000),
                    EpisodeGoal::Housing(HouseLevel::Estate, 200),
                    EpisodeGoal::Housing(HouseLevel::Apartment, 2400),
                    EpisodeGoal::YearlyProfit(2500),
                    EpisodeGoal::Quest(0)
                ]
            );

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.episode_goals,
                vec![
                    EpisodeGoal::Treasury(7500),
                    EpisodeGoal::YearlyProfit(1500),
                    EpisodeGoal::SetAsideGoods(ResourceType::Bronze, 8)
                ]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.episode_goals,
                vec![
                    EpisodeGoal::YearlyProduction(ResourceType::Bronze, 12),
                    EpisodeGoal::SetAsideGoods(ResourceType::Bronze, 16),
                    EpisodeGoal::YearlyProduction(ResourceType::Wood, 16),
                    EpisodeGoal::SetAsideGoods(ResourceType::Wood, 24),
                    EpisodeGoal::YearlyProduction(ResourceType::Marble, 24),
                    EpisodeGoal::SetAsideGoods(ResourceType::Marble, 32),
                ]
            );
        }
    }

    #[test]
    fn parse_the_birth_of_atlantis() {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = adventure_from_pak(format!("{game_root}/Adventures/#The Birth of Atlantis.pak"));

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.episode_goals,
                vec![EpisodeGoal::Population(1000), EpisodeGoal::Housing(HouseLevel::Homestead, 500)]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.episode_goals,
                vec![
                    EpisodeGoal::Slay(0),
                    EpisodeGoal::Population(1500),
                    EpisodeGoal::Housing(HouseLevel::Apartment, 1000),
                    EpisodeGoal::Pyramid(PyramidType::ModestPyramid),
                ]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.episode_goals,
                vec![
                    EpisodeGoal::Sanctuary(God::Artemis),
                    EpisodeGoal::Quest(0),
                    EpisodeGoal::Population(2000),
                    EpisodeGoal::Housing(HouseLevel::Townhouse, 1500)
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.episode_goals,
                vec![
                    EpisodeGoal::Rule(9),  // Cennia
                    EpisodeGoal::Rule(10), // Mt Geimm
                    EpisodeGoal::Rule(11), // Mt Phaesus
                ]
            );

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(
                episode_5.episode_goals,
                vec![
                    EpisodeGoal::Quest(0),
                    EpisodeGoal::Quest(1),
                    EpisodeGoal::Population(3000),
                    EpisodeGoal::Housing(HouseLevel::Mansion, 100)
                ]
            );

            let episode_6 = adventure.parent_episodes.get(5).expect("Episode 6");
            assert_eq!(episode_6.episode_goals, vec![EpisodeGoal::Sanctuary(God::Poseidon)]);

            let episode_7 = adventure.parent_episodes.get(6).expect("Episode 7");
            assert_eq!(
                episode_7.episode_goals,
                vec![
                    EpisodeGoal::Quest(0),
                    EpisodeGoal::Population(5000),
                    EpisodeGoal::Pyramid(PyramidType::PyramidOfThePantheon),
                    EpisodeGoal::Hippodrome(40),
                ]
            );

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.episode_goals,
                vec![EpisodeGoal::TradingPartners(9), EpisodeGoal::Housing(HouseLevel::Estate, 60)]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.episode_goals,
                vec![
                    EpisodeGoal::Slay(0),
                    EpisodeGoal::SetAsideGoods(ResourceType::Orichalc, 32),
                    EpisodeGoal::SetAsideGoods(ResourceType::Bronze, 32)
                ]
            );
        }
    }

    #[test]
    fn parse_enlightenment_in_the_west() {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = adventure_from_pak(format!("{game_root}/Adventures/&Enlightenment in the West.pak"));

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.episode_goals,
                vec![
                    EpisodeGoal::SetAsideGoods(ResourceType::Marble, 48),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 20),
                    EpisodeGoal::Housing(HouseLevel::Townhouse, 1200)
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.episode_goals,
                vec![
                    EpisodeGoal::Sanctuary(God::Demeter),
                    EpisodeGoal::Sanctuary(God::Athena),
                    EpisodeGoal::Sanctuary(God::Dionysus),
                    EpisodeGoal::SetAsideGoods(ResourceType::Marble, 48),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 20)
                ]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.episode_goals,
                vec![
                    EpisodeGoal::Quest(0),
                    EpisodeGoal::Pyramid(PyramidType::MuseumAtlantika),
                    EpisodeGoal::YearlyProfit(6000),
                    EpisodeGoal::SetAsideGoods(ResourceType::Marble, 48),
                    EpisodeGoal::SetAsideGoods(ResourceType::OliveOil, 20)
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.episode_goals,
                vec![
                    EpisodeGoal::Rule(7), // Diapraea
                    EpisodeGoal::Rule(6), // Azium
                    EpisodeGoal::Population(4000),
                    EpisodeGoal::Pyramid(PyramidType::TempleOfOlympus),
                ]
            );

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.episode_goals,
                vec![
                    EpisodeGoal::Sanctuary(God::Aphrodite),
                    EpisodeGoal::Population(2000),
                    EpisodeGoal::Quest(0)
                ]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.episode_goals,
                vec![EpisodeGoal::Slay(0), EpisodeGoal::YearlyProduction(ResourceType::Orichalc, 64)]
            );

            let colony_episode_3 = adventure.colony_episodes.get(2).expect("Colony episode 3");
            assert_eq!(
                colony_episode_3.episode_goals,
                vec![
                    EpisodeGoal::Pyramid(PyramidType::SmallMonumentToTheSky),
                    EpisodeGoal::Pyramid(PyramidType::MonumentToTheSky),
                    EpisodeGoal::Pyramid(PyramidType::GrantMonumentToTheSky)
                ]
            );

            let colony_episode_4 = adventure.colony_episodes.get(3).expect("Colony episode 4");
            assert_eq!(
                colony_episode_4.episode_goals,
                vec![
                    EpisodeGoal::Pyramid(PyramidType::ObservatoryKosmika),
                    EpisodeGoal::Housing(HouseLevel::Townhouse, 1000)
                ]
            );
        }
    }

    fn adventure_from_pak(path: impl AsRef<Path>) -> Adventure {
        let mut reader = BufReader::new(File::open(path).expect("pak file"));
        let pak_data = PakData::read_from(&mut reader).expect("valid pak data");

        return Adventure::from_pak(&pak_data, &AdventureText::default());
    }
}
