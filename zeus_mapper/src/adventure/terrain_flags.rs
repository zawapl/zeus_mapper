use crate::constants::data_constant::data_constants;
use crate::prelude::DataConstant;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::ops::BitOr;

data_constants!(TerrainTypeFlag<u32> {
    Forest       = 0,
    Rock         = 1,
    WaterShallow = 2,
    Building     = 3,
    Disaster     = 4,
    Park         = 5,
    Road         = 6,
    Scrub        = 7,
    Flood        = 8,
    Slope        = 9,
    AccessRamp   = 10,
    Meadow       = 11,
    _0x1000      = 12,
    _0x2000      = 13,
    Wall         = 14,
    _0x8000      = 15,
    Sand         = 16,
    Marble       = 17, // BlackMarble = Marble + Copper
    Marshland    = 18,
    OutOfBounds  = 19,
    Copper       = 20,
    Silver       = 21, // Orichalc = Copper + Silver
    HighSlope    = 22,
    HalfSlope    = 23,
    SolidLava    = 24,
    Earthquake   = 25,
    WaterDeep    = 26,
    _0x8000000   = 27,
    Sanctuary    = 28,
    Corner       = 29,
    _0x40000000  = 30,
    Landslide    = 31,
});

#[derive(Default, Clone, Copy, PartialEq)]
pub struct TerrainTypeFlags(pub u32);

impl TerrainTypeFlags {
    pub fn is(&self, terrain_type_flag: TerrainTypeFlag) -> bool {
        return self.is_any_of(terrain_type_flag);
    }

    pub fn is_all_of<T: Into<TerrainTypeFlags>>(&self, other: T) -> bool {
        let self_bits = self.0;
        let other_bits = other.into().0;
        return (self_bits & other_bits) == other_bits;
    }

    pub fn is_any_of<T: Into<TerrainTypeFlags>>(&self, other: T) -> bool {
        let self_bits = self.0;
        let other_bits = other.into().0;
        return (self_bits & other_bits) != 0;
    }
}

impl BitOr for TerrainTypeFlags {
    type Output = TerrainTypeFlags;

    fn bitor(self, rhs: Self) -> Self::Output {
        return TerrainTypeFlags(self.0 | rhs.0);
    }
}

impl BitOr for TerrainTypeFlag {
    type Output = TerrainTypeFlags;

    fn bitor(self, rhs: Self) -> Self::Output {
        return TerrainTypeFlags::from(self) | TerrainTypeFlags::from(rhs);
    }
}

impl BitOr<TerrainTypeFlag> for TerrainTypeFlags {
    type Output = TerrainTypeFlags;

    fn bitor(self, rhs: TerrainTypeFlag) -> Self::Output {
        return self | TerrainTypeFlags::from(rhs);
    }
}

impl From<TerrainTypeFlag> for TerrainTypeFlags {
    fn from(value: TerrainTypeFlag) -> Self {
        return TerrainTypeFlags(1 << (value as u32));
    }
}

impl From<u32> for TerrainTypeFlags {
    fn from(value: u32) -> Self {
        return TerrainTypeFlags(value);
    }
}

impl Debug for TerrainTypeFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let flags = TerrainTypeFlag::values()
            .iter()
            .rev()
            .filter(|terrain_type_flag| self.is(**terrain_type_flag))
            .map(|terrain_type_flag| format!("{:?}", terrain_type_flag))
            .collect::<Vec<String>>()
            .join(" | ");
        return write!(f, "[{}]", flags);
    }
}
