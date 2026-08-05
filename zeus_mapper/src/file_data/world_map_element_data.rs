use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
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

#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct WorldMapElementData {
    pub variant: u8, // 0 = invisible / 1 = old_city / 2 = label ?
    pub data_a: [u8; 3],
    pub x: u16,
    pub y: u16,
    pub sprite_width: u16,
    pub sprite_height: u16,
    // 121 = Zeus_Interface.sg3#1090
    // Zeus_empire_bits.BMP starts at 1038 (note atlantean sprites are in Poseidon_loaded.sg3#247)
    pub sprite_id: u8,
    pub unknown: [u8; 11], // @0 is set to 13 for atlantean rival, @5 is label position
    pub region_name: u8,   // also something else for a old_city?
    pub city_name: u8,
    pub data_d: [u8; 14],
    pub custom_names: String,
}

impl WorldMapElementData {
    pub(crate) fn read_arr_from(reader: &mut impl Read, custom_names: bool) -> io::Result<BoxedArray<Self, 200>> {
        let compressed_size = i32::read_from(reader)?;
        let mut result = Vec::with_capacity(200);

        if compressed_size < 0 {
            for _ in 0..200 {
                result.push(Self::read_from(reader, custom_names)?);
            }
        } else {
            let mut explode_reader = explode::ExplodeReader::new(reader);

            for _ in 0..200 {
                result.push(Self::read_from(&mut explode_reader, custom_names)?);
            }
        }

        return Ok(BoxedArray::from_vec(result));
    }

    fn read_from(reader: &mut impl Read, custom_names: bool) -> io::Result<Self> {
        let custom_names_length = if custom_names { 32 } else { 0 };

        return Ok(WorldMapElementData {
            variant: ReadFrom::read_from(reader)?,
            data_a: ReadFrom::read_from(reader)?,
            x: ReadFrom::read_from(reader)?,
            y: ReadFrom::read_from(reader)?,
            sprite_width: ReadFrom::read_from(reader)?,
            sprite_height: ReadFrom::read_from(reader)?,
            sprite_id: ReadFrom::read_from(reader)?,
            unknown: ReadFrom::read_from(reader)?,
            region_name: ReadFrom::read_from(reader)?,
            city_name: ReadFrom::read_from(reader)?,
            data_d: ReadFrom::read_from(reader)?,
            custom_names: read_string_from(reader, custom_names_length)?,
        });
    }
    pub(crate) fn write_arr_to<W: Write, const N: usize>(
        data: &BoxedArray<Self, N>,
        writer: &mut W,
        custom_names: bool,
    ) -> io::Result<usize> {
        let mut uncompressed = Cursor::new(vec![]);

        for world_map_element_data in data.iter() {
            world_map_element_data.write_to(&mut uncompressed, custom_names)?;
        }

        let compressed = implode_bytes(uncompressed.get_ref().as_slice(), COMPRESSION_MODE, DICTIONARY_SIZE)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

        let mut bytes = 0;
        bytes += WriteTo::write_to(&(compressed.len() as i32), writer)?;
        bytes += WriteTo::write_to(&compressed, writer)?;
        return Ok(bytes);
    }

    fn write_to<W: Write>(&self, writer: &mut W, custom_names: bool) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.variant, writer)?;
        bytes += WriteTo::write_to(&self.data_a, writer)?;
        bytes += WriteTo::write_to(&self.x, writer)?;
        bytes += WriteTo::write_to(&self.y, writer)?;
        bytes += WriteTo::write_to(&self.sprite_width, writer)?;
        bytes += WriteTo::write_to(&self.sprite_height, writer)?;
        bytes += WriteTo::write_to(&self.sprite_id, writer)?;
        bytes += WriteTo::write_to(&self.unknown, writer)?;
        bytes += WriteTo::write_to(&self.region_name, writer)?;
        bytes += WriteTo::write_to(&self.city_name, writer)?;
        bytes += WriteTo::write_to(&self.data_d, writer)?;

        if custom_names {
            bytes += write_string_to(&self.custom_names, writer, 32)?;
        }

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write_without_custom_names() -> io::Result<()> {
        let original = base();

        let mut buffer = vec![];

        original.write_to(&mut buffer, false)?;

        let deserialized = WorldMapElementData::read_from(&mut Cursor::new(buffer), false)?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    #[test]
    fn read_write_with_custom_names() -> io::Result<()> {
        let mut original = base();
        original.custom_names = "Custom Name".to_string();

        let mut buffer = vec![];

        original.write_to(&mut buffer, true)?;

        let deserialized = WorldMapElementData::read_from(&mut Cursor::new(buffer), true)?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    fn base() -> WorldMapElementData {
        return WorldMapElementData {
            variant: 1,
            data_a: [2, 3, 4],
            x: 5,
            y: 6,
            sprite_width: 7,
            sprite_height: 8,
            sprite_id: 9,
            unknown: [10; 11],
            region_name: 11,
            city_name: 12,
            data_d: [13; 14],
            custom_names: "".to_string(),
        };
    }
}
