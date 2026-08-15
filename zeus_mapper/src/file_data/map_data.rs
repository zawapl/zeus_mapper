use crate::file_data::manifest_data::ManifestData;
use crate::file_data::manifest_data::read_segment;
use crate::file_data::mythology_data::MythologyData;
use crate::file_data::real_episode_data::RealEpisodeData;
use crate::file_data::trade_route_data::TradeRouteData;
use crate::file_data::world_location_data::WorldLocationData;
use crate::file_data::world_map_element_data::WorldMapElementData;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_compressed_boxed_array_from;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_compressed;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use std::io;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Default, LogDifferences)]
pub struct MapData {
    pub version_1: u32,
    pub version_2: u32,
    pub manifest: BoxedArray<ManifestData, 300>,
    pub sprite: BoxedArray<u32, 51984>,
    pub root_offset: BoxedArray<u8, 51984>,
    pub terrain: BoxedArray<u32, 51984>,
    pub tile_size: BoxedArray<u8, 51984>,
    pub random: BoxedArray<u8, 51984>,
    pub constant_1_0x00: BoxedArray<u8, 51984>,
    pub seed_1: u32,
    pub seed_2: u32,
    pub unknown_1: u32,
    pub unknown_2: u32,
    pub scenario_data: RealEpisodeData,
    pub meadow: BoxedArray<u8, 51984>,
    pub unknown_3: BoxedArray<u8, 18628>,
    pub world_map_elements: BoxedArray<WorldMapElementData, 200>,
    pub trade_routes: BoxedArray<TradeRouteData, 232>,
    pub constant_2_0xff: BoxedArray<u8, 51984>,
    pub constant_3: BoxedArray<u32, 9>,
    pub prices: BoxedArray<u32, 36>,
    pub scrub: BoxedArray<u8, 51984>,
    pub elevation: BoxedArray<u8, 51984>,
    pub elevation_rotation: BoxedArray<u8, 51984>,
    pub world_locations: BoxedArray<WorldLocationData, 22>,
    pub background_image: u32,
    pub unknown_5: Vec<Vec<u8>>,
    pub mythology: MythologyData,
    pub unknown_6: [BoxedArray<u8, 76>; 6],
    pub unknown_7: u32,
    pub unknown_8: u32,
    pub unknown_9: [BoxedArray<u8, 36>; 10],
}

impl MapData {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return ReadFrom::read_from(reader);
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        return WriteTo::write_to(self, writer);
    }

    pub(crate) fn read_maps<R: Read + Seek>(reader: &mut R, count: usize) -> io::Result<Vec<Self>> {
        let mut result = Vec::with_capacity(count);

        loop {
            // TODO a better mechanism to this maybe: https://doc.rust-lang.org/std/io/trait.Seek.html#method.stream_len
            let start_pos = reader.stream_position()?;
            reader.seek(SeekFrom::End(0))?;
            let end_pos = reader.stream_position()?;

            if start_pos == end_pos {
                return Ok(result);
            }

            reader.seek(SeekFrom::Start(start_pos))?;
            result.push(MapData::read_from(reader)?);
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version_2 != 33 {
            return Err(format!("version_2 is {}, expected 33", self.version_2));
        }

        let is_old_format = (227..=247).contains(&self.version_1);
        let is_new_format = (318..=321).contains(&self.version_1);
        if !is_old_format && !is_new_format {
            return Err(format!(
                "version_1 is {}, expected the known old-format (227..=247) or new-format (318..=321) range",
                self.version_1
            ));
        }

        if let Some(offset) = self.constant_1_0x00.as_ref().iter().position(|b| *b != 0) {
            return Err(format!("constant_1_0x00[{offset}] is non-zero"));
        }

        if let Some(offset) = self.constant_2_0xff.as_ref().iter().position(|b| *b != 0xFF) {
            return Err(format!("constant_2_0xff[{offset}] is not 0xFF"));
        }

        const CONSTANT_3_EXPECTED: [u32; 9] = [180, 70, 100, 180, 70, 100, 0, 100, 0];
        if *self.constant_3 != CONSTANT_3_EXPECTED {
            return Err(format!("constant_3 is {:?}, expected {:?}", *self.constant_3, CONSTANT_3_EXPECTED));
        }

        self.scenario_data.validate().map_err(|e| format!("scenario_data: {e}"))?;

        for (i, trade_route) in self.trade_routes.iter().enumerate() {
            trade_route.validate().map_err(|e| format!("trade_routes[{i}]: {e}"))?;
        }

        for (i, world_location) in self.world_locations.iter().enumerate() {
            world_location.validate().map_err(|e| format!("world_locations[{i}]: {e}"))?;
        }

        return Ok(());
    }
}

impl ReadFrom for MapData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let mut header = [0; 4];
        reader.read_exact(&mut header)?;

        if header.ne("MAPS".as_bytes()) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unknown file header for .map file: {:?}.", header),
            ));
        }

        let version_1 = ReadFrom::read_from(reader)?;
        let version_2 = ReadFrom::read_from(reader)?;

        let manifest: BoxedArray<ManifestData, 300> = ReadFrom::read_from(reader)?;
        let include_custom_names = manifest[17].size == 72;
        let include_world_locations_extras = manifest[25].size == 572;
        let manifest_field_28 = manifest[27].clone();
        let include_pyramids = manifest[28].size == 300;

        return Ok(MapData {
            version_1,
            version_2,
            manifest,
            sprite: read_compressed_boxed_array_from(reader)?,
            root_offset: read_compressed_boxed_array_from(reader)?,
            terrain: read_compressed_boxed_array_from(reader)?,
            tile_size: read_compressed_boxed_array_from(reader)?,
            random: read_compressed_boxed_array_from(reader)?,
            constant_1_0x00: read_compressed_boxed_array_from(reader)?,
            seed_1: ReadFrom::read_from(reader)?,
            seed_2: ReadFrom::read_from(reader)?,
            unknown_1: ReadFrom::read_from(reader)?,
            unknown_2: ReadFrom::read_from(reader)?,
            scenario_data: ReadFrom::read_from(reader)?,
            meadow: read_compressed_boxed_array_from(reader)?,
            unknown_3: ReadFrom::read_from(reader)?,
            world_map_elements: WorldMapElementData::read_arr_from(reader, include_custom_names)?,
            trade_routes: TradeRouteData::read_arr_from(reader)?,
            constant_2_0xff: read_compressed_boxed_array_from(reader)?,
            constant_3: read_compressed_boxed_array_from(reader)?,
            prices: ReadFrom::read_from(reader)?,
            scrub: read_compressed_boxed_array_from(reader)?,
            elevation: read_compressed_boxed_array_from(reader)?,
            elevation_rotation: read_compressed_boxed_array_from(reader)?,
            world_locations: WorldLocationData::read_arr_from(reader, include_world_locations_extras)?,
            background_image: ReadFrom::read_from(reader)?,
            unknown_5: read_segment(reader, &manifest_field_28)?,
            mythology: MythologyData::read_from(reader, include_pyramids)?,
            unknown_6: ReadFrom::read_from(reader)?,
            unknown_7: ReadFrom::read_from(reader)?,
            unknown_8: ReadFrom::read_from(reader)?,
            unknown_9: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for MapData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        let include_custom_names = self.manifest[17].size == 72;
        let include_world_locations_extras = self.manifest[25].size == 572;
        let include_pyramids = self.manifest[28].size == 300;

        bytes += write_string_to("MAPS", writer, 4)?;
        bytes += WriteTo::write_to(&self.version_1, writer)?;
        bytes += WriteTo::write_to(&self.version_2, writer)?;
        bytes += WriteTo::write_to(&self.manifest, writer)?;
        bytes += write_compressed(&self.sprite, writer)?;
        bytes += write_compressed(&self.root_offset, writer)?;
        bytes += write_compressed(&self.terrain, writer)?;
        bytes += write_compressed(&self.tile_size, writer)?;
        bytes += write_compressed(&self.random, writer)?;
        bytes += write_compressed(&self.constant_1_0x00, writer)?;
        bytes += WriteTo::write_to(&self.seed_1, writer)?;
        bytes += WriteTo::write_to(&self.seed_2, writer)?;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.unknown_2, writer)?;
        bytes += WriteTo::write_to(&self.scenario_data, writer)?;
        bytes += write_compressed(&self.meadow, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
        bytes += WorldMapElementData::write_arr_to(&self.world_map_elements, writer, include_custom_names)?;
        bytes += write_compressed(&self.trade_routes, writer)?;
        bytes += write_compressed(&self.constant_2_0xff, writer)?;
        bytes += write_compressed(&self.constant_3, writer)?;
        bytes += WriteTo::write_to(&self.prices, writer)?;
        bytes += write_compressed(&self.scrub, writer)?;
        bytes += write_compressed(&self.elevation, writer)?;
        bytes += write_compressed(&self.elevation_rotation, writer)?;
        bytes += WorldLocationData::write_arr_to(&self.world_locations, writer, include_world_locations_extras)?;
        bytes += WriteTo::write_to(&self.background_image, writer)?;
        bytes += WriteTo::write_to(&self.unknown_5, writer)?;
        bytes += MythologyData::write_to(&self.mythology, writer, include_pyramids)?;
        bytes += WriteTo::write_to(&self.unknown_6, writer)?;
        bytes += WriteTo::write_to(&self.unknown_7, writer)?;
        bytes += WriteTo::write_to(&self.unknown_8, writer)?;
        bytes += WriteTo::write_to(&self.unknown_9, writer)?;

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use crate::file_data::map_data::MapData;
    use std::fs;
    use std::fs::File;
    use std::io;
    use std::io::BufReader;
    use std::io::Seek;
    use std::io::SeekFrom;

    #[test]
    fn validate_map_files() -> io::Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            for entry in fs::read_dir(format!("{game_root}/Adventures"))? {
                let adventure_folder = entry?.path();

                if adventure_folder.is_dir() {
                    for adventure_file in fs::read_dir(adventure_folder)? {
                        let path = adventure_file?.path();

                        let extension = path.extension().map_or("", |x| x.to_str().unwrap_or(""));

                        if extension == "map" {
                            let mut reader = File::open(&path).map(BufReader::new)?;
                            let map_data = MapData::read_from(&mut reader)?;

                            let current_pos = reader.stream_position()?;
                            reader.seek(SeekFrom::End(0))?;
                            let end_pos = reader.stream_position()?;
                            let unread = end_pos - current_pos;

                            assert_eq!(unread, 0, "{path:?}: {} unread bytes", end_pos - current_pos);

                            if let Err(e) = map_data.validate() {
                                panic!("{path:?}: {e}");
                            }
                        }
                    }
                }
            }
        }

        return Ok(());
    }
}
