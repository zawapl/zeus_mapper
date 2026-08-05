use crate::map_helper::map_accessor::MapAccessor;
use crate::map_helper::map_accessor::MapPosition;
use crate::prelude::TerrainTypeFlag::Building;
use crate::prelude::TerrainTypeFlag::Rock;
use crate::prelude::TerrainTypeFlag::Slope;

pub fn calculate_visibility<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> bool {
    let terrain_type = map_accessor.terrain_flags_at(pos);
    if terrain_type.is_any_of(Rock | Slope | Building) {
        let tile_size = map_accessor.tile_size_at(pos) & 0xf;
        let root_offset = map_accessor.root_offset_at(pos);

        // rotation specific
        if tile_size == 1 {
            return root_offset == 9;
        } else if tile_size == 2 {
            return root_offset == 18;
        }
    }
    return true;
}
