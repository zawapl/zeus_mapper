use crate::adventure::God;
use crate::adventure::Monster;
use crate::prelude::BoxedArray;
use crate::prelude::DataConstant;
use crate::prelude::MythologyData;
use my_macros::LogDifferences;

#[derive(LogDifferences, PartialEq, Debug)]
pub struct Mythology {
    pub proponent_gods: Vec<(God, SanctuaryAllowed)>,
    pub opponent_gods: Vec<God>,
    pub monster: Monster,
    pub max_sanctuaries: u8,
}

impl Mythology {
    pub(crate) fn from_data(mythology_data: &MythologyData) -> Mythology {
        return Mythology {
            proponent_gods: mythology_data
                .proponent_gods
                .iter()
                .zip(mythology_data.sanctuaries_allowed.iter())
                .filter_map(|(id, &allowed)| God::try_resolve(id).map(|god| (god, allowed != 0)))
                .collect(),
            opponent_gods: mythology_data.opponent_gods.iter().filter_map(|i| God::try_resolve(i)).collect(),
            monster: Monster::try_resolve(&mythology_data.monster).unwrap_or(Monster::Hydra),
            max_sanctuaries: mythology_data.max_sanctuaries as u8,
        };
    }

    pub(crate) fn to_data(&self) -> MythologyData {
        let mut proponent_gods = [u32::MAX; 12];
        let mut sanctuaries_allowed = [0u8; 12];
        for (i, (god, sanctuary_allowed)) in self.proponent_gods.iter().take(12).enumerate() {
            proponent_gods[i] = god.value();
            sanctuaries_allowed[i] = *sanctuary_allowed as u8;
        }

        let mut opponent_gods = [u32::MAX; 12];
        for (i, god) in self.opponent_gods.iter().take(12).enumerate() {
            opponent_gods[i] = god.value();
        }

        return MythologyData {
            opponent_gods,
            proponent_gods,
            monster: self.monster.value(),
            constant_1_0xff: BoxedArray::from_vec(vec![0xFF; 96]),
            constant_2_0x00: [0; 12],
            sanctuaries_allowed,
            max_sanctuaries: self.max_sanctuaries as u32,
            max_pyramids: 0,
            pyramids: vec![],
        };
    }
}

type SanctuaryAllowed = bool;

#[cfg(test)]
mod tests {
    use crate::adventure::God;
    use crate::adventure::Monster;
    use crate::adventure::adventure::Adventure;
    use std::io::Result;

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.mythology.opponent_gods, vec![God::Ares, God::Hera]);
            assert_eq!(
                episode_1.mythology.proponent_gods,
                vec![
                    (God::Athena, true),
                    (God::Artemis, true),
                    (God::Hermes, false),
                    (God::Aphrodite, true),
                    (God::Dionysus, false),
                    (God::Apollo, true)
                ]
            );
            assert_eq!(episode_1.mythology.monster, Monster::Minotaur);
            assert_eq!(episode_1.mythology.max_sanctuaries, 4);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.mythology.opponent_gods, vec![God::Ares, God::Hera]);
            assert_eq!(
                episode_2.mythology.proponent_gods,
                vec![
                    (God::Athena, true),
                    (God::Artemis, true),
                    (God::Hermes, true),
                    (God::Aphrodite, true),
                    (God::Dionysus, true),
                    (God::Apollo, true)
                ]
            );
            assert_eq!(episode_2.mythology.monster, Monster::Minotaur);
            assert_eq!(episode_2.mythology.max_sanctuaries, 4);

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(colony_episode.mythology.opponent_gods, vec![God::Ares, God::Hera]);
            assert_eq!(
                colony_episode.mythology.proponent_gods,
                vec![
                    (God::Athena, false),
                    (God::Artemis, false),
                    (God::Hermes, false),
                    (God::Aphrodite, false),
                    (God::Dionysus, false),
                    (God::Apollo, false)
                ]
            );
            assert_eq!(colony_episode.mythology.monster, Monster::Minotaur);
            assert_eq!(colony_episode.mythology.max_sanctuaries, 0);
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.mythology.opponent_gods,
                vec![God::Poseidon, God::Apollo, God::Artemis, God::Ares, God::Aphrodite]
            );
            assert_eq!(
                episode_1.mythology.proponent_gods,
                vec![
                    (God::Athena, false),
                    (God::Hermes, false),
                    (God::Zeus, false),
                    (God::Hephaestus, false)
                ]
            );
            assert_eq!(episode_1.mythology.monster, Monster::Cyclops);
            assert_eq!(episode_1.mythology.max_sanctuaries, 0);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.mythology.opponent_gods,
                vec![God::Poseidon, God::Apollo, God::Artemis, God::Ares, God::Aphrodite]
            );
            assert_eq!(
                episode_2.mythology.proponent_gods,
                vec![
                    (God::Athena, false),
                    (God::Hermes, true),
                    (God::Zeus, false),
                    (God::Hephaestus, false)
                ]
            );
            assert_eq!(episode_2.mythology.monster, Monster::Cyclops);
            assert_eq!(episode_2.mythology.max_sanctuaries, 1);

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(
                episode_3.mythology.opponent_gods,
                vec![God::Poseidon, God::Apollo, God::Artemis, God::Ares, God::Aphrodite]
            );
            assert_eq!(
                episode_3.mythology.proponent_gods,
                vec![
                    (God::Athena, false),
                    (God::Hermes, true),
                    (God::Zeus, false),
                    (God::Hephaestus, false)
                ]
            );
            assert_eq!(episode_3.mythology.monster, Monster::Cyclops);
            assert_eq!(episode_3.mythology.max_sanctuaries, 1);

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(
                episode_4.mythology.opponent_gods,
                vec![God::Poseidon, God::Apollo, God::Artemis, God::Ares, God::Aphrodite]
            );
            assert_eq!(
                episode_4.mythology.proponent_gods,
                vec![
                    (God::Athena, true),
                    (God::Hermes, true),
                    (God::Zeus, true),
                    (God::Hephaestus, false)
                ]
            );
            assert_eq!(episode_4.mythology.monster, Monster::Cyclops);
            assert_eq!(episode_4.mythology.max_sanctuaries, 3);

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(
                episode_5.mythology.opponent_gods,
                vec![God::Poseidon, God::Apollo, God::Artemis, God::Ares, God::Aphrodite]
            );
            assert_eq!(
                episode_5.mythology.proponent_gods,
                vec![
                    (God::Athena, true),
                    (God::Hermes, true),
                    (God::Zeus, true),
                    (God::Hephaestus, false)
                ]
            );
            assert_eq!(episode_5.mythology.monster, Monster::Cyclops);
            assert_eq!(episode_5.mythology.max_sanctuaries, 3);

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(
                colony_episode_1.mythology.opponent_gods,
                vec![God::Poseidon, God::Apollo, God::Artemis, God::Ares, God::Aphrodite]
            );
            assert_eq!(
                colony_episode_1.mythology.proponent_gods,
                vec![
                    (God::Athena, false),
                    (God::Hermes, true),
                    (God::Zeus, false),
                    (God::Hephaestus, true)
                ]
            );
            assert_eq!(colony_episode_1.mythology.monster, Monster::Cyclops);
            assert_eq!(colony_episode_1.mythology.max_sanctuaries, 2);

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(
                colony_episode_2.mythology.opponent_gods,
                vec![God::Poseidon, God::Apollo, God::Artemis, God::Ares, God::Aphrodite]
            );
            assert_eq!(
                colony_episode_2.mythology.proponent_gods,
                vec![
                    (God::Athena, true),
                    (God::Hermes, false),
                    (God::Zeus, true),
                    (God::Hephaestus, true)
                ]
            );
            assert_eq!(colony_episode_2.mythology.monster, Monster::Cyclops);
            assert_eq!(colony_episode_2.mythology.max_sanctuaries, 3);
        }

        return Ok(());
    }
}
