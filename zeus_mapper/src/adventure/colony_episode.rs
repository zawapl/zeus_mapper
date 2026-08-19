use crate::adventure::building_type::BuildingType;
use crate::adventure::city_map::CityMap;
use crate::adventure::episode_goals::EpisodeGoal;
use crate::adventure::event::Event;
use crate::adventure::mythology::Mythology;
use crate::adventure::pyramid::PyramidSetting;
use crate::adventure::resource::ResourceType;
use crate::l10n::adventure_text::AdventureText;
use crate::l10n::adventure_text::ColonyText;
use crate::prelude::PakData;
use my_macros::LogDifferences;

pub(crate) const COLONY_SETTINGS_OFFSET: usize = 10;

#[derive(LogDifferences, PartialEq, Debug)]
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
    pub pyramid_settings: Vec<PyramidSetting>,
    pub city_map: CityMap,
}

impl ColonyEpisode {
    pub(crate) fn vec_from_data(pak_data: &PakData, adventure_text: &AdventureText) -> Vec<Self> {
        let mut result = Vec::with_capacity(10);
        let new_file_ver = pak_data.settings_data.version_2 == 26;

        let colony_count = usize::min(
            pak_data.settings_data.colony_episodes_available as usize,
            pak_data.map_data.len().saturating_sub(1),
        );

        for (i, _colony) in pak_data.settings_data.real_episode_data[COLONY_SETTINGS_OFFSET..]
            .iter()
            .take(colony_count)
            .enumerate()
        {
            let settings_index = COLONY_SETTINGS_OFFSET + i;
            let real_episode_data = &pak_data.settings_data.real_episode_data[settings_index];
            let settings_building_flags = real_episode_data.building_flags.as_slice();
            let map_building_flags = pak_data.map_data[1 + i].scenario_data.building_flags.as_slice();
            let building_flags = if settings_building_flags.iter().any(|&flag| flag != 0) {
                settings_building_flags
            } else {
                map_building_flags
            };

            let events = &pak_data.settings_data.events[settings_index];
            let raw_event_count = pak_data.settings_data.colony_event_counts.get(i).copied().unwrap_or(0) as usize;

            // todo can this be replaced with a version check?
            let event_count = if raw_event_count > 0 || events.first().map(|event| event.event_type) == Some(0) {
                raw_event_count
            } else {
                events.iter().take_while(|event| event.event_type != 0).count().saturating_sub(1)
            };

            let mythology = Mythology::from_data(&pak_data.settings_data.mythology[settings_index]);

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
                panhellenic_games: real_episode_data.panhellenic_games == 0,
                building_allowed: BuildingType::vec_from_data(building_flags),
                episode_goals: pak_data
                    .settings_data
                    .colony_episode_goals
                    .get(i)
                    .map(|row| {
                        let goal_count = pak_data.settings_data.colony_episode_goal_counts.get(i).copied().unwrap_or(0) as usize;
                        let real_goal_slots = row.get(..goal_count.min(row.len())).unwrap_or(&[]);
                        EpisodeGoal::vec_from_episode_goal_data(real_goal_slots, new_file_ver, events)
                    })
                    .unwrap_or_default(),
                events: Event::vec_from_data(events, event_count, new_file_ver),
                city_resources: ResourceType::vec_from_data(&real_episode_data.city_resources, new_file_ver),
                pyramid_settings: PyramidSetting::vec_from_data(&pak_data.settings_data.mythology[settings_index].pyramids),
                city_map: CityMap::from_map_data(&pak_data.map_data[1 + i]),
                mythology,
            });
        }

        result.shrink_to_fit();

        return result;
    }

    pub(crate) fn to_text(&self) -> ColonyText {
        return ColonyText {
            title: self.title.clone(),
            introduction: self.introduction_text.clone(),
            complete: self.complete_text.clone(),
            selection: self.selection_text.clone(),
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::adventure::adventure::Adventure;
    use crate::prelude::ResourceType;
    use std::io::Result;

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(colony_episode.title, "The Wild Coast");
            assert_eq!(colony_episode.panhellenic_games, false);
            assert_eq!(
                colony_episode.city_resources,
                vec![
                    ResourceType::Fish,
                    ResourceType::Meat,
                    ResourceType::Bronze,
                    ResourceType::Fleece,
                    ResourceType::Orichalc
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(colony_episode_1.title, "The Shepherd's Tale");
            assert_eq!(colony_episode_1.panhellenic_games, true);
            assert_eq!(
                colony_episode_1.city_resources,
                vec![ResourceType::Fish, ResourceType::Cheese, ResourceType::Fleece]
            );

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(colony_episode_2.title, "The Hero Errs");
            assert_eq!(colony_episode_2.panhellenic_games, true);
            assert_eq!(colony_episode_2.city_resources, vec![]);
        }

        return Ok(());
    }
}
