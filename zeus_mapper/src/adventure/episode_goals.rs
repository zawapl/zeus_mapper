use crate::adventure::God;
use crate::adventure::MonetaryAmount;
use crate::adventure::Monster;
use crate::constants::data_constant::data_constants;
use crate::differ::default_differ_impl;
use my_macros::LogDifferences;

#[derive(PartialEq, Debug)]
pub enum EpisodeGoal {
    Population(u32),
    Treasury(MonetaryAmount),
    Sanctuary(God),
    Sanctuaries(u8),
    Army(ArmyType, u32),
    Quest(God, QuestType),
    Slay(Monster),
    YearlyProduction(ResourceType, u32),
    Rule(u8),
    YearlyProfit(MonetaryAmount),
    Housing(HouseLevel, u32),
    TradingPartners(u32),
    SetAsideGoods(ResourceType, u32),
    Pyramid(PyramidType),
    Pyramids(u8),
    Hippodrome(u32),
}

default_differ_impl!(EpisodeGoal);

#[derive(LogDifferences)]
pub struct OwnCitySettings {
    pub traded_resources: Vec<TradedGoods>,
    pub military_strength: u8,
    pub economy_strength: u8,
    pub favour: u16,
}

#[derive(LogDifferences)]
pub struct PyramidSetting {
    pub pyramid_type: PyramidType,
    pub allowed: bool,
}

#[derive(LogDifferences)]
pub struct TradedGoods {
    pub resource_id: ResourceType,
    pub amount: u8,
    pub buying: bool,
}

#[derive(PartialEq, Debug)]
pub enum QuestType {
    Type0,
    Type1,
}

data_constants!(ResourceType<u8>{
    Urchin = 1,
    Fish = 2,
    Meat = 3,
    Cheese = 4,
    Carrot = 5,
    Onion = 6,
    Wheat = 7,
    Orange = 8,
    Wood = 9,
    Bronze = 10,
    Marble = 11,
    Grape = 12,
    Olive = 13,
    Fleece = 14,
    Horse = 15,
    BlackMarble = 16,
    Orichalc = 17,
    Armor = 18,
    Sculpture = 19,
    OliveOil = 20,
    Wine = 21,
    Chariot = 22,
});

#[derive(PartialEq, Debug)]
pub enum PyramidType {
    ModestPyramid([PyramidColor; 2]),
    Pyramid([PyramidColor; 3]),
    GreatPyramid([PyramidColor; 4]),
    MajesticPyramid([PyramidColor; 5]),
    SmallMonumentToTheSky([PyramidColor; 3]),
    MonumentToTheSky([PyramidColor; 3]),
    GrantMonumentToTheSky([PyramidColor; 4]),
    MinorShrine(God, [PyramidColor; 2]),
    Shrine(God, [PyramidColor; 3]),
    MajorShrine(God, [PyramidColor; 3]),
    PyramidOfThePantheon([PyramidColor; 3]),
    AltarOfOlympus([PyramidColor; 4]),
    TempleOfOlympus([PyramidColor; 3]),
    ObservatoryKosmika([PyramidColor; 3]),
    MuseumAtlantika([PyramidColor; 2]),
}

default_differ_impl!(PyramidType);

#[derive(PartialEq, Debug)]
pub enum PyramidColor {
    White,
    Black,
}

#[derive(PartialEq, Debug)]
pub enum ArmyType {
    Untrained,
    Elite,
    EliteUpgraded,
    Naval,
}

#[derive(PartialEq, Debug)]
pub enum HouseLevel {
    Common(u8),
    Elite(u8),
}
