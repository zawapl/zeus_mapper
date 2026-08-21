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
    pub fn vec_from_data(episode_events: &BoxedArray<EventData, 150>, count: usize, new_file_ver: bool) -> Vec<Event> {
        return episode_events
            .iter()
            .take(count)
            .map(|event| Event::from_data(event, new_file_ver))
            .collect();
    }

    pub fn from_data(event: &EventData, new_file_ver: bool) -> Event {
        if event.event_type == 1 && matches!(event.subtype, 1 | 2 | 7) {
            return Event::MilitaryRequest(MilitaryRequest::from_data(event));
        }

        return match event.event_type {
            2 => Event::Invasion(Invasion::from_data(event)),
            4 => Event::Quest(Quest::from_data(event)),
            3 | 5 | 24 | 25 | 28 => Event::Disaster(Disaster::from_data(event)),
            8 => Event::WageIncrease(WageIncrease::from_data(event)),
            9 => Event::WageDecrease(WageDecrease::from_data(event)),
            13 | 14 | 15 | 16 | 21 | 22 => Event::TradeChange(TradeChange::from_data(event, new_file_ver)),
            19 if matches!(event.subtype, 2 | 3) => Event::TradeChange(TradeChange::from_data(event, new_file_ver)),
            19 => Event::CityStatusChange(CityStatusChange::from_data(event)),
            23 => Event::Gift(Gift::from_data(event, new_file_ver)),
            26 => Event::MonsterInvasion(MonsterInvasion::from_data(event)),
            27 => Event::GodInvasion(GodInvasion::from_data(event)),
            _ => Event::GoodsRequest(GoodsRequest::from_data(event, new_file_ver)),
        };
    }

    pub fn vec_to_data(events: &[Event]) -> BoxedArray<EventData, 150> {
        let mut data = vec![EventData::default(); 150];

        for (slot, event) in events.iter().enumerate() {
            if slot < data.len() {
                data[slot] = event.to_data(slot);
            }
        }

        return BoxedArray::from_vec(data);
    }

    pub fn to_data(&self, slot: usize) -> EventData {
        let mut event = match self {
            Event::GoodsRequest(goods_request) => goods_request.to_data(),
            Event::MilitaryRequest(military_request) => military_request.to_data(),
            Event::Gift(gift) => gift.to_data(),
            Event::Quest(quest) => quest.to_data(),
            Event::Invasion(invasion) => invasion.to_data(),
            Event::MonsterInvasion(monster_invasion) => monster_invasion.to_data(),
            Event::GodInvasion(god_invasion) => god_invasion.to_data(),
            Event::Disaster(disaster) => disaster.to_data(),
            Event::WageIncrease(wage_increase) => wage_increase.to_data(),
            Event::WageDecrease(wage_decrease) => wage_decrease.to_data(),
            Event::TradeChange(trade_change) => trade_change.to_data(),
            Event::CityStatusChange(city_status_change) => city_status_change.to_data(),
            Event::RivalArmyChange(rival_army_change) => rival_army_change.to_data(),
        };
        event.id = slot as u16;

        return event;
    }
}

default_differ_impl!(Event);

#[derive(PartialEq, Debug)]
pub struct GoodsRequest {
    pub subtype: GoodsRequestSubtype,
    pub city_min: CityId,
    pub city_max: CityId,
    pub amount_min: u16,
    pub amount_max: u16,
    pub occurrence: Occurrence,
    pub warning_months: u8,
}

#[derive(PartialEq, Debug)]
pub enum GoodsRequestSubtype {
    GeneralRequest { resources: Vec<ResourceType> },
    Festival { resources: Vec<ResourceType>, god: God },
    Construction { resources: Vec<ResourceType> },
    Famine,
    FinancialWoes,
}

impl GoodsRequest {
    fn from_data(event: &EventData, new_file_ver: bool) -> GoodsRequest {
        let resources = items_from_data(event, new_file_ver);

        let subtype = match event.subtype {
            0 => GoodsRequestSubtype::GeneralRequest { resources },
            3 => {
                let god = God::try_resolve(&(*event.god_or_mon_or_warship_id as u32)).unwrap_or(God::Zeus);
                GoodsRequestSubtype::Festival { resources, god }
            }
            4 => GoodsRequestSubtype::Construction { resources },
            5 => GoodsRequestSubtype::Famine,
            6 | _ => GoodsRequestSubtype::FinancialWoes,
        };

        let (city_min, city_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);
        let (amount_min, amount_max) = range_from_data(event.fixed_amount, event.min_amount, event.max_amount);

        return GoodsRequest {
            subtype,
            city_min,
            city_max,
            amount_min,
            amount_max,
            occurrence: Occurrence::from_data(event),
            warning_months: event.warnings as u8,
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = match &self.subtype {
            GoodsRequestSubtype::GeneralRequest { resources } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                EventData {
                    subtype: 0,
                    first_item,
                    second_item,
                    third_item,
                    ..EventData::default()
                }
            }
            GoodsRequestSubtype::Festival { resources, god } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                EventData {
                    subtype: 3,
                    first_item,
                    second_item,
                    third_item,
                    god_or_mon_or_warship_id: UnconfirmedSign(god.value() as u16),
                    ..EventData::default()
                }
            }
            GoodsRequestSubtype::Construction { resources } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                EventData {
                    subtype: 4,
                    first_item,
                    second_item,
                    third_item,
                    ..EventData::default()
                }
            }
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
        (data.fixed_target, data.min_target, data.max_target) = range_to_data(self.city_min, self.city_max);
        (data.fixed_amount, data.min_amount, data.max_amount) = range_to_data(self.amount_min, self.amount_max);
        data.warnings = self.warning_months as i16;

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub struct MilitaryRequest {
    pub subtype: MilitaryRequestSubtype,
    pub city_min: CityId,
    pub city_max: CityId,
    pub outcome: CityAttackOutcome,
    pub occurrence: Occurrence,
    pub warning_months: u8,
}

#[derive(PartialEq, Debug)]
pub enum MilitaryRequestSubtype {
    CityUnderAttack { attacking_city: CityId },
    CityAttacksRival { attacked_city: CityId },
    GreekCityTerrorized { monster_index: u16 },
}

impl MilitaryRequest {
    fn from_data(event: &EventData) -> MilitaryRequest {
        let outcome = CityAttackOutcome::try_resolve(&(*event.eff_on_city & 0xFF)).unwrap_or(CityAttackOutcome::Unaffected);

        let subtype = match event.subtype {
            1 => MilitaryRequestSubtype::CityUnderAttack {
                attacking_city: event.other_city as u16,
            },
            2 => MilitaryRequestSubtype::CityAttacksRival {
                attacked_city: event.other_city as u16,
            },
            7 | _ => MilitaryRequestSubtype::GreekCityTerrorized {
                monster_index: *event.god_or_mon_or_warship_id,
            },
        };

        let (city_min, city_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);

        return MilitaryRequest {
            subtype,
            city_min,
            city_max,
            outcome,
            occurrence: Occurrence::from_data(event),
            warning_months: event.warnings as u8,
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = match &self.subtype {
            MilitaryRequestSubtype::CityUnderAttack {
                attacking_city: second_city,
            } => EventData {
                subtype: 1,
                other_city: *second_city as u8,
                ..EventData::default()
            },
            MilitaryRequestSubtype::CityAttacksRival {
                attacked_city: second_city,
            } => EventData {
                subtype: 2,
                other_city: *second_city as u8,
                ..EventData::default()
            },
            MilitaryRequestSubtype::GreekCityTerrorized { monster_index } => EventData {
                subtype: 7,
                god_or_mon_or_warship_id: UnconfirmedSign(*monster_index),
                ..EventData::default()
            },
        };
        data.event_type = 1;
        (data.fixed_target, data.min_target, data.max_target) = range_to_data(self.city_min, self.city_max);
        data.eff_on_city = UnconfirmedSign(self.outcome.value());
        data.warnings = self.warning_months as i16;

        return with_occurrence(data, &self.occurrence);
    }
}

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
        let (city_min, city_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);
        let (amount_min, amount_max) = range_from_data(event.fixed_amount, event.min_amount, event.max_amount);

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

        (data.fixed_target, data.min_target, data.max_target) = range_to_data(self.city_min, self.city_max);
        (data.fixed_amount, data.min_amount, data.max_amount) = range_to_data(self.amount_min, self.amount_max);

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub struct Quest {
    pub god: God,
    pub city_min: CityId,
    pub city_max: CityId,
    pub reward: MonumentReward,
    pub hero: Hero,
    pub quest_type: QuestType,
    pub trigger: EventToTrigger,
    pub occurrence: Occurrence,
}

impl Quest {
    fn from_data(event: &EventData) -> Quest {
        let (city_min, city_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);

        return Quest {
            god: God::try_resolve(&(event.subtype as u32)).unwrap_or(God::Zeus),
            quest_type: if event.quest == 0 { QuestType::Type0 } else { QuestType::Type1 },
            city_min,
            city_max,
            reward: MonumentReward::try_resolve(&*event.loot_type).unwrap_or(MonumentReward::None),
            hero: Hero::try_resolve(&*event.god_or_mon_or_warship_id).unwrap_or(Hero::Hero1),
            trigger: EventToTrigger::from_data(event.on_success, *event.trig_reason),
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let (on_success, trig_reason) = self.trigger.to_data();
        let (fixed_target, min_target, max_target) = range_to_data(self.city_min, self.city_max);

        return with_occurrence(
            EventData {
                event_type: 4,
                subtype: self.god.value() as u16,
                quest: if self.quest_type == QuestType::Type0 { 0 } else { 1 },
                fixed_target,
                min_target,
                max_target,
                loot_type: UnconfirmedSign(self.reward.value()),
                god_or_mon_or_warship_id: UnconfirmedSign(self.hero.value()),
                on_success,
                trig_reason: UnconfirmedSign(trig_reason),
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct Invasion {
    pub city_min: CityId,
    pub city_max: CityId,
    pub amount_min: u16,
    pub amount_max: u16,
    pub marker_min: u16,
    pub marker_max: u16,
    pub warships: u16,
    pub occurrence: Occurrence,
    pub warning_months: u8,
}

impl Invasion {
    fn from_data(event: &EventData) -> Invasion {
        let (city_min, city_max) = range_from_data(event.source_fixed, event.source_min, event.source_max);
        let (amount_min, amount_max) = range_from_data(event.fixed_amount, event.min_amount, event.max_amount);
        let (marker_min, marker_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);

        return Invasion {
            city_min,
            city_max,
            warships: *event.god_or_mon_or_warship_id,
            amount_min,
            amount_max,
            marker_min,
            marker_max,
            occurrence: Occurrence::from_data(event),
            warning_months: event.warnings as u8,
        };
    }

    fn to_data(&self) -> EventData {
        let mut data = EventData {
            event_type: 2,
            god_or_mon_or_warship_id: UnconfirmedSign(self.warships),
            warnings: self.warning_months as i16,
            ..EventData::default()
        };

        (data.source_fixed, data.source_min, data.source_max) = range_to_data(self.city_min, self.city_max);
        (data.fixed_amount, data.min_amount, data.max_amount) = range_to_data(self.amount_min, self.amount_max);
        (data.fixed_target, data.min_target, data.max_target) = range_to_data(self.marker_min, self.marker_max);

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub struct MonsterInvasion {
    pub subtype: MonsterInvasionSubtype,
    pub monsters: Vec<MonsterSlot>,
    pub monument: bool,
    pub target: [MonsterTarget; 3],
    pub aggression: u8,
    pub next_event: EventToTrigger,
}
#[derive(PartialEq, Debug)]
pub enum MonsterInvasionSubtype {
    MonsterInCity,
    MonsterUnleashed { occurrence: Occurrence },
    MonsterInvades { occurrence: Occurrence, warning_months: u8 },
}

impl MonsterInvasion {
    fn from_data(event: &EventData) -> MonsterInvasion {
        let subtype = match event.subtype {
            0 => MonsterInvasionSubtype::MonsterInCity,
            1 => MonsterInvasionSubtype::MonsterUnleashed {
                occurrence: Occurrence::from_data(event),
            },
            2 | _ => MonsterInvasionSubtype::MonsterInvades {
                occurrence: Occurrence::from_data(event),
                warning_months: event.warnings as u8,
            },
        };

        let monsters = [event.first_item, event.second_item, event.third_item]
            .into_iter()
            .filter_map(|id| MonsterSlot::try_resolve(&(id as u8)))
            .collect();

        let target =
            [event.mtar1 as u16, event.mtar2, event.mtar3].map(|value| MonsterTarget::try_resolve(&value).unwrap_or(MonsterTarget::Random));

        let next_event = EventToTrigger::from_data(event.on_success, *event.trig_reason);

        return MonsterInvasion {
            subtype,
            monsters,
            monument: event.monument != 0,
            target,
            aggression: *event.magg as u8,
            next_event,
        };
    }

    fn to_data(&self) -> EventData {
        let mut monster_ids = self.monsters.iter();
        let first_item = monster_ids.next().map(|slot| slot.value() as i16).unwrap_or(-1);
        let second_item = monster_ids.next().map(|slot| slot.value() as i16).unwrap_or(-1);
        let third_item = monster_ids.next().map(|slot| slot.value() as i16).unwrap_or(-1);

        let mut targets = self.target.iter();
        let mtar1 = targets.next().map(MonsterTarget::value).unwrap_or(0);
        let mtar2 = targets.next().map(MonsterTarget::value).unwrap_or(0);
        let mtar3 = targets.next().map(MonsterTarget::value).unwrap_or(0);
        let (on_success, trig_reason) = self.next_event.to_data();

        let mut event_data = EventData {
            event_type: 26,
            first_item,
            second_item,
            third_item,
            mtar1: mtar1 as u8,
            monument: if self.monument { 1 } else { 0 },
            mtar2,
            mtar3,
            magg: UnconfirmedSign(self.aggression as u16),
            on_success,
            trig_reason: UnconfirmedSign(trig_reason),
            ..EventData::default()
        };

        return match &self.subtype {
            MonsterInvasionSubtype::MonsterInCity => event_data,
            MonsterInvasionSubtype::MonsterUnleashed { occurrence } => {
                event_data.subtype = 1;
                with_occurrence(event_data, occurrence)
            }
            MonsterInvasionSubtype::MonsterInvades {
                warning_months,
                occurrence,
            } => {
                event_data.subtype = 2;
                event_data.warnings = *warning_months as i16;
                with_occurrence(event_data, occurrence)
            }
        };
    }
}

#[derive(PartialEq, Debug)]
pub struct GodInvasion {
    pub gods: Vec<u16>,
    pub occurrence: Occurrence,
}

impl GodInvasion {
    fn from_data(event: &EventData) -> GodInvasion {
        let gods = [event.first_item, event.second_item, event.third_item]
            .into_iter()
            .filter(|&id| id >= 0)
            .map(|id| id as u16)
            .collect();

        return GodInvasion {
            gods,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let mut god_ids = self.gods.iter();
        let first_item = god_ids.next().map(|&id| id as i16).unwrap_or(-1);
        let second_item = god_ids.next().map(|&id| id as i16).unwrap_or(-1);
        let third_item = god_ids.next().map(|&id| id as i16).unwrap_or(-1);

        return with_occurrence(
            EventData {
                event_type: 27,
                first_item,
                second_item,
                third_item,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct Disaster {
    pub disaster_type: DisasterSubtype,
    pub marker_min: u8,
    pub marker_max: u8,
    pub occurrence: Occurrence,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DisasterSubtype {
    Earthquake,
    Landslide,
    LavaFlow,
    TidalWave { permanent: bool },
    SinkLand,
}

impl Disaster {
    fn from_data(event: &EventData) -> Disaster {
        let disaster_type = match event.event_type {
            3 => DisasterSubtype::Earthquake,
            5 => DisasterSubtype::Landslide,
            24 => DisasterSubtype::LavaFlow,
            25 => DisasterSubtype::TidalWave {
                permanent: event.permanent_flag > 0,
            },
            28 | _ => DisasterSubtype::SinkLand,
        };
        let (marker_min, marker_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);

        return Disaster {
            disaster_type,
            marker_min: marker_min as u8,
            marker_max: marker_max as u8,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let (event_type, permanent_flag) = match self.disaster_type {
            DisasterSubtype::Earthquake => (3, 0),
            DisasterSubtype::Landslide => (5, 0),
            DisasterSubtype::LavaFlow => (24, 0),
            DisasterSubtype::TidalWave { permanent } => (25, if permanent { 1 } else { 0 }),
            DisasterSubtype::SinkLand => (28, 0),
        };

        let (fixed_target, min_target, max_target) = range_to_data(self.marker_min as u16, self.marker_max as u16);

        return with_occurrence(
            EventData {
                event_type,
                fixed_target,
                min_target,
                max_target,
                permanent_flag,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct WageIncrease {
    pub amount_min: u16,
    pub amount_max: u16,
    pub occurrence: Occurrence,
}

impl WageIncrease {
    fn from_data(event: &EventData) -> WageIncrease {
        let (amount_min, amount_max) = range_from_data(event.fixed_amount, event.min_amount, event.max_amount);

        return WageIncrease {
            amount_min,
            amount_max,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let (fixed_amount, min_amount, max_amount) = range_to_data(self.amount_min, self.amount_max);

        return with_occurrence(
            EventData {
                event_type: 8,
                fixed_amount,
                min_amount,
                max_amount,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct WageDecrease {
    pub amount_min: u16,
    pub amount_max: u16,
    pub occurrence: Occurrence,
}

impl WageDecrease {
    fn from_data(event: &EventData) -> WageDecrease {
        let (amount_min, amount_max) = range_from_data(event.fixed_amount, event.min_amount, event.max_amount);

        return WageDecrease {
            amount_min,
            amount_max,
            occurrence: Occurrence::from_data(event),
        };
    }

    fn to_data(&self) -> EventData {
        let (fixed_amount, min_amount, max_amount) = range_to_data(self.amount_min, self.amount_max);

        return with_occurrence(
            EventData {
                event_type: 9,
                fixed_amount,
                min_amount,
                max_amount,
                ..EventData::default()
            },
            &self.occurrence,
        );
    }
}

#[derive(PartialEq, Debug)]
pub struct TradeChange {
    pub subtype: TradeChangeSubtype,
    pub occurrence: Occurrence,
}

#[derive(PartialEq, Debug)]
pub enum TradeChangeSubtype {
    DemandIncrease {
        city_min: CityId,
        city_max: CityId,
        resources: Vec<ResourceType>,
    },
    DemandDecrease {
        city_min: CityId,
        city_max: CityId,
        resources: Vec<ResourceType>,
    },
    SupplyIncrease {
        city_min: CityId,
        city_max: CityId,
        resources: Vec<ResourceType>,
    },
    SupplyDecrease {
        city_min: CityId,
        city_max: CityId,
        resources: Vec<ResourceType>,
    },
    PriceIncrease {
        resources: Vec<ResourceType>,
        amount_min: u8,
        amount_max: u8,
    },
    PriceDecrease {
        resources: Vec<ResourceType>,
        amount_min: u8,
        amount_max: u8,
    },
    TradeShutsDown {
        city_min: CityId,
        city_max: CityId,
    },
    TradeOpensUp {
        city_min: CityId,
        city_max: CityId,
    },
}

impl TradeChange {
    fn from_data(event: &EventData, new_file_ver: bool) -> TradeChange {
        let (city_min, city_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);
        let resources = items_from_data(event, new_file_ver);
        let (amount_min, amount_max) = range_from_data(event.fixed_amount, event.min_amount, event.max_amount);
        let (amount_min, amount_max) = (amount_min as u8, amount_max as u8);
        let occurrence = Occurrence::from_data(event);

        let subtype = match (event.event_type, event.subtype) {
            (13, _) => TradeChangeSubtype::DemandIncrease {
                city_min,
                city_max,
                resources,
            },
            (14, _) => TradeChangeSubtype::DemandDecrease {
                city_min,
                city_max,
                resources,
            },
            (15, _) => TradeChangeSubtype::PriceIncrease {
                resources,
                amount_min,
                amount_max,
            },
            (16, _) => TradeChangeSubtype::PriceDecrease {
                resources,
                amount_min,
                amount_max,
            },
            (19, 2) => TradeChangeSubtype::TradeShutsDown { city_min, city_max },
            (19, 3) => TradeChangeSubtype::TradeOpensUp { city_min, city_max },
            (21, _) => TradeChangeSubtype::SupplyIncrease {
                city_min,
                city_max,
                resources,
            },
            (22, _) | _ => TradeChangeSubtype::SupplyDecrease {
                city_min,
                city_max,
                resources,
            },
        };

        return TradeChange { subtype, occurrence };
    }

    fn to_data(&self) -> EventData {
        let data = match &self.subtype {
            TradeChangeSubtype::DemandIncrease {
                city_min,
                city_max,
                resources,
            } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                let (fixed_target, min_target, max_target) = range_to_data(*city_min, *city_max);
                EventData {
                    event_type: 13,
                    first_item,
                    second_item,
                    third_item,
                    fixed_target,
                    min_target,
                    max_target,
                    ..EventData::default()
                }
            }
            TradeChangeSubtype::DemandDecrease {
                city_min,
                city_max,
                resources,
            } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                let (fixed_target, min_target, max_target) = range_to_data(*city_min, *city_max);
                EventData {
                    event_type: 14,
                    first_item,
                    second_item,
                    third_item,
                    fixed_target,
                    min_target,
                    max_target,
                    ..EventData::default()
                }
            }
            TradeChangeSubtype::SupplyIncrease {
                city_min,
                city_max,
                resources,
            } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                let (fixed_target, min_target, max_target) = range_to_data(*city_min, *city_max);
                EventData {
                    event_type: 21,
                    first_item,
                    second_item,
                    third_item,
                    fixed_target,
                    min_target,
                    max_target,
                    ..EventData::default()
                }
            }
            TradeChangeSubtype::SupplyDecrease {
                city_min,
                city_max,
                resources,
            } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                let (fixed_target, min_target, max_target) = range_to_data(*city_min, *city_max);
                EventData {
                    event_type: 22,
                    first_item,
                    second_item,
                    third_item,
                    fixed_target,
                    min_target,
                    max_target,
                    ..EventData::default()
                }
            }
            TradeChangeSubtype::PriceIncrease {
                resources,
                amount_min,
                amount_max,
            } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                let (fixed_amount, min_amount, max_amount) = range_to_data(*amount_min as u16, *amount_max as u16);
                EventData {
                    event_type: 15,
                    first_item,
                    second_item,
                    third_item,
                    fixed_amount,
                    min_amount,
                    max_amount,
                    ..EventData::default()
                }
            }
            TradeChangeSubtype::PriceDecrease {
                resources,
                amount_min,
                amount_max,
            } => {
                let (first_item, second_item, third_item) = items_to_data(resources);
                let (fixed_amount, min_amount, max_amount) = range_to_data(*amount_min as u16, *amount_max as u16);
                EventData {
                    event_type: 16,
                    first_item,
                    second_item,
                    third_item,
                    fixed_amount,
                    min_amount,
                    max_amount,
                    ..EventData::default()
                }
            }
            TradeChangeSubtype::TradeShutsDown { city_min, city_max } => {
                let (fixed_target, min_target, max_target) = range_to_data(*city_min, *city_max);
                EventData {
                    event_type: 19,
                    subtype: 2,
                    fixed_target,
                    min_target,
                    max_target,
                    ..EventData::default()
                }
            }
            TradeChangeSubtype::TradeOpensUp { city_min, city_max } => {
                let (fixed_target, min_target, max_target) = range_to_data(*city_min, *city_max);
                EventData {
                    event_type: 19,
                    subtype: 3,
                    fixed_target,
                    min_target,
                    max_target,
                    ..EventData::default()
                }
            }
        };

        return with_occurrence(data, &self.occurrence);
    }
}

#[derive(PartialEq, Debug)]
pub struct CityStatusChange {
    pub subtype: CityStatusChangeSubtype,
    pub city_min: CityId,
    pub city_max: CityId,
    pub occurrence: Occurrence,
}

#[derive(PartialEq, Debug)]
pub enum CityStatusChangeSubtype {
    RivalBecomesAlly,
    CityBecomesRival,
    CityBecomesVassal,
    GodDisaster { god: God, duration: u8 },
    MilitaryBuildup { amount_min: u8, amount_max: u8 },
    MilitaryDecline { amount_min: u8, amount_max: u8 },
    EconomicProsperity { amount_min: u8, amount_max: u8 },
    EconomicDecline { amount_min: u8, amount_max: u8 },
    CityBecomesActive,
    CityBecomesInactive,
    CityAppears,
    CityDisappears,
    RebellionOver,
    CityConquered { attacking_city: CityId },
}

impl CityStatusChange {
    fn from_data(event: &EventData) -> CityStatusChange {
        let (amount_min, amount_max) = range_from_data(event.fixed_amount, event.min_amount, event.max_amount);
        let (amount_min, amount_max) = (amount_min as u8, amount_max as u8);
        let (city_min, city_max) = range_from_data(event.fixed_target, event.min_target, event.max_target);

        let subtype = match event.subtype {
            9 => CityStatusChangeSubtype::RivalBecomesAlly,
            10 => CityStatusChangeSubtype::CityBecomesRival,
            11 => CityStatusChangeSubtype::CityBecomesVassal,
            13 => {
                let god = God::try_resolve(&(*event.god_or_mon_or_warship_id as u32)).unwrap_or(God::Zeus);
                CityStatusChangeSubtype::GodDisaster {
                    god,
                    duration: event.warnings as u8,
                }
            }
            14 => CityStatusChangeSubtype::MilitaryBuildup { amount_min, amount_max },
            15 => CityStatusChangeSubtype::MilitaryDecline { amount_min, amount_max },
            16 => CityStatusChangeSubtype::EconomicProsperity { amount_min, amount_max },
            17 => CityStatusChangeSubtype::EconomicDecline { amount_min, amount_max },
            18 => CityStatusChangeSubtype::CityBecomesActive,
            19 => CityStatusChangeSubtype::CityBecomesInactive,
            20 => CityStatusChangeSubtype::CityAppears,
            21 => CityStatusChangeSubtype::CityDisappears,
            23 => CityStatusChangeSubtype::RebellionOver,
            24 | _ => CityStatusChangeSubtype::CityConquered {
                attacking_city: event.other_city as u16,
            },
        };

        return CityStatusChange {
            subtype,
            city_min,
            city_max,
            occurrence: Occurrence::from_data(event),
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
            CityStatusChangeSubtype::GodDisaster { god, duration } => EventData {
                subtype: 13,
                god_or_mon_or_warship_id: UnconfirmedSign(god.value() as u16),
                warnings: *duration as i16,
                ..EventData::default()
            },
            CityStatusChangeSubtype::MilitaryBuildup { amount_min, amount_max } => {
                let (fixed_amount, min_amount, max_amount) = range_to_data(*amount_min as u16, *amount_max as u16);
                EventData {
                    subtype: 14,
                    fixed_amount,
                    min_amount,
                    max_amount,
                    ..EventData::default()
                }
            }
            CityStatusChangeSubtype::MilitaryDecline { amount_min, amount_max } => {
                let (fixed_amount, min_amount, max_amount) = range_to_data(*amount_min as u16, *amount_max as u16);
                EventData {
                    subtype: 15,
                    fixed_amount,
                    min_amount,
                    max_amount,
                    ..EventData::default()
                }
            }
            CityStatusChangeSubtype::EconomicProsperity { amount_min, amount_max } => {
                let (fixed_amount, min_amount, max_amount) = range_to_data(*amount_min as u16, *amount_max as u16);
                EventData {
                    subtype: 16,
                    fixed_amount,
                    min_amount,
                    max_amount,
                    ..EventData::default()
                }
            }
            CityStatusChangeSubtype::EconomicDecline { amount_min, amount_max } => {
                let (fixed_amount, min_amount, max_amount) = range_to_data(*amount_min as u16, *amount_max as u16);
                EventData {
                    subtype: 17,
                    fixed_amount,
                    min_amount,
                    max_amount,
                    ..EventData::default()
                }
            }
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
            CityStatusChangeSubtype::CityConquered { attacking_city: conqueror } => EventData {
                subtype: 24,
                other_city: *conqueror as u8,
                ..EventData::default()
            },
        };
        data.event_type = 19;
        (data.fixed_target, data.min_target, data.max_target) = range_to_data(self.city_min, self.city_max);

        return with_occurrence(data, &self.occurrence);
    }
}

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

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Occurrence {
    OneTime { month: u16, year_min: u16, year_max: u16 },
    Recurring { month: u16, year_min: u16, year_max: u16 },
    Triggered { year_min: u16, year_max: u16 },
    EpisodeComplete,
}

default_differ_impl!(Occurrence);

impl Occurrence {
    fn from_data(event: &EventData) -> Occurrence {
        let (year_min, year_max) = range_from_data(event.fixed_time, event.min_time, event.max_time);

        if event.flags & 0x1 != 0 {
            return Occurrence::Triggered { year_min, year_max };
        }

        if event.flags & 0x2 != 0 {
            return Occurrence::Recurring {
                month: event.month as u16,
                year_min,
                year_max,
            };
        }

        if event.flags & 0x20000 != 0 {
            return Occurrence::EpisodeComplete;
        }

        return Occurrence::OneTime {
            month: event.month as u16,
            year_min,
            year_max,
        };
    }

    fn to_data(self) -> (u8, u32, i16, i16, i16) {
        let (month, year_min, year_max, flags) = match self {
            Occurrence::OneTime { month, year_min, year_max } => (month, year_min, year_max, 0u32),
            Occurrence::Recurring { month, year_min, year_max } => (month, year_min, year_max, 0x2),
            Occurrence::Triggered { year_min, year_max } => (0, year_min, year_max, 0x1),
            Occurrence::EpisodeComplete => (0, 1, 1, 0x20000),
        };

        let (fixed_time, min_time, max_time) = range_to_data(year_min, year_max);

        return (month as u8, flags, fixed_time, min_time, max_time);
    }
}

#[derive(PartialEq, Debug)]
pub struct EventToTrigger {
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

data_constants!(CityAttackOutcome<u16> {
    Unaffected = 0,
    Destroyed = 1,
    Conquered = 2,
});

data_constants!(MonumentReward<u16> {
    None = 0,
    Small = 1,
    Large = 2,
});

// Which of a Quest's 3 candidate heroes must complete it
data_constants!(Hero<u16> {
    Hero1 = 0,
    Hero2 = 1,
    Hero3 = 2,
});

data_constants!(MonsterSlot<u8> {
    Monster1 = 0,
    Monster2 = 1,
    IndependentMonster = 2,
});

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

fn range_from_data(fixed: i16, min: i16, max: i16) -> (u16, u16) {
    return if fixed != -1 {
        (fixed as u16, fixed as u16)
    } else {
        (min as u16, max as u16)
    };
}

fn range_to_data(min: u16, max: u16) -> (i16, i16, i16) {
    if min == max {
        return (min as i16, -1, -1);
    }
    return (-1, min as i16, max as i16);
}

fn items_from_data(event: &EventData, new_file_ver: bool) -> Vec<ResourceType> {
    return [event.first_item, event.second_item, event.third_item]
        .into_iter()
        .filter(|&id| id >= 0)
        .filter_map(|id| ResourceType::try_resolve_for_format(&(id as i8), new_file_ver))
        .collect();
}

fn items_to_data(resources: &[ResourceType]) -> (i16, i16, i16) {
    let mut ids = resources.iter().map(|resource| resource.value() as i16);

    return (ids.next().unwrap_or(-1), ids.next().unwrap_or(-1), ids.next().unwrap_or(-1));
}

fn with_occurrence(mut event: EventData, occurrence: &Occurrence) -> EventData {
    let (month, occurrence_flags, fixed_time, min_time, max_time) = occurrence.to_data();
    event.month = month;
    event.flags |= occurrence_flags;
    event.fixed_time = fixed_time;
    event.min_time = min_time;
    event.max_time = max_time;

    return event;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::adventure::Adventure;
    use crate::file_data::resource_id::ResourceId;
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
                        subtype: TradeChangeSubtype::SupplyDecrease {
                            city_min: 6,
                            city_max: 6,
                            resources: vec![ResourceType::Sculpture]
                        },
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::Orichalc, ResourceType::Sculpture, ResourceType::OliveOil,]
                        },
                        city_min: 1,
                        city_max: 1,
                        amount_min: 6,
                        amount_max: 10,
                        occurrence: Occurrence::Recurring {
                            month: 2,
                            year_min: 1,
                            year_max: 2
                        },
                        warning_months: 8
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::BlackMarble, ResourceType::Wood, ResourceType::Wine]
                        },
                        city_min: 2,
                        city_max: 2,
                        amount_min: 6,
                        amount_max: 10,
                        occurrence: Occurrence::Recurring {
                            month: 11,
                            year_min: 2,
                            year_max: 3
                        },
                        warning_months: 8
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::Fleece, ResourceType::Sculpture]
                        },
                        city_min: 3,
                        city_max: 4,
                        amount_min: 8,
                        amount_max: 12,
                        occurrence: Occurrence::Recurring {
                            month: 5,
                            year_min: 4,
                            year_max: 5
                        },
                        warning_months: 6
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::GodDisaster {
                            god: God::Hera,
                            duration: 6
                        },
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::Recurring {
                            month: 9,
                            year_min: 5,
                            year_max: 5
                        }
                    }),
                    Event::TradeChange(TradeChange {
                        subtype: TradeChangeSubtype::TradeShutsDown { city_min: 6, city_max: 6 },
                        occurrence: Occurrence::Recurring {
                            month: 1,
                            year_min: 8,
                            year_max: 8
                        }
                    }),
                    Event::TradeChange(TradeChange {
                        subtype: TradeChangeSubtype::TradeOpensUp { city_min: 6, city_max: 6 },
                        occurrence: Occurrence::Recurring {
                            month: 11,
                            year_min: 8,
                            year_max: 8
                        }
                    }),
                    Event::TradeChange(TradeChange {
                        subtype: TradeChangeSubtype::PriceIncrease {
                            resources: vec![ResourceType::Wood, ResourceType::Armor, ResourceId::Wine],
                            amount_min: 20,
                            amount_max: 20
                        },
                        occurrence: Occurrence::Recurring {
                            month: 3,
                            year_min: 12,
                            year_max: 12
                        }
                    }),
                    Event::WageIncrease(WageIncrease {
                        amount_min: 15,
                        amount_max: 15,
                        occurrence: Occurrence::OneTime {
                            month: 5,
                            year_min: 22,
                            year_max: 22
                        }
                    }),
                    Event::TradeChange(TradeChange {
                        subtype: TradeChangeSubtype::PriceDecrease {
                            resources: vec![ResourceType::Wood, ResourceType::Armor, ResourceId::Wine],
                            amount_min: 15,
                            amount_max: 15
                        },
                        occurrence: Occurrence::Recurring {
                            month: 6,
                            year_min: 26,
                            year_max: 26
                        }
                    }),
                    Event::GodInvasion(GodInvasion {
                        gods: vec![11],
                        occurrence: Occurrence::Recurring {
                            month: 10,
                            year_min: 3,
                            year_max: 4
                        }
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
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime {
                            month: 2,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime {
                            month: 5,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime {
                            month: 8,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime {
                            month: 0,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::OneTime {
                            month: 0,
                            year_min: 2,
                            year_max: 2
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime {
                            month: 0,
                            year_min: 3,
                            year_max: 3
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime {
                            month: 0,
                            year_min: 4,
                            year_max: 4
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 2,
                            year_max: 2
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 3,
                            year_max: 3
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 4,
                            year_max: 4
                        }
                    }),
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityAppears,
                        city_min: 16,
                        city_max: 16,
                        occurrence: Occurrence::OneTime {
                            month: 1,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityAppears,
                        city_min: 17,
                        city_max: 17,
                        occurrence: Occurrence::OneTime {
                            month: 8,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityAppears,
                        city_min: 18,
                        city_max: 18,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 2,
                            year_max: 2
                        }
                    }),
                    Event::Invasion(Invasion {
                        city_min: 13,
                        city_max: 15,
                        warships: 0,
                        amount_min: 24,
                        amount_max: 48,
                        marker_min: 9,
                        marker_max: 11,
                        occurrence: Occurrence::Recurring {
                            month: 5,
                            year_min: 2,
                            year_max: 4
                        },
                        warning_months: 4
                    }),
                    Event::Invasion(Invasion {
                        city_min: 16,
                        city_max: 18,
                        warships: 0,
                        amount_min: 18,
                        amount_max: 48,
                        marker_min: 1,
                        marker_max: 8,
                        occurrence: Occurrence::Recurring {
                            month: 8,
                            year_min: 2,
                            year_max: 5
                        },
                        warning_months: 4
                    }),
                    Event::Invasion(Invasion {
                        city_min: 6,
                        city_max: 12,
                        warships: 0,
                        amount_min: 8,
                        amount_max: 64,
                        marker_min: 1,
                        marker_max: 11,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 1,
                            year_max: 8
                        },
                        warning_months: 2
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 2,
                        city_max: 5,
                        occurrence: Occurrence::Recurring {
                            month: 4,
                            year_min: 1,
                            year_max: 4
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 2,
                        city_max: 5,
                        occurrence: Occurrence::OneTime {
                            month: 1,
                            year_min: 0,
                            year_max: 2
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 2,
                        city_max: 5,
                        occurrence: Occurrence::OneTime {
                            month: 8,
                            year_min: 0,
                            year_max: 1
                        }
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::GreekCityTerrorized { monster_index: 0 },
                        city_min: 3,
                        city_max: 3,
                        outcome: CityAttackOutcome::Unaffected,
                        occurrence: Occurrence::OneTime {
                            month: 1,
                            year_min: 0,
                            year_max: 1
                        },
                        warning_months: 4
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::GreekCityTerrorized { monster_index: 1 },
                        city_min: 4,
                        city_max: 4,
                        outcome: CityAttackOutcome::Unaffected,
                        occurrence: Occurrence::OneTime {
                            month: 5,
                            year_min: 1,
                            year_max: 2
                        },
                        warning_months: 4
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::GreekCityTerrorized { monster_index: 2 },
                        city_min: 5,
                        city_max: 5,
                        outcome: CityAttackOutcome::Unaffected,
                        occurrence: Occurrence::OneTime {
                            month: 10,
                            year_min: 2,
                            year_max: 3
                        },
                        warning_months: 4
                    }),
                    Event::Invasion(Invasion {
                        city_min: 10,
                        city_max: 12,
                        warships: 8,
                        amount_min: 48,
                        amount_max: 72,
                        marker_min: 1,
                        marker_max: 11,
                        occurrence: Occurrence::Recurring {
                            month: 0,
                            year_min: 4,
                            year_max: 6
                        },
                        warning_months: 5
                    })
                ]
            );

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(
                colony_episode.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::OneTime {
                            month: 0,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 12 },
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime {
                            month: 0,
                            year_min: 2,
                            year_max: 2
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime {
                            month: 9,
                            year_min: 0,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 11,
                        city_max: 11,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityAppears,
                        city_min: 21,
                        city_max: 21,
                        occurrence: Occurrence::OneTime {
                            month: 2,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityAppears,
                        city_min: 20,
                        city_max: 20,
                        occurrence: Occurrence::OneTime {
                            month: 5,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::Invasion(Invasion {
                        city_min: 21,
                        city_max: 21,
                        warships: 0,
                        amount_min: 12,
                        amount_max: 12,
                        marker_min: 1,
                        marker_max: 3,
                        occurrence: Occurrence::Recurring {
                            month: 2,
                            year_min: 1,
                            year_max: 1
                        },
                        warning_months: 2
                    }),
                    Event::Invasion(Invasion {
                        city_min: 1,
                        city_max: 8,
                        warships: 0,
                        amount_min: 24,
                        amount_max: 32,
                        marker_min: 1,
                        marker_max: 3,
                        occurrence: Occurrence::Recurring {
                            month: 9,
                            year_min: 2,
                            year_max: 4
                        },
                        warning_months: 4
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 14,
                        city_max: 14,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 15,
                        city_max: 15,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::OneTime {
                            month: 10,
                            year_min: 0,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 14,
                        city_max: 14,
                        occurrence: Occurrence::OneTime {
                            month: 8,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 15,
                        city_max: 15,
                        occurrence: Occurrence::OneTime {
                            month: 5,
                            year_min: 2,
                            year_max: 2
                        }
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker_min: 1,
                        marker_max: 2,
                        occurrence: Occurrence::OneTime {
                            month: 11,
                            year_min: 0,
                            year_max: 1
                        }
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker_min: 3,
                        marker_max: 4,
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 1,
                            year_max: 3
                        }
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker_min: 5,
                        marker_max: 5,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 2,
                            year_max: 4
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                        city_min: 20,
                        city_max: 20,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                        city_min: 21,
                        city_max: 21,
                        occurrence: Occurrence::EpisodeComplete
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
                        subtype: CityStatusChangeSubtype::CityConquered { attacking_city: 5 }, // Tenedos
                        city_min: 10,                                                          // Troy
                        city_max: 10,
                        occurrence: Occurrence::OneTime {
                            month: 2,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::Recurring {
                            month: 3,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityDisappears,
                        city_min: 10,
                        city_max: 10,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryDecline {
                            amount_min: 5,
                            amount_max: 5
                        },
                        city_min: 9, // Ismarus
                        city_max: 9,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::EconomicDecline {
                            amount_min: 3,
                            amount_max: 3
                        },
                        city_min: 9,
                        city_max: 9,
                        occurrence: Occurrence::OneTime {
                            month: 7,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                        city_min: 4, // Sparta
                        city_max: 4,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityDisappears,
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesInactive,
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityAppears,
                        city_min: 12,
                        city_max: 12,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesActive,
                        city_min: 12,
                        city_max: 12,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryDecline {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 8,
                        city_max: 8,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityDisappears,
                        city_min: 5,
                        city_max: 5,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                ]
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.events,
                vec![
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::Food, ResourceType::OliveOil, ResourceType::Wine]
                        },
                        city_min: 6,
                        city_max: 7,
                        amount_min: 5,
                        amount_max: 20,
                        occurrence: Occurrence::OneTime {
                            month: 1,
                            year_min: 0,
                            year_max: 0
                        },
                        warning_months: 2
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::Wine, ResourceType::Food, ResourceType::OliveOil]
                        },
                        city_min: 6,
                        city_max: 7,
                        amount_min: 5,
                        amount_max: 20,
                        occurrence: Occurrence::Recurring {
                            month: 2,
                            year_min: 1,
                            year_max: 3
                        },
                        warning_months: 2
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::OliveOil, ResourceType::Wine, ResourceType::Food]
                        },
                        city_min: 6,
                        city_max: 7,
                        amount_min: 5,
                        amount_max: 20,
                        occurrence: Occurrence::OneTime {
                            month: 11,
                            year_min: 1,
                            year_max: 3
                        },
                        warning_months: 2
                    }),
                ]
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.events,
                vec![
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::Food, ResourceType::OliveOil, ResourceType::Wine]
                        },
                        city_min: 6,
                        city_max: 7,
                        amount_min: 10,
                        amount_max: 30,
                        occurrence: Occurrence::OneTime {
                            month: 6,
                            year_min: 0,
                            year_max: 0
                        },
                        warning_months: 2
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::Wine, ResourceType::Food, ResourceType::OliveOil]
                        },
                        city_min: 6,
                        city_max: 7,
                        amount_min: 10,
                        amount_max: 30,
                        occurrence: Occurrence::Recurring {
                            month: 1,
                            year_min: 1,
                            year_max: 3
                        },
                        warning_months: 2
                    }),
                    Event::GoodsRequest(GoodsRequest {
                        subtype: GoodsRequestSubtype::GeneralRequest {
                            resources: vec![ResourceType::OliveOil, ResourceType::Wine, ResourceType::Food]
                        },
                        city_min: 6,
                        city_max: 7,
                        amount_min: 10,
                        amount_max: 30,
                        occurrence: Occurrence::Recurring {
                            month: 8,
                            year_min: 1,
                            year_max: 3
                        },
                        warning_months: 2
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityDisappears,
                        city_min: 12,
                        city_max: 12,
                        occurrence: Occurrence::OneTime {
                            month: 0,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesActive,
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::GodInvasion(GodInvasion {
                        gods: vec![God::Poseidon.value() as u16],
                        occurrence: Occurrence::Triggered { year_min: 0, year_max: 0 }
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::Earthquake,
                        marker_min: 1,
                        marker_max: 3,
                        occurrence: Occurrence::OneTime {
                            month: 2,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::LavaFlow,
                        marker_min: 4,
                        marker_max: 5,
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::Disaster(Disaster {
                        disaster_type: DisasterSubtype::TidalWave { permanent: false },
                        marker_min: 6,
                        marker_max: 8,
                        occurrence: Occurrence::OneTime {
                            month: 1,
                            year_min: 1,
                            year_max: 1
                        }
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        subtype: MonsterInvasionSubtype::MonsterUnleashed {
                            occurrence: Occurrence::OneTime {
                                month: 0,
                                year_min: 1,
                                year_max: 1
                            }
                        },
                        monsters: vec![MonsterSlot::Monster1],
                        monument: false,
                        target: [MonsterTarget::Troops, MonsterTarget::Sea, MonsterTarget::Sea],
                        aggression: 2,
                        next_event: EventToTrigger {
                            event_id: 5,
                            trigger_type: TriggerType::DirectResult,
                        },
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::MilitaryBuildup {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::OneTime {
                            month: 7,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::EconomicProsperity {
                            amount_min: 1,
                            amount_max: 1
                        },
                        city_min: 4,
                        city_max: 4,
                        occurrence: Occurrence::OneTime {
                            month: 8,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                ]
            );

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(
                episode_5.events,
                vec![
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 6,
                        city_max: 6,
                        occurrence: Occurrence::OneTime {
                            month: 1,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesRival,
                        city_min: 7,
                        city_max: 7,
                        occurrence: Occurrence::OneTime {
                            month: 1,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::GodInvasion(GodInvasion {
                        gods: vec![God::Poseidon.value() as u16, God::Apollo.value() as u16],
                        occurrence: Occurrence::Recurring {
                            month: 7,
                            year_min: 2,
                            year_max: 4
                        }
                    }),
                    Event::Invasion(Invasion {
                        city_min: 6,
                        city_max: 7,
                        warships: 2,
                        amount_min: 5,
                        amount_max: 10,
                        marker_min: 9,
                        marker_max: 16,
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 1,
                            year_max: 2
                        },
                        warning_months: 4
                    }),
                    Event::Invasion(Invasion {
                        city_min: 6,
                        city_max: 7,
                        warships: 6,
                        amount_min: 20,
                        amount_max: 40,
                        marker_min: 9,
                        marker_max: 16,
                        occurrence: Occurrence::Recurring {
                            month: 10,
                            year_min: 2,
                            year_max: 5
                        },
                        warning_months: 4
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::CityUnderAttack { attacking_city: 6 },
                        city_min: 1,
                        city_max: 2,
                        outcome: CityAttackOutcome::Conquered,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 6,
                            year_max: 6
                        },
                        warning_months: 4
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::CityUnderAttack { attacking_city: 7 },
                        city_min: 1,
                        city_max: 2,
                        outcome: CityAttackOutcome::Conquered,
                        occurrence: Occurrence::OneTime {
                            month: 9,
                            year_min: 8,
                            year_max: 8
                        },
                        warning_months: 4
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        subtype: MonsterInvasionSubtype::MonsterInvades {
                            occurrence: Occurrence::OneTime {
                                month: 0,
                                year_min: 0,
                                year_max: 0
                            },
                            warning_months: 0
                        },
                        monsters: vec![MonsterSlot::IndependentMonster],
                        monument: true,
                        target: [MonsterTarget::Random, MonsterTarget::Random, MonsterTarget::Random],
                        aggression: 1,
                        next_event: EventToTrigger {
                            event_id: -1,
                            trigger_type: TriggerType::DirectResult,
                        },
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::CityUnderAttack { attacking_city: 7 },
                        city_min: 6,
                        city_max: 6,
                        outcome: CityAttackOutcome::Conquered,
                        occurrence: Occurrence::Recurring {
                            month: 0,
                            year_min: 2,
                            year_max: 10
                        },
                        warning_months: 4
                    }),
                    Event::MilitaryRequest(MilitaryRequest {
                        subtype: MilitaryRequestSubtype::CityUnderAttack { attacking_city: 6 },
                        city_min: 7,
                        city_max: 7,
                        outcome: CityAttackOutcome::Conquered,
                        occurrence: Occurrence::Recurring {
                            month: 1,
                            year_min: 2,
                            year_max: 10
                        },
                        warning_months: 4
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
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::Gift(Gift {
                        city_min: 0,
                        city_max: 0,
                        resource: ResourceType::OliveOil,
                        amount_min: 32,
                        amount_max: 32,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        subtype: MonsterInvasionSubtype::MonsterInvades {
                            occurrence: Occurrence::OneTime {
                                month: 5,
                                year_min: 3,
                                year_max: 3
                            },
                            warning_months: 12
                        },
                        monsters: vec![MonsterSlot::IndependentMonster],
                        monument: true,
                        target: [MonsterTarget::Random, MonsterTarget::Random, MonsterTarget::Random],
                        aggression: 0,
                        next_event: EventToTrigger {
                            event_id: -1,
                            trigger_type: TriggerType::DirectResult,
                        },
                    }),
                ]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.events,
                vec![
                    Event::GodInvasion(GodInvasion {
                        gods: vec![God::Apollo.value() as u16],
                        occurrence: Occurrence::Triggered { year_min: 1, year_max: 1 }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityAppears,
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityBecomesActive,
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::OneTime {
                            month: 5,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::Gift(Gift {
                        city_min: 0,
                        city_max: 0,
                        resource: ResourceType::Fish,
                        amount_min: 32,
                        amount_max: 32,
                        occurrence: Occurrence::OneTime {
                            month: 3,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::Gift(Gift {
                        city_min: 0,
                        city_max: 0,
                        resource: ResourceType::OliveOil,
                        amount_min: 32,
                        amount_max: 32,
                        occurrence: Occurrence::OneTime {
                            month: 4,
                            year_min: 0,
                            year_max: 0
                        }
                    }),
                    Event::CityStatusChange(CityStatusChange {
                        subtype: CityStatusChangeSubtype::CityDisappears,
                        city_min: 13,
                        city_max: 13,
                        occurrence: Occurrence::EpisodeComplete
                    }),
                    Event::MonsterInvasion(MonsterInvasion {
                        subtype: MonsterInvasionSubtype::MonsterUnleashed {
                            occurrence: Occurrence::OneTime {
                                month: 10,
                                year_min: 2,
                                year_max: 2
                            }
                        },
                        monsters: vec![MonsterSlot::Monster2],
                        monument: true,
                        target: [MonsterTarget::Random, MonsterTarget::Random, MonsterTarget::Random],
                        aggression: 0,
                        next_event: EventToTrigger {
                            event_id: 0,
                            trigger_type: TriggerType::DirectResult
                        },
                    })
                ]
            );
        }

        return Ok(());
    }
}
