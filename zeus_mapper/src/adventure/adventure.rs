use crate::adventure::AdventureType;
use crate::adventure::Civilization;
use crate::adventure::MonetaryAmount;
use crate::adventure::NextEpisode;
use crate::adventure::building_type::BuildingType;
use crate::adventure::city_map::CityMap;
use crate::adventure::colony_episode::COLONY_SETTINGS_OFFSET;
use crate::adventure::colony_episode::ColonyEpisode;
use crate::adventure::episode_goals::EpisodeGoal;
use crate::adventure::event::Event;
use crate::adventure::parent_city_episode::ParentEpisode;
use crate::adventure::pyramid::PyramidSetting;
use crate::adventure::resource::ResourceType;
use crate::adventure::resource::TradedGoods;
use crate::adventure::trade_route::TradeRoute;
use crate::adventure::world_location::WorldLocation;
use crate::l10n::adventure_text::AdventureText;
use crate::prelude::BasicEpisodeData;
use crate::prelude::BoxedArray;
use crate::prelude::DataConstant;
use crate::prelude::EpisodeGoalData;
use crate::prelude::EventData;
use crate::prelude::MythologyData;
use crate::prelude::PakData;
use crate::prelude::RealEpisodeData;
use crate::prelude::SettingsData;
use my_macros::LogDifferences;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;

// There are issues with adventures with long names, UI allows for ~21, longest observed is 23
// Might need a proper investigation to find what a safe limit is
const MAX_ADVENTURE_NAME_LENGTH: usize = 23;

#[derive(LogDifferences, PartialEq, Debug)]
pub struct Adventure {
    pub title: String,
    pub introduction_text: String,
    pub complete_text: String,
    pub civilization: Civilization,
    pub adventure_type: AdventureType,
    pub bitmap: u16,
    pub world_bitmap: u32,
    pub parent_city: CityMap,
    // add world properties here
    pub prices: Vec<u32>,
    pub world_locations: Vec<WorldLocation>,
    pub trade_routes: Vec<TradeRoute>,
    pub available_colonies: u8, // probably remove this
    pub start_year: i16,
    pub initial_funds: MonetaryAmount,
    pub parent_episodes: Vec<ParentEpisode>,
    pub colony_episodes: Vec<ColonyEpisode>,
}

impl Adventure {
    pub fn from_pak(pak_data: &PakData, adventure_text: &AdventureText) -> Adventure {
        // `prices`/`world_locations` live in `MapData`, not `SettingsData` - resaving an adventure
        // into the new settings format (`SettingsData.version_2 == 26`) does not migrate `MapData`'s
        // own embedded resource ids (confirmed: resaving `The Odyssey` into the new settings format
        // left `MapData` byte-identical, including its resource ids, until the map itself was later
        // separately re-edited). `MapData.version_1` is `MapData`'s own format marker and tracks
        // whether its embedded resource ids use the new numbering instead - old-format maps observed
        // in the range 234-247, new-format maps at 319-321 across every real adventure surveyed, a
        // clean gap with no observed values in between.
        let map_new_file_ver = pak_data.map_data[0].version_1 >= 300;

        // `colony_episodes_available` sometimes claims more colonies than the file actually backs
        // with map data (e.g. `bigcity.pak`: `colony_episodes_available == 1` but only the parent
        // map is present) - `ColonyEpisode::vec_from_data` already clamps to what's actually usable,
        // so `available_colonies` is read from its result rather than the raw field to stay
        // consistent with it (and with what `Adventure::to_pak` writes back).
        let colony_episodes = ColonyEpisode::vec_from_data(pak_data, adventure_text);

        return Adventure {
            title: adventure_text.title.to_owned(),
            introduction_text: adventure_text.introduction.to_owned(),
            complete_text: adventure_text.complete.to_owned(),
            civilization: pak_data.civilization(),
            adventure_type: AdventureType::try_resolve(&pak_data.settings_data.adventure_type).unwrap_or(AdventureType::ZeusCampaign),
            bitmap: pak_data.settings_data.bitmap as u16,
            world_bitmap: pak_data.map_data[0].background_image,
            parent_city: CityMap::from_map_data(&pak_data.map_data[0]),
            prices: ResourceType::prices_from_data(pak_data.map_data[0].prices.as_slice(), map_new_file_ver),
            parent_episodes: ParentEpisode::vec_from_data(pak_data, adventure_text),
            available_colonies: colony_episodes.len() as u8,
            colony_episodes,
            initial_funds: pak_data.settings_data.real_episode_data[0].starting_cash,
            start_year: pak_data.settings_data.real_episode_data[0].start_date,
            world_locations: WorldLocation::vec_from_data(
                &pak_data.map_data[0].world_locations,
                &pak_data.map_data[0].world_map_elements,
                map_new_file_ver,
            ),
            trade_routes: TradeRoute::vec_from_data(&pak_data.map_data[0].trade_routes),
        };
    }

    pub fn read_from(folder: impl AsRef<Path>) -> io::Result<Adventure> {
        let folder = folder.as_ref();
        let name = folder_name(folder)?;

        let mut pak_reader = File::open(folder.join(format!("{name}.pak"))).map(BufReader::new)?;
        let pak_data = PakData::read_from(&mut pak_reader)?;

        let mut text_reader = File::open(folder.join(format!("{name}.txt"))).map(BufReader::new)?;
        let adventure_text = AdventureText::read_from(&mut text_reader)?;

        return Ok(Adventure::from_pak(&pak_data, &adventure_text));
    }

    pub fn to_pak(&self) -> PakData {
        let mut basic_episode_data: [BasicEpisodeData; 20] = Default::default();
        let mut real_episode_data: [RealEpisodeData; 14] = Default::default();
        let mut mythology: [MythologyData; 14] = Default::default();
        let mut events: [BoxedArray<EventData, 150>; 14] = Default::default();
        let mut colony_episode_goals: [[EpisodeGoalData; 6]; 4] = Default::default();
        let mut colony_episode_goal_counts = [0u32; 4];
        let mut parent_episode_goal_counts = [0u32; 10];
        let mut parent_episode_goals: [[EpisodeGoalData; 6]; 10] = Default::default();
        let mut parent_city_favor = [0u32; 10];

        real_episode_data[0].civilization = self.civilization.value();
        real_episode_data[0].starting_cash = self.initial_funds;
        real_episode_data[0].start_date = self.start_year;

        let episode_count = usize::min(self.parent_episodes.len(), basic_episode_data.len());

        for (i, episode) in self.parent_episodes.iter().take(episode_count).enumerate() {
            let this_episode_data = BasicEpisodeData {
                exists: 1,
                constant_1_0x00: 0,
                unknown_1: u32::MAX,
                episode_no: (i + 1) as u32,
                unknown_2: u32::MAX,
                unknown_3: u32::MAX,
                unknown_4: Default::default(),
                next_episode: episode.next_episode.value(),
                episode_type: 0,
                name_padding: 0,
                name: String::new(),
            };
            real_episode_data[i].basic_episode_data = this_episode_data.clone();
            basic_episode_data[i] = this_episode_data;

            real_episode_data[i].panhellenic_games = if episode.panhellenic_games { 0 } else { 1 };
            real_episode_data[i].building_flags = BuildingType::vec_to_data(&episode.building_allowed);
            real_episode_data[i].city_resources = ResourceType::vec_to_data(&episode.city_resources);
            mythology[i] = episode.mythology.to_data();
            let (pyramids, max_pyramids) = PyramidSetting::vec_to_data(&episode.pyramid_settings);
            mythology[i].pyramids = pyramids;
            mythology[i].max_pyramids = max_pyramids;
            events[i] = Event::vec_to_data(&episode.events);

            let (sold, bought, quantity) = TradedGoods::vec_to_data::<4, 40>(&episode.world_settings.traded_resources);
            real_episode_data[i].city_resources_sold = sold;
            real_episode_data[i].city_resources_bought = bought;
            real_episode_data[i].city_resources_quantity = BoxedArray::from_vec(quantity.to_vec());
            if let Some(favour) = parent_city_favor.get_mut(i) {
                *favour = episode.world_settings.favour as u32;
            }

            let (goal_slots, goal_count) = EpisodeGoal::vec_to_episode_goal_data(&episode.episode_goals);
            parent_episode_goals[i] = goal_slots;
            parent_episode_goal_counts[i] = goal_count;
        }

        for (i, colony_episode) in self.colony_episodes.iter().enumerate() {
            let settings_index = COLONY_SETTINGS_OFFSET + i;

            if settings_index < basic_episode_data.len() {
                let this_episode_data = BasicEpisodeData {
                    exists: 1,
                    constant_1_0x00: 0,
                    unknown_1: u32::MAX,
                    episode_no: (settings_index + 1) as u32,
                    unknown_2: u32::MAX,
                    unknown_3: u32::MAX,
                    unknown_4: Default::default(),
                    next_episode: NextEpisode::Parent.value(),
                    episode_type: 0,
                    name_padding: 0,
                    name: String::new(),
                };
                real_episode_data[settings_index].basic_episode_data = this_episode_data.clone();
                basic_episode_data[settings_index] = this_episode_data;
            }
            if let Some(this_episode_data) = real_episode_data.get_mut(settings_index) {
                this_episode_data.panhellenic_games = if colony_episode.panhellenic_games { 0 } else { 1 };
                this_episode_data.building_flags = BuildingType::vec_to_data(&colony_episode.building_allowed);
                this_episode_data.city_resources = ResourceType::vec_to_data(&colony_episode.city_resources);
            }
            if let Some(this_mythology) = mythology.get_mut(settings_index) {
                *this_mythology = colony_episode.mythology.to_data();
                let (pyramids, max_pyramids) = PyramidSetting::vec_to_data(&colony_episode.pyramid_settings);
                this_mythology.pyramids = pyramids;
                this_mythology.max_pyramids = max_pyramids;
            }
            if let Some(this_events) = events.get_mut(settings_index) {
                *this_events = Event::vec_to_data(&colony_episode.events);
            }
            if let Some(this_goals) = colony_episode_goals.get_mut(i) {
                let (goal_slots, goal_count) = EpisodeGoal::vec_to_episode_goal_data(&colony_episode.episode_goals);
                *this_goals = goal_slots;
                if let Some(this_count) = colony_episode_goal_counts.get_mut(i) {
                    *this_count = goal_count;
                }
            }
        }

        let mut map_data = vec![self.parent_city.to_map_data()];
        for colony_episode in &self.colony_episodes {
            map_data.push(colony_episode.city_map.to_map_data());
        }

        let (world_locations, world_map_elements) = WorldLocation::vec_to_data(&self.world_locations, self.civilization);
        let trade_routes = TradeRoute::vec_to_data(&self.trade_routes);
        let prices = BoxedArray::from_vec(self.prices.clone());

        // `SettingsData.colony_location_names` mirrors `world_locations[].name` for the up to 4
        // Colony-type locations, in scan order - not stored independently on `Adventure`, always
        // derived from `world_locations` itself; see DATA_MAPPING.md.
        const COLONY_LOCATION_TYPE: u8 = 1;
        let mut colony_location_names = [u32::MAX; 4];
        let mut colony_slots = colony_location_names.iter_mut();
        for location in world_locations.iter().filter(|l| l.location_type == COLONY_LOCATION_TYPE) {
            if let Some(slot) = colony_slots.next() {
                *slot = location.name as u32;
            }
        }

        for map in &mut map_data {
            map.scenario_data.civilization = self.civilization.value();
            map.world_locations = world_locations.clone();
            map.world_map_elements = world_map_elements.clone();
            map.trade_routes = trade_routes.clone();
            map.prices = prices.clone();
            map.manifest[25].size = 572;
        }

        map_data[0].background_image = self.world_bitmap;

        if let Some(first_parent_episode) = self.parent_episodes.first() {
            map_data[0].scenario_data.building_flags = BuildingType::vec_to_data(&first_parent_episode.building_allowed);
        }
        for (i, colony_episode) in self.colony_episodes.iter().enumerate() {
            if let Some(map) = map_data.get_mut(1 + i) {
                map.scenario_data.building_flags = BuildingType::vec_to_data(&colony_episode.building_allowed);
            }
        }

        let mut map_data_duplicate = match self.colony_episodes.last() {
            Some(last_colony_episode) => {
                let mut last_map = last_colony_episode.city_map.to_map_data();
                last_map.scenario_data.building_flags = BuildingType::vec_to_data(&last_colony_episode.building_allowed);
                last_map
            }
            None => self.parent_city.to_map_data(),
        };
        map_data_duplicate.scenario_data.civilization = self.civilization.value();
        map_data_duplicate.world_locations = world_locations;
        map_data_duplicate.world_map_elements = world_map_elements;
        map_data_duplicate.trade_routes = trade_routes;
        map_data_duplicate.prices = prices;
        map_data_duplicate.manifest[25].size = 572;
        if self.colony_episodes.is_empty()
            && let Some(first_parent_episode) = self.parent_episodes.first()
        {
            map_data_duplicate.scenario_data.building_flags = BuildingType::vec_to_data(&first_parent_episode.building_allowed);
        }

        let mut parent_event_counts = [0u32; 10];
        for (i, episode) in self.parent_episodes.iter().take(parent_event_counts.len()).enumerate() {
            parent_event_counts[i] = episode.events.len() as u32;
        }

        let mut colony_event_counts = [0u32; 4];
        for (i, colony_episode) in self.colony_episodes.iter().take(colony_event_counts.len()).enumerate() {
            colony_event_counts[i] = colony_episode.events.len() as u32;
        }

        let settings_data = SettingsData {
            parent_episodes: self.parent_episodes.len() as u32,
            colony_episodes_available: self.colony_episodes.len() as u32,
            basic_episode_data,
            real_episode_data,
            colony_location_names,
            mythology,
            events: BoxedArray::from_vec(events.into()),
            adventure_type: self.adventure_type.value(),
            map_data: map_data_duplicate,
            parent_event_counts,
            colony_event_counts,
            parent_city_favor,
            bitmap: self.bitmap as u32,
            tab_visibility: [1; 11],
            world_map_enabled: 1,
            colony_episode_goal_counts,
            colony_episode_goals,
            parent_episode_goal_counts,
            parent_episode_goals,
            ..Default::default()
        };

        return PakData { settings_data, map_data };
    }

    pub fn to_text(&self) -> AdventureText {
        return AdventureText {
            title: self.title.clone(),
            introduction: self.introduction_text.clone(),
            complete: self.complete_text.clone(),
            episodes_text: self.parent_episodes.iter().map(ParentEpisode::to_text).collect(),
            colony_text: self.colony_episodes.iter().map(ColonyEpisode::to_text).collect(),
        };
    }

    pub fn write_to(&self, folder: impl AsRef<Path>) -> io::Result<()> {
        let folder = folder.as_ref();
        let name = folder_name(folder)?;
        validate_adventure_name(&name)?;

        if let Err(err) = fs::remove_dir_all(&folder)
            && err.kind() != ErrorKind::NotFound
        {
            return Err(err);
        }
        fs::create_dir_all(folder)?;

        let pak_data = self.to_pak();

        let mut pak_writer = File::create(folder.join(format!("{name}.pak"))).map(BufWriter::new)?;
        pak_data.write_to(&mut pak_writer)?;
        pak_writer.flush()?;

        let mut set_writer = File::create(folder.join(format!("{name}.set"))).map(BufWriter::new)?;
        pak_data.settings_data.write_to(&mut set_writer)?;
        set_writer.flush()?;

        for (i, map_data) in pak_data.map_data.iter().enumerate() {
            let suffix = if i == 0 { "P".to_string() } else { format!("C{i}") };
            let mut map_writer = File::create(folder.join(format!("{name}{suffix}.map"))).map(BufWriter::new)?;
            map_data.write_to(&mut map_writer)?;
            map_writer.flush()?;
        }

        let mut text_writer = File::create(folder.join(format!("{name}.txt"))).map(BufWriter::new)?;
        self.to_text().write_to(&mut text_writer)?;
        text_writer.flush()?;

        return Ok(());
    }
}

fn folder_name(folder: &Path) -> io::Result<String> {
    return folder
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .ok_or_else(|| Error::new(io::ErrorKind::InvalidInput, format!("folder path has no folder name: {folder:?}")));
}

fn validate_adventure_name(name: &str) -> io::Result<()> {
    if name.chars().count() > MAX_ADVENTURE_NAME_LENGTH {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!(
                "adventure name {name:?} is {} characters long, longer than the {MAX_ADVENTURE_NAME_LENGTH} limit",
                name.chars().count()
            ),
        ));
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use crate::adventure::adventure::Adventure;
    use crate::adventure::adventure::validate_adventure_name;
    use crate::prelude::AdventureText;
    use crate::prelude::AdventureType;
    use crate::prelude::Civilization;
    use crate::prelude::DataConstant;
    use crate::prelude::PakData;
    use crate::prelude::ResourceType;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn validate_adventure_name_accepts_the_longest_known_real_name() -> Result<()> {
        validate_adventure_name("The Sinking of Atlantis")?;

        return Ok(());
    }

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            assert_eq!(adventure.title, "The Youngest Twins");
            assert_eq!(adventure.complete_text, "n/a");
            assert_eq!(adventure.civilization, Civilization::Atlantean);
            assert_eq!(adventure.adventure_type, AdventureType::PoseidonCustom);
            assert_eq!(adventure.bitmap, 14);
            assert_eq!(adventure.world_bitmap, 11);
            assert_eq!(adventure.start_year, -2600);
            assert_eq!(adventure.initial_funds, 10_000);
            assert_eq!(adventure.parent_episodes.len(), 2);
            assert_eq!(adventure.colony_episodes.len(), 1);
            assert_price_eq(&adventure.prices, ResourceType::Urchin, 36);
            assert_price_eq(&adventure.prices, ResourceType::Fish, 36);
            assert_price_eq(&adventure.prices, ResourceType::Meat, 45);
            assert_price_eq(&adventure.prices, ResourceType::Cheese, 45);
            assert_price_eq(&adventure.prices, ResourceType::Carrot, 40);
            assert_price_eq(&adventure.prices, ResourceType::Onion, 40);
            assert_price_eq(&adventure.prices, ResourceType::Wheat, 30);
            assert_price_eq(&adventure.prices, ResourceType::Orange, 40);
            assert_price_eq(&adventure.prices, ResourceType::Wood, 75);
            assert_price_eq(&adventure.prices, ResourceType::Bronze, 110);
            assert_price_eq(&adventure.prices, ResourceType::Marble, 84);
            assert_price_eq(&adventure.prices, ResourceType::Grape, 42);
            assert_price_eq(&adventure.prices, ResourceType::Olive, 38);
            assert_price_eq(&adventure.prices, ResourceType::Fleece, 60);
            assert_price_eq(&adventure.prices, ResourceType::BlackMarble, 88);
            assert_price_eq(&adventure.prices, ResourceType::Orichalc, 140);
            assert_price_eq(&adventure.prices, ResourceType::Armor, 200);
            assert_price_eq(&adventure.prices, ResourceType::Sculpture, 640);
            assert_price_eq(&adventure.prices, ResourceType::OliveOil, 145);
            assert_price_eq(&adventure.prices, ResourceType::Wine, 160);
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            assert_eq!(adventure.title, "The Odyssey");
            assert_eq!(adventure.civilization, Civilization::Greek);
            assert_eq!(adventure.adventure_type, AdventureType::ZeusCustom);
            assert_eq!(adventure.bitmap, 7);
            assert_eq!(adventure.world_bitmap, 9);
            assert_eq!(adventure.start_year, -1200);
            assert_eq!(adventure.initial_funds, 8000);
            assert_eq!(adventure.parent_episodes.len(), 5);
            assert_eq!(adventure.colony_episodes.len(), 2);
            assert_price_eq(&adventure.prices, ResourceType::Urchin, 36);
            assert_price_eq(&adventure.prices, ResourceType::Fish, 36);
            assert_price_eq(&adventure.prices, ResourceType::Meat, 45);
            assert_price_eq(&adventure.prices, ResourceType::Cheese, 45);
            assert_price_eq(&adventure.prices, ResourceType::Carrot, 40);
            assert_price_eq(&adventure.prices, ResourceType::Onion, 40);
            assert_price_eq(&adventure.prices, ResourceType::Wheat, 30);
            assert_price_eq(&adventure.prices, ResourceType::Orange, 0);
            assert_price_eq(&adventure.prices, ResourceType::Wood, 75);
            assert_price_eq(&adventure.prices, ResourceType::Bronze, 110);
            assert_price_eq(&adventure.prices, ResourceType::Marble, 84);
            assert_price_eq(&adventure.prices, ResourceType::Grape, 42);
            assert_price_eq(&adventure.prices, ResourceType::Olive, 38);
            assert_price_eq(&adventure.prices, ResourceType::Fleece, 60);
            assert_price_eq(&adventure.prices, ResourceType::BlackMarble, 0);
            assert_price_eq(&adventure.prices, ResourceType::Orichalc, 0);
            assert_price_eq(&adventure.prices, ResourceType::Armor, 200);
            assert_price_eq(&adventure.prices, ResourceType::Sculpture, 640);
            assert_price_eq(&adventure.prices, ResourceType::OliveOil, 145);
            assert_price_eq(&adventure.prices, ResourceType::Wine, 160);
        }

        return Ok(());
    }

    #[test]
    fn parse_two_worlds_collide() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let path = format!("{game_root}/Adventures/^Two Worlds Collide.pak");
            let mut reader = BufReader::new(File::open(path)?);
            let pak_data = PakData::read_from(&mut reader)?;
            let adventure = Adventure::from_pak(&pak_data, &AdventureText::default());

            assert_eq!(adventure.civilization, Civilization::Greek);
            assert_eq!(adventure.adventure_type, AdventureType::PoseidonCampaign);
            assert_eq!(adventure.bitmap, 4);
            assert_eq!(adventure.world_bitmap, 14);
            assert_eq!(adventure.start_year, -1900);
            assert_eq!(adventure.initial_funds, 4000);
            assert_eq!(adventure.parent_episodes.len(), 6);
            assert_eq!(adventure.colony_episodes.len(), 2);
            assert_price_eq(&adventure.prices, ResourceType::Urchin, 36);
            assert_price_eq(&adventure.prices, ResourceType::Fish, 36);
            assert_price_eq(&adventure.prices, ResourceType::Meat, 45);
            assert_price_eq(&adventure.prices, ResourceType::Cheese, 45);
            assert_price_eq(&adventure.prices, ResourceType::Carrot, 40);
            assert_price_eq(&adventure.prices, ResourceType::Onion, 40);
            assert_price_eq(&adventure.prices, ResourceType::Wheat, 30);
            assert_price_eq(&adventure.prices, ResourceType::Orange, 40);
            assert_price_eq(&adventure.prices, ResourceType::Wood, 75);
            assert_price_eq(&adventure.prices, ResourceType::Bronze, 110);
            assert_price_eq(&adventure.prices, ResourceType::Marble, 84);
            assert_price_eq(&adventure.prices, ResourceType::Grape, 42);
            assert_price_eq(&adventure.prices, ResourceType::Olive, 38);
            assert_price_eq(&adventure.prices, ResourceType::Fleece, 60);
            assert_price_eq(&adventure.prices, ResourceType::BlackMarble, 88);
            assert_price_eq(&adventure.prices, ResourceType::Orichalc, 140);
            assert_price_eq(&adventure.prices, ResourceType::Armor, 200);
            assert_price_eq(&adventure.prices, ResourceType::Sculpture, 640);
            assert_price_eq(&adventure.prices, ResourceType::OliveOil, 145);
            assert_price_eq(&adventure.prices, ResourceType::Wine, 160);
        }

        return Ok(());
    }

    #[test]
    fn all_real_adventures_round_trip() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventures_dir = PathBuf::from(format!("{game_root}/Adventures"));

            let mut pak_paths = vec![];
            collect_paks(&adventures_dir, &mut pak_paths)?;

            for path in pak_paths {
                let mut reader = BufReader::new(File::open(&path)?);
                let pak_data = PakData::read_from(&mut reader)?;

                let text_path = path.with_extension("txt");
                let adventure_text = if text_path.is_file() {
                    let mut text_reader = BufReader::new(File::open(&text_path)?);
                    AdventureText::read_from(&mut text_reader)?
                } else {
                    AdventureText::default()
                };

                let adventure = Adventure::from_pak(&pak_data, &adventure_text);

                let reconstructed_pak_data = adventure.to_pak();
                let reconstructed_text = adventure.to_text();
                let round_tripped = Adventure::from_pak(&reconstructed_pak_data, &reconstructed_text);

                assert_eq!(round_tripped, adventure, "{path:?} did not round-trip");
            }
        }

        return Ok(());
    }

    fn collect_paks(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();

            if path.is_dir() {
                collect_paks(&path, out)?;
            } else if path.extension().is_some_and(|e| e == "pak") {
                out.push(path);
            }
        }

        return Ok(());
    }

    fn assert_price_eq(prices: &Vec<u32>, resource_type: ResourceType, expected_price: u32) {
        assert_eq!(prices[resource_type.value() as usize], expected_price);
    }
}
