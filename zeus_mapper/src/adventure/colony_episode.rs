use crate::adventure::MonetaryAmount;
use crate::adventure::building_type::BuildingType;
use crate::adventure::city_map::CityMap;
use crate::adventure::episode_goals::EpisodeGoal;
use crate::adventure::episode_goals::OwnCitySettings;
use crate::adventure::episode_goals::PyramidSetting;
use crate::adventure::episode_goals::ResourceType;
use crate::adventure::event::Event;
use crate::adventure::mythology::Mythology;
use crate::l10n::adventure_text::AdventureText;
use crate::prelude::PakData;
use my_macros::LogDifferences;

#[derive(LogDifferences)]
pub struct ColonyEpisode {
    pub title: String,
    pub introduction_text: String,
    pub complete_text: String,
    pub selection_text: String,
    pub panhellenic_games: bool,
    pub mythology: Mythology,
    pub building_allowed: Vec<BuildingType>,
    pub episode_goals: Vec<EpisodeGoal>,
    pub events: Vec<Event>,
    pub city_resources: Vec<ResourceType>,
    pub world_settings: OwnCitySettings,
    pub pyramid_settings: Vec<PyramidSetting>,
    pub initial_funds: MonetaryAmount,
    pub city_map: CityMap,
}

impl ColonyEpisode {
    pub(crate) fn vec_from_data(pak_data: &PakData, adventure_text: &AdventureText) -> Vec<Self> {
        let mut result = Vec::with_capacity(10);

        for (i, _colony) in pak_data.settings_data.real_episode_data[10..]
            .iter()
            .take(pak_data.settings_data.colony_episodes_available as usize)
            .enumerate()
        {
            result.push(ColonyEpisode {
                title: adventure_text
                    .colony_text
                    .get(i)
                    .map(|colony_text| colony_text.title.to_owned())
                    .unwrap_or_default(),
                introduction_text: adventure_text
                    .colony_text
                    .get(i)
                    .map(|colony_text| colony_text.introduction.to_owned())
                    .unwrap_or_default(),
                complete_text: adventure_text
                    .colony_text
                    .get(i)
                    .map(|colony_text| colony_text.complete.to_owned())
                    .unwrap_or_default(),
                selection_text: adventure_text
                    .colony_text
                    .get(i)
                    .map(|colony_text| colony_text.selection.to_owned())
                    .unwrap_or_default(),
                panhellenic_games: false,
                mythology: Mythology {
                    proponent_gods: vec![],
                    opponent_gods: vec![],
                    max_sanctuaries: 0,
                },
                building_allowed: vec![],
                episode_goals: vec![],
                events: vec![],
                city_resources: vec![],
                world_settings: OwnCitySettings {
                    traded_resources: vec![],
                    military_strength: 0,
                    economy_strength: 0,
                    favour: 0,
                },
                pyramid_settings: vec![],
                initial_funds: 0,
                city_map: CityMap::from_map_data(&pak_data.map_data[1 + i]),
            });
        }

        result.shrink_to_fit();

        return result;
    }
}
