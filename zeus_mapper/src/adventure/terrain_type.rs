use crate::adventure::terrain_type::TerrainType::*;
use crate::constants::data_constant::data_constants;
use crate::prelude::DataConstant;
use crate::prelude::Unresolved;
use std::ops::Deref;

data_constants!(TerrainType<u32> {
    Empty = 0x0,
    Forest = 0x1,
    Rock = 0x2,
    WaterShallow = 0x4,
    Building = 0x8,
    Earthquake = 0x10,
    Park = 0x20,
    Road = 0x40,
    Scrub = 0x80,
    RoadScrub = 0xc0,
    ForestScrub = 0x81,
    EarthquakeScrub = 0x90,
    Flood = 0x100,
    ForestFlood = 0x101,
    WaterShallowFlood = 0x104,
    ScrubFlood = 0x180,
    ForestScrubFlood = 0x181,
    Slope = 0x200,
    SlopeForest = 0x201,
    SlopeRock = 0x202,
    SlopeWaterShallow = 0x204,
    AccessRamp = 0x600,
    AccessRampForest = 0x601,
    RoadAccessRamp = 0x640,
    Meadow = 0x800,
    MeadowRoad = 0x840,
    Wall = 0x4000,
    Sand = 0x10080,
    Marble = 0x20000,
    Marshland = 0x40000,
    OutOfBounds = 0x80000,
    Copper = 0x100002,
    BlackMarble = 0x120000,
    Silver = 0x200002,
    Orichalc = 0x300000,
    Cliff = 0x300002,
    SlopeCliff = 0x300202,
    SlopeHigh = 0x400200,
    SlopeHighRock = 0x400202,
    SlopeHighCliff = 0x700202,
    SlopeHalf = 0x800200,
    SlopeHalfAccessRamp = 0x800600,
    SolidLava = 0x1000000,
    Lava = 0x2000000,
    ScrubLava = 0x2000080,
    FloodScrubLava = 0x2000180,
    WaterDeep = 0x4000004,
    WaterDeepFlood = 0x4000104,
    Sanctuary = 0x10000000,
    SlopeCorner = 0x20400200,
    SlopeCornerRock = 0x20400202,
    SlopeCornerWaterShallow = 0x20400204,
    SlopeCornerSilverRock = 0x20600202,
    SlopeCornerAccessRamp = 0x20400600,
    SlopeCornerForest = 0x20400201,
    SlopeCornerForestCliff = 0x20700202,
    SlopeCornerBig = 0x20c00200,
    Landslide = 0x80000000,
    LandslideForest = 0x80000001,
    LandslideScrub = 0x80000080,
    LandslideSlope = 0x80000200,
    LandslideSlopeHigh = 0x80400200,
    LandslideSlopeCornerHigh = 0xa0400200,
});

pub type TerrainFlags = Unresolved<TerrainType>;

impl TerrainFlags {
    pub fn is_type_of(&self, other: &TerrainType) -> bool {
        let self_bits = *self.deref();
        if other == &Empty {
            return self_bits == Empty.value();
        }
        let other_bits = *other as u32;
        return (self_bits & other_bits) == other_bits;
    }

    pub fn all_of<T: Into<Self>, const N: usize>(values: [T; N]) -> Self {
        let mut result = 0;
        for val in values {
            result |= val.into().deref();
        }
        return Unresolved::new(result);
    }

    pub fn is_any_of<T: Into<Self>>(&self, other: T) -> bool {
        return self.deref() & other.into().deref() != 0;
    }

    pub fn is_all_of<T: Into<Self>>(&self, other: T) -> bool {
        let unresolved = other.into();
        return self.deref() & unresolved.deref() == *unresolved.deref();
    }

    #[allow(unused)]
    pub fn is_disaster(&self) -> bool {
        return self.is_any_of(TerrainFlags::all_of([Earthquake, Lava, Flood, Landslide]));
    }
}
