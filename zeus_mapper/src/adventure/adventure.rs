use crate::adventure::Civilization;
use crate::adventure::MonetaryAmount;
use crate::adventure::building_type::BuildingType;
use crate::adventure::city_map::CityMap;
use crate::adventure::colony_episode::ColonyEpisode;
use crate::adventure::parent_city_episode::ParentEpisode;
use crate::l10n::adventure_text::AdventureText;
use crate::prelude::DataConstant;
use crate::prelude::PakData;
use crate::prelude::SettingsData;
use my_macros::LogDifferences;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

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
        let mut settings_data = SettingsData::default();

        settings_data.version_1 = 8871;

        settings_data.real_episode_data[0].civilization = self.civilization.value();
        settings_data.real_episode_data[0].starting_cash = self.initial_funds;
        settings_data.real_episode_data[0].start_date = self.start_year;
        settings_data.colony_episodes_available = self.colony_episodes.len() as u32;

        settings_data.parent_episodes = self.parent_episodes.len() as u32;

        let episode_count = usize::min(self.parent_episodes.len(), settings_data.basic_episode_data.len());

        for (i, episode) in self.parent_episodes.iter().take(episode_count).enumerate() {
            settings_data.basic_episode_data[i].next_episode = episode.next_episode.value();
            settings_data.real_episode_data[i].panhellenic_games = if episode.panhellenic_games { 0 } else { 1 };
            settings_data.real_episode_data[i].building_flags = BuildingType::vec_to_data(&episode.building_allowed);
            settings_data.mythology[i] = episode.mythology.to_data();
        }

        let mut map_data = vec![self.parent_city.to_map_data()];
        for colony_episode in &self.colony_episodes {
            map_data.push(colony_episode.city_map.to_map_data());
        }

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

        fs::create_dir_all(folder)?;

        let pak_data = self.to_pak();

        let mut pak_writer = File::create(folder.join(format!("{name}.pak")))?;
        pak_data.write_to(&mut pak_writer)?;

        let mut set_writer = File::create(folder.join(format!("{name}.set")))?;
        pak_data.settings_data.write_to(&mut set_writer)?;

        for (i, map_data) in pak_data.map_data.iter().enumerate() {
            let suffix = if i == 0 { "P".to_string() } else { format!("C{i}") };
            let mut map_writer = File::create(folder.join(format!("{name}{suffix}.map")))?;
            map_data.write_to(&mut map_writer)?;
        }

        let mut text_writer = File::create(folder.join(format!("{name}.txt")))?;
        self.to_text().write_to(&mut text_writer)?;

        return Ok(());
    }
}

fn folder_name(folder: &Path) -> io::Result<String> {
    return folder
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("folder path has no folder name: {folder:?}")));
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
