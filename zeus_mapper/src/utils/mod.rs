pub(crate) mod read_utils;
pub(crate) mod write_utils;

pub const MAX_MAP_SIZE: u32 = 228;
pub const TILE_MAP_IMAGE_INDEX_START: u32 = 201;
pub const TILE_MAP_MAP_DATA_IMAGE_OFFSET: u32 = 48951;
pub const TILE_MAP_IMAGE_INDEX_MASK: u32 = 0x3fff;

/// Tile-adjacency direction on the isometric map.
///
/// `N`/`S`/`W`/`E` are the tiles directly up/down/left/right on screen.
/// On this isometric grid those screen-adjacent tiles are diagonal neighbours in tile-coordinate terms,
/// while `NW`/`NE`/`SW`/`SE` are the axis-aligned tile-coordinate neighbours.
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    N,
    S,
    W,
    E,
    NW,
    NE,
    SW,
    SE,
}

#[allow(dead_code)]
impl Direction {
    /// True for `N`/`S`/`W`/`E`, the screen up/down/left/right tiles that are diagonal neighbours in
    /// tile-coordinate terms on this isometric grid.
    pub fn is_diagonal(&self) -> bool {
        return *self == Direction::N || *self == Direction::S || *self == Direction::W || *self == Direction::E;
    }

    pub fn values() -> [Self; 8] {
        return [
            Direction::N,
            Direction::S,
            Direction::W,
            Direction::E,
            Direction::NW,
            Direction::NE,
            Direction::SW,
            Direction::SE,
        ];
    }
}
