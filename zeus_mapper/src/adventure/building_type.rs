use crate::constants::data_constant::data_constants;
use crate::prelude::DataConstant;
use crate::utils::boxed_array::BoxedArray;

data_constants!(BuildingType<u16> {
    Mint = 1,
    HuntingLodge = 2,
    OlivePress = 3,
    GrandAgora = 4,
    // Unused = 5,
    Corral = 6,
    Granary = 7,
    CommonAgora = 8,
    Storehouse = 9,
    TradeBuilding = 10,
    CultureBuildingUnique = 11,
    CultureBDestination = 12,
    CultureBuildingBasic = 13,
    Winery = 14,
    SculptureStudio = 15,
    CultureADestination = 16,
    CultureASource = 17,
    Fountain = 18,
    HorseRanch = 19,
    CultureBSource = 20,
    Maintenance = 21,
    Infirmary = 22,
    TaxOffice = 23,
    Watchpost = 24,
    Palace = 25,
    HeroHall = 26,
    RoadBlock = 27,
    WaterCrossing = 28,
    Column = 29,
    Park = 30,
    Avenue = 31,
    Boulevards = 32,
    Wall = 33,
    Tower = 34,
    Gatehouse = 35,
    Bench = 36,
    Gazebo = 37,
    FlowerGarden = 38,
    HedgeMaze = 39,
    FishPond = 40,
    Armory = 41,
    Wharf = 42,
    EliteHousing = 43,
    Hippodrome = 44,
    ChariotFactory = 45,
    TallObelisk = 46,
    Sundial = 47,
    Topiary = 48,
    Spring = 49,
    StoneCircle = 50,
    ShortObelisk = 51,
    WaterPark = 52,
    LeapingDolphin = 53,
    Orrery = 54,
    ShellGarden = 55,
    Baths = 56,
    BirdsBath = 57,
});

impl BuildingType {
    pub(crate) fn vec_from_data(data: &[u16]) -> Vec<Self> {
        return data
            .iter()
            .enumerate()
            .filter(|(_, v)| v > &&0)
            .filter_map(|(i, _)| BuildingType::try_resolve(&(i as u16)))
            .collect();
    }

    pub(crate) fn vec_to_data(buildings: &[BuildingType]) -> BoxedArray<u16, 100> {
        let mut data = [0; 100];

        for building in buildings {
            let index = building.value() as usize;
            if index < data.len() {
                data[index] = 1;
            }
        }

        return BoxedArray::from_vec(data.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::adventure::Adventure;
    use std::io::Result;

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.building_allowed,
                all_buildings_except([
                    BuildingType::Corral,
                    BuildingType::HeroHall,
                    BuildingType::Armory,
                    BuildingType::Wharf,
                    BuildingType::ChariotFactory,
                ])
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.building_allowed, all_buildings_except([BuildingType::Corral]));

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(
                colony_episode.building_allowed,
                all_buildings_except([
                    BuildingType::FishPond,
                    BuildingType::FlowerGarden,
                    BuildingType::Gatehouse,
                    BuildingType::HeroHall,
                    BuildingType::HorseRanch,
                    BuildingType::Mint,
                    BuildingType::SculptureStudio,
                    BuildingType::Tower,
                    BuildingType::Wharf,
                    BuildingType::Wall,
                    BuildingType::Hippodrome,
                    BuildingType::Corral,
                    BuildingType::ChariotFactory
                ])
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
                episode_1.building_allowed,
                all_buildings_except([
                    BuildingType::Mint,
                    BuildingType::GrandAgora,
                    BuildingType::SculptureStudio,
                    BuildingType::HorseRanch,
                    BuildingType::HeroHall,
                    BuildingType::Wall,
                    BuildingType::Tower,
                    BuildingType::Gatehouse,
                    BuildingType::FishPond,
                    BuildingType::Armory,
                    BuildingType::Wharf,
                    BuildingType::EliteHousing,
                ])
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.building_allowed,
                all_buildings_except([
                    BuildingType::Mint,
                    BuildingType::HorseRanch,
                    BuildingType::HeroHall,
                    BuildingType::Wharf
                ])
            );

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.building_allowed,
                all_buildings_except([
                    BuildingType::Mint,
                    BuildingType::HorseRanch,
                    BuildingType::HeroHall,
                    BuildingType::Wharf
                ])
            );

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(episode_4.building_allowed, all_buildings_except([BuildingType::Mint]));

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(episode_5.building_allowed, all_buildings_except([BuildingType::Mint]));

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.building_allowed,
                all_buildings_except([BuildingType::Mint, BuildingType::Winery, BuildingType::Armory])
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.building_allowed,
                all_buildings_except([
                    BuildingType::CultureBuildingUnique,
                    BuildingType::Winery,
                    BuildingType::SculptureStudio,
                    BuildingType::HorseRanch,
                ])
            );
        }

        return Ok(());
    }

    fn all_buildings_except<const N: usize>(excluded: [BuildingType; N]) -> Vec<BuildingType> {
        return BuildingType::values()
            .iter()
            .filter(|building_type| !excluded.contains(building_type))
            .cloned()
            .collect();
    }
}
