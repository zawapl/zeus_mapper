use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::read_utils::read_vec_from;
use crate::utils::write_utils::COMPRESSION_MODE;
use crate::utils::write_utils::DICTIONARY_SIZE;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use pklib::implode::implode_bytes;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, Default, LogDifferences)]
pub struct WorldLocationData {
    pub exists: u16,
    pub location_type: u8,
    pub unknown_3: BoxedArray<u8, 94>,
    pub trade_quantities: BoxedArray<u8, 36>,
    pub unknown_133: BoxedArray<u8, 55>,
    pub buying: [u8; 8],
    pub selling: [u8; 8],
    pub variant: u32,
    pub leader_name: u32,
    pub attitude: u32,
    pub economical_strength: u32,
    pub military_strength: u32,
    pub tribute: u32,
    pub rec: u32,
    pub pay: u32,
    pub pay_resource: [u16; 2],
    pub unknown_240: BoxedArray<u8, 116>,
    pub favour: u32,
    pub unknown_360: [u8; 8],
    pub active: u32,
    pub visible: u32,
    pub unknown_376: BoxedArray<u8, 100>,
    pub unknown_476: Vec<u8>,
    pub trade_route_visible: u8,
    pub custom_name: String,
    pub custom_leader_name: String,
    pub tail: Vec<u8>,
}

impl WorldLocationData {
    pub(crate) fn read_arr_from(reader: &mut impl Read, include_extras: bool) -> io::Result<BoxedArray<Self, 22>> {
        let mut result = Vec::with_capacity(22);

        let compressed_size = i32::read_from(reader)?;

        if compressed_size < 0 {
            for _ in 0..22 {
                result.push(Self::read_from(reader, include_extras)?);
            }
        } else {
            let mut explode_reader = explode::ExplodeReader::new(reader);

            for _ in 0..22 {
                result.push(Self::read_from(&mut explode_reader, include_extras)?);
            }
        }

        return Ok(BoxedArray::from_vec(result));
    }

    fn read_from(reader: &mut impl Read, include_extras: bool) -> io::Result<Self> {
        let unknown_476_length = if include_extras { 28 } else { 0 };
        let strings_length = if include_extras { 32 } else { 0 };
        let tail_length = if include_extras { 3 } else { 0 };

        return Ok(WorldLocationData {
            exists: ReadFrom::read_from(reader)?,
            location_type: ReadFrom::read_from(reader)?,
            unknown_3: ReadFrom::read_from(reader)?,
            trade_quantities: ReadFrom::read_from(reader)?,
            unknown_133: ReadFrom::read_from(reader)?,
            buying: ReadFrom::read_from(reader)?,
            selling: ReadFrom::read_from(reader)?,
            variant: ReadFrom::read_from(reader)?,
            leader_name: ReadFrom::read_from(reader)?,
            attitude: ReadFrom::read_from(reader)?,
            economical_strength: ReadFrom::read_from(reader)?,
            military_strength: ReadFrom::read_from(reader)?,
            tribute: ReadFrom::read_from(reader)?,
            rec: ReadFrom::read_from(reader)?,
            pay: ReadFrom::read_from(reader)?,
            pay_resource: ReadFrom::read_from(reader)?,
            unknown_240: ReadFrom::read_from(reader)?,
            favour: ReadFrom::read_from(reader)?,
            unknown_360: ReadFrom::read_from(reader)?,
            active: ReadFrom::read_from(reader)?,
            visible: ReadFrom::read_from(reader)?,
            unknown_376: ReadFrom::read_from(reader)?,
            unknown_476: read_vec_from(reader, unknown_476_length)?,
            trade_route_visible: if include_extras { ReadFrom::read_from(reader)? } else { 4 },
            custom_name: read_string_from(reader, strings_length)?,
            custom_leader_name: read_string_from(reader, strings_length)?,
            tail: read_vec_from(reader, tail_length)?,
        });
    }
    pub(crate) fn write_arr_to<W: Write, const N: usize>(
        data: &BoxedArray<Self, N>,
        writer: &mut W,
        include_extras: bool,
    ) -> io::Result<usize> {
        let mut uncompressed = Cursor::new(vec![]);

        for world_location_data in data.iter() {
            world_location_data.write_to(&mut uncompressed, include_extras)?;
        }

        let compressed = implode_bytes(uncompressed.get_ref().as_slice(), COMPRESSION_MODE, DICTIONARY_SIZE)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        let mut bytes = 0;
        bytes += WriteTo::write_to(&(compressed.len() as i32), writer)?;
        bytes += WriteTo::write_to(&compressed, writer)?;
        return Ok(bytes);
    }

    fn write_to<W: Write>(&self, writer: &mut W, include_extras: bool) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.exists, writer)?;
        bytes += WriteTo::write_to(&self.location_type, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
        bytes += WriteTo::write_to(&self.trade_quantities, writer)?;
        bytes += WriteTo::write_to(&self.unknown_133, writer)?;
        bytes += WriteTo::write_to(&self.buying, writer)?;
        bytes += WriteTo::write_to(&self.selling, writer)?;
        bytes += WriteTo::write_to(&self.variant, writer)?;
        bytes += WriteTo::write_to(&self.leader_name, writer)?;
        bytes += WriteTo::write_to(&self.attitude, writer)?;
        bytes += WriteTo::write_to(&self.economical_strength, writer)?;
        bytes += WriteTo::write_to(&self.military_strength, writer)?;
        bytes += WriteTo::write_to(&self.tribute, writer)?;
        bytes += WriteTo::write_to(&self.rec, writer)?;
        bytes += WriteTo::write_to(&self.pay, writer)?;
        bytes += WriteTo::write_to(&self.pay_resource, writer)?;
        bytes += WriteTo::write_to(&self.unknown_240, writer)?;
        bytes += WriteTo::write_to(&self.favour, writer)?;
        bytes += WriteTo::write_to(&self.unknown_360, writer)?;
        bytes += WriteTo::write_to(&self.active, writer)?;
        bytes += WriteTo::write_to(&self.visible, writer)?;
        bytes += WriteTo::write_to(&self.unknown_376, writer)?;

        if include_extras {
            bytes += WriteTo::write_to(&self.unknown_476, writer)?;
            bytes += WriteTo::write_to(&self.trade_route_visible, writer)?;
            bytes += write_string_to(&self.custom_name, writer, 32)?;
            bytes += write_string_to(&self.custom_leader_name, writer, 32)?;
            bytes += WriteTo::write_to(&self.tail, writer)?;
        }

        return Ok(bytes);
    }
}

// todo delete this
#[allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum LocationType {
    Parent    = 0,
    Colony    = 1,
    Foreign   = 2,
    Distant   = 4,
    Enchanted = 5,
    Unknown,
}

impl ReadFrom for LocationType {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let id: u8 = ReadFrom::read_from(reader)?;
        match id {
            0 => Ok(LocationType::Parent),
            1 => Ok(LocationType::Colony),
            2 => Ok(LocationType::Foreign),
            4 => Ok(LocationType::Distant),
            5 => Ok(LocationType::Enchanted),
            _ => Ok(LocationType::Unknown),
        }
    }
}
