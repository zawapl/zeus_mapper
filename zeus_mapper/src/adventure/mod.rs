use crate::constants::data_constant::data_constants;

pub(crate) mod adventure;
pub(crate) mod building_type;
pub(crate) mod city_map;
pub(crate) mod colony_episode;
pub(crate) mod episode_goals;
pub(crate) mod event;
pub(crate) mod mythology;
pub(crate) mod parent_city_episode;
pub(crate) mod terrain_flags;

data_constants!(Civilization<u32>{
    Greek = 0,
    Atlantean = 1,
});

data_constants!(God<u32> {
    Zeus = 0,
    Poseidon = 1,
    Demeter = 2,
    Apollo = 3,
    Artemis = 4,
    Ares = 5,
    Aphrodite = 6,
    Hermes = 7,
    Athena = 8,
    Hephaestus = 9,
    Dionysus = 10,
    Hades = 11,
    Hera = 12,
    Atlas = 13,
});

impl God {
    pub fn get_monster(&self) -> Monster {
        return match self {
            God::Zeus => Monster::Cyclops,
            God::Poseidon => Monster::Kraken,
            God::Demeter => Monster::Medusa,
            God::Apollo => Monster::Scylla,
            God::Artemis => Monster::CalydonianBoar,
            God::Ares => Monster::Dragon,
            God::Aphrodite => Monster::Hector,
            God::Hermes => Monster::Minotaur,
            God::Athena => Monster::Hydra,
            God::Hephaestus => Monster::Talos,
            God::Dionysus => Monster::Maenad,
            God::Hades => Monster::Cerberus,
            God::Hera => Monster::Sphinx,
            God::Atlas => Monster::Chimera,
        };
    }
}

data_constants!(Monster<u32> {
    Hydra = 0,
    Kraken = 1,
    Cerberus = 2,
    Cyclops = 3,
    Minotaur = 4,
    Medusa = 5,
    CalydonianBoar = 6,
    Dragon = 7,
    Talos = 8,
    Hector = 9,
    Scylla = 10,
    Maenad = 11,
    Sphinx = 12,
    Chimera = 13,
    Harpy = 14,
    Echidna = 15,
});

impl Monster {
    pub fn god(&self) -> Option<God> {
        return match self {
            Monster::Hydra => Some(God::Athena),
            Monster::Kraken => Some(God::Poseidon),
            Monster::Cerberus => Some(God::Hades),
            Monster::Cyclops => Some(God::Zeus),
            Monster::Minotaur => Some(God::Hermes),
            Monster::Medusa => Some(God::Demeter),
            Monster::CalydonianBoar => Some(God::Artemis),
            Monster::Dragon => Some(God::Ares),
            Monster::Talos => Some(God::Hephaestus),
            Monster::Hector => Some(God::Aphrodite),
            Monster::Scylla => Some(God::Apollo),
            Monster::Maenad => Some(God::Dionysus),
            Monster::Sphinx => Some(God::Hera),
            Monster::Chimera => Some(God::Atlas),
            _ => None,
        };
    }
}

data_constants!(NextEpisode<u32>{
    Victory = 0,
    Colony = 1,
    Parent = 2,
});

pub type MonetaryAmount = u32;

#[derive(PartialEq, Debug)]
pub struct CityId(u16);
