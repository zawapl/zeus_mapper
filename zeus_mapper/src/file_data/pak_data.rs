use crate::adventure::AdventureType;
use crate::adventure::Civilization;
use crate::constants::data_constant::DataConstant;
use crate::file_data::map_data::MapData;
use crate::file_data::settings_data::SettingsData;
use crate::prelude::ValidationError;
use crate::utils::validation::ValidationResult;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, LogDifferences)]
pub struct PakData {
    pub settings_data: SettingsData,
    pub map_data: Vec<MapData>,
}

impl Default for PakData {
    fn default() -> Self {
        return PakData {
            settings_data: Default::default(),
            map_data: vec![Default::default()],
        };
    }
}

impl PakData {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let settings_data = SettingsData::read_from(reader)?;
        let map_data = MapData::read_maps(reader, 1 + settings_data.colony_episodes_available as usize)?;
        return Ok(PakData { settings_data, map_data });
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        return WriteTo::write_to(self, writer);
    }

    pub fn civilization(&self) -> Civilization {
        let adventure_type = AdventureType::try_resolve(&self.settings_data.adventure_type).unwrap_or(AdventureType::PoseidonCustom);
        if matches!(
            adventure_type,
            AdventureType::ZeusCampaign | AdventureType::ZeusCustom | AdventureType::Tutorial
        ) {
            return Civilization::Greek;
        }

        return Civilization::try_resolve(&self.settings_data.real_episode_data[0].civilization).unwrap_or(Civilization::Greek);
    }

    pub fn validate(&self) -> ValidationResult {
        self.settings_data
            .validate()
            .map_err(ValidationError::add_parent("settings_data"))?;

        for (i, map_data) in self.map_data.iter().enumerate() {
            map_data.validate().map_err(ValidationError::add_parent(format!("map_data[{i}]")))?;
        }

        self.validate_attacking_cities()?;
        self.validate_on_success_targets()?;

        return Ok(());
    }

    fn validate_attacking_cities(&self) -> ValidationResult {
        let valid_attackers = self.map_data[0]
            .world_locations
            .iter()
            .enumerate()
            .filter(|(_, location)| location.location_type != 0 && location.location_type != 1)
            .map(|(i, _)| i as u16)
            .collect::<Vec<_>>();

        for (i, row) in self.settings_data.events.iter().enumerate() {
            let real_count = self.real_event_count(i);

            for (j, event) in row.iter().take(real_count).enumerate() {
                let is_attacker_field =
                    (event.event_type == 1 && matches!(event.subtype, 1 | 2)) || (event.event_type == 19 && event.subtype == 24);
                if !is_attacker_field {
                    continue;
                }

                let attacking_city = event.other_city as u16;

                if !valid_attackers.contains(&attacking_city) {
                    return Err(ValidationError::expected_one_of(
                        format!("settings_data.events[{i}][{j}].other_city"),
                        attacking_city,
                        valid_attackers.as_ref(),
                    ));
                }
            }
        }

        return Ok(());
    }

    fn validate_on_success_targets(&self) -> ValidationResult {
        for (i, row) in self.settings_data.events.iter().enumerate() {
            let real_count = self.real_event_count(i);
            let row_events = &row[..real_count];

            for (j, event) in row_events.iter().enumerate() {
                if !matches!(event.event_type, 4 | 26) || event.on_success < 0 {
                    continue;
                }

                if row_events.get(event.on_success as usize).is_none() {
                    return Err(ValidationError::expected_range(
                        format!("settings_data.events[{i}][{j}].on_success"),
                        event.on_success,
                        0,
                        real_count.saturating_sub(1) as i16,
                    ));
                }
            }
        }

        return Ok(());
    }

    fn real_event_count(&self, row_index: usize) -> usize {
        if row_index < self.settings_data.parent_event_counts.len() {
            return self.settings_data.parent_event_counts.get(row_index).copied().unwrap_or(0) as usize;
        }

        return self
            .settings_data
            .colony_event_counts
            .get(row_index - self.settings_data.parent_event_counts.len())
            .copied()
            .unwrap_or(0) as usize;
    }
}

impl WriteTo for PakData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.settings_data, writer)?;
        bytes += WriteTo::write_to(&self.map_data, writer)?;

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use crate::file_data::pak_data::PakData;
    use crate::prelude::BasicEpisodeData;
    use crate::prelude::BoxedArray;
    use crate::prelude::EpisodeGoalData;
    use crate::prelude::EventData;
    use crate::prelude::ManifestData;
    use crate::prelude::MapData;
    use crate::prelude::MythologyData;
    use crate::prelude::PyramidData;
    use crate::prelude::RealEpisodeData;
    use crate::prelude::SettingsData;
    use crate::prelude::TradeRouteData;
    use crate::prelude::TradeRoutePointData;
    use crate::prelude::UnconfirmedSign;
    use crate::prelude::WorldLocationData;
    use crate::prelude::WorldMapElementData;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Cursor;
    use std::io::Result;
    use std::io::Seek;
    use std::io::SeekFrom;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn default_is_valid() {
        if let Err(e) = PakData::default().validate() {
            panic!("{e}");
        }
    }

    #[test]
    fn validate_pak_files() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let mut pak_paths = vec![];
            collect_paks(&PathBuf::from(format!("{game_root}/Adventures")), &mut pak_paths)?;

            for path in pak_paths {
                let mut reader = File::open(&path).map(BufReader::new)?;
                let pak_data = PakData::read_from(&mut reader)?;

                let current_pos = reader.stream_position()?;
                reader.seek(SeekFrom::End(0))?;
                let end_pos = reader.stream_position()?;
                let unread = end_pos - current_pos;

                assert_eq!(unread, 0, "{path:?}: {unread} unread bytes");

                if let Err(e) = pak_data.validate() {
                    panic!("{path:?}: {e}");
                }
            }
        }

        return Ok(());
    }

    fn collect_paks(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                collect_paks(&path, out)?;
            } else if path.extension().map_or(false, |e| e == "pak") {
                out.push(path);
            }
        }

        return Ok(());
    }

    #[test]
    fn byte_identical_adventures_round_trip_exactly() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            // Not all adventures complete the round trip with byte identical results - below ones do
            let adventures = [
                "#The Birth of Atlantis.pak",
                "&Enlightenment in the West.pak",
                "@Athens through the Ages.pak",
                "@Life in the Mediterranean.pak",
                "Open Play Economic 2/Open Play Economic 2.pak",
                "Open Play Military 2/Open Play Military 2.pak",
                "The Founding of Troy/The Founding of Troy.pak",
                "The Mayan Revolution/The Mayan Revolution.pak",
                "The Odyssey/The Odyssey.pak",
                "The Sinking of Atlantis/The Sinking of Atlantis.pak",
                "The Youngest Twins/The Youngest Twins.pak",
                "[Atlantis Reborn.pak",
                "]Proetus and Bellerophon.pak",
                "^Two Worlds Collide.pak",
                "}Open Play Sandbox.pak",
            ];

            for name in adventures {
                let path = PathBuf::from(format!("{game_root}/Adventures/{name}"));
                let original = fs::read(&path)?;
                let mut reader = Cursor::new(&original);
                let pak_data = PakData::read_from(&mut reader)?;

                let mut regenerated = vec![];
                pak_data.write_to(&mut regenerated)?;

                assert_eq!(original, regenerated, "{name} did not round-trip byte-identically");
            }
        }

        return Ok(());
    }

    #[test]
    fn write_read_round_trip() -> Result<()> {
        let mut test_generator = TestDataGenerator::default();
        let pak_data: PakData = test_generator.generate_data();

        let mut buffer = vec![];

        pak_data.write_to(&mut buffer)?;

        let reconstructed = PakData::read_from(&mut Cursor::new(&mut buffer))?;

        assert_eq!(reconstructed, pak_data, "PakData mismatch");

        return Ok(());
    }

    /// Helper trait for generating test data
    trait GenerateData<T> {
        fn generate_data(&mut self) -> T;
    }

    /// Test data generator that assigns incrementing values to all the fields
    #[derive(Default)]
    struct TestDataGenerator(usize);

    impl GenerateData<PakData> for TestDataGenerator {
        fn generate_data(&mut self) -> PakData {
            return PakData {
                settings_data: self.generate_data(),
                map_data: (0..2).map(|_| self.generate_data()).collect(),
            };
        }
    }

    impl GenerateData<SettingsData> for TestDataGenerator {
        fn generate_data(&mut self) -> SettingsData {
            let mut draft = SettingsData {
                version_1: self.generate_data(),
                version_2: 26,
                parent_episodes: self.generate_data(),
                colony_episodes_used: self.generate_data(),
                colony_episodes_available: 1,
                basic_episode_data: self.generate_data(),
                real_episode_data: self.generate_data(),
                unknown_1: self.generate_data(),
                colony_location_names: self.generate_data(),
                unknown_2: self.generate_data(),
                mythology: self.generate_data(),
                events: self.generate_data(),
                adventure_type: self.generate_data(),
                constant_1_0x00: self.generate_data(),
                data_length: self.generate_data(),
                unknown_3: self.generate_data(),
                map_data: self.generate_data(),
                padding: vec![],
                parent_event_counts: self.generate_data(),
                colony_event_counts: self.generate_data(),
                unused_blocks: self.generate_data(),
                parent_city_favor: self.generate_data(),
                constant_2_0x00: self.generate_data(),
                bitmap: self.generate_data(),
                tab_visibility: self.generate_data(),
                constant_3_0x01: self.generate_data(),
                unknown_4: self.generate_data(),
                constant_4_0x01: self.generate_data(),
                unknown_5: self.generate_data(),
                constant_5_0x01: self.generate_data(),
                unknown_6: self.generate_data(),
                unknown_7: self.generate_data(),
                constant_6_0x01: self.generate_data(),
                unknown_8: self.generate_data(),
                constant_7_0x01: self.generate_data(),
                constant_8_0x00: self.generate_data(),
                world_map_enabled: self.generate_data(),
                constant_9_0x01: self.generate_data(),
                unknown_9: self.generate_data(),
                unknown_10: self.generate_data(),
                colony_episode_goal_counts: self.generate_data(),
                colony_episode_goals: self.generate_data(),
                parent_episode_goal_counts: self.generate_data(),
                parent_episode_goals: self.generate_data(),
            };

            // `padding` is slack space sized by how many bytes everything ahead of it (including
            // the compressed `map_data`) actually took up on the wire, which isn't known until
            // it's written. Write once with an empty placeholder, then read back to learn the
            // length it needs to be populated with to round-trip byte-identically.
            let mut buffer = vec![];
            draft.write_to(&mut buffer).expect("draft SettingsData should write");
            let reread = SettingsData::read_from(&mut Cursor::new(&buffer)).expect("draft SettingsData should read back");

            draft.padding = (0..reread.padding.len()).map(|_| self.generate_data()).collect();

            return draft;
        }
    }

    impl GenerateData<MapData> for TestDataGenerator {
        fn generate_data(&mut self) -> MapData {
            let mut manifest: BoxedArray<ManifestData, 300> = self.generate_data();
            manifest[17].size = 72; // include_custom_names
            manifest[25].size = 572; // include_world_locations_extras
            manifest[27].size = 10; // unknown_5 row size
            manifest[27].count = 3; // unknown_5 row count
            manifest[28].size = 300; // include_pyramids

            return MapData {
                version_1: self.generate_data(),
                version_2: self.generate_data(),
                manifest,
                sprite: self.generate_data(),
                root_offset: self.generate_data(),
                terrain: self.generate_data(),
                tile_size: self.generate_data(),
                random: self.generate_data(),
                constant_1_0x00: self.generate_data(),
                seed_1: self.generate_data(),
                seed_2: self.generate_data(),
                camera_x: self.generate_data(),
                camera_y: self.generate_data(),
                scenario_data: self.generate_data(),
                meadow: self.generate_data(),
                unknown_3: self.generate_data(),
                world_map_elements: self.generate_data(),
                trade_routes: self.generate_data(),
                constant_2_0xff: self.generate_data(),
                constant_3: BoxedArray::from_vec(vec![180, 70, 100, 180, 70, 100, 0, 100, 0]),
                prices: self.generate_data(),
                scrub: self.generate_data(),
                elevation: self.generate_data(),
                elevation_rotation: self.generate_data(),
                world_locations: self.generate_data(),
                background_image: self.generate_data(),
                unknown_5: (0..3).map(|_| (0..10).map(|_| self.generate_data()).collect()).collect(),
                mythology: self.generate_data(),
                unknown_6: self.generate_data(),
                unknown_7: self.generate_data(),
                unknown_8: self.generate_data(),
                unknown_9: self.generate_data(),
            };
        }
    }

    impl GenerateData<ManifestData> for TestDataGenerator {
        fn generate_data(&mut self) -> ManifestData {
            return ManifestData {
                compressed: self.generate_data(),
                address: self.generate_data(),
                size: self.generate_data(),
                count: self.generate_data(),
                unknown_1: self.generate_data(),
            };
        }
    }

    impl GenerateData<MythologyData> for TestDataGenerator {
        fn generate_data(&mut self) -> MythologyData {
            return MythologyData {
                opponent_gods: self.generate_data(),
                proponent_gods: self.generate_data(),
                monster: self.generate_data(),
                constant_1_0xff: self.generate_data(),
                constant_2_0x00: self.generate_data(),
                sanctuaries_allowed: self.generate_data(),
                max_sanctuaries: self.generate_data(),
                max_pyramids: self.generate_data(),
                pyramids: (0..6).map(|_| self.generate_data()).collect(),
            };
        }
    }

    impl GenerateData<PyramidData> for TestDataGenerator {
        fn generate_data(&mut self) -> PyramidData {
            return PyramidData {
                pyramid_type: self.generate_data(),
                deity: self.generate_data(),
                coloration: self.generate_data(),
            };
        }
    }

    impl GenerateData<EventData> for TestDataGenerator {
        fn generate_data(&mut self) -> EventData {
            return EventData {
                id: self.generate_data(),
                event_type: self.generate_data(),
                month: self.generate_data(),
                item_chosen: self.generate_data(),
                first_item: self.generate_data(),
                second_item: self.generate_data(),
                third_item: self.generate_data(),
                amount: self.generate_data(),
                fixed_amount: self.generate_data(),
                min_amount: self.generate_data(),
                max_amount: self.generate_data(),
                time: self.generate_data(),
                fixed_time: self.generate_data(),
                min_time: self.generate_data(),
                max_time: self.generate_data(),
                target: self.generate_data(),
                fixed_target: self.generate_data(),
                min_target: self.generate_data(),
                max_target: self.generate_data(),
                on_success: self.generate_data(),
                on_failure: self.generate_data(),
                flags: self.generate_data(),
                warnings: self.generate_data(),
                time_ctr: self.generate_data(),
                status: self.generate_data(),
                need_msg_res: self.generate_data(),
                triggerer: self.generate_data(),
                god_or_mon_or_warship_id: self.generate_data(),
                mtar1: self.generate_data(),
                mtar2: self.generate_data(),
                mtar3: self.generate_data(),
                magg: self.generate_data(),
                unkown_row: self.generate_data(),
                trigger_on_1: self.generate_data(),
                trigger_on_2: self.generate_data(),
                eff_on_city: self.generate_data(),
                source: self.generate_data(),
                source_fixed: self.generate_data(),
                source_min: self.generate_data(),
                source_max: self.generate_data(),
                subtype: self.generate_data(),
                prev_amount: self.generate_data(),
                related_to_triggered_evt: self.generate_data(),
                constant_1_0x00: self.generate_data(),
                trig_reason: self.generate_data(),
                constant_2_0x00: self.generate_data(),
                constant_3_0x00: self.generate_data(),
                other_city: self.generate_data(),
                loot_type: self.generate_data(),
                loot_amount: self.generate_data(),
                constant_4_0x00: self.generate_data(),
                ally_city: self.generate_data(),
                ally_strength: self.generate_data(),
                to_strength: self.generate_data(),
                constant_5_0x00: self.generate_data(),
                permanent_flag: self.generate_data(),
                quest: self.generate_data(),
                tail: self.generate_data(),
            };
        }
    }

    impl GenerateData<TradeRouteData> for TestDataGenerator {
        fn generate_data(&mut self) -> TradeRouteData {
            return TradeRouteData {
                constant_1_0x05: self.generate_data(),
                points: self.generate_data(),
                distance: self.generate_data(),
                route_type: self.generate_data(),
                points_count: self.generate_data(),
                exists: self.generate_data(),
                constant_2_0x00: self.generate_data(),
            };
        }
    }

    impl GenerateData<TradeRoutePointData> for TestDataGenerator {
        fn generate_data(&mut self) -> TradeRoutePointData {
            return TradeRoutePointData {
                x: self.generate_data(),
                y: self.generate_data(),
                unknown_1: self.generate_data(),
            };
        }
    }

    impl GenerateData<WorldLocationData> for TestDataGenerator {
        fn generate_data(&mut self) -> WorldLocationData {
            return WorldLocationData {
                exists: self.generate_data(),
                location_type: self.generate_data(),
                name: self.generate_data(),
                slot_index: self.generate_data(),
                unknown_1: self.generate_data(),
                trade_quantities: self.generate_data(),
                unknown_2: self.generate_data(),
                buying: self.generate_data(),
                selling: self.generate_data(),
                civilization: self.generate_data(),
                leader_name: self.generate_data(),
                attitude: self.generate_data(),
                economical_strength: self.generate_data(),
                military_strength: self.generate_data(),
                tribute: self.generate_data(),
                tribute_rec_amount: self.generate_data(),
                tribute_pay_amount: self.generate_data(),
                tribute_pay_resource: self.generate_data(),
                tribute_rec_resource: self.generate_data(),
                unknown_3: self.generate_data(),
                favour: self.generate_data(),
                constant_1_0x00: self.generate_data(),
                active_old: self.generate_data(),
                favour_or_visibility: self.generate_data(),
                constant_3_0x00: self.generate_data(),
                active_new: self.generate_data(),
                visible: self.generate_data(),
                unknown_4: self.generate_data(),
                unknown_5: (0..28).map(|_| self.generate_data()).collect(),
                trade_route_visible: self.generate_data(),
                custom_name: self.generate_data(),
                custom_leader_name: self.generate_data(),
                tail: (0..3).map(|_| self.generate_data()).collect(),
            };
        }
    }

    impl GenerateData<WorldMapElementData> for TestDataGenerator {
        fn generate_data(&mut self) -> WorldMapElementData {
            return WorldMapElementData {
                variant: self.generate_data(),
                data_a: self.generate_data(),
                x: self.generate_data(),
                y: self.generate_data(),
                sprite_width: self.generate_data(),
                sprite_height: self.generate_data(),
                sprite_id: self.generate_data(),
                unknown_1: self.generate_data(),
                label_position: self.generate_data(),
                unknown_2: self.generate_data(),
                region_name: self.generate_data(),
                city_name: self.generate_data(),
                data_d: self.generate_data(),
                custom_names: self.generate_data(),
            };
        }
    }

    impl GenerateData<RealEpisodeData> for TestDataGenerator {
        fn generate_data(&mut self) -> RealEpisodeData {
            return RealEpisodeData {
                start_date: self.generate_data(),
                constant_1_0x00: self.generate_data(),
                months_elapsed: self.generate_data(),
                constant_2_0x00: self.generate_data(),
                starting_cash: self.generate_data(),
                unknown_1: self.generate_data(),
                map_size: self.generate_data(),
                map_size_duplicate: self.generate_data(),
                first_tile_index: self.generate_data(),
                map_size_margin: self.generate_data(),
                text_buffer_1: self.generate_data(),
                text_buffer_2: self.generate_data(),
                civilization: self.generate_data(),
                wolf_x: self.generate_data(),
                wolf_y: self.generate_data(),
                fish_x: self.generate_data(),
                fish_y: self.generate_data(),
                urchin_x: self.generate_data(),
                urchin_y: self.generate_data(),
                constant_3_0x00: self.generate_data(),
                invasion_x: self.generate_data(),
                invasion_y: self.generate_data(),
                panhellenic_games: self.generate_data(),
                colonies_done: self.generate_data(),
                deer_x: self.generate_data(),
                deer_y: self.generate_data(),
                unknown_3: self.generate_data(),
                earthquake_area: self.generate_data(),
                entry_x: self.generate_data(),
                entry_y: self.generate_data(),
                exit_x: self.generate_data(),
                exit_y: self.generate_data(),
                disaster_x: self.generate_data(),
                disaster_y: self.generate_data(),
                river_entry_x: self.generate_data(),
                river_entry_y: self.generate_data(),
                river_exit_x: self.generate_data(),
                river_exit_y: self.generate_data(),
                unknown_4: self.generate_data(),
                tropical: self.generate_data(),
                boar_x: self.generate_data(),
                boar_y: self.generate_data(),
                building_flags: self.generate_data(),
                constant_4_0x00: self.generate_data(),
                monster_x: self.generate_data(),
                monster_y: self.generate_data(),
                disembark_x: self.generate_data(),
                disembark_y: self.generate_data(),
                constant_5_0x00: self.generate_data(),
                landslide_x: self.generate_data(),
                landslide_y: self.generate_data(),
                constant_6_0x00: self.generate_data(),
                basic_episode_data: self.generate_data(),
                city_resources: self.generate_data(),
                city_resources_bought: self.generate_data(),
                constant_7_0x00: self.generate_data(),
                city_resources_sold: self.generate_data(),
                unknown_5: self.generate_data(),
                city_resources_quantity: self.generate_data(),
            };
        }
    }

    impl GenerateData<BasicEpisodeData> for TestDataGenerator {
        fn generate_data(&mut self) -> BasicEpisodeData {
            return BasicEpisodeData {
                exists: self.generate_data(),
                constant_1_0x00: self.generate_data(),
                unknown_1: self.generate_data(),
                episode_no: self.generate_data(),
                unknown_2: self.generate_data(),
                unknown_3: self.generate_data(),
                unknown_4: self.generate_data(),
                next_episode: self.generate_data(),
                episode_type: self.generate_data(),
                name_padding: self.generate_data(),
                name: self.generate_data(),
            };
        }
    }

    impl GenerateData<EpisodeGoalData> for TestDataGenerator {
        fn generate_data(&mut self) -> EpisodeGoalData {
            return EpisodeGoalData {
                goal_type: self.generate_data(),
                resource_id: self.generate_data(),
                amount: self.generate_data(),
                unknown_1: self.generate_data(),
            };
        }
    }

    impl<T, const N: usize> GenerateData<BoxedArray<T, N>> for TestDataGenerator
    where
        T: Default,
        TestDataGenerator: GenerateData<T>,
    {
        fn generate_data(&mut self) -> BoxedArray<T, N> {
            let vec = (0..N).map(|_i| self.generate_data()).collect();
            return BoxedArray::from_vec(vec);
        }
    }

    impl<T, const N: usize> GenerateData<[T; N]> for TestDataGenerator
    where
        TestDataGenerator: GenerateData<BoxedArray<T, N>>,
    {
        fn generate_data(&mut self) -> [T; N] {
            let boxed_array: BoxedArray<T, N> = self.generate_data();
            return *boxed_array.0;
        }
    }

    impl TestDataGenerator {
        fn next(&mut self) -> usize {
            self.0 += 1;
            return self.0;
        }
    }

    impl GenerateData<u8> for TestDataGenerator {
        fn generate_data(&mut self) -> u8 {
            return self.next() as u8;
        }
    }

    impl GenerateData<i8> for TestDataGenerator {
        fn generate_data(&mut self) -> i8 {
            return self.next() as i8;
        }
    }

    impl GenerateData<u16> for TestDataGenerator {
        fn generate_data(&mut self) -> u16 {
            return self.next() as u16;
        }
    }

    impl<T> GenerateData<UnconfirmedSign<T>> for TestDataGenerator
    where
        TestDataGenerator: GenerateData<T>,
    {
        fn generate_data(&mut self) -> UnconfirmedSign<T> {
            return UnconfirmedSign(self.generate_data());
        }
    }

    impl GenerateData<i16> for TestDataGenerator {
        fn generate_data(&mut self) -> i16 {
            return self.next() as i16;
        }
    }

    impl GenerateData<u32> for TestDataGenerator {
        fn generate_data(&mut self) -> u32 {
            return self.next() as u32;
        }
    }

    impl GenerateData<i32> for TestDataGenerator {
        fn generate_data(&mut self) -> i32 {
            return self.next() as i32;
        }
    }

    impl GenerateData<u64> for TestDataGenerator {
        fn generate_data(&mut self) -> u64 {
            return self.next() as u64;
        }
    }

    impl GenerateData<String> for TestDataGenerator {
        fn generate_data(&mut self) -> String {
            return self.next().to_string();
        }
    }
}
