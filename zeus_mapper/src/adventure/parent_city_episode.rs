use crate::adventure::NextEpisode;
use crate::adventure::building_type::BuildingType;
use crate::adventure::episode_goals::EpisodeGoal;
use crate::adventure::episode_goals::OwnCitySettings;
use crate::adventure::episode_goals::PyramidSetting;
use crate::adventure::episode_goals::ResourceType;
use crate::adventure::event::Event;
use crate::adventure::mythology::Mythology;
use crate::l10n::adventure_text::AdventureText;
use crate::l10n::adventure_text::EpisodeText;
use crate::prelude::DataConstant;
use crate::prelude::PakData;
use my_macros::LogDifferences;

#[derive(LogDifferences)]
pub struct ParentEpisode {
    pub title: String,
    pub introduction_text: String,
    pub complete_text: String,
    pub next_episode: NextEpisode,
    pub panhellenic_games: bool,
    pub mythology: Mythology,
    pub building_allowed: Vec<BuildingType>,
    pub episode_goals: Vec<EpisodeGoal>,
    pub events: Vec<Event>,
    pub city_resources: Vec<ResourceType>,
    pub world_settings: OwnCitySettings,
    pub pyramid_settings: Vec<PyramidSetting>,
}

impl ParentEpisode {
    pub(crate) fn vec_from_data(pak_data: &PakData, adventure_text: &AdventureText) -> Vec<Self> {
        let mut result = vec![];

        let episode_count = usize::min(
            pak_data.settings_data.parent_episodes as usize,
            pak_data.settings_data.basic_episode_data.len(),
        );

        for i in 0..episode_count {
            result.push(ParentEpisode {
                title: adventure_text
                    .episodes_text
                    .get(i)
                    .map(|episode_text| episode_text.title.to_owned())
                    .unwrap_or_default(),
                introduction_text: adventure_text
                    .episodes_text
                    .get(i)
                    .map(|episode_text| episode_text.introduction.to_owned())
                    .unwrap_or_default(),
                complete_text: adventure_text
                    .episodes_text
                    .get(i)
                    .map(|episode_text| episode_text.complete.to_owned())
                    .unwrap_or_default(),
                next_episode: NextEpisode::try_resolve(&pak_data.settings_data.basic_episode_data[i].next_episode)
                    .unwrap_or(NextEpisode::Victory),
                panhellenic_games: pak_data.settings_data.real_episode_data[i].panhellenic_games == 0,
                mythology: Mythology::from_data(&pak_data.settings_data.mythology[i]),
                building_allowed: BuildingType::vec_from_data(&pak_data.settings_data.real_episode_data[i].building_flags),
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
            });
        }

        return result;
    }

    pub(crate) fn to_text(&self) -> EpisodeText {
        return EpisodeText {
            title: self.title.clone(),
            introduction: self.introduction_text.clone(),
            complete: self.complete_text.clone(),
        };
    }
}
