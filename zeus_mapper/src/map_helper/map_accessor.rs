use crate::adventure::city_map::CityMap;
pub(crate) use crate::map_helper::map_position::MapPosition;
use crate::prelude::MapData;
use crate::prelude::TerrainTypeFlags;

pub trait MapAccessor {
    fn root_offset_at(&self, pos: &MapPosition) -> u8;

    fn terrain_flags_at(&self, pos: &MapPosition) -> TerrainTypeFlags;

    fn tile_size_at(&self, pos: &MapPosition) -> u8;

    fn random_at(&self, pos: &MapPosition) -> u8;

    fn meadow_at(&self, pos: &MapPosition) -> u8;

    fn scrub_at(&self, pos: &MapPosition) -> u8;

    fn elevation_at(&self, pos: &MapPosition) -> u8;
}

impl MapAccessor for MapData {
    fn root_offset_at(&self, pos: &MapPosition) -> u8 {
        return self.root_offset[pos.index()];
    }

    fn terrain_flags_at(&self, pos: &MapPosition) -> TerrainTypeFlags {
        return TerrainTypeFlags(self.terrain[pos.index()]);
    }

    fn tile_size_at(&self, pos: &MapPosition) -> u8 {
        return self.tile_size[pos.index()];
    }

    fn random_at(&self, pos: &MapPosition) -> u8 {
        return self.random[pos.index()];
    }

    fn meadow_at(&self, pos: &MapPosition) -> u8 {
        return self.meadow[pos.index()];
    }

    fn scrub_at(&self, pos: &MapPosition) -> u8 {
        return self.scrub[pos.index()];
    }

    fn elevation_at(&self, pos: &MapPosition) -> u8 {
        return self.elevation[pos.index()];
    }
}

impl MapAccessor for CityMap {
    fn root_offset_at(&self, pos: &MapPosition) -> u8 {
        return self.root_offset[pos.index()];
    }

    fn terrain_flags_at(&self, pos: &MapPosition) -> TerrainTypeFlags {
        return TerrainTypeFlags(self.terrain[pos.index()]);
    }

    fn tile_size_at(&self, pos: &MapPosition) -> u8 {
        return self.tile_size[pos.index()];
    }

    fn random_at(&self, pos: &MapPosition) -> u8 {
        return self.random[pos.index()];
    }

    fn meadow_at(&self, pos: &MapPosition) -> u8 {
        return self.meadow[pos.index()];
    }

    fn scrub_at(&self, pos: &MapPosition) -> u8 {
        return self.scrub[pos.index()];
    }

    fn elevation_at(&self, pos: &MapPosition) -> u8 {
        return self.elevation[pos.index()];
    }
}
