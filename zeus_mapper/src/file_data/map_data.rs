use crate::file_data::manifest_data::ManifestData;
use crate::file_data::manifest_data::read_segment;
use crate::file_data::mythology_data::MythologyData;
use crate::file_data::real_episode_data::RealEpisodeData;
use crate::file_data::trade_route_data::TradeRouteData;
use crate::file_data::world_location_data::WorldLocationData;
use crate::file_data::world_map_element_data::WorldMapElementData;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_compressed_vec_from;
use crate::utils::read_utils::read_vec_from;
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

#[derive(Debug, LogDifferences)]
pub struct MapData {
    pub version_1: u32,
    pub version_2: u32,
    pub manifest: Vec<ManifestData>,
    pub sprite: Vec<u32>,
    pub root_offset: Vec<u8>,
    pub terrain: Vec<u32>,
    pub tile_size: Vec<u8>,
    pub random: Vec<u8>,
    pub field_10: Vec<u8>,
    pub seed_1: u32,
    pub seed_2: u32,
    pub field_13: u32,
    pub field_14: u32,
    pub scenario_data: RealEpisodeData,
    pub meadow: Vec<u8>,
    pub field_17: [u8; 18628],
    pub world_map_elements: Vec<WorldMapElementData>,
    pub trade_routes: Vec<TradeRouteData>,
    pub field_20: Vec<u8>,
    pub field_21: Vec<u8>,
    pub prices: [u32; 36],
    pub scrub: Vec<u8>,
    pub elevation: Vec<u8>,
    pub elevation_rotation: Vec<u8>,
    pub world_locations: Vec<WorldLocationData>,
    pub background_image: u32,
    pub field_28: Vec<Vec<u8>>,
    pub mythology: MythologyData,
    pub field_30: [[u8; 76]; 6],
    pub field_31: u32,
    pub field_32: u32,
    pub field_33: [[u8; 36]; 10],
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

        let manifest: Vec<ManifestData> = read_vec_from(reader, 300)?;
        let include_custom_names = manifest[17].size == 72;
        let include_world_locations_extras = manifest[25].size == 572;
        let manifest_field_28 = manifest[27].clone();
        let include_pyramids = manifest[28].size == 300;

        return Ok(MapData {
            version_1,
            version_2,
            manifest,
            sprite: read_compressed_vec_from(reader, 51984)?,
            root_offset: read_compressed_vec_from(reader, 51984)?,
            terrain: read_compressed_vec_from(reader, 51984)?,
            tile_size: read_compressed_vec_from(reader, 51984)?,
            random: read_compressed_vec_from(reader, 51984)?,
            field_10: read_compressed_vec_from(reader, 51984)?,
            seed_1: ReadFrom::read_from(reader)?,
            seed_2: ReadFrom::read_from(reader)?,
            field_13: ReadFrom::read_from(reader)?,
            field_14: ReadFrom::read_from(reader)?,
            scenario_data: ReadFrom::read_from(reader)?,
            meadow: read_compressed_vec_from(reader, 51984)?,
            field_17: ReadFrom::read_from(reader)?,
            world_map_elements: WorldMapElementData::read_vec_from(reader, include_custom_names)?,
            trade_routes: TradeRouteData::read_vec_from(reader)?,
            field_20: read_compressed_vec_from(reader, 51984)?,
            field_21: read_compressed_vec_from(reader, 36)?,
            prices: ReadFrom::read_from(reader)?,
            scrub: read_compressed_vec_from(reader, 51984)?,
            elevation: read_compressed_vec_from(reader, 51984)?,
            elevation_rotation: read_compressed_vec_from(reader, 51984)?,
            world_locations: WorldLocationData::read_vec_from(reader, include_world_locations_extras)?,
            background_image: ReadFrom::read_from(reader)?,
            field_28: read_segment(reader, &manifest_field_28)?,
            mythology: MythologyData::read_from(reader, include_pyramids)?,
            field_30: ReadFrom::read_from(reader)?,
            field_31: ReadFrom::read_from(reader)?,
            field_32: ReadFrom::read_from(reader)?,
            field_33: ReadFrom::read_from(reader)?,
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
        bytes += write_compressed(&self.field_10, writer)?;
        bytes += WriteTo::write_to(&self.seed_1, writer)?;
        bytes += WriteTo::write_to(&self.seed_2, writer)?;
        bytes += WriteTo::write_to(&self.field_13, writer)?;
        bytes += WriteTo::write_to(&self.field_14, writer)?;
        bytes += WriteTo::write_to(&self.scenario_data, writer)?;
        bytes += write_compressed(&self.meadow, writer)?;
        bytes += WriteTo::write_to(&self.field_17, writer)?;
        bytes += WorldMapElementData::write_vec_to(&self.world_map_elements, writer, include_custom_names)?;
        bytes += write_compressed(&self.trade_routes, writer)?;
        bytes += write_compressed(&self.field_20, writer)?;
        bytes += write_compressed(&self.field_21, writer)?;
        bytes += WriteTo::write_to(&self.prices, writer)?;
        bytes += write_compressed(&self.scrub, writer)?;
        bytes += write_compressed(&self.elevation, writer)?;
        bytes += write_compressed(&self.elevation_rotation, writer)?;
        bytes += WorldLocationData::write_vec_to(&self.world_locations, writer, include_world_locations_extras)?;
        bytes += WriteTo::write_to(&self.background_image, writer)?;
        bytes += WriteTo::write_to(&self.field_28, writer)?;
        bytes += MythologyData::write_to(&self.mythology, writer, include_pyramids)?;
        bytes += WriteTo::write_to(&self.field_30, writer)?;
        bytes += WriteTo::write_to(&self.field_31, writer)?;
        bytes += WriteTo::write_to(&self.field_32, writer)?;
        bytes += WriteTo::write_to(&self.field_33, writer)?;

        return Ok(bytes);
    }
}

impl Default for MapData {
    fn default() -> Self {
        return MapData {
            version_1: 0,
            version_2: 0,
            manifest: vec![ManifestData::default(); 300],
            sprite: vec![0; 51984],
            root_offset: vec![0; 51984],
            terrain: vec![0; 51984],
            tile_size: vec![0; 51984],
            random: vec![0; 51984],
            field_10: vec![0; 51984],
            seed_1: 0,
            seed_2: 0,
            field_13: 0,
            field_14: 0,
            scenario_data: Default::default(),
            meadow: vec![0; 51984],
            field_17: [0; 18628],
            world_map_elements: vec![WorldMapElementData::default(); 200],
            trade_routes: vec![TradeRouteData::default(); 232],
            field_20: vec![0; 51984],
            field_21: vec![0; 36],
            prices: [0; 36],
            scrub: vec![0; 51984],
            elevation: vec![0; 51984],
            elevation_rotation: vec![0; 51984],
            world_locations: vec![WorldLocationData::default(); 22],
            background_image: 0,
            field_28: vec![],
            mythology: Default::default(),
            field_30: [[0; 76]; 6],
            field_31: 0,
            field_32: 0,
            field_33: [[0; 36]; 10],
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::file_data::map_data::MapData;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Seek;
    use std::io::SeekFrom;

    #[test]
    fn test_map_files() {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not file_data");

        let adventures_folder = fs::read_dir(format!("{}/Adventures", game_root)).unwrap();

        let mut files_tested = 0;

        for adventure in adventures_folder {
            let folder_path = adventure.unwrap().path();
            if folder_path.is_dir() {
                let files = fs::read_dir(folder_path).unwrap();
                for file in files {
                    let file_path = file.unwrap().path();
                    let extension = file_path.extension().map_or("", |x| x.to_str().unwrap_or(""));
                    if extension == "map" {
                        let mut reader = File::open(&file_path).map(BufReader::new).unwrap();
                        let zeus_map_file = MapData::read_from(&mut reader).unwrap();

                        let current_pos = reader.stream_position().unwrap();
                        reader.seek(SeekFrom::End(0)).unwrap();
                        let end_pos = reader.stream_position().unwrap();
                        let unread = end_pos - current_pos;

                        assert_eq!(unread, 0);

                        assert_eq!(zeus_map_file.version_2, 33);

                        // let mut bytes = vec![];
                        // let mut buffer_writer = Cursor::new(&mut bytes);
                        // let written = zeus_map_file.write_to(&mut buffer_writer).unwrap();
                        //
                        // assert_eq!(written, bytes.len());
                        //
                        // let mut buffer_reader = Cursor::new(&mut bytes);
                        // let reconstructed = MapData::read_from(&mut buffer_reader).unwrap();
                        // LogDifferences::log_differences(
                        //     &zeus_map_file,
                        //     &reconstructed,
                        //     format!("{:?}", file_path.file_name().unwrap())
                        // );

                        files_tested += 1;
                    }
                }
            }
        }

        assert!(files_tested >= 8);
    }
}
