use crate::constants::data_constant::data_constants;
use crate::prelude::DataConstant;
use crate::utils::boxed_array::BoxedArray;

data_constants!(BuildingType<u16> {
    Mint = 1,
    HuntingLodge = 2,
    OlivePress = 3,
    GrandAgora = 4,
    Reserved = 5,
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
