use crate::file_data::basic_episode_data::BasicEpisodeData;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use std::io::Read;
use std::io::Write;

#[derive(Debug, PartialEq, Default, LogDifferences)]
pub struct RealEpisodeData {
    pub start_date: i16,
    pub field_2: [u8; 8],
    pub months_elapsed: u16,
    pub field_4: [u8; 20],
    pub starting_cash: u32,
    pub field_6: [u8; 8],
    pub map_size: u32,
    pub field_8: [u8; 12],
    pub text_buffer_1: String,
    pub text_buffer_2: String,
    pub civilization: u32,
    pub wolf_x: [u16; 4],
    pub wolf_y: [u16; 4],
    pub fish_x: [u16; 8],
    pub fish_y: [u16; 8],
    pub urchin_x: [u16; 8],
    pub urchin_y: [u16; 8],
    pub field_17: [u8; 12],
    pub invasion_x: [u16; 16],
    pub invasion_y: [u16; 16],
    pub panhellenic_games: u16,
    pub colonies_done: u16,
    pub deer_x: [u16; 4],
    pub deer_y: [u16; 4],
    pub field_24: BoxedArray<u8, 76>,
    pub earthquake_area: [u16; 2],
    pub entry_x: u16,
    pub entry_y: u16,
    pub exit_x: u16,
    pub exit_y: u16,
    pub disaster_x: [u16; 8],
    pub disaster_y: [u16; 8],
    pub river_entry_x: u16,
    pub river_entry_y: u16,
    pub river_exit_x: u16,
    pub river_exit_y: u16,
    pub field_36: BoxedArray<u8, 40>,
    pub tropical: u32,
    pub boar_x: [u16; 8],
    pub boar_y: [u16; 8],
    pub building_flags: BoxedArray<u16, 100>,
    pub field_41: [u8; 16],
    pub monster_x: [u32; 3],
    pub monster_y: [u32; 3],
    pub disembark_x: [u16; 6],
    pub disembark_y: [u16; 6],
    pub field_46: BoxedArray<u8, 276>,
    pub landslide_x: [u16; 6],
    pub landslide_y: [u16; 6],
    pub field_49: [u8; 8],
    pub basic_episode_data: BasicEpisodeData,
    pub city_resources: [u8; 20],
    pub city_resources_bought: [u8; 4],
    pub field_53: [u8; 2],
    pub city_resources_sold: [u8; 4],
    pub field_55: [u8; 2],
    pub city_resources_quantity: BoxedArray<u8, 40>,
}

impl ReadFrom for RealEpisodeData {
    fn read_from(reader: &mut impl Read) -> std::io::Result<Self> {
        return Ok(RealEpisodeData {
            start_date: ReadFrom::read_from(reader)?,
            field_2: ReadFrom::read_from(reader)?,
            months_elapsed: ReadFrom::read_from(reader)?,
            field_4: ReadFrom::read_from(reader)?,
            starting_cash: ReadFrom::read_from(reader)?,
            field_6: ReadFrom::read_from(reader)?,
            map_size: ReadFrom::read_from(reader)?,
            field_8: ReadFrom::read_from(reader)?,
            text_buffer_1: read_string_from(reader, 64)?,
            text_buffer_2: read_string_from(reader, 524)?,
            civilization: ReadFrom::read_from(reader)?,
            wolf_x: ReadFrom::read_from(reader)?,
            wolf_y: ReadFrom::read_from(reader)?,
            fish_x: ReadFrom::read_from(reader)?,
            fish_y: ReadFrom::read_from(reader)?,
            urchin_x: ReadFrom::read_from(reader)?,
            urchin_y: ReadFrom::read_from(reader)?,
            field_17: ReadFrom::read_from(reader)?,
            invasion_x: ReadFrom::read_from(reader)?,
            invasion_y: ReadFrom::read_from(reader)?,
            panhellenic_games: ReadFrom::read_from(reader)?,
            colonies_done: ReadFrom::read_from(reader)?,
            deer_x: ReadFrom::read_from(reader)?,
            deer_y: ReadFrom::read_from(reader)?,
            field_24: ReadFrom::read_from(reader)?,
            earthquake_area: ReadFrom::read_from(reader)?,
            entry_x: ReadFrom::read_from(reader)?,
            entry_y: ReadFrom::read_from(reader)?,
            exit_x: ReadFrom::read_from(reader)?,
            exit_y: ReadFrom::read_from(reader)?,
            disaster_x: ReadFrom::read_from(reader)?,
            disaster_y: ReadFrom::read_from(reader)?,
            river_entry_x: ReadFrom::read_from(reader)?,
            river_entry_y: ReadFrom::read_from(reader)?,
            river_exit_x: ReadFrom::read_from(reader)?,
            river_exit_y: ReadFrom::read_from(reader)?,
            field_36: ReadFrom::read_from(reader)?,
            tropical: ReadFrom::read_from(reader)?,
            boar_x: ReadFrom::read_from(reader)?,
            boar_y: ReadFrom::read_from(reader)?,
            building_flags: ReadFrom::read_from(reader)?,
            field_41: ReadFrom::read_from(reader)?,
            monster_x: ReadFrom::read_from(reader)?,
            monster_y: ReadFrom::read_from(reader)?,
            disembark_x: ReadFrom::read_from(reader)?,
            disembark_y: ReadFrom::read_from(reader)?,
            field_46: ReadFrom::read_from(reader)?,
            landslide_x: ReadFrom::read_from(reader)?,
            landslide_y: ReadFrom::read_from(reader)?,
            field_49: ReadFrom::read_from(reader)?,
            basic_episode_data: ReadFrom::read_from(reader)?,
            city_resources: ReadFrom::read_from(reader)?,
            city_resources_bought: ReadFrom::read_from(reader)?,
            field_53: ReadFrom::read_from(reader)?,
            city_resources_sold: ReadFrom::read_from(reader)?,
            field_55: ReadFrom::read_from(reader)?,
            city_resources_quantity: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for RealEpisodeData {
    fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.start_date, writer)?;
        bytes += WriteTo::write_to(&self.field_2, writer)?;
        bytes += WriteTo::write_to(&self.months_elapsed, writer)?;
        bytes += WriteTo::write_to(&self.field_4, writer)?;
        bytes += WriteTo::write_to(&self.starting_cash, writer)?;
        bytes += WriteTo::write_to(&self.field_6, writer)?;
        bytes += WriteTo::write_to(&self.map_size, writer)?;
        bytes += WriteTo::write_to(&self.field_8, writer)?;
        bytes += write_string_to(&self.text_buffer_1, writer, 64)?;
        bytes += write_string_to(&self.text_buffer_2, writer, 524)?;
        bytes += WriteTo::write_to(&self.civilization, writer)?;
        bytes += WriteTo::write_to(&self.wolf_x, writer)?;
        bytes += WriteTo::write_to(&self.wolf_y, writer)?;
        bytes += WriteTo::write_to(&self.fish_x, writer)?;
        bytes += WriteTo::write_to(&self.fish_y, writer)?;
        bytes += WriteTo::write_to(&self.urchin_x, writer)?;
        bytes += WriteTo::write_to(&self.urchin_y, writer)?;
        bytes += WriteTo::write_to(&self.field_17, writer)?;
        bytes += WriteTo::write_to(&self.invasion_x, writer)?;
        bytes += WriteTo::write_to(&self.invasion_y, writer)?;
        bytes += WriteTo::write_to(&self.panhellenic_games, writer)?;
        bytes += WriteTo::write_to(&self.colonies_done, writer)?;
        bytes += WriteTo::write_to(&self.deer_x, writer)?;
        bytes += WriteTo::write_to(&self.deer_y, writer)?;
        bytes += WriteTo::write_to(&self.field_24, writer)?;
        bytes += WriteTo::write_to(&self.earthquake_area, writer)?;
        bytes += WriteTo::write_to(&self.entry_x, writer)?;
        bytes += WriteTo::write_to(&self.entry_y, writer)?;
        bytes += WriteTo::write_to(&self.exit_x, writer)?;
        bytes += WriteTo::write_to(&self.exit_y, writer)?;
        bytes += WriteTo::write_to(&self.disaster_x, writer)?;
        bytes += WriteTo::write_to(&self.disaster_y, writer)?;
        bytes += WriteTo::write_to(&self.river_entry_x, writer)?;
        bytes += WriteTo::write_to(&self.river_entry_y, writer)?;
        bytes += WriteTo::write_to(&self.river_exit_x, writer)?;
        bytes += WriteTo::write_to(&self.river_exit_y, writer)?;
        bytes += WriteTo::write_to(&self.field_36, writer)?;
        bytes += WriteTo::write_to(&self.tropical, writer)?;
        bytes += WriteTo::write_to(&self.boar_x, writer)?;
        bytes += WriteTo::write_to(&self.boar_y, writer)?;
        bytes += WriteTo::write_to(&self.building_flags, writer)?;
        bytes += WriteTo::write_to(&self.field_41, writer)?;
        bytes += WriteTo::write_to(&self.monster_x, writer)?;
        bytes += WriteTo::write_to(&self.monster_y, writer)?;
        bytes += WriteTo::write_to(&self.disembark_x, writer)?;
        bytes += WriteTo::write_to(&self.disembark_y, writer)?;
        bytes += WriteTo::write_to(&self.field_46, writer)?;
        bytes += WriteTo::write_to(&self.landslide_x, writer)?;
        bytes += WriteTo::write_to(&self.landslide_y, writer)?;
        bytes += WriteTo::write_to(&self.field_49, writer)?;
        bytes += WriteTo::write_to(&self.basic_episode_data, writer)?;
        bytes += WriteTo::write_to(&self.city_resources, writer)?;
        bytes += WriteTo::write_to(&self.city_resources_bought, writer)?;
        bytes += WriteTo::write_to(&self.field_53, writer)?;
        bytes += WriteTo::write_to(&self.city_resources_sold, writer)?;
        bytes += WriteTo::write_to(&self.field_55, writer)?;
        bytes += WriteTo::write_to(&self.city_resources_quantity, writer)?;

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::boxed_array::BoxedArray;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write() -> io::Result<()> {
        let original = RealEpisodeData {
            start_date: 1,
            field_2: seq_u8(2),
            months_elapsed: 3,
            field_4: seq_u8(4),
            starting_cash: 5,
            field_6: seq_u8(6),
            map_size: 7,
            field_8: seq_u8(8),
            text_buffer_1: "Text buffer one".to_owned(),
            text_buffer_2: "Text buffer two".to_owned(),
            civilization: 9,
            wolf_x: seq_u16(10),
            wolf_y: seq_u16(11),
            fish_x: seq_u16(12),
            fish_y: seq_u16(13),
            urchin_x: seq_u16(14),
            urchin_y: seq_u16(15),
            field_17: seq_u8(16),
            invasion_x: seq_u16(17),
            invasion_y: seq_u16(18),
            panhellenic_games: 19,
            colonies_done: 20,
            deer_x: seq_u16(21),
            deer_y: seq_u16(22),
            field_24: BoxedArray::from_vec(seq_u8::<76>(23).to_vec()),
            earthquake_area: seq_u16(24),
            entry_x: 25,
            entry_y: 26,
            exit_x: 27,
            exit_y: 28,
            disaster_x: seq_u16(29),
            disaster_y: seq_u16(30),
            river_entry_x: 31,
            river_entry_y: 32,
            river_exit_x: 33,
            river_exit_y: 34,
            field_36: BoxedArray::from_vec(seq_u8::<40>(35).to_vec()),
            tropical: 36,
            boar_x: seq_u16(37),
            boar_y: seq_u16(38),
            building_flags: BoxedArray::from_vec(seq_u16::<100>(39).to_vec()),
            field_41: seq_u8(40),
            monster_x: seq_u32(41),
            monster_y: seq_u32(42),
            disembark_x: seq_u16(43),
            disembark_y: seq_u16(44),
            field_46: BoxedArray::from_vec(seq_u8::<276>(45).to_vec()),
            landslide_x: seq_u16(46),
            landslide_y: seq_u16(47),
            field_49: seq_u8(48),
            basic_episode_data: BasicEpisodeData {
                exists: 49,
                field_2: 50,
                field_3: 51,
                episode_no: 52,
                field_5: 53,
                field_6: 54,
                field_7: BoxedArray::from_vec(seq_u8::<60>(55).to_vec()),
                next_episode: 56,
                episode_type: 57,
                name_padding: 58,
                name: "Basic episode".to_owned(),
            },
            city_resources: seq_u8(59),
            city_resources_bought: seq_u8(60),
            field_53: seq_u8(61),
            city_resources_sold: seq_u8(62),
            field_55: seq_u8(63),
            city_resources_quantity: BoxedArray::from_vec(seq_u8::<40>(64).to_vec()),
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = RealEpisodeData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    fn seq_u8<const N: usize>(start: u8) -> [u8; N] {
        let mut result = [0u8; N];
        for (i, value) in result.iter_mut().enumerate() {
            *value = start.wrapping_add(i as u8);
        }
        return result;
    }

    fn seq_u16<const N: usize>(start: u16) -> [u16; N] {
        let mut result = [0u16; N];
        for (i, value) in result.iter_mut().enumerate() {
            *value = start.wrapping_add(i as u16);
        }
        return result;
    }

    fn seq_u32<const N: usize>(start: u32) -> [u32; N] {
        let mut result = [0u32; N];
        for (i, value) in result.iter_mut().enumerate() {
            *value = start.wrapping_add(i as u32);
        }
        return result;
    }
}
