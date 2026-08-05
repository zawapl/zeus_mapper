use crate::prelude::MapData;
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
            sprite: map_data.sprite.clone(),
            root_offset: map_data.root_offset.clone(),
            terrain: map_data.terrain.clone(),
            tile_size: map_data.tile_size.clone(),
            random: map_data.random.clone(),
            meadow: map_data.meadow.clone(),
            scrub: map_data.scrub.clone(),
            elevation: map_data.elevation.clone(),
            entry_point: (map_data.scenario_data.entry_x, map_data.scenario_data.entry_y),
            exit_point: (map_data.scenario_data.exit_x, map_data.scenario_data.exit_y),
            fishing_spots: zip_positions(&map_data.scenario_data.fish_x, &map_data.scenario_data.fish_y),
            wolf_spawn: zip_positions(&map_data.scenario_data.wolf_x, &map_data.scenario_data.wolf_y),
        };
    }
}

fn zip_positions<const N: usize>(a: &[u16; N], b: &[u16; N]) -> Vec<(u16, u16)> {
    return a.iter().zip(b).map(|(&x, &y)| (x, y)).collect();
}
