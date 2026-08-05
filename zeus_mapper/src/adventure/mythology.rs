use crate::adventure::God;
use crate::prelude::DataConstant;
use crate::prelude::MythologyData;
use my_macros::LogDifferences;

#[derive(LogDifferences)]
pub struct Mythology {
    pub proponent_gods: Vec<(God, SanctuaryAllowed)>,
    pub opponent_gods: Vec<God>,
    pub max_sanctuaries: u8,
}

impl Mythology {
    pub(crate) fn from_data(mythology_data: &MythologyData) -> Mythology {
        return Mythology {
            proponent_gods: mythology_data
                .proponent_gods
                .iter()
                .filter_map(|i| God::try_resolve(i))
                .map(|god| {
                    return (god, mythology_data.sanctuaries_allowed.contains(&(god as u8)));
                })
                .collect(),
            opponent_gods: mythology_data.opponent_gods.iter().filter_map(|i| God::try_resolve(i)).collect(),
            max_sanctuaries: mythology_data.max_sanctuaries as u8,
        };
    }

    pub(crate) fn to_data(&self) -> MythologyData {
        let mut mythology_data = MythologyData::default();

        for (i, (god, sanctuary_allowed)) in self.proponent_gods.iter().take(12).enumerate() {
            mythology_data.proponent_gods[i] = god.value();
            if *sanctuary_allowed {
                mythology_data.sanctuaries_allowed[i] = god.value() as u8;
            }
        }

        for (i, god) in self.opponent_gods.iter().take(12).enumerate() {
            mythology_data.opponent_gods[i] = god.value();
        }

        mythology_data.max_sanctuaries = self.max_sanctuaries as u32;

        return mythology_data;
    }
}

type SanctuaryAllowed = bool;
