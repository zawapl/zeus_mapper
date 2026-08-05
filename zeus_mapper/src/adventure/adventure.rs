use crate::adventure::Civilization;
use crate::adventure::MonetaryAmount;
use crate::adventure::city_map::CityMap;
use crate::adventure::colony_episode::ColonyEpisode;
use crate::adventure::parent_city_episode::ParentEpisode;
use crate::l10n::adventure_text::AdventureText;
use crate::prelude::DataConstant;
use crate::prelude::PakData;
use my_macros::LogDifferences;

#[derive(LogDifferences)]
pub struct Adventure {
    pub title: String,
    pub introduction_text: String,
    pub complete_text: String,
    pub civilization: Civilization,
    pub bitmap: u16,
    pub parent_city: CityMap,
    pub parent_episodes: Vec<ParentEpisode>,
    pub colony_episodes: Vec<ColonyEpisode>,
    pub available_colonies: u8,
    pub initial_funds: MonetaryAmount,
    pub start_year: i16,
}

impl Adventure {
    pub fn from_pak(pak_data: &PakData, adventure_text: &AdventureText) -> Adventure {
        return Adventure {
            title: adventure_text.title.to_owned(),
            introduction_text: adventure_text.introduction.to_owned(),
            complete_text: adventure_text.complete.to_owned(),
            civilization: Civilization::try_resolve(&pak_data.settings_data.real_episode_data[0].civilization)
                .unwrap_or(Civilization::Greek),
            bitmap: 0,
            parent_city: CityMap::from_map_data(&pak_data.map_data[0]),
            parent_episodes: ParentEpisode::vec_from_data(pak_data, adventure_text),
            colony_episodes: ColonyEpisode::vec_from_data(pak_data, adventure_text),
            available_colonies: pak_data.settings_data.colony_episodes_available as u8,
            initial_funds: pak_data.settings_data.real_episode_data[0].starting_cash,
            start_year: pak_data.settings_data.real_episode_data[0].start_date,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::adventure::adventure::Adventure;
    use crate::l10n::adventure_text::AdventureText;
    use crate::prelude::PakData;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;

    #[test]
    fn test_zeus_and_europa() -> Result<()> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

        let mut pak_reader = File::open(format!("{}/Adventures/#Zeus and Europa.pak", game_root)).map(BufReader::new)?;
        let pak_data = PakData::read_from(&mut pak_reader)?;

        let mut text_reader = File::open(format!("{}/Adventures/Text_Template.txt", game_root)).map(BufReader::new)?;
        let text_template = AdventureText::read_from(&mut text_reader)?;

        let adventure = Adventure::from_pak(&pak_data, &text_template);

        assert_eq!(adventure.title, "INSERT TEXT HERE");

        return Ok(());
    }
}
