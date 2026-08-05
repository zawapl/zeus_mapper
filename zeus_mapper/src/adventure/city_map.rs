use crate::prelude::MapData;
use crate::utils::boxed_array::BoxedArray;
use my_macros::LogDifferences;

#[derive(LogDifferences)]
pub struct CityMap {
    pub map_size: u32,
    pub tropical: bool,
    pub sprite: Vec<u32>,
    pub root_offset: Vec<u8>,
    pub terrain: Vec<u32>,
    pub tile_size: Vec<u8>,
    pub random: Vec<u8>,
    pub meadow: Vec<u8>,
    pub scrub: Vec<u8>,
    pub elevation: Vec<u8>,
    pub entry_point: (u16, u16),
    pub exit_point: (u16, u16),
    pub fishing_spots: Vec<(u16, u16)>,
    pub wolf_spawn: Vec<(u16, u16)>,
}

impl CityMap {
    pub fn from_map_data(map_data: &MapData) -> Self {
        return CityMap {
            map_size: map_data.scenario_data.map_size,
            tropical: map_data.scenario_data.tropical != 0,
            sprite: map_data.sprite.to_vec(),
            root_offset: map_data.root_offset.to_vec(),
            terrain: map_data.terrain.to_vec(),
            tile_size: map_data.tile_size.to_vec(),
            random: map_data.random.to_vec(),
            meadow: map_data.meadow.to_vec(),
            scrub: map_data.scrub.to_vec(),
            elevation: map_data.elevation.to_vec(),
            entry_point: (map_data.scenario_data.entry_x, map_data.scenario_data.entry_y),
            exit_point: (map_data.scenario_data.exit_x, map_data.scenario_data.exit_y),
            fishing_spots: zip_positions(&map_data.scenario_data.fish_x, &map_data.scenario_data.fish_y),
            wolf_spawn: zip_positions(&map_data.scenario_data.wolf_x, &map_data.scenario_data.wolf_y),
        };
    }

    pub fn to_map_data(&self) -> MapData {
        let mut map_data = MapData::default();

        map_data.version_2 = 33;

        map_data.scenario_data.map_size = self.map_size;
        map_data.scenario_data.tropical = if self.tropical { 1 } else { 0 };
        map_data.sprite = BoxedArray::from_vec(self.sprite.clone());
        map_data.root_offset = BoxedArray::from_vec(self.root_offset.clone());
        map_data.terrain = BoxedArray::from_vec(self.terrain.clone());
        map_data.tile_size = BoxedArray::from_vec(self.tile_size.clone());
        map_data.random = BoxedArray::from_vec(self.random.clone());
        map_data.meadow = BoxedArray::from_vec(self.meadow.clone());
        map_data.scrub = BoxedArray::from_vec(self.scrub.clone());
        map_data.elevation = BoxedArray::from_vec(self.elevation.clone());
        map_data.scenario_data.entry_x = self.entry_point.0;
        map_data.scenario_data.entry_y = self.entry_point.1;
        map_data.scenario_data.exit_x = self.exit_point.0;
        map_data.scenario_data.exit_y = self.exit_point.1;

        let (fish_x, fish_y) = unzip_positions(&self.fishing_spots);
        map_data.scenario_data.fish_x = fish_x;
        map_data.scenario_data.fish_y = fish_y;

        let (wolf_x, wolf_y) = unzip_positions(&self.wolf_spawn);
        map_data.scenario_data.wolf_x = wolf_x;
        map_data.scenario_data.wolf_y = wolf_y;

        return map_data;
    }
}

fn zip_positions<const N: usize>(a: &[u16; N], b: &[u16; N]) -> Vec<(u16, u16)> {
    return a.iter().zip(b).map(|(&x, &y)| (x, y)).collect();
}

fn unzip_positions<const N: usize>(positions: &[(u16, u16)]) -> ([u16; N], [u16; N]) {
    let mut x = [0u16; N];
    let mut y = [0u16; N];

    for (i, &(px, py)) in positions.iter().take(N).enumerate() {
        x[i] = px;
        y[i] = py;
    }

    return (x, y);
}
