use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

// count = 232
// size = 324
#[derive(Debug, Copy, Clone, LogDifferences)]
pub struct TradeRouteData {
    pub header: [u8; 8],
    pub points: [TradeRoutePointData; 50],
    pub distance: [u8; 12],
    pub route_type: u8,
    pub points_count: u8,
    pub exists: u8,
    pub unknown: u8,
}
// @320 = sea/land route?
// @321 = points count
// @322 = exists?

impl TradeRouteData {
    pub(crate) fn read_vec_from(reader: &mut impl Read) -> io::Result<Vec<Self>> {
        let mut result = Vec::with_capacity(232);

        let compressed_size = i32::read_from(reader)?;

        if compressed_size < 0 {
            for _ in 0..232 {
                result.push(Self::read_from(reader)?);
            }
        } else {
            let mut explode_reader = explode::ExplodeReader::new(reader);

            for _ in 0..232 {
                result.push(Self::read_from(&mut explode_reader)?);
            }
        }

        return Ok(result);
    }
}

impl ReadFrom for TradeRouteData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(TradeRouteData {
            header: ReadFrom::read_from(reader)?,
            points: ReadFrom::read_from(reader)?,
            distance: ReadFrom::read_from(reader)?,
            route_type: ReadFrom::read_from(reader)?,
            points_count: ReadFrom::read_from(reader)?,
            exists: ReadFrom::read_from(reader)?,
            unknown: ReadFrom::read_from(reader)?,
        });
    }
}

impl Default for TradeRouteData {
    fn default() -> Self {
        return TradeRouteData {
            header: [0; 8],
            points: [TradeRoutePointData::default(); 50],
            distance: [0; 12],
            route_type: 0,
            points_count: 0,
            exists: 0,
            unknown: 0,
        };
    }
}

impl WriteTo for TradeRouteData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.header, writer)?;
        bytes += WriteTo::write_to(&self.points, writer)?;
        bytes += WriteTo::write_to(&self.distance, writer)?;
        bytes += WriteTo::write_to(&self.route_type, writer)?;
        bytes += WriteTo::write_to(&self.points_count, writer)?;
        bytes += WriteTo::write_to(&self.exists, writer)?;
        bytes += WriteTo::write_to(&self.unknown, writer)?;
        return Ok(bytes);
    }
}

#[derive(Debug, Copy, Clone, Default, LogDifferences)]
pub struct TradeRoutePointData {
    pub x: u16,
    pub y: u16,
    pub unknown: u16,
}

impl ReadFrom for TradeRoutePointData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(TradeRoutePointData {
            x: ReadFrom::read_from(reader)?,
            y: ReadFrom::read_from(reader)?,
            unknown: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for TradeRoutePointData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.x, writer)?;
        bytes += WriteTo::write_to(&self.y, writer)?;
        bytes += WriteTo::write_to(&self.unknown, writer)?;
        return Ok(bytes);
    }
}
