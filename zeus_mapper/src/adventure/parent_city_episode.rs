use crate::adventure::NextEpisode;
use crate::adventure::building_type::BuildingType;
use crate::adventure::episode_goals::EpisodeGoal;
use crate::adventure::event::Event;
use crate::adventure::mythology::Mythology;
use crate::adventure::pyramid::PyramidSetting;
use crate::adventure::resource::ResourceType;
use crate::adventure::resource::TradedGoods;
use crate::l10n::adventure_text::AdventureText;
use crate::l10n::adventure_text::EpisodeText;
use crate::prelude::DataConstant;
use crate::prelude::PakData;
use my_macros::LogDifferences;

#[derive(LogDifferences, PartialEq, Debug)]
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
    pub world_settings: ParentCityWorldSettings,
    pub pyramid_settings: Vec<PyramidSetting>,
}

/// The player's own city's world-facing stats, in the same shape `WorldLocation` carries for
/// every other city - it's assembled from `SettingsData` fields for trade/favour rather than
/// read directly off one `WorldLocationData` record.
#[derive(LogDifferences, PartialEq, Debug)]
pub struct ParentCityWorldSettings {
    pub traded_resources: Vec<TradedGoods>,
    pub favour: u16,
}

impl ParentEpisode {
    pub(crate) fn vec_from_data(pak_data: &PakData, adventure_text: &AdventureText) -> Vec<Self> {
        let mut result = vec![];

        let episode_count = usize::min(
            pak_data.settings_data.parent_episodes as usize,
            pak_data.settings_data.basic_episode_data.len(),
        );

        let map_building_flags = pak_data.map_data[0].scenario_data.building_flags.as_slice();

        let new_file_ver = pak_data.settings_data.version_2 == 26;

        for i in 0..episode_count {
            let settings_building_flags = pak_data.settings_data.real_episode_data[i].building_flags.as_slice();
            let building_flags = if settings_building_flags.iter().any(|&flag| flag != 0) {
                settings_building_flags
            } else {
                map_building_flags
            };

            let event_count = pak_data.settings_data.parent_event_counts.get(i).copied().unwrap_or(0) as usize;
            let goal_row = pak_data
                .settings_data
                .parent_episode_goals
                .get(i)
                .map(|row| row.as_slice())
                .unwrap_or(&[]);
            // `parent_episode_goal_counts[i]` gates real vs. leftover-template slots for both file
            // generations - old-format rows have a genuine goal immediately followed by a leftover
            // editor-template slot duplicating it (same or a differing kind/count), and the count is
            // what distinguishes them (confirmed against `#Zeus and Europa.pak`, `&Perseus and
            // Medusa.pak`, `@Athens through the Ages.pak`, `Open Play Economic 2`, and
            // `Open Play Military 2`).
            let goal_count = pak_data.settings_data.parent_episode_goal_counts.get(i).copied().unwrap_or(0) as usize;
            let real_goal_slots = goal_row.get(..goal_count.min(goal_row.len())).unwrap_or(&[]);

            let mythology = Mythology::from_data(&pak_data.settings_data.mythology[i]);

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
                building_allowed: BuildingType::vec_from_data(building_flags),
                episode_goals: EpisodeGoal::vec_from_episode_goal_data(
                    real_goal_slots,
                    new_file_ver,
                    &pak_data.settings_data.events[i],
                    &mythology,
                ),
                events: Event::vec_from_data(&pak_data.settings_data.events[i], event_count, new_file_ver),
                city_resources: ResourceType::vec_from_data(&pak_data.settings_data.real_episode_data[i].city_resources, new_file_ver),
                world_settings: ParentCityWorldSettings {
                    traded_resources: TradedGoods::vec_from_data(
                        &pak_data.settings_data.real_episode_data[i].city_resources_sold,
                        &pak_data.settings_data.real_episode_data[i].city_resources_bought,
                        pak_data.settings_data.real_episode_data[i].city_resources_quantity.as_slice(),
                        new_file_ver,
                    ),
                    favour: pak_data.settings_data.parent_city_favor.get(i).copied().unwrap_or(0) as u16,
                },
                pyramid_settings: PyramidSetting::vec_from_data(&pak_data.settings_data.mythology[i].pyramids),
                mythology,
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

#[cfg(test)]
mod tests {
    use crate::adventure::NextEpisode;
    use crate::adventure::adventure::Adventure;
    use crate::adventure::resource::TradedGoods;
    use crate::prelude::ResourceType;
    use std::io::Result;

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.title, "The Azium Empire");
            assert_eq!(episode_1.panhellenic_games, false);
            assert_eq!(
                episode_1.city_resources,
                vec![
                    ResourceType::Meat,
                    ResourceType::Carrot,
                    ResourceType::Wheat,
                    ResourceType::Wood,
                    ResourceType::Marble,
                    ResourceType::Grape,
                    ResourceType::Olive,
                    ResourceType::Fleece,
                    ResourceType::BlackMarble
                ]
            );
            assert_eq!(episode_1.next_episode, NextEpisode::Colony);
            assert_eq!(
                episode_1.world_settings.traded_resources,
                vec![
                    TradedGoods::Selling(ResourceType::OliveOil, 24),
                    TradedGoods::Selling(ResourceType::Wine, 12),
                    TradedGoods::Buying(ResourceType::Bronze, 24),
                    TradedGoods::Buying(ResourceType::Armor, 12)
                ]
            );
            assert_eq!(episode_1.world_settings.favour, 65);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.title, "The Long Road to Victory");
            assert_eq!(episode_2.panhellenic_games, false);
            assert_eq!(
                episode_2.city_resources,
                vec![
                    ResourceType::Meat,
                    ResourceType::Carrot,
                    ResourceType::Wheat,
                    ResourceType::Wood,
                    ResourceType::Marble,
                    ResourceType::Grape,
                    ResourceType::Olive,
                    ResourceType::Fleece,
                    ResourceType::BlackMarble
                ]
            );
            assert_eq!(episode_2.next_episode, NextEpisode::Victory);
            assert_eq!(episode_2.world_settings.traded_resources, vec![]);
            // Still uncertain what the below need to be:
            // assert_eq!(episode_2.world_settings.favour, 50);
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.title, "The Great War Ends");
            assert_eq!(episode_1.panhellenic_games, true);
            assert_eq!(
                episode_1.city_resources,
                vec![ResourceType::Fish, ResourceType::Cheese, ResourceType::Grape, ResourceType::Olive]
            );
            assert_eq!(episode_1.next_episode, NextEpisode::Parent);
            assert_eq!(
                episode_1.world_settings.traded_resources,
                vec![
                    TradedGoods::Selling(ResourceType::Fish, 36),
                    TradedGoods::Selling(ResourceType::Olive, 36),
                    TradedGoods::Buying(ResourceType::Fleece, 24),
                    TradedGoods::Buying(ResourceType::Armor, 36)
                ]
            );
            assert_eq!(episode_1.world_settings.favour, 75);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.title, "The Absent Hero");
            assert_eq!(episode_2.panhellenic_games, true);
            assert_eq!(
                episode_2.city_resources,
                vec![ResourceType::Fish, ResourceType::Cheese, ResourceType::Grape, ResourceType::Olive]
            );
            assert_eq!(episode_2.next_episode, NextEpisode::Colony);
            assert_eq!(
                episode_2.world_settings.traded_resources,
                vec![
                    TradedGoods::Selling(ResourceType::Fish, 36),
                    TradedGoods::Selling(ResourceType::Olive, 36),
                    TradedGoods::Buying(ResourceType::Fleece, 24),
                    TradedGoods::Buying(ResourceType::Armor, 24)
                ]
            );
            assert_eq!(episode_2.world_settings.favour, 70);

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(episode_3.title, "Suitors, Suitors Everywhere");
            assert_eq!(episode_3.panhellenic_games, true);
            assert_eq!(
                episode_3.city_resources,
                vec![ResourceType::Fish, ResourceType::Cheese, ResourceType::Grape, ResourceType::Olive]
            );
            assert_eq!(episode_3.next_episode, NextEpisode::Colony);
            assert_eq!(
                episode_3.world_settings.traded_resources,
                vec![
                    TradedGoods::Selling(ResourceType::Fish, 36),
                    TradedGoods::Selling(ResourceType::Olive, 36),
                    TradedGoods::Buying(ResourceType::Fleece, 24),
                    TradedGoods::Buying(ResourceType::Armor, 24)
                ]
            );
            assert_eq!(episode_3.world_settings.favour, 70);

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(episode_4.title, "The Goddess' Advice");
            assert_eq!(episode_4.panhellenic_games, true);
            assert_eq!(
                episode_4.city_resources,
                vec![ResourceType::Fish, ResourceType::Cheese, ResourceType::Grape, ResourceType::Olive]
            );
            assert_eq!(episode_4.next_episode, NextEpisode::Parent);
            assert_eq!(episode_4.world_settings.traded_resources, vec![]);
            assert_eq!(episode_4.world_settings.favour, 50);

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(episode_5.title, "The Hero Returns");
            assert_eq!(episode_5.panhellenic_games, true);
            assert_eq!(
                episode_5.city_resources,
                vec![ResourceType::Fish, ResourceType::Cheese, ResourceType::Grape, ResourceType::Olive]
            );
            assert_eq!(episode_5.next_episode, NextEpisode::Victory);
            assert_eq!(episode_5.world_settings.traded_resources, vec![]);
            assert_eq!(episode_5.world_settings.favour, 50);
        }

        return Ok(());
    }
}
