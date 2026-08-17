use crate::pkware::explode;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::to_usize;
use crate::utils::validation::ValidationError;
use crate::utils::validation::ValidationResult;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

// count = 232
// size = 324
#[derive(Debug, Clone, PartialEq, LogDifferences)]
pub struct TradeRouteData {
    pub constant_1_0x05: u64,
    pub points: BoxedArray<TradeRoutePointData, 50>,
    pub distance: [u8; 12],
    pub route_type: u8,
    pub points_count: u8,
    pub exists: u8,
    pub constant_2_0x00: u8,
}
// @320 = sea/land route?
// @321 = points count
// @322 = exists?

impl Default for TradeRouteData {
    fn default() -> Self {
        return TradeRouteData {
            constant_1_0x05: 5,
            points: Default::default(),
            distance: [0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF],
            route_type: 1,
            points_count: 0,
            exists: 0,
            constant_2_0x00: 0,
        };
    }
}

impl TradeRouteData {
    pub fn validate(&self) -> ValidationResult {
        if self.constant_1_0x05 != 5 {
            return Err(ValidationError::expected_exactly("constant_1_0x05", self.constant_1_0x05, 5));
        }

        if self.constant_2_0x00 != 0 {
            return Err(ValidationError::expected_exactly("constant_2_0x00", self.constant_2_0x00, 0));
        }

        return Ok(());
    }

    pub(crate) fn read_arr_from(reader: &mut impl Read) -> io::Result<BoxedArray<Self, 232>> {
        let mut result = Vec::with_capacity(232);

        let compressed_size = i32::read_from(reader)?;

        if compressed_size < 0 {
            for _ in 0..232 {
                result.push(Self::read_from(reader)?);
            }
        } else {
            let mut compressed = vec![0; to_usize(compressed_size)?];
            reader.read_exact(&mut compressed)?;
            let mut decompressed_reader = Cursor::new(explode(&compressed)?);

            for _ in 0..232 {
                result.push(Self::read_from(&mut decompressed_reader)?);
            }
        }

        return Ok(BoxedArray::from_vec(result));
    }
}

impl ReadFrom for TradeRouteData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(TradeRouteData {
            constant_1_0x05: ReadFrom::read_from(reader)?,
            points: ReadFrom::read_from(reader)?,
            distance: ReadFrom::read_from(reader)?,
            route_type: ReadFrom::read_from(reader)?,
            points_count: ReadFrom::read_from(reader)?,
            exists: ReadFrom::read_from(reader)?,
            constant_2_0x00: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for TradeRouteData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.constant_1_0x05, writer)?;
        bytes += WriteTo::write_to(&self.points, writer)?;
        bytes += WriteTo::write_to(&self.distance, writer)?;
        bytes += WriteTo::write_to(&self.route_type, writer)?;
        bytes += WriteTo::write_to(&self.points_count, writer)?;
        bytes += WriteTo::write_to(&self.exists, writer)?;
        bytes += WriteTo::write_to(&self.constant_2_0x00, writer)?;
        return Ok(bytes);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default, LogDifferences)]
pub struct TradeRoutePointData {
    pub x: u16,
    pub y: u16,
    pub unknown_1: u16,
}

impl ReadFrom for TradeRoutePointData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(TradeRoutePointData {
            x: ReadFrom::read_from(reader)?,
            y: ReadFrom::read_from(reader)?,
            unknown_1: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for TradeRoutePointData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.x, writer)?;
        bytes += WriteTo::write_to(&self.y, writer)?;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write_point() -> io::Result<()> {
        let original = TradeRoutePointData { x: 1, y: 2, unknown_1: 3 };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = TradeRoutePointData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    #[test]
    fn read_write_route() -> io::Result<()> {
        let mut points = vec![];
        for i in 0..50 {
            points.push(TradeRoutePointData {
                x: i,
                y: i + 1,
                unknown_1: i + 2,
            });
        }

        let original = TradeRouteData {
            constant_1_0x05: 0x0807060504030201,
            points: BoxedArray::from_vec(points),
            distance: [9; 12],
            route_type: 10,
            points_count: 11,
            exists: 12,
            constant_2_0x00: 13,
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = TradeRouteData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }
}
