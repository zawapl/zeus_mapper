use crate::adventure::CityId;
use crate::adventure::God;
use crate::adventure::Monster;
use crate::adventure::episode_goals::QuestType;
use crate::adventure::episode_goals::ResourceType;
use crate::differ::default_differ_impl;
use my_macros::LogDifferences;

#[derive(LogDifferences)]
pub struct Event {
    pub details: EventDetails,
    pub occurrence: Occurrence,
    pub warning_months: u8,
}

#[derive(PartialEq, Debug)]
pub enum EventDetails {
    GeneralGoodsRequest(CityId, ResourceType, u16),
    FestivalGoodsRequest(CityId, ResourceType, u16, God),
    ConstructionGoodsRequest(CityId, ResourceType, u16),
    FamineGoodsRequest(CityId, u16),
    FinancialWoesGoodsRequest(CityId, u16),
    OffensiveMilitaryRequest(CityId, CityId, CityAttackOutcome),
    DefensiveMilitaryRequest(CityId, CityId, CityAttackOutcome),
    MonsterMilitaryRequest(CityId, Monster, CityAttackOutcome),
    Gift(CityId, ResourceType, u16),
    Quest(God, QuestType, CityId, MonumentReward, EventToTrigger),
    Invasion(CityId, u16, [u8; 2]),
    MonsterInCity(MonsterInvasion),
    MonsterUnleashed(MonsterInvasion),
    MonsterInvades(MonsterInvasion),
    GodInvasion(God),
    Disaster(Disaster),
    WageIncrease(u8),
    WageDecrease(u8),
    TradeDemandChange(CityId, ResourceType, IsIncrease, u8),
    TradeSupplyChange(CityId, ResourceType, IsIncrease, u8),
    PriceChange(ResourceType, IsIncrease, u8),
    TradingStatusChange(CityId, bool),
    RivalBecomesAlly(CityId),
    CityBecomesAlly(CityId),
    CityBecomesVassal(CityId),
    CityDestroyedByGod(CityId, God),
    MilitaryChange(CityId, IsIncrease, u8),
    EconomyChange(CityId, IsIncrease, u8),
    CityActivityChange(CityId, bool),
    CityVisibilityChange(CityId, bool),
    RebellionOver(CityId),
    CityConquered(CityId, CityId),
    RivalArmyAway(CityId),
}

default_differ_impl!(EventDetails);

#[derive(PartialEq, Debug)]
pub enum CityAttackOutcome {
    Unaffected,
    Destroyed,
    Conquered,
}

#[derive(PartialEq, Debug)]
pub struct MonsterInvasion {
    pub monster: Monster,
    pub monument: bool,
    pub target: Vec<MonsterTarget>,
    pub aggression: u8,
    pub event_on_success: EventToTrigger,
}

#[derive(PartialEq, Debug)]
pub enum MonsterTarget {
    Food,
    Sea,
    Industry,
    Military,
    Money,
    Troops,
    Common,
    Aesthetic,
    Mythological,
    Best,
    Random,
}

#[derive(PartialEq, Debug)]
pub struct Disaster {
    pub disaster_type: DisasterSubtype,
    pub marker: u8,
    pub permanent: bool,
}

#[derive(PartialEq, Debug)]
pub enum DisasterSubtype {
    Earthquake,
    LavaFlow,
    TidalWave,
    Landslide,
    SinkLand,
}

#[derive(PartialEq, Debug)]
pub struct EventToTrigger {
    pub event_id: u8,
    pub trigger_type: TriggerType,
}

#[derive(PartialEq, Debug)]
pub enum TriggerType {
    DirectResult,
    Incidental,
    InSpiteOf,
    NoCause,
    Cyclical,
    Specific,
    Auto,
}

#[derive(PartialEq, Debug)]
pub enum Occurrence {
    OneTime(Month, BetweenYears),
    Repeating(Month, BetweenYears),
    Triggered(BetweenYears),
}

default_differ_impl!(Occurrence);

#[derive(PartialEq, Debug)]
pub struct BetweenYears(u16, u16);

pub type Month = u16;
pub type IsIncrease = bool;

#[derive(PartialEq, Debug)]
pub enum MonumentReward {
    None,
    Small,
    Large,
}
