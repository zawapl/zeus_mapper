use crate::adventure::CityId;
use crate::adventure::God;
use crate::adventure::episode_goals::QuestType;
use crate::adventure::resource::ResourceType;
use crate::constants::data_constant::data_constants;
use crate::differ::default_differ_impl;
use crate::prelude::BoxedArray;
use crate::prelude::DataConstant;
use crate::prelude::EventData;
use crate::prelude::UnconfirmedSign;

pub type WarningMonths = u8;

#[derive(PartialEq, Debug)]
pub enum Event {
    GoodsRequest(GoodsRequest),
    MilitaryRequest(MilitaryRequest),
    Gift(Gift),
    Quest(Quest),
    Invasion(Invasion),
    MonsterInvasion(MonsterInvasion),
    GodInvasion(GodInvasion),
    Disaster(Disaster),
    WageIncrease(WageIncrease),
    WageDecrease(WageDecrease),
    TradeChange(TradeChange),
    CityStatusChange(CityStatusChange),
    RivalArmyChange(RivalArmyChange),
}

impl Event {
    /// The slot each `Event` occupies is derived from its position in `episode_events`, not
    /// stored on `Event` itself; see `vec_to_data` for the inverse.
    ///
    /// **Assumptions**: real events are densely packed starting at slot `0` (unlike most other
    /// dense arrays in this format, slot `0` is a real event, not a sentinel/always-unused slot);
    /// `count` is the episode's real event count, sourced from `SettingsData.parent_event_counts`/
    /// `colony_event_counts`.
    /// Slots at and beyond `count` hold real-looking but confirmed-unused editor template data,
    /// the same "extra populated slots" pattern documented for `basic_episode_data`/
    /// `real_episode_data` in `DATA_MAPPING.md` - `id` alone can't distinguish real from unused
    /// since it always mirrors its own slot index either way.
    pub(crate) fn vec_from_data(episode_events: &BoxedArray<EventData, 150>, count: usize, new_file_ver: bool) -> Vec<Event> {
        return episode_events
            .iter()
            .take(count)
            .map(|event| Event::from_data(event, new_file_ver))
            .collect();
    }

    pub(crate) fn vec_to_data(events: &[Event]) -> BoxedArray<EventData, 150> {
        let mut data = vec![EventData::default(); 150];

        for (slot, event) in events.iter().enumerate() {
            if slot < data.len() {
                data[slot] = event.to_data(slot);
            }
        }

        return BoxedArray::from_vec(data);
    }

    // Dispatch table documented in DATA_MAPPING.md; each arm delegates entirely to the relevant
    // type's own `from_data`, which fills in a default for whatever it can't decode. Falls back to
    // `GoodsRequest` for any `event_type`/`subtype` combination this doesn't recognize at all.
    // TODO: defaults are provisional, to be revisited once more real adventures have been surveyed.
    fn from_data(event: &EventData, new_file_ver: bool) -> Event {
        // `subtype` `1`/`2` is shared with some other, still-unidentified `event_type == 1` kind
        // that leaves `fixed_target` unset (a `min_target`/`max_target` range instead) - a real
        // `CityUnderAttack`/`CityAttacksRival` always names one specific city, so gate on that too.
        let is_city_attack = matches!(event.subtype, 1 | 2) && event.fixed_target >= 0;
        if event.event_type == 1 && (event.subtype == 7 || is_city_attack) {
            return Event::MilitaryRequest(MilitaryRequest::from_data(event));
        }

        return match event.event_type {
            2 => Event::Invasion(Invasion::from_data(event)),
            4 => Event::Quest(Quest::from_data(event)),
            3 | 5 | 24 | 25 | 28 => Event::Disaster(Disaster::from_data(event)),
            8 => Event::WageIncrease(WageIncrease::from_data(event)),
            9 => Event::WageDecrease(WageDecrease::from_data(event)),
            13 | 14 | 15 | 16 | 21 | 22 => Event::TradeChange(TradeChange::from_data(event)),
            19 if matches!(event.subtype, 2 | 3) => Event::TradeChange(TradeChange::from_data(event)),
            19 => Event::CityStatusChange(CityStatusChange::from_data(event)),
            23 => Event::Gift(Gift::from_data(event, new_file_ver)),
            26 => Event::MonsterInvasion(MonsterInvasion::from_data(event)),
            27 => Event::GodInvasion(GodInvasion::from_data(event)),
            _ => Event::GoodsRequest(GoodsRequest::from_data(event)),
        };
    }

    fn to_data(&self, slot: usize) -> EventData {
        let mut event = match self {
            Event::GoodsRequest(request) => request.to_data(),
            Event::MilitaryRequest(request) => request.to_data(),
            Event::Gift(gift) => gift.to_data(),
            Event::Quest(quest) => quest.to_data(),
            Event::Invasion(invasion) => invasion.to_data(),
            Event::MonsterInvasion(invasion) => invasion.to_data(),
            Event::GodInvasion(invasion) => invasion.to_data(),
            Event::Disaster(disaster) => disaster.to_data(),
            Event::WageIncrease(wage_increase) => wage_increase.to_data(),
            Event::WageDecrease(wage_decrease) => wage_decrease.to_data(),
            Event::TradeChange(change) => change.to_data(),
            Event::CityStatusChange(change) => change.to_data(),
            Event::RivalArmyChange(change) => change.to_data(),
        };
        event.id = slot as u16;

        return event;
    }
}

default_differ_impl!(Event);

// Collapses a fixed-or-range quantity to one representative value; drops the range's `max` bound
// (see DATA_MAPPING.md) since `Event` variants only have room for one quantity.
fn resolve_range(fixed: i16, min: i16) -> u16 {
    return if fixed != -1 { fixed as u16 } else { min as u16 };
}

// Stamps `occurrence`'s month/flags/time range onto `event`; used by the various `to_data`
// methods for every event variant that carries an `Occurrence` (i.e. all but
// `MonsterInvasionSubtype::MonsterInCity`).
fn with_occurrence(mut event: EventData, occurrence: &Occurrence) -> EventData {
    let (month, occurrence_flags, fixed_time, min_time, max_time) = occurrence.to_data();
    event.month = month;
    event.flags |= occurrence_flags;
    event.fixed_time = fixed_time;
    event.min_time = min_time;
    event.max_time = max_time;

    return event;
}

pub type Month = u16;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Occurrence {
    OneTime(Month, BetweenYears),
    Repeating(Month, BetweenYears),
    Triggered(BetweenYears),
    EpisodeComplete,
}

default_differ_impl!(Occurrence);

impl Occurrence {
    /// Parses the occurrence. `event.warnings` is never folded in here - it's a separate
    /// concept (how far ahead the event is telegraphed to the player), not part of when the
    /// event actually fires.
    fn from_data(event: &EventData) -> Occurrence {
        let between_years = BetweenYears::from_data(event.fixed_time, event.min_time, event.max_time);

        if event.flags & 0x1 != 0 {
            return Occurrence::Triggered(between_years);
        }
        if event.flags & 0x2 != 0 {
            return Occurrence::Repeating(event.month as Month, between_years);
        }
        if event.flags & 0x20000 != 0 {
            return Occurrence::EpisodeComplete;
        }
        return Occurrence::OneTime(event.month as Month, between_years);
    }

    /// Returns `(month, flags, fixed_time, min_time, max_time)`.
    fn to_data(self) -> (u8, u32, i16, i16, i16) {
        let (month, between_years, flags) = match self {
            Occurrence::OneTime(month, between_years) => (month, between_years, 0u32),
            Occurrence::Repeating(month, between_years) => (month, between_years, 0x2),
            Occurrence::Triggered(between_years) => (0, between_years, 0x1),
            Occurrence::EpisodeComplete => (0, BetweenYears(1, 1), 0x20000),
        };
        let (fixed_time, min_time, max_time) = between_years.to_data();

        return (month as u8, flags, fixed_time, min_time, max_time);
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct BetweenYears(pub u16, pub u16);

impl BetweenYears {
    fn from_data(fixed: i16, min: i16, max: i16) -> BetweenYears {
        return if fixed != -1 {
            BetweenYears(fixed as u16, fixed as u16)
        } else {
            BetweenYears(min as u16, max as u16)
        };
    }

    /// Returns `(fixed_time, min_time, max_time)`.
    fn to_data(self) -> (i16, i16, i16) {
        if self.0 == self.1 {
            return (self.0 as i16, -1, -1);
        }
        return (-1, self.0 as i16, self.1 as i16);
    }
}

#[derive(PartialEq, Debug)]
pub struct GoodsRequest {
    pub subtype: GoodsRequestSubtype,
    pub city: CityId,
    pub amount: u16,
    pub warning_months: WarningMonths,
    pub occurrence: Occurrence,
}

default_differ_impl!(GoodsRequest);

impl GoodsRequest {
    fn from_data(event: &EventData) -> GoodsRequest {
        let resource = ResourceType::try_resolve(&(event.first_item as i8)).unwrap_or(ResourceType::Urchin);

        let subtype = match event.subtype {
            3 => {
                let god = God::try_resolve(&(*event.god_or_mon_or_warship_id as u32)).unwrap_or(God::Zeus);
                GoodsRequestSubtype::Festival(resource, god)
            }
            4 => GoodsRequestSubtype::Construction(resource),
            5 => GoodsRequestSubtype::Famine,
            6 => GoodsRequestSubtype::FinancialWoes,
            // Covers `0` (`GeneralRequest`'s real encoding) and any unrecognized subtype.
            _ => GoodsRequestSubtype::GeneralRequest(resource),
        };

        return GoodsRequest {
            subtype,
            city: event.other_city as u16,
            amount: resolve_range(event.fixed_amount, event.min_amount),
            warning_months: event.warnings as u8,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = match &self.subtype {
            GoodsRequestSubtype::GeneralRequest(resource) => EventData {
                subtype: 0,
                first_item: resource.value() as i16,
                ..EventData::default()
            },
            GoodsRequestSubtype::Festival(resource, god) => EventData {
                subtype: 3,
                first_item: resource.value() as i16,
                god_or_mon_or_warship_id: UnconfirmedSign(god.value() as u16),
                ..EventData::default()
            },
            GoodsRequestSubtype::Construction(resource) => EventData {
                subtype: 4,
                first_item: resource.value() as i16,
                ..EventData::default()
            },
            GoodsRequestSubtype::Famine => EventData {
                subtype: 5,
                ..EventData::default()
            },
            GoodsRequestSubtype::FinancialWoes => EventData {
                subtype: 6,
                ..EventData::default()
            },
        };
        data.event_type = 1;
        data.other_city = self.city as u8;
        data.fixed_amount = self.amount as i16;
        data.warnings = self.warning_months as i16;

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub enum GoodsRequestSubtype {
    GeneralRequest(ResourceType),
    Festival(ResourceType, God),
    Construction(ResourceType),
    Famine,
    FinancialWoes,
}

#[derive(PartialEq, Debug)]
pub struct MilitaryRequest {
    pub subtype: MilitaryRequestSubtype,
    pub city: CityId,
    pub outcome: CityAttackOutcome,
    pub warning_months: WarningMonths,
    pub occurrence: Occurrence,
}

default_differ_impl!(MilitaryRequest);

impl MilitaryRequest {
    fn from_data(event: &EventData) -> MilitaryRequest {
        let outcome = CityAttackOutcome::try_resolve(&(*event.eff_on_city & 0xFF)).unwrap_or(CityAttackOutcome::Unaffected);

        let subtype = match event.subtype {
            1 => MilitaryRequestSubtype::CityUnderAttack(event.other_city as u16),
            2 => MilitaryRequestSubtype::CityAttacksRival(event.other_city as u16),
            // Covers `7` and any unrecognized subtype.
            _ => MilitaryRequestSubtype::GreekCityTerrorized(*event.god_or_mon_or_warship_id),
        };

        return MilitaryRequest {
            subtype,
            city: resolve_range(event.fixed_target, event.min_target),
            outcome,
            warning_months: event.warnings as u8,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = match &self.subtype {
            MilitaryRequestSubtype::CityUnderAttack(second_city) => EventData {
                subtype: 1,
                other_city: *second_city as u8,
                ..EventData::default()
            },
            MilitaryRequestSubtype::CityAttacksRival(second_city) => EventData {
                subtype: 2,
                other_city: *second_city as u8,
                ..EventData::default()
            },
            MilitaryRequestSubtype::GreekCityTerrorized(monster_index) => EventData {
                subtype: 7,
                god_or_mon_or_warship_id: UnconfirmedSign(*monster_index),
                ..EventData::default()
            },
        };
        data.event_type = 1;
        data.fixed_target = self.city as i16;
        data.eff_on_city = UnconfirmedSign(self.outcome.value());
        data.warnings = self.warning_months as i16;

        return with_occurrence(data, &self.occurrence);
    }
}

/// `CityUnderAttack`'s payload is the city doing the attacking - confirmed against `The Odyssey`,
/// where a reciprocal pair of `CityUnderAttack` events threatens cities `6` and `7` with each
/// other. `CityAttacksRival` is assumed to carry the same shape by symmetry, not yet confirmed
/// against real data of its own.
///
/// `GreekCityTerrorized`'s payload is the raw in-game index selecting which of the episode's
/// monster sources attacks (`0`/`1` for the first/second opponent god's signature monster, `2` for
/// the episode's own custom monster), not a resolved `Monster` - see `DATA_MAPPING.md`.
#[derive(PartialEq, Debug)]
pub enum MilitaryRequestSubtype {
    CityUnderAttack(CityId),
    CityAttacksRival(CityId),
    GreekCityTerrorized(u16),
}

data_constants!(CityAttackOutcome<u16> {
    Unaffected = 0,
    Destroyed = 1,
    Conquered = 2,
});

/// `city_min`/`city_max` and `amount_min`/`amount_max` are equal unless the underlying event
/// encodes the target city/gift amount as a range - the same `fixed`/`min`/`max` triples
/// `Invasion`/`MilitaryRequest` use for their own city fields, not `other_city` (confirmed against
/// `The Odyssey`: `other_city` is always `0`, while `fixed_target`/`min_target`/`max_target` carry
/// real data).
#[derive(PartialEq, Debug)]
pub struct Gift {
    pub city_min: CityId,
    pub city_max: CityId,
    pub resource: ResourceType,
    pub amount_min: u16,
    pub amount_max: u16,
    pub occurrence: Occurrence,
}

impl Gift {
    fn from_data(event: &EventData, new_file_ver: bool) -> Gift {
        let (city_min, city_max) = if event.fixed_target != -1 {
            (event.fixed_target as u16, event.fixed_target as u16)
        } else {
            (event.min_target as u16, event.max_target as u16)
        };
        let (amount_min, amount_max) = if event.fixed_amount != -1 {
            (event.fixed_amount as u16, event.fixed_amount as u16)
        } else {
            (event.min_amount as u16, event.max_amount as u16)
        };

        return Gift {
            city_min,
            city_max,
            resource: ResourceType::try_resolve_for_format(&(event.first_item as i8), new_file_ver).unwrap_or(ResourceType::Urchin),
            amount_min,
            amount_max,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = EventData {
            event_type: 23,
            first_item: self.resource.value() as i16,
            ..EventData::default()
        };

        if self.city_min == self.city_max {
            data.fixed_target = self.city_min as i16;
        } else {
            data.fixed_target = -1;
            data.min_target = self.city_min as i16;
            data.max_target = self.city_max as i16;
        }

        if self.amount_min == self.amount_max {
            data.fixed_amount = self.amount_min as i16;
        } else {
            data.fixed_amount = -1;
            data.min_amount = self.amount_min as i16;
            data.max_amount = self.amount_max as i16;
        }

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub struct Quest {
    pub god: God,
    pub quest_type: QuestType,
    pub city: CityId,
    pub reward: MonumentReward,
    pub trigger: EventToTrigger,
    pub occurrence: Occurrence,
}

impl Quest {
    fn from_data(event: &EventData) -> Quest {
        return Quest {
            god: God::try_resolve(&(event.subtype as u32)).unwrap_or(God::Zeus),
            quest_type: if event.quest == 0 { QuestType::Type0 } else { QuestType::Type1 },
            city: event.other_city as u16,
            reward: MonumentReward::try_resolve(&*event.loot_type).unwrap_or(MonumentReward::None),
            trigger: EventToTrigger::from_data(event.on_success, *event.trig_reason),
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let (on_success, trig_reason) = self.trigger.to_data();

        return with_occurrence(
            EventData {
                event_type: 4,
                subtype: self.god.value() as u16,
                quest: if self.quest_type == QuestType::Type0 { 0 } else { 1 },
                other_city: self.city as u8,
                loot_type: UnconfirmedSign(self.reward.value()),
                on_success,
                trig_reason: UnconfirmedSign(trig_reason),
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

data_constants!(MonumentReward<u16> {
    None = 0,
    Small = 1,
    Large = 2,
});

#[derive(PartialEq, Debug)]
pub struct EventToTrigger {
    /// `-1` means no event is triggered - confirmed against the real editor, which renders
    /// `EventData.on_success`'s `-1` sentinel as `-1` directly, not `65535`.
    pub event_id: i16,
    pub trigger_type: TriggerType,
}

impl EventToTrigger {
    fn from_data(event_id: i16, trigger_type: u16) -> EventToTrigger {
        return EventToTrigger {
            event_id,
            trigger_type: TriggerType::try_resolve(&trigger_type).unwrap_or(TriggerType::DirectResult),
        };
    }

    fn to_data(&self) -> (i16, u16) {
        return (self.event_id, self.trigger_type.value());
    }
}

data_constants!(TriggerType<u16> {
    DirectResult = 0,
    Incidental = 1,
    InSpiteOf = 2,
    NoCause = 3,
    Cyclical = 4,
    Specific = 5,
    Auto = 6,
});

/// `city_min`/`city_max` are equal unless the underlying event encodes the target city as a range.
/// `amount_min`/`amount_max` is the invading force size; `marker_min`/`marker_max` selects which
/// map markers the invaders spawn/land at.
#[derive(PartialEq, Debug)]
pub struct Invasion {
    pub city_min: CityId,
    pub city_max: CityId,
    pub warships: u16,
    pub amount_min: u16,
    pub amount_max: u16,
    pub marker_min: u16,
    pub marker_max: u16,
    pub warning_months: WarningMonths,
    pub occurrence: Occurrence,
}

impl Invasion {
    fn from_data(event: &EventData) -> Invasion {
        let (city_min, city_max) = if event.source_fixed != -1 {
            (event.source_fixed as u16, event.source_fixed as u16)
        } else {
            (event.source_min as u16, event.source_max as u16)
        };
        let (amount_min, amount_max) = if event.fixed_amount != -1 {
            (event.fixed_amount as u16, event.fixed_amount as u16)
        } else {
            (event.min_amount as u16, event.max_amount as u16)
        };
        let (marker_min, marker_max) = if event.fixed_target != -1 {
            (event.fixed_target as u16, event.fixed_target as u16)
        } else {
            (event.min_target as u16, event.max_target as u16)
        };

        return Invasion {
            city_min,
            city_max,
            warships: *event.god_or_mon_or_warship_id,
            amount_min,
            amount_max,
            marker_min,
            marker_max,
            warning_months: event.warnings as u8,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = EventData {
            event_type: 2,
            god_or_mon_or_warship_id: UnconfirmedSign(self.warships),
            warnings: self.warning_months as i16,
            ..EventData::default()
        };

        if self.city_min == self.city_max {
            data.source_fixed = self.city_min as i16;
        } else {
            data.source_fixed = -1;
            data.source_min = self.city_min as i16;
            data.source_max = self.city_max as i16;
        }

        if self.amount_min == self.amount_max {
            data.fixed_amount = self.amount_min as i16;
        } else {
            data.fixed_amount = -1;
            data.min_amount = self.amount_min as i16;
            data.max_amount = self.amount_max as i16;
        }

        if self.marker_min == self.marker_max {
            data.fixed_target = self.marker_min as i16;
        } else {
            data.fixed_target = -1;
            data.min_target = self.marker_min as i16;
            data.max_target = self.marker_max as i16;
        }

        return with_occurrence(data, &self.occurrence);
    }
}

/// A monster's involvement in an episode, from being placed in a city to invading it.
///
/// `attack` is common to every `MonsterInvasionSubtype`; `Occurrence` is not, since
/// `MonsterInvasionSubtype::MonsterInCity` has no occurrence of its own.
#[derive(PartialEq, Debug)]
pub struct MonsterInvasion {
    pub attack: MonsterAttack,
    pub subtype: MonsterInvasionSubtype,
}

default_differ_impl!(MonsterInvasion);

// `MonsterUnleashed` reuses the general `month`/`fixed_time` fields, but confirmed against
// `The Odyssey` with two real records: a raw `month` of `0` means the between-years range comes
// from `fixed_time`/`min_time`/`max_time` as usual, while a non-zero `month` is already the real
// target month on its own - `fixed_time` is a leftover template value in that case, and between
// years defaults to `(0, 0)`.
fn monster_unleashed_occurrence_from_data(event: &EventData) -> Occurrence {
    if event.month != 0 {
        return Occurrence::OneTime(event.month as Month, BetweenYears(0, 0));
    }
    return Occurrence::from_data(event);
}

impl MonsterInvasion {
    fn from_data(event: &EventData) -> MonsterInvasion {
        let subtype = match event.subtype {
            1 => MonsterInvasionSubtype::MonsterUnleashed(monster_unleashed_occurrence_from_data(event)),
            2 => MonsterInvasionSubtype::MonsterInvades(event.warnings as u8, Occurrence::from_data(event)),
            // Covers `0` and any unrecognized subtype.
            _ => MonsterInvasionSubtype::MonsterInCity,
        };

        return MonsterInvasion {
            attack: MonsterAttack::from_data(event),
            subtype,
        };
    }

    fn to_data(&self) -> EventData {
        return match &self.subtype {
            MonsterInvasionSubtype::MonsterInCity => self.attack.to_data(26, 0),
            MonsterInvasionSubtype::MonsterUnleashed(occurrence) => with_occurrence(self.attack.to_data(26, 1), occurrence),
            MonsterInvasionSubtype::MonsterInvades(warning_months, occurrence) => {
                let mut data = self.attack.to_data(26, 2);
                data.warnings = *warning_months as i16;

                with_occurrence(data, occurrence)
            }
        };
    }
}

#[derive(PartialEq, Debug)]
pub enum MonsterInvasionSubtype {
    MonsterInCity,
    MonsterUnleashed(Occurrence),
    MonsterInvades(WarningMonths, Occurrence),
}

#[derive(PartialEq, Debug)]
pub struct MonsterAttack {
    /// Raw in-game indices selecting which of the episode's monster sources take part (`0`/`1` for
    /// the first/second opponent god's signature monster, `2` for the episode's own custom
    /// monster) - the same scheme as `MilitaryRequestSubtype::GreekCityTerrorized`, not a resolved
    /// `Monster`.
    pub monsters: Vec<u8>,
    pub monument: bool,
    pub target: Vec<MonsterTarget>,
    pub aggression: u8,
    pub event_on_success: EventToTrigger,
}

impl MonsterAttack {
    fn from_data(event: &EventData) -> MonsterAttack {
        let monsters = [event.first_item, event.second_item, event.third_item]
            .into_iter()
            .filter(|&id| id >= 0)
            .map(|id| id as u8)
            .collect();

        let target = [*event.mtar1 & 0xFF, *event.mtar2, *event.mtar3]
            .into_iter()
            .filter_map(|value| MonsterTarget::try_resolve(&value))
            .collect();

        let event_on_success = EventToTrigger::from_data(event.on_success, *event.trig_reason);

        return MonsterAttack {
            monsters,
            monument: (*event.mtar1 >> 8) != 0,
            target,
            aggression: *event.magg as u8,
            event_on_success,
        };
    }

    fn to_data(&self, event_type: u8, subtype: u16) -> EventData {
        let mut monsters = self.monsters.iter();
        let first_item = monsters.next().map(|&id| id as i16).unwrap_or(-1);
        let second_item = monsters.next().map(|&id| id as i16).unwrap_or(-1);
        let third_item = monsters.next().map(|&id| id as i16).unwrap_or(-1);

        let mut targets = self.target.iter();
        let mtar1_target = targets.next().map(MonsterTarget::value).unwrap_or(0);
        let mtar2 = targets.next().map(MonsterTarget::value).unwrap_or(0);
        let mtar3 = targets.next().map(MonsterTarget::value).unwrap_or(0);
        let monument_bit = if self.monument { 0x0100 } else { 0 };
        let (on_success, trig_reason) = self.event_on_success.to_data();

        return EventData {
            event_type,
            subtype,
            first_item,
            second_item,
            third_item,
            mtar1: UnconfirmedSign(mtar1_target | monument_bit),
            mtar2: UnconfirmedSign(mtar2),
            mtar3: UnconfirmedSign(mtar3),
            magg: UnconfirmedSign(self.aggression as u16),
            on_success,
            trig_reason: UnconfirmedSign(trig_reason),
            ..EventData::default()
        };
    }
}

data_constants!(MonsterTarget<u16> {
    Food = 0,
    Sea = 1,
    Industry = 2,
    Military = 3,
    Money = 4,
    Troops = 5,
    Common = 6,
    Aesthetic = 7,
    Mythological = 8,
    Best = 9,
    Random = 10,
});

#[derive(PartialEq, Debug)]
pub struct GodInvasion {
    /// Raw god ids for up to 3 slots (`u16::MAX` in an unused slot), the same
    /// `first_item`/`second_item`/`third_item` fields `MonsterAttack.monsters` uses - confirmed
    /// against `The Odyssey`, where the raw ids matched `1`/`3` (`Poseidon`/`Apollo`) for a
    /// location whose `opponent_gods` starts with exactly those two, not `god_or_mon_or_warship_id`
    /// (always `0` here).
    pub gods: [u16; 3],
    pub occurrence: Occurrence,
}

impl GodInvasion {
    fn from_data(event: &EventData) -> GodInvasion {
        let gods = [event.first_item as u16, event.second_item as u16, event.third_item as u16];
        let occurrence = Occurrence::from_data(event);

        return GodInvasion { gods, occurrence };
    }

    fn to_data(&self) -> EventData {
        return with_occurrence(
            EventData {
                event_type: 27,
                first_item: self.gods[0] as i16,
                second_item: self.gods[1] as i16,
                third_item: self.gods[2] as i16,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct Disaster {
    pub disaster_type: DisasterSubtype,
    pub marker: u8,
    pub permanent: bool,
    pub occurrence: Occurrence,
}

impl Disaster {
    // `disaster_type` is encoded as `event_type` itself (see DATA_MAPPING.md), so it's always
    // resolvable for the `event_type`s this is dispatched for; `unwrap_or` is defensive only.
    fn from_data(event: &EventData) -> Disaster {
        return Disaster {
            disaster_type: DisasterSubtype::try_resolve(&event.event_type).unwrap_or(DisasterSubtype::Earthquake),
            marker: resolve_range(event.fixed_target, event.min_target) as u8,
            permanent: event.permanent_flag > 0,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        return with_occurrence(
            EventData {
                event_type: self.disaster_type.value(),
                fixed_target: self.marker as i16,
                permanent_flag: if self.permanent { 1 } else { 0 },
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

data_constants!(DisasterSubtype<u8> {
    Earthquake = 3,
    Landslide = 5,
    LavaFlow = 24,
    TidalWave = 25,
    SinkLand = 28,
});

#[derive(PartialEq, Debug)]
pub struct WageIncrease {
    pub amount: u8,
    pub occurrence: Occurrence,
}

impl WageIncrease {
    fn from_data(event: &EventData) -> WageIncrease {
        let amount = resolve_range(event.fixed_amount, event.min_amount) as u8;
        let occurrence = Occurrence::from_data(event);

        return WageIncrease { amount, occurrence };
    }

    fn to_data(&self) -> EventData {
        return with_occurrence(
            EventData {
                event_type: 8,
                fixed_amount: self.amount as i16,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct WageDecrease {
    pub amount: u8,
    pub occurrence: Occurrence,
}

impl WageDecrease {
    fn from_data(event: &EventData) -> WageDecrease {
        let amount = resolve_range(event.fixed_amount, event.min_amount) as u8;
        let occurrence = Occurrence::from_data(event);

        return WageDecrease { amount, occurrence };
    }

    fn to_data(&self) -> EventData {
        return with_occurrence(
            EventData {
                event_type: 9,
                fixed_amount: self.amount as i16,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

/// A change in a resource's trade demand, supply, price, or open/closed status at a city.
///
/// `occurrence` is the only field common to every `TradeChangeSubtype`: `city` is absent from the
/// price variants, and the traded `amount` is absent from the shut-down/opens-up variants.
#[derive(PartialEq, Debug)]
pub struct TradeChange {
    pub occurrence: Occurrence,
    pub subtype: TradeChangeSubtype,
}

default_differ_impl!(TradeChange);

impl TradeChange {
    fn from_data(event: &EventData) -> TradeChange {
        let city = event.other_city as u16;
        let amount = resolve_range(event.fixed_amount, event.min_amount);
        let resource = ResourceType::try_resolve(&(event.first_item as i8)).unwrap_or(ResourceType::Urchin);
        let occurrence = Occurrence::from_data(event);

        let subtype = match (event.event_type, event.subtype) {
            (14, _) => TradeChangeSubtype::DemandDecrease(city, resource, amount as u8),
            (15, _) => TradeChangeSubtype::PriceIncrease(resource, amount as u8),
            (16, _) => TradeChangeSubtype::PriceDecrease(resource, amount as u8),
            (19, 2) => TradeChangeSubtype::TradeShutsDown(city),
            (19, 3) => TradeChangeSubtype::TradeOpensUp(city),
            (21, _) => TradeChangeSubtype::SupplyIncrease(city, resource, amount as u8),
            (22, _) => TradeChangeSubtype::SupplyDecrease(city, resource, amount as u8),
            // Covers `(13, _)` (`DemandIncrease`'s real encoding) and any unrecognized combination.
            _ => TradeChangeSubtype::DemandIncrease(city, resource, amount as u8),
        };

        return TradeChange { occurrence, subtype };
    }

    fn to_data(&self) -> EventData {
        let data = match &self.subtype {
            TradeChangeSubtype::DemandIncrease(city, resource, amount) => EventData {
                event_type: 13,
                other_city: *city as u8,
                first_item: resource.value() as i16,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            TradeChangeSubtype::DemandDecrease(city, resource, amount) => EventData {
                event_type: 14,
                other_city: *city as u8,
                first_item: resource.value() as i16,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            TradeChangeSubtype::SupplyIncrease(city, resource, amount) => EventData {
                event_type: 21,
                other_city: *city as u8,
                first_item: resource.value() as i16,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            TradeChangeSubtype::SupplyDecrease(city, resource, amount) => EventData {
                event_type: 22,
                other_city: *city as u8,
                first_item: resource.value() as i16,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            TradeChangeSubtype::PriceIncrease(resource, amount) => EventData {
                event_type: 15,
                first_item: resource.value() as i16,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            TradeChangeSubtype::PriceDecrease(resource, amount) => EventData {
                event_type: 16,
                first_item: resource.value() as i16,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            TradeChangeSubtype::TradeShutsDown(city) => EventData {
                event_type: 19,
                subtype: 2,
                other_city: *city as u8,
                ..EventData::default()
            },
            TradeChangeSubtype::TradeOpensUp(city) => EventData {
                event_type: 19,
                subtype: 3,
                other_city: *city as u8,
                ..EventData::default()
            },
        };

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub enum TradeChangeSubtype {
    DemandIncrease(CityId, ResourceType, u8),
    DemandDecrease(CityId, ResourceType, u8),
    SupplyIncrease(CityId, ResourceType, u8),
    SupplyDecrease(CityId, ResourceType, u8),
    PriceIncrease(ResourceType, u8),
    PriceDecrease(ResourceType, u8),
    TradeShutsDown(CityId),
    TradeOpensUp(CityId),
}

/// A change in a city's diplomatic standing, strength, activity, or visibility.
///
/// `city_min`/`city_max`/`occurrence` are common to every `CityStatusChangeSubtype`. `city_min`
/// and `city_max` are equal unless the underlying event encodes a fixed-or-range quantity as a
/// range (see `to_data`).
#[derive(PartialEq, Debug)]
pub struct CityStatusChange {
    pub city_min: CityId,
    pub city_max: CityId,
    pub occurrence: Occurrence,
    pub subtype: CityStatusChangeSubtype,
}

default_differ_impl!(CityStatusChange);

impl CityStatusChange {
    fn from_data(event: &EventData) -> CityStatusChange {
        let amount = resolve_range(event.fixed_amount, event.min_amount);
        let (city_min, city_max) = if event.fixed_target != -1 {
            (event.fixed_target as u16, event.fixed_target as u16)
        } else {
            (event.min_target as u16, event.max_target as u16)
        };

        let subtype = match event.subtype {
            10 => CityStatusChangeSubtype::CityBecomesRival,
            11 => CityStatusChangeSubtype::CityBecomesVassal,
            13 => {
                let god = God::try_resolve(&(*event.god_or_mon_or_warship_id as u32)).unwrap_or(God::Zeus);
                CityStatusChangeSubtype::GodDisaster(god, event.warnings as u8)
            }
            14 => CityStatusChangeSubtype::MilitaryBuildup(amount as u8),
            15 => CityStatusChangeSubtype::MilitaryDecline(amount as u8),
            16 => CityStatusChangeSubtype::EconomicProsperity(amount as u8),
            17 => CityStatusChangeSubtype::EconomicDecline(amount as u8),
            18 => CityStatusChangeSubtype::CityBecomesActive,
            19 => CityStatusChangeSubtype::CityBecomesInactive,
            20 => CityStatusChangeSubtype::CityAppears,
            21 => CityStatusChangeSubtype::CityDisappears,
            23 => CityStatusChangeSubtype::RebellionOver,
            24 => CityStatusChangeSubtype::CityConqueredBy(event.other_city as u16),
            // Covers `9` (`RivalBecomesAlly`'s real encoding) and any unrecognized subtype.
            _ => CityStatusChangeSubtype::RivalBecomesAlly,
        };

        return CityStatusChange {
            city_min,
            city_max,
            occurrence: Occurrence::from_data(event),
            subtype,
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = match &self.subtype {
            CityStatusChangeSubtype::RivalBecomesAlly => EventData {
                subtype: 9,
                ..EventData::default()
            },
            CityStatusChangeSubtype::CityBecomesRival => EventData {
                subtype: 10,
                ..EventData::default()
            },
            CityStatusChangeSubtype::CityBecomesVassal => EventData {
                subtype: 11,
                ..EventData::default()
            },
            CityStatusChangeSubtype::GodDisaster(god, warning_months) => EventData {
                subtype: 13,
                god_or_mon_or_warship_id: UnconfirmedSign(god.value() as u16),
                warnings: *warning_months as i16,
                ..EventData::default()
            },
            CityStatusChangeSubtype::MilitaryBuildup(amount) => EventData {
                subtype: 14,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            CityStatusChangeSubtype::MilitaryDecline(amount) => EventData {
                subtype: 15,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            CityStatusChangeSubtype::EconomicProsperity(amount) => EventData {
                subtype: 16,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            CityStatusChangeSubtype::EconomicDecline(amount) => EventData {
                subtype: 17,
                fixed_amount: *amount as i16,
                ..EventData::default()
            },
            CityStatusChangeSubtype::CityBecomesActive => EventData {
                subtype: 18,
                ..EventData::default()
            },
            CityStatusChangeSubtype::CityBecomesInactive => EventData {
                subtype: 19,
                ..EventData::default()
            },
            CityStatusChangeSubtype::CityAppears => EventData {
                subtype: 20,
                ..EventData::default()
            },
            CityStatusChangeSubtype::CityDisappears => EventData {
                subtype: 21,
                ..EventData::default()
            },
            CityStatusChangeSubtype::RebellionOver => EventData {
                subtype: 23,
                ..EventData::default()
            },
            CityStatusChangeSubtype::CityConqueredBy(conqueror) => EventData {
                subtype: 24,
                other_city: *conqueror as u8,
                ..EventData::default()
            },
        };
        data.event_type = 19;

        if self.city_min == self.city_max {
            data.fixed_target = self.city_min as i16;
            data.min_target = -1;
            data.max_target = -1;
        } else {
            data.fixed_target = -1;
            data.min_target = self.city_min as i16;
            data.max_target = self.city_max as i16;
        }

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub enum CityStatusChangeSubtype {
    RivalBecomesAlly,
    CityBecomesRival,
    CityBecomesVassal,
    GodDisaster(God, WarningMonths), // todo not warning months, but duration - make naming clearer
    MilitaryBuildup(u8),
    MilitaryDecline(u8),
    EconomicProsperity(u8),
    EconomicDecline(u8),
    CityBecomesActive,
    CityBecomesInactive,
    CityAppears,
    CityDisappears,
    RebellionOver,
    CityConqueredBy(CityId),
}

/// The rival city's army strength changing over time.
///
/// Not modeled by `from_data` (see DATA_MAPPING.md): its raw subtype wasn't observed in any real
/// adventure surveyed, so `RivalArmyChange` is `to_data`-only.
#[derive(PartialEq, Debug)]
pub struct RivalArmyChange {
    pub city: CityId,
    pub occurrence: Occurrence,
}

impl RivalArmyChange {
    fn to_data(&self) -> EventData {
        return with_occurrence(
            EventData {
                event_type: 20,
                other_city: self.city as u8,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::adventure::Adventure;
    use std::io::Result;

    #[test]
    fn parse_the_founding_of_troy() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Founding of Troy"))?;

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.events,
                vec![
                    Event::TradeChange(TradeChange {
                        occurrence: Occurrence::OneTime(3, BetweenYears(0, 0)),
                        subtype: TradeChangeSubtype::SupplyDecrease(0, ResourceType::Sculpture, 5),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Orichalc),
                        city: 0,
                        amount: 6,
                        warning_months: 8,
                        occurrence: Occurrence::Repeating(2, BetweenYears(1, 2)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::BlackMarble),
                        city: 0,
                        amount: 6,
                        warning_months: 8,
                        occurrence: Occurrence::Repeating(11, BetweenYears(2, 3)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Fleece),
                        city: 0,
                        amount: 8,
                        warning_months: 6,
                        occurrence: Occurrence::Repeating(5, BetweenYears(4, 5)),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::Repeating(9, BetweenYears(5, 5)),
                        subtype: CityStatusChangeSubtype::GodDisaster(God::Hera, 6),
                    }),
                    Event::TradeChange(TradeChange {
                        occurrence: Occurrence::Repeating(1, BetweenYears(8, 8)),
                        subtype: TradeChangeSubtype::TradeShutsDown(0),
                    }),
                    Event::TradeChange(TradeChange {
                        occurrence: Occurrence::Repeating(11, BetweenYears(8, 8)),
                        subtype: TradeChangeSubtype::TradeOpensUp(0),
                    }),
                    Event::TradeChange(TradeChange {
                        occurrence: Occurrence::Repeating(3, BetweenYears(12, 12)),
                        subtype: TradeChangeSubtype::PriceIncrease(ResourceType::Wood, 20),
                    }),
                    Event::WageIncrease(WageIncrease {
                        amount: 15,
                        occurrence: Occurrence::OneTime(5, BetweenYears(22, 22)),
                    }),
                    Event::TradeChange(TradeChange {
                        occurrence: Occurrence::Repeating(6, BetweenYears(26, 26)),
                        subtype: TradeChangeSubtype::PriceDecrease(ResourceType::Wood, 15),
                    }),
                    Event::GodInvasion(GodInvasion {
                        gods: [11, u16::MAX, u16::MAX],
                        occurrence: Occurrence::Repeating(10, BetweenYears(3, 4)),
                    }),
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime(2, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime(5, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime(8, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime(3, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime(6, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime(0, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::OneTime(0, BetweenYears(2, 2)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime(0, BetweenYears(3, 3)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime(0, BetweenYears(4, 4)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime(6, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::OneTime(6, BetweenYears(2, 2)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime(6, BetweenYears(3, 3)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime(6, BetweenYears(4, 4)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 16,
                        city_max: 16,
                        occurrence: Occurrence::OneTime(1, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::CityAppears,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 17,
                        city_max: 17,
                        occurrence: Occurrence::OneTime(8, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::CityAppears,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 18,
                        city_max: 18,
                        occurrence: Occurrence::OneTime(4, BetweenYears(2, 2)),
                        subtype: CityStatusChangeSubtype::CityAppears,
                    }),
                    Event::Invasion(Invasion {
                        city_min: 13,
                        city_max: 15,
                        warships: 0,
                        amount_min: 24,
                        amount_max: 48,
                        marker_min: 9,
                        marker_max: 11,
                        warning_months: 4,
                        occurrence: Occurrence::Repeating(5, BetweenYears(2, 4)),
                    }),
                    Event::Invasion(Invasion {
                        city_min: 16,
                        city_max: 18,
                        warships: 0,
                        amount_min: 18,
                        amount_max: 48,
                        marker_min: 1,
                        marker_max: 8,
                        warning_months: 4,
                        occurrence: Occurrence::Repeating(8, BetweenYears(2, 5)),
                    }),
                    Event::Invasion(Invasion {
                        city_min: 6,
                        city_max: 12,
                        warships: 0,
                        amount_min: 8,
                        amount_max: 64,
                        marker_min: 1,
                        marker_max: 11,
                        warning_months: 2,
                        occurrence: Occurrence::OneTime(6, BetweenYears(1, 8)),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 2,
                        city_max: 5,
                        occurrence: Occurrence::Repeating(4, BetweenYears(1, 4)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 2,
                        city_max: 5,
                        occurrence: Occurrence::OneTime(1, BetweenYears(0, 2)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 2,
                        city_max: 5,
                        occurrence: Occurrence::OneTime(8, BetweenYears(0, 1)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::GreekCityTerrorized(0),
                        city: 3,
                        outcome: CityAttackOutcome::Unaffected,
                        warning_months: 4,
                        occurrence: Occurrence::OneTime(1, BetweenYears(0, 1)),
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::GreekCityTerrorized(1),
                        city: 4,
                        outcome: CityAttackOutcome::Unaffected,
                        warning_months: 4,
                        occurrence: Occurrence::OneTime(5, BetweenYears(1, 2)),
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::GreekCityTerrorized(2),
                        city: 5,
                        outcome: CityAttackOutcome::Unaffected,
                        warning_months: 4,
                        occurrence: Occurrence::OneTime(10, BetweenYears(2, 3)),
                    }),
                    Event::Invasion(Invasion {
                        city_min: 10,
                        city_max: 12,
                        warships: 8,
                        amount_min: 48,
                        amount_max: 72,
                        marker_min: 1,
                        marker_max: 11,
                        warning_months: 5,
                        occurrence: Occurrence::Repeating(0, BetweenYears(4, 6)),
                    })
                ]
            );

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(
                colony_episode.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime(6, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12)
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::OneTime(0, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12)
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime(6, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12)
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime(0, BetweenYears(2, 2)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(12)
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime(9, BetweenYears(0, 1)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1)
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime(3, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1)
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime(4, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1)
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 21,
                        city_max: 21,
                        occurrence: Occurrence::OneTime(2, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityAppears
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 20,
                        city_max: 20,
                        occurrence: Occurrence::OneTime(5, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityAppears
                    }),
                    Event::Invasion(Invasion {
                        city_min: 21,
                        city_max: 21,
                        warships: 0,
                        amount_min: 12,
                        amount_max: 12,
                        marker_min: 1,
                        marker_max: 3,
                        warning_months: 2,
                        occurrence: Occurrence::Repeating(2, BetweenYears(1, 1))
                    }),
                    Event::Invasion(Invasion {
                        city_min: 1,
                        city_max: 8,
                        warships: 0,
                        amount_min: 24,
                        amount_max: 32,
                        marker_min: 1,
                        marker_max: 3,
                        warning_months: 4,
                        occurrence: Occurrence::Repeating(9, BetweenYears(2, 4))
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 14,
                        city_max: 14,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 15,
                        city_max: 15,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::OneTime(10, BetweenYears(0, 1)),
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 14,
                        city_max: 14,
                        occurrence: Occurrence::OneTime(8, BetweenYears(1, 1)),
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 15,
                        city_max: 15,
                        occurrence: Occurrence::OneTime(5, BetweenYears(2, 2)),
                        subtype: CityStatusChangeSubtype::CityBecomesRival
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker: 1, // needs to be marker_min=1, marker_max=2
                        permanent: false,
                        occurrence: Occurrence::OneTime(11, BetweenYears(0, 1))
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker: 3, // needs to be marker_min=3, marker_max=4
                        permanent: false,
                        occurrence: Occurrence::OneTime(3, BetweenYears(1, 3))
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker: 5, // needs to be marker_min=5, marker_max=5
                        permanent: false,
                        occurrence: Occurrence::OneTime(4, BetweenYears(2, 4))
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 20,
                        city_max: 20,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesInactive
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 21,
                        city_max: 21,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesInactive
                    })
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 10, // Troy
                        city_max: 10,
                        occurrence: Occurrence::OneTime(2, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityConqueredBy(5), // Tenedos
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::Repeating(3, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime(4, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityDisappears,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 9, // Ismarus
                        city_max: 9,
                        occurrence: Occurrence::OneTime(6, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryDecline(5),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime(7, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::EconomicDecline(3),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 4, // Sparta
                        city_max: 4,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityDisappears,
                    }),
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 12,
                        city_max: 12,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityAppears,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 12,
                        city_max: 12,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityBecomesActive,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::MilitaryDecline(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 5,
                        city_max: 5,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityDisappears,
                    }),
                ]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.events,
                vec![
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Wine), // this should have 3 valued: Food, Oil, Wine
                        city: 0,                                                          // city_min=6, city_max=7
                        amount: 5,                                                        // min_amount=5, max_amount=20
                        warning_months: 2,
                        occurrence: Occurrence::OneTime(1, BetweenYears(0, 0)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Armor), // this should have 3 valued: Wine, Food, Oil
                        city: 0,                                                           // city_min=6, city_max=7
                        amount: 5,                                                         // min_amount=5, max_amount=20
                        warning_months: 2,
                        occurrence: Occurrence::Repeating(2, BetweenYears(1, 3)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Orichalc), // this should have 3 valued: Oil, Wine, Food
                        city: 0,                                                              // city_min=6, city_max=7
                        amount: 5,                                                            // min_amount=5, max_amount=20
                        warning_months: 2,
                        occurrence: Occurrence::OneTime(11, BetweenYears(1, 3)),
                    }),
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.events,
                vec![
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Wine),
                        city: 0,
                        amount: 10,
                        warning_months: 2,
                        occurrence: Occurrence::OneTime(6, BetweenYears(0, 0)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Armor),
                        city: 0,
                        amount: 10,
                        warning_months: 2,
                        occurrence: Occurrence::Repeating(1, BetweenYears(1, 3)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::Orichalc),
                        city: 0,
                        amount: 10,
                        warning_months: 2,
                        occurrence: Occurrence::Repeating(8, BetweenYears(1, 3)),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 12,
                        city_max: 12,
                        occurrence: Occurrence::OneTime(0, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityDisappears,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::OneTime(3, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityBecomesActive,
                    }),
                    Event::GodInvasion(GodInvasion {
                        gods: [God::Poseidon.value() as u16, u16::MAX, u16::MAX],
                        occurrence: Occurrence::Triggered(BetweenYears(0, 0)),
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::Earthquake,
                        marker: 1,        // marker_min=1 marker_max=3
                        permanent: false, // this field only makes sense to TidalWave
                        occurrence: Occurrence::OneTime(2, BetweenYears(1, 1)),
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker: 4,
                        permanent: false,
                        occurrence: Occurrence::OneTime(3, BetweenYears(1, 1)),
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::TidalWave,
                        marker: 6,
                        permanent: false,
                        occurrence: Occurrence::OneTime(1, BetweenYears(1, 1)),
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        attack: MonsterAttack {
                            monsters: vec![0],
                            monument: false,
                            target: vec![MonsterTarget::Troops, MonsterTarget::Sea, MonsterTarget::Sea],
                            aggression: 2,
                            event_on_success: EventToTrigger {
                                event_id: 5,
                                trigger_type: TriggerType::DirectResult,
                            },
                        },
                        subtype: MonsterInvasionSubtype::MonsterUnleashed(Occurrence::OneTime(0, BetweenYears(1, 1))),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::OneTime(7, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::MilitaryBuildup(1),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::OneTime(8, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::EconomicProsperity(1),
                    }),
                ]
            );

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(
                episode_5.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime(1, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime(1, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                    }),
                    Event::GodInvasion(GodInvasion {
                        gods: [God::Poseidon.value() as u16, God::Apollo.value() as u16, u16::MAX],
                        occurrence: Occurrence::Repeating(7, BetweenYears(2, 4)),
                    }),
                    Event::Invasion(Invasion {
                        city_min: 6,
                        city_max: 7,
                        warships: 2,
                        amount_min: 5,
                        amount_max: 10,
                        marker_min: 9,
                        marker_max: 16,
                        warning_months: 4,
                        occurrence: Occurrence::OneTime(3, BetweenYears(1, 2)),
                    }),
                    Event::Invasion(Invasion {
                        city_min: 6,
                        city_max: 7,
                        warships: 6,
                        amount_min: 20,
                        amount_max: 40,
                        marker_min: 9,
                        marker_max: 16,
                        warning_months: 4,
                        occurrence: Occurrence::Repeating(10, BetweenYears(2, 5)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::OliveOil),
                        city: 6,
                        amount: 5,
                        warning_months: 4,
                        occurrence: Occurrence::OneTime(4, BetweenYears(6, 6)),
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest(ResourceType::OliveOil),
                        city: 7,
                        amount: 5,
                        warning_months: 4,
                        occurrence: Occurrence::OneTime(9, BetweenYears(8, 8)),
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        attack: MonsterAttack {
                            monsters: vec![2],
                            monument: true,
                            target: vec![MonsterTarget::Random, MonsterTarget::Random, MonsterTarget::Random],
                            aggression: 1,
                            event_on_success: EventToTrigger {
                                event_id: -1,
                                trigger_type: TriggerType::DirectResult,
                            },
                        },
                        subtype: MonsterInvasionSubtype::MonsterInvades(0, Occurrence::OneTime(0, BetweenYears(0, 0))),
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::CityUnderAttack(7),
                        city: 6, // city_min=6, city_max=6
                        outcome: CityAttackOutcome::Conquered,
                        warning_months: 4,
                        occurrence: Occurrence::Repeating(0, BetweenYears(2, 10)),
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::CityUnderAttack(6),
                        city: 7, // city_min=7, city_max=7
                        outcome: CityAttackOutcome::Conquered,
                        warning_months: 4,
                        occurrence: Occurrence::Repeating(1, BetweenYears(2, 10)),
                    }),
                ]
            );

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.events,
                vec![
                    Event::Gift(Gift {
                        city_min: 0,
                        city_max: 0,
                        resource: ResourceType::Fish,
                        amount_min: 32,
                        amount_max: 32,
                        occurrence: Occurrence::OneTime(3, BetweenYears(0, 0)),
                    }),
                    Event::Gift(Gift {
                        city_min: 0,
                        city_max: 0,
                        resource: ResourceType::OliveOil,
                        amount_min: 32,
                        amount_max: 32,
                        occurrence: Occurrence::OneTime(4, BetweenYears(0, 0)),
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        attack: MonsterAttack {
                            monsters: vec![2],
                            monument: true,
                            target: vec![MonsterTarget::Random, MonsterTarget::Random, MonsterTarget::Random],
                            aggression: 0,
                            event_on_success: EventToTrigger {
                                event_id: -1,
                                trigger_type: TriggerType::DirectResult,
                            },
                        },
                        subtype: MonsterInvasionSubtype::MonsterInvades(12, Occurrence::OneTime(5, BetweenYears(3, 3))),
                    }),
                ]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.events,
                vec![
                    Event::GodInvasion(GodInvasion {
                        gods: [God::Apollo.value() as u16, u16::MAX, u16::MAX],
                        occurrence: Occurrence::Triggered(BetweenYears(1, 1))
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::OneTime(4, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityAppears,
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::OneTime(5, BetweenYears(0, 0)),
                        subtype: CityStatusChangeSubtype::CityBecomesActive,
                    }),
                    Event::Gift(Gift {
                        city_min: 0,
                        city_max: 0,
                        resource: ResourceType::Fish,
                        amount_min: 32,
                        amount_max: 32,
                        occurrence: Occurrence::OneTime(3, BetweenYears(0, 0)),
                    }),
                    Event::Gift(Gift {
                        city_min: 0,
                        city_max: 0,
                        resource: ResourceType::OliveOil,
                        amount_min: 32,
                        amount_max: 32,
                        occurrence: Occurrence::OneTime(4, BetweenYears(0, 0)),
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::EpisodeComplete,
                        subtype: CityStatusChangeSubtype::CityDisappears,
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        attack: MonsterAttack {
                            monsters: vec![1],
                            monument: true,
                            target: vec![MonsterTarget::Random, MonsterTarget::Random, MonsterTarget::Random],
                            aggression: 0,
                            event_on_success: EventToTrigger {
                                event_id: 0,
                                trigger_type: TriggerType::DirectResult
                            },
                        },
                        subtype: MonsterInvasionSubtype::MonsterUnleashed(Occurrence::OneTime(10, BetweenYears(0, 0)))
                    })
                ]
            );
        }

        return Ok(());
    }
}
