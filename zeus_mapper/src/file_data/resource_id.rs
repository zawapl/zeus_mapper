use crate::constants::data_constant::DataConstant;
use crate::constants::data_constant::data_constants;

data_constants!(ResourceId<i8>{
    Urchin = 1,
    Fish = 2,
    Meat = 3,
    Cheese = 4,
    Carrot = 5,
    Onion = 6,
    Wheat = 7,
    Orange = 8,
    Wood = 9,
    Bronze = 10,
    Marble = 11,
    Grape = 12,
    Olive = 13,
    Fleece = 14,
    Horse = 15,
    BlackMarble = 16,
    Orichalc = 17,
    Armor = 18,
    Sculpture = 19,
    OliveOil = 20,
    Wine = 21,
    Chariot = 22,
    Drachmas = 23,
    Troops = 24,
    Food = 25,
    Hero = 26,
});

impl ResourceId {
    pub fn try_resolve_for_format(id: &i8, new_file_ver: bool) -> Option<ResourceId> {
        if new_file_ver {
            return ResourceId::try_resolve(id);
        }

        return ResourceId::try_resolve(&ResourceId::map_to_new_id(*id));
    }

    fn map_to_new_id(old_format_id: i8) -> i8 {
        return match old_format_id {
            ..0 => -1,
            0..8 => old_format_id,
            8..15 => old_format_id + 1,
            15..19 => old_format_id + 3,
            19.. => old_format_id + 4,
        };
    }

    fn map_to_old_id(new_format_id: i8) -> i8 {
        return match new_format_id {
            ..0 => -1,
            0..8 => new_format_id,
            8 => -1, // Orange
            9..16 => new_format_id - 1,
            16 => -1, // BlackMarble
            17 => -1, // Orichalc
            18..22 => new_format_id - 3,
            22 => -1, // Chariot
            23.. => new_format_id - 4,
        };
    }

    // Map new resource id to one in the desired version
    pub fn map_id_to_version(new_format_id: i8, new_file_ver: bool) -> i8 {
        if new_file_ver || new_format_id < 0 {
            return new_format_id;
        }

        return Self::map_to_old_id(new_format_id);
    }

    // Resources with a real per-adventure price (`Adventure.prices`) - everything except `Horse`
    // (only ever used to build units) and `Chariot` (never observed as a priced good in any real
    // adventure). Confirmed against every real adventure's demand/price/supply-change events, which
    // never target either of those two.
    pub fn tradeable_resource_ids() -> [i8; 20] {
        return [
            ResourceId::Urchin.value(),
            ResourceId::Fish.value(),
            ResourceId::Meat.value(),
            ResourceId::Cheese.value(),
            ResourceId::Carrot.value(),
            ResourceId::Onion.value(),
            ResourceId::Wheat.value(),
            ResourceId::Orange.value(),
            ResourceId::Wood.value(),
            ResourceId::Bronze.value(),
            ResourceId::Marble.value(),
            ResourceId::Grape.value(),
            ResourceId::Olive.value(),
            ResourceId::Fleece.value(),
            ResourceId::BlackMarble.value(),
            ResourceId::Orichalc.value(),
            ResourceId::Armor.value(),
            ResourceId::Sculpture.value(),
            ResourceId::OliveOil.value(),
            ResourceId::Wine.value(),
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_to_new_and_back_round_trips() {
        for old_id in 1..=22i8 {
            let new_id = ResourceId::map_to_new_id(old_id);
            assert_eq!(ResourceId::map_id_to_version(new_id, false), old_id);
        }
    }

    #[test]
    fn try_resolve_for_format_new_file_ver() {
        assert_eq!(ResourceId::try_resolve_for_format(&16, true), Some(ResourceId::BlackMarble));
        assert_eq!(ResourceId::try_resolve_for_format(&24, true), Some(ResourceId::Troops));
    }

    #[test]
    fn try_resolve_for_format_old_file_ver() {
        // Old format has no `Orange` (new id 8), so old raw `8` (`Wood` in the new format) reads as `Wood`.
        assert_eq!(ResourceId::try_resolve_for_format(&8, false), Some(ResourceId::Wood));
        // Old raw `13` is one short of new `Fleece` (`14`) because `Orange` (`8`) was skipped.
        assert_eq!(ResourceId::try_resolve_for_format(&13, false), Some(ResourceId::Fleece));
        // Old raw `19` is four short of new `Drachmas` (`23`).
        assert_eq!(ResourceId::try_resolve_for_format(&19, false), Some(ResourceId::Drachmas));
    }

    #[test]
    fn new_to_old_id_maps_event_target_ids() {
        assert_eq!(ResourceId::map_id_to_version(24, false), 20);
        assert_eq!(ResourceId::map_id_to_version(25, false), 21);
        assert_eq!(ResourceId::map_id_to_version(26, false), 22);
    }

    #[test]
    fn new_to_old_id_passes_through_sentinel_and_new_file_ver() {
        assert_eq!(ResourceId::map_id_to_version(-1, false), -1);
        assert_eq!(ResourceId::map_id_to_version(16, true), 16);
    }

    #[test]
    fn map_id_to_version_returns_negative_for_ids_without_an_old_format_representation() {
        for id in [
            ResourceId::Orange,
            ResourceId::BlackMarble,
            ResourceId::Orichalc,
            ResourceId::Chariot,
        ] {
            assert_eq!(ResourceId::map_id_to_version(id.value(), false), -1);
        }
    }
}
