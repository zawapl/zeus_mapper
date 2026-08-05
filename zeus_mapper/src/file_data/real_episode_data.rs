use crate::file_data::basic_episode_data::BasicEpisodeData;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use std::io::Read;
use std::io::Write;

#[derive(Debug, LogDifferences)]
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
    pub field_24: [u8; 76],
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
    pub field_36: [u8; 40],
    pub tropical: u32,
    pub boar_x: [u16; 8],
    pub boar_y: [u16; 8],
    pub building_flags: [u16; 100],
    pub field_41: [u8; 16],
    pub monster_x: [u32; 3],
    pub monster_y: [u32; 3],
    pub disembark_x: [u16; 6],
    pub disembark_y: [u16; 6],
    pub field_46: [u8; 276],
    pub landslide_x: [u16; 6],
    pub landslide_y: [u16; 6],
    pub field_49: [u8; 8],
    pub basic_episode_data: BasicEpisodeData,
    pub city_resources: [u8; 20],
    pub city_resources_bought: [u8; 4],
    pub field_53: [u8; 2],
    pub city_resources_sold: [u8; 4],
    pub field_55: [u8; 2],
    pub city_resources_quantity: [u8; 40],
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

impl Default for RealEpisodeData {
    fn default() -> Self {
        return RealEpisodeData {
            start_date: 0,
            field_2: [0; 8],
            months_elapsed: 0,
            field_4: [0; 20],
            starting_cash: 0,
            field_6: [0; 8],
            map_size: 0,
            field_8: [0; 12],
            text_buffer_1: "".to_string(),
            text_buffer_2: "".to_string(),
            civilization: 0,
            wolf_x: [0; 4],
            wolf_y: [0; 4],
            fish_x: [0; 8],
            fish_y: [0; 8],
            urchin_x: [0; 8],
            urchin_y: [0; 8],
            field_17: [0; 12],
            invasion_x: [0; 16],
            invasion_y: [0; 16],
            panhellenic_games: 0,
            colonies_done: 0,
            deer_x: [0; 4],
            deer_y: [0; 4],
            field_24: [0; 76],
            earthquake_area: [0; 2],
            entry_x: 0,
            entry_y: 0,
            exit_x: 0,
            exit_y: 0,
            disaster_x: [0; 8],
            disaster_y: [0; 8],
            river_entry_x: 0,
            river_entry_y: 0,
            river_exit_x: 0,
            river_exit_y: 0,
            field_36: [0; 40],
            tropical: 0,
            boar_x: [0; 8],
            boar_y: [0; 8],
            building_flags: [0; 100],
            field_41: [0; 16],
            monster_x: [0; 3],
            monster_y: [0; 3],
            disembark_x: [0; 6],
            disembark_y: [0; 6],
            field_46: [0; 276],
            landslide_x: [0; 6],
            landslide_y: [0; 6],
            field_49: [0; 8],
            basic_episode_data: Default::default(),
            city_resources: [0; 20],
            city_resources_bought: [0; 4],
            field_53: [0; 2],
            city_resources_sold: [0; 4],
            field_55: [0; 2],
            city_resources_quantity: [0; 40],
        };
    }
}
