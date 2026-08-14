use crate::prelude::MapData;
use crate::prelude::RealEpisodeData;
use crate::utils::boxed_array::BoxedArray;
use my_macros::LogDifferences;

#[derive(LogDifferences, PartialEq, Debug)]
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
    pub fishing_spots: Vec<Option<(u16, u16)>>,
    pub wolf_spawn: Vec<Option<(u16, u16)>>,
    pub urchin_spawn: Vec<Option<(u16, u16)>>,
    pub invasion_points: Vec<Option<(u16, u16)>>,
    pub deer_spawn: Vec<Option<(u16, u16)>>,
    pub disaster_points: Vec<Option<(u16, u16)>>,
    pub boar_spawn: Vec<Option<(u16, u16)>>,
    pub monster_spawn: Vec<Option<(u16, u16)>>,
    pub disembark_points: Vec<Option<(u16, u16)>>,
    pub landslide_spawn: Vec<Option<(u16, u16)>>,
    pub earthquake_area: Option<(u16, u16)>,
    pub river_entry: Option<(u16, u16)>,
    pub river_exit: Option<(u16, u16)>,
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
            urchin_spawn: zip_positions(&map_data.scenario_data.urchin_x, &map_data.scenario_data.urchin_y),
            invasion_points: zip_positions(&map_data.scenario_data.invasion_x, &map_data.scenario_data.invasion_y),
            deer_spawn: zip_positions(&map_data.scenario_data.deer_x, &map_data.scenario_data.deer_y),
            disaster_points: zip_positions(&map_data.scenario_data.disaster_x, &map_data.scenario_data.disaster_y),
            boar_spawn: zip_positions(&map_data.scenario_data.boar_x, &map_data.scenario_data.boar_y),
            monster_spawn: zip_positions_u32(&map_data.scenario_data.monster_x, &map_data.scenario_data.monster_y),
            disembark_points: zip_positions(&map_data.scenario_data.disembark_x, &map_data.scenario_data.disembark_y),
            landslide_spawn: zip_positions(&map_data.scenario_data.landslide_x, &map_data.scenario_data.landslide_y),
            earthquake_area: zip_position(map_data.scenario_data.earthquake_area[0], map_data.scenario_data.earthquake_area[1]),
            river_entry: zip_position(map_data.scenario_data.river_entry_x, map_data.scenario_data.river_entry_y),
            river_exit: zip_position(map_data.scenario_data.river_exit_x, map_data.scenario_data.river_exit_y),
        };
    }

    pub fn to_map_data(&self) -> MapData {
        let (fish_x, fish_y) = unzip_positions(&self.fishing_spots);
        let (wolf_x, wolf_y) = unzip_positions(&self.wolf_spawn);
        let (urchin_x, urchin_y) = unzip_positions(&self.urchin_spawn);
        let (invasion_x, invasion_y) = unzip_positions(&self.invasion_points);
        let (deer_x, deer_y) = unzip_positions(&self.deer_spawn);
        let (disaster_x, disaster_y) = unzip_positions(&self.disaster_points);
        let (boar_x, boar_y) = unzip_positions(&self.boar_spawn);
        let (monster_x, monster_y) = unzip_positions_u32(&self.monster_spawn);
        let (disembark_x, disembark_y) = unzip_positions(&self.disembark_points);
        let (landslide_x, landslide_y) = unzip_positions(&self.landslide_spawn);
        let earthquake_area = unzip_position(self.earthquake_area);
        let (river_entry_x, river_entry_y) = unzip_position(self.river_entry);
        let (river_exit_x, river_exit_y) = unzip_position(self.river_exit);

        let scenario_data = RealEpisodeData {
            start_date: -500,
            constant_1_0x00: Default::default(),
            months_elapsed: 0,
            constant_2_0x00: Default::default(),
            starting_cash: 1000,
            unknown_1: Default::default(),
            map_size: self.map_size,
            unknown_2: Default::default(),
            text_buffer_1: "Brief description".to_owned(),
            text_buffer_2: "Brief description of this episode, for players. History, aims and tips etc.".to_owned(),
            civilization: 0,
            wolf_x,
            wolf_y,
            fish_x,
            fish_y,
            urchin_x,
            urchin_y,
            constant_3_0x00: Default::default(),
            invasion_x,
            invasion_y,
            panhellenic_games: 65535,
            colonies_done: 0,
            deer_x,
            deer_y,
            unknown_3: Default::default(),
            earthquake_area: [earthquake_area.0, earthquake_area.1],
            entry_x: self.entry_point.0,
            entry_y: self.entry_point.1,
            exit_x: self.exit_point.0,
            exit_y: self.exit_point.1,
            disaster_x,
            disaster_y,
            river_entry_x,
            river_entry_y,
            river_exit_x,
            river_exit_y,
            unknown_4: Default::default(),
            tropical: if self.tropical { 1 } else { 0 },
            boar_x,
            boar_y,
            building_flags: Default::default(),
            constant_4_0x00: Default::default(),
            monster_x,
            monster_y,
            disembark_x,
            disembark_y,
            constant_5_0x00: Default::default(),
            landslide_x,
            landslide_y,
            constant_6_0x00: Default::default(),
            basic_episode_data: Default::default(),
            city_resources: Default::default(),
            city_resources_bought: Default::default(),
            constant_7_0x00: Default::default(),
            city_resources_sold: Default::default(),
            unknown_5: Default::default(),
            city_resources_quantity: Default::default(),
        };

        return MapData {
            // 321 is the most common new-format value observed across real adventures (see
            // `Adventure::from_pak`'s `map_new_file_ver`) - resource ids are always written in the
            // new numbering (see `ResourceType::try_resolve_for_format`), so `version_1` must land in
            // the new-format range (>= 300) for a round trip through `Adventure` to read them back
            // correctly.
            version_1: 321,
            version_2: 33,
            manifest: Default::default(),
            sprite: BoxedArray::from_vec(self.sprite.clone()),
            root_offset: BoxedArray::from_vec(self.root_offset.clone()),
            terrain: BoxedArray::from_vec(self.terrain.clone()),
            tile_size: BoxedArray::from_vec(self.tile_size.clone()),
            random: BoxedArray::from_vec(self.random.clone()),
            constant_1_0x00: Default::default(),
            seed_1: 0,
            seed_2: 0,
            unknown_1: 0,
            unknown_2: 0,
            scenario_data,
            meadow: BoxedArray::from_vec(self.meadow.clone()),
            unknown_3: Default::default(),
            world_map_elements: Default::default(),
            trade_routes: Default::default(),
            constant_2_0xff: BoxedArray::from_vec(vec![0xFF; 51984]),
            unknown_4: Default::default(),
            prices: Default::default(),
            scrub: BoxedArray::from_vec(self.scrub.clone()),
            elevation: BoxedArray::from_vec(self.elevation.clone()),
            elevation_rotation: Default::default(),
            world_locations: Default::default(),
            background_image: 0,
            unknown_5: Default::default(),
            mythology: Default::default(),
            unknown_6: Default::default(),
            unknown_7: 0,
            unknown_8: 0,
            unknown_9: Default::default(),
        };
    }
}

fn zip_position(x: u16, y: u16) -> Option<(u16, u16)> {
    return if x == u16::MAX { None } else { Some((x, y)) };
}

fn unzip_position(position: Option<(u16, u16)>) -> (u16, u16) {
    return position.unwrap_or((u16::MAX, u16::MAX));
}

fn zip_positions<const N: usize>(a: &[u16; N], b: &[u16; N]) -> Vec<Option<(u16, u16)>> {
    let mut positions: Vec<Option<(u16, u16)>> = a
        .iter()
        .zip(b)
        .map(|(&x, &y)| if x == u16::MAX { None } else { Some((x, y)) })
        .collect();

    while positions.last() == Some(&None) {
        positions.pop();
    }

    return positions;
}

fn unzip_positions<const N: usize>(positions: &[Option<(u16, u16)>]) -> ([u16; N], [u16; N]) {
    let mut x = [u16::MAX; N];
    let mut y = [u16::MAX; N];

    for (i, position) in positions.iter().take(N).enumerate() {
        if let Some((px, py)) = position {
            x[i] = *px;
            y[i] = *py;
        }
    }

    return (x, y);
}

fn zip_positions_u32<const N: usize>(a: &[u32; N], b: &[u32; N]) -> Vec<Option<(u16, u16)>> {
    let mut positions: Vec<Option<(u16, u16)>> = a
        .iter()
        .zip(b)
        .map(|(&x, &y)| if x == u32::MAX { None } else { Some((x as u16, y as u16)) })
        .collect();

    while positions.last() == Some(&None) {
        positions.pop();
    }

    return positions;
}

fn unzip_positions_u32<const N: usize>(positions: &[Option<(u16, u16)>]) -> ([u32; N], [u32; N]) {
    let mut x = [u32::MAX; N];
    let mut y = [u32::MAX; N];

    for (i, position) in positions.iter().take(N).enumerate() {
        if let Some((px, py)) = position {
            x[i] = *px as u32;
            y[i] = *py as u32;
        }
    }

    return (x, y);
}
