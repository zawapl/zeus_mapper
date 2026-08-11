use crate::adventure::Civilization;
use crate::adventure::MonetaryAmount;
use crate::adventure::resource::ResourceType;
use crate::adventure::resource::TradedGoods;
use crate::constants::data_constant::data_constants;
use crate::differ::default_differ_impl;
use crate::prelude::BoxedArray;
use crate::prelude::DataConstant;
use crate::prelude::WorldLocationData;
use crate::prelude::WorldMapElementData;
use my_macros::LogDifferences;

#[derive(LogDifferences, Debug, PartialEq)]
pub struct WorldLocation {
    pub marker_variant: u8, // fixme they should be derivable
    pub position: (u16, u16),
    pub name: EntityName,
    pub label_position: LabelPosition,
    pub city_number: usize,
    pub location_type: WorldLocationType,
}

/// Which kind of world-map location this is, carrying only the fields relevant to that kind.
///
/// Several fields the raw format stores for every location (e.g. `civilization`/`favour`/tribute
/// on a `ParentCity`, `name`/`civilization` on an `EnchantedPlace`) are intentionally not modeled
/// here even where real, non-zero data exists for them - they don't appear to be meaningful for
/// those location types in-game. `to_data` writes sensible defaults back for whatever a variant
/// doesn't carry, the same way `empty_extras_shaped_location` does for unused slots.
#[derive(PartialEq, Debug)]
pub enum WorldLocationType {
    ParentCity(ParentCity),
    Colony(Colony),
    ForeignCity(ForeignCity),
    DistantCity(DistantCity),
    EnchantedPlace(EnchantedPlace),
}

default_differ_impl!(WorldLocationType);

#[derive(PartialEq, Debug)]
pub struct ParentCity {
    pub leader_name: EntityName,
}

#[derive(PartialEq, Debug)]
pub struct Colony {
    pub leader_name: EntityName,
    pub active: bool,
    pub paying_tribute: bool,
    pub received_tribute: (ResourceType, MonetaryAmount),
    pub paid_tribute: (ResourceType, MonetaryAmount),
    pub military_strength: u32,
    pub economy_strength: u32,
    pub favour: u32,
    pub traded_goods: Vec<TradedGoods>,
}

#[derive(PartialEq, Debug)]
pub struct ForeignCity {
    pub leader_name: EntityName,
    pub civilization: LocationCivilization,
    pub active: bool,
    pub visible: bool,
    pub relation: Relationship,
    pub paying_tribute: bool,
    pub received_tribute: (ResourceType, MonetaryAmount),
    pub paid_tribute: (ResourceType, MonetaryAmount),
    pub military_strength: u32,
    pub economy_strength: u32,
    pub favour: u32,
    pub traded_goods: Vec<TradedGoods>,
}

#[derive(PartialEq, Debug)]
pub struct DistantCity {
    pub leader_name: EntityName,
    pub direction: WorldDirection,
    pub active: bool,
    pub visible: bool,
    pub military_strength: u32,
    pub economy_strength: u32,
    pub traded_goods: Vec<TradedGoods>,
}

#[derive(PartialEq, Debug)]
pub struct EnchantedPlace {
    pub active: bool,
    pub visible: bool,
    pub military_strength: u32,
}

impl WorldLocationType {
    // Dispatch on the raw `location_type` byte; falls back to `ForeignCity` for any unrecognized
    // value, mirroring `Event::from_data`'s fallback to `GoodsRequest`.
    fn from_data(location: &WorldLocationData, new_file_ver: bool) -> WorldLocationType {
        let leader_name = if location.leader_name == 255 {
            EntityName::Custom(location.custom_leader_name.clone())
        } else {
            EntityName::TextId(location.leader_name)
        };
        let active = resolve_active(location, new_file_ver);

        return match location.location_type {
            0 => WorldLocationType::ParentCity(ParentCity { leader_name }),
            1 => WorldLocationType::Colony(Colony {
                leader_name,
                active,
                paying_tribute: location.tribute != 0,
                received_tribute: (
                    resolve_pay_resource(location.tribute_rec_resource, new_file_ver),
                    location.tribute_rec_amount,
                ),
                paid_tribute: (
                    resolve_pay_resource(location.tribute_pay_resource, new_file_ver),
                    location.tribute_pay_amount,
                ),
                military_strength: location.military_strength,
                economy_strength: location.economical_strength,
                favour: resolve_favour(location, new_file_ver),
                traded_goods: TradedGoods::vec_from_data(
                    &location.selling,
                    &location.buying,
                    location.trade_quantities.as_slice(),
                    new_file_ver,
                ),
            }),
            4 => WorldLocationType::DistantCity(DistantCity {
                leader_name,
                // `civilization` is repurposed on `Distant` records to encode this marker's
                // off-map compass direction rather than a nation skin - see `WorldDirection`.
                direction: WorldDirection::try_resolve(&location.civilization).unwrap_or(WorldDirection::None),
                active,
                visible: location.visible == 0,
                military_strength: location.military_strength,
                economy_strength: location.economical_strength,
                traded_goods: TradedGoods::vec_from_data(
                    &location.selling,
                    &location.buying,
                    location.trade_quantities.as_slice(),
                    new_file_ver,
                ),
            }),
            5 => WorldLocationType::EnchantedPlace(EnchantedPlace {
                active,
                visible: location.visible == 0,
                military_strength: location.military_strength,
            }),
            // Covers `2` (`Foreign`'s real encoding) and any unrecognized value.
            _ => WorldLocationType::ForeignCity(ForeignCity {
                leader_name,
                civilization: LocationCivilization::try_resolve(&location.civilization).unwrap_or(LocationCivilization::Greek),
                active,
                visible: location.visible == 0,
                relation: Relationship::try_resolve(&location.attitude).unwrap_or(Relationship::Ally),
                paying_tribute: location.tribute != 0,
                received_tribute: (
                    resolve_pay_resource(location.tribute_rec_resource, new_file_ver),
                    location.tribute_rec_amount,
                ),
                paid_tribute: (
                    resolve_pay_resource(location.tribute_pay_resource, new_file_ver),
                    location.tribute_pay_amount,
                ),
                military_strength: location.military_strength,
                economy_strength: location.economical_strength,
                favour: resolve_favour(location, new_file_ver),
                traded_goods: TradedGoods::vec_from_data(
                    &location.selling,
                    &location.buying,
                    location.trade_quantities.as_slice(),
                    new_file_ver,
                ),
            }),
        };
    }

    // Returns `(raw_location_type, WorldLocationData field overrides)`; `WorldLocation::to_data`
    // layers these onto a base record built from the fields common to every variant.
    fn to_data(&self) -> (u8, WorldLocationData) {
        let mut data = empty_extras_shaped_location();

        let (location_type, leader_name) = match self {
            WorldLocationType::ParentCity(parent) => (0, &parent.leader_name),
            WorldLocationType::Colony(colony) => {
                data.active_new = if colony.active { 1 } else { 0 };
                data.tribute = if colony.paying_tribute { 1 } else { 0 };
                data.tribute_rec_resource = colony.received_tribute.0.value() as u16;
                data.tribute_rec_amount = colony.received_tribute.1;
                data.tribute_pay_resource = colony.paid_tribute.0.value() as u16;
                data.tribute_pay_amount = colony.paid_tribute.1;
                data.military_strength = colony.military_strength;
                data.economical_strength = colony.economy_strength;
                data.favour_new = colony.favour;
                let (selling, buying, trade_quantities) = TradedGoods::vec_to_data::<8, 37>(&colony.traded_goods);
                data.selling = selling;
                data.buying = buying;
                data.trade_quantities = BoxedArray::from_vec(trade_quantities.to_vec());
                (1, &colony.leader_name)
            }
            WorldLocationType::ForeignCity(foreign) => {
                data.civilization = foreign.civilization.value();
                data.active_new = if foreign.active { 1 } else { 0 };
                data.visible = if foreign.visible { 0 } else { 4 };
                data.attitude = foreign.relation.value();
                data.tribute = if foreign.paying_tribute { 1 } else { 0 };
                data.tribute_rec_resource = foreign.received_tribute.0.value() as u16;
                data.tribute_rec_amount = foreign.received_tribute.1;
                data.tribute_pay_resource = foreign.paid_tribute.0.value() as u16;
                data.tribute_pay_amount = foreign.paid_tribute.1;
                data.military_strength = foreign.military_strength;
                data.economical_strength = foreign.economy_strength;
                data.favour_new = foreign.favour;
                let (selling, buying, trade_quantities) = TradedGoods::vec_to_data::<8, 37>(&foreign.traded_goods);
                data.selling = selling;
                data.buying = buying;
                data.trade_quantities = BoxedArray::from_vec(trade_quantities.to_vec());
                (2, &foreign.leader_name)
            }
            WorldLocationType::DistantCity(distant) => {
                data.civilization = distant.direction.value();
                data.active_new = if distant.active { 1 } else { 0 };
                data.visible = if distant.visible { 0 } else { 4 };
                data.military_strength = distant.military_strength;
                data.economical_strength = distant.economy_strength;
                let (selling, buying, trade_quantities) = TradedGoods::vec_to_data::<8, 37>(&distant.traded_goods);
                data.selling = selling;
                data.buying = buying;
                data.trade_quantities = BoxedArray::from_vec(trade_quantities.to_vec());
                (4, &distant.leader_name)
            }
            WorldLocationType::EnchantedPlace(enchanted) => {
                data.active_new = if enchanted.active { 1 } else { 0 };
                data.visible = if enchanted.visible { 0 } else { 4 };
                data.military_strength = enchanted.military_strength;
                return (5, data);
            }
        };

        let (leader_name, custom_leader_name) = match leader_name {
            EntityName::Custom(name) => (255, name.clone()),
            EntityName::TextId(id) => (*id, String::new()),
        };
        data.leader_name = leader_name;
        data.custom_leader_name = custom_leader_name;

        return (location_type, data);
    }

    /// This location's marker sprite id and pixel size on the world map.
    ///
    /// A formula over each variant's own state, confirmed against ~340 real records with zero
    /// exceptions for every variant but `ParentCity`. `ParentCity`'s own sprite depends purely on
    /// the adventure's overall `Civilization` - unlike every other variant, it has no
    /// active/inactive state of its own (the player's home city is always "founded"). Confirmed
    /// against 44 of 45 real non-tutorial records; the one exception (`The Odyssey V2`) is an
    /// already-known format-conversion artifact - see `Colony`'s own note on the same file below.
    /// `Colony`'s formula additionally depends on the adventure's overall `Civilization`
    /// (`Adventure.civilization`, a different concept from `ForeignCity`'s own `civilization` or
    /// `DistantCity`'s `direction`). See the "sprite_id/sprite_size investigation" note in
    /// DATA_MAPPING.md for the full methodology.
    ///
    /// **Assumptions**: `active` (`WorldLocationType`'s own resolved field - see `resolve_active`)
    /// is this location's single "visually active" state. The raw format actually splits this
    /// across two format-version-gated bytes (`WorldLocationData.active_new`/`active_old` - see
    /// `resolve_active`), but `to_pak` (the only caller of `sprite`) always emits new-format maps,
    /// so only the new-format sprite mapping is reachable here. On `Colony` specifically, an
    /// old-format `active_old` marker renders in-game as a third, visually distinct sprite (`70`,
    /// confirmed by `}Open Play Sandbox`, the one real record with `active_old` set) rather than
    /// the new-format active sprite (`47`) - a real gap for round-tripping such a record's exact
    /// original sprite through `to_pak`, accepted the same way `ParentCity`'s `Odyssey V2` gap
    /// below is.
    pub(crate) fn sprite(&self, adventure_civilization: Civilization) -> (u8, (u16, u16)) {
        return match self {
            WorldLocationType::ParentCity(_) => match adventure_civilization {
                Civilization::Greek => PARENT_GREEK_SPRITE,
                Civilization::Atlantean => PARENT_ATLANTEAN_SPRITE,
            },
            WorldLocationType::Colony(colony) => {
                // fixme still doesn't seem to match what is seen in game, needs more investigation
                if colony.active {
                    COLONY_ATLANTEAN_ACTIVE_SPRITE
                } else {
                    match adventure_civilization {
                        Civilization::Greek => COLONY_GREEK_INACTIVE_SPRITE,
                        Civilization::Atlantean => COLONY_ATLANTEAN_INACTIVE_SPRITE,
                    }
                }
            }
            WorldLocationType::ForeignCity(foreign) => {
                if foreign.active {
                    FOREIGN_ACTIVE_SPRITES[foreign.civilization.value() as usize]
                } else {
                    FOREIGN_GENERIC_SPRITE
                }
            }
            WorldLocationType::DistantCity(distant) => {
                let direction_index = distant.direction.value() as usize;
                if distant.active {
                    DISTANT_ACTIVE_SPRITES[direction_index]
                } else {
                    DISTANT_INACTIVE_SPRITES[direction_index]
                }
            }
            WorldLocationType::EnchantedPlace(enchanted) => {
                if enchanted.active {
                    ENCHANTED_ACTIVE_SPRITE
                } else {
                    ENCHANTED_INACTIVE_SPRITE
                }
            }
        };
    }
}

const PARENT_GREEK_SPRITE: (u8, (u16, u16)) = (69, (33, 38));
const PARENT_ATLANTEAN_SPRITE: (u8, (u16, u16)) = (46, (35, 39));
const COLONY_GREEK_ACTIVE_SPRITE: (u8, (u16, u16)) = (70, (23, 25));
const COLONY_GREEK_INACTIVE_SPRITE: (u8, (u16, u16)) = (71, (23, 24));

const COLONY_ATLANTEAN_ACTIVE_SPRITE: (u8, (u16, u16)) = (47, (26, 25));
const COLONY_ATLANTEAN_INACTIVE_SPRITE: (u8, (u16, u16)) = (48, (26, 25));

const ENCHANTED_ACTIVE_SPRITE: (u8, (u16, u16)) = (85, (33, 35));
const ENCHANTED_INACTIVE_SPRITE: (u8, (u16, u16)) = (86, (34, 33));

const FOREIGN_GENERIC_SPRITE: (u8, (u16, u16)) = (117, (14, 13));

// Indexed by `LocationCivilization.value()` (`Greek`=0 .. `Atlantean`=9).
const FOREIGN_ACTIVE_SPRITES: [(u8, (u16, u16)); 10] = [
    (72, (32, 33)),
    (73, (33, 33)),
    (74, (32, 37)),
    (75, (33, 31)),
    (76, (35, 28)),
    (49, (36, 39)),
    (50, (32, 40)),
    (51, (33, 38)),
    (52, (34, 32)),
    (53, (38, 34)),
];

// Indexed by `WorldDirection.value()` (`None`=0 .. `Nw`=8).
const DISTANT_ACTIVE_SPRITES: [(u8, (u16, u16)); 9] = [
    (87, (27, 22)),
    (88, (29, 38)),
    (89, (35, 28)),
    (90, (51, 22)),
    (91, (37, 28)),
    (92, (27, 38)),
    (93, (38, 30)),
    (94, (51, 22)),
    (95, (36, 29)),
];

// Indexed by `WorldDirection.value()` (`None`=0 .. `Nw`=8).
const DISTANT_INACTIVE_SPRITES: [(u8, (u16, u16)); 9] = [
    (117, (14, 13)),
    (118, (14, 29)),
    (119, (28, 21)),
    (120, (39, 12)),
    (121, (29, 22)),
    (122, (15, 30)),
    (123, (28, 22)),
    (124, (39, 13)),
    (125, (14, 13)),
];

// Old-format (`new_file_ver == false`) locations leave `favour_new` `0` for most locations - the
// real value instead lives in `favour_old`, confirmed against `The Odyssey`. A few old-format
// locations do carry a real, nonzero value in `favour_new` itself, which is preferred when
// present. Only `Colony`/`ForeignCity` keep a `favour` field at all, so this is never called for
// `ParentCity` (which had no real favour value in either format per DATA_MAPPING.md) or the other
// variants.
fn resolve_favour(location: &WorldLocationData, new_file_ver: bool) -> u32 {
    if !new_file_ver && location.favour_new == 0 {
        return location.favour_old as u32;
    }
    return location.favour_new;
}

// Unlike `favour`, old-format locations always leave `active_new` `0` - the real value lives
// entirely in `active_old` instead, with no exceptions, confirmed against 250 real
// `Colony`/`ForeignCity`/`DistantCity`/`EnchantedPlace` records. Not called for `ParentCity`,
// which has no "not yet founded" state to toggle - see `WorldLocationType::sprite`.
fn resolve_active(location: &WorldLocationData, new_file_ver: bool) -> bool {
    if new_file_ver {
        location.active_new != 0
    } else {
        location.active_old != 0
    }
}

impl WorldLocation {
    fn from_data(slot: usize, location: &WorldLocationData, element: &WorldMapElementData, new_file_ver: bool) -> WorldLocation {
        let name = if location.name == 255 {
            EntityName::Custom(location.custom_name.clone())
        } else {
            EntityName::TextId(location.name as u32)
        };

        // A marker-less location (see the "one observed exception" in DATA_MAPPING.md) has no real
        // label placement byte to read, so default it to the common case.
        let label_position = if element.variant == 0 {
            LabelPosition::Bottom
        } else {
            LabelPosition::try_resolve(&element.label_position).unwrap_or(LabelPosition::Bottom)
        };

        return WorldLocation {
            city_number: slot,
            position: (element.x, element.y),
            marker_variant: element.variant,
            label_position,
            name,
            location_type: WorldLocationType::from_data(location, new_file_ver),
        };
    }

    fn to_data(&self, adventure_civilization: Civilization) -> (WorldLocationData, WorldMapElementData) {
        let (name, custom_name) = match &self.name {
            EntityName::Custom(name) => (255, name.clone()),
            EntityName::TextId(id) => (*id as u8, String::new()),
        };

        let (location_type, mut location) = self.location_type.to_data();
        location.exists = 1;
        location.location_type = location_type;
        location.name = name;
        location.slot_index = self.city_number as u8;
        location.custom_name = custom_name;

        let (sprite_id, sprite_size) = self.location_type.sprite(adventure_civilization);

        // `region_name` is stamped in by `vec_to_data`, which knows the slot; see DATA_MAPPING.md.
        let element = WorldMapElementData {
            variant: self.marker_variant,
            data_a: [2, 0, 0],
            x: self.position.0,
            y: self.position.1,
            sprite_width: sprite_size.0,
            sprite_height: sprite_size.1,
            sprite_id,
            unknown_a: Default::default(),
            label_position: self.label_position.value(),
            unknown_b: Default::default(),
            region_name: 0,
            city_name: 0,
            data_d: Default::default(),
            custom_names: String::new(),
        };

        return (location, element);
    }

    pub(crate) fn vec_from_data(
        locations: &BoxedArray<WorldLocationData, 22>,
        elements: &BoxedArray<WorldMapElementData, 200>,
        new_file_ver: bool,
    ) -> Vec<WorldLocation> {
        // `elements[i]` is NOT reliably the marker for `locations[i]`: an unused location slot
        // (`exists == 0`) shifts every later location's real marker forward by one raw index in
        // `elements`, so slots must be paired via the `region_name` back-link instead of position -
        // see DATA_MAPPING.md.
        let default_element = WorldMapElementData::default();

        return locations
            .iter()
            .enumerate()
            .filter(|(_, location)| location.exists != 0)
            .map(|(slot, location)| {
                let element = elements
                    .iter()
                    .find(|element| element.variant != 0 && element.region_name as usize == slot)
                    .unwrap_or(&default_element);
                WorldLocation::from_data(slot, location, element, new_file_ver)
            })
            .collect();
    }

    pub(crate) fn vec_to_data(
        world_locations: &[WorldLocation],
        adventure_civilization: Civilization,
    ) -> (BoxedArray<WorldLocationData, 22>, BoxedArray<WorldMapElementData, 200>) {
        // `WorldLocation::default()` padding would be unsafe here - see DATA_MAPPING.md.
        let mut locations = vec![empty_extras_shaped_location(); 22];
        let mut elements = vec![WorldMapElementData::default(); 200];

        for world_location in world_locations {
            if world_location.city_number < locations.len() {
                let (location_data, mut element_data) = world_location.to_data(adventure_civilization);
                element_data.region_name = world_location.city_number as u8;
                locations[world_location.city_number] = location_data;
                elements[world_location.city_number] = element_data;
            }
        }

        return (BoxedArray::from_vec(locations), BoxedArray::from_vec(elements));
    }
}

fn resolve_pay_resource(resource_id: u16, new_file_ver: bool) -> ResourceType {
    return ResourceType::try_resolve_for_format(&(resource_id as u8), new_file_ver).unwrap_or(ResourceType::Drachmas);
}

fn empty_extras_shaped_location() -> WorldLocationData {
    return WorldLocationData {
        exists: 0,
        location_type: 0,
        name: 0,
        slot_index: 0,
        unknown_3: Default::default(),
        trade_quantities: Default::default(),
        unknown_133: Default::default(),
        buying: [0; 8],
        selling: [0; 8],
        civilization: 0,
        leader_name: 0,
        attitude: 0,
        economical_strength: 0,
        military_strength: 0,
        tribute: 0,
        tribute_rec_amount: 0,
        tribute_pay_amount: 0,
        tribute_pay_resource: 0,
        tribute_rec_resource: 0,
        unknown_240: Default::default(),
        favour_old: 0,
        unknown_341: Default::default(),
        active_old: 0,
        unknown_353: Default::default(),
        favour_new: 0,
        unknown_360: Default::default(),
        active_new: 0,
        visible: 0,
        unknown_376: Default::default(),
        unknown_476: vec![0; 28],
        trade_route_visible: 4,
        custom_name: String::new(),
        custom_leader_name: String::new(),
        tail: vec![0; 3],
    };
}
#[derive(PartialEq, Debug)]
pub enum EntityName {
    Custom(String),
    TextId(u32),
}

default_differ_impl!(EntityName);

// The world-map sprite set ("nation skin") a location's marker uses - a different, unrelated
// encoding from the adventure-level `Civilization` (`SettingsData.real_episode_data[i]`).
// Confirmed against `The Youngest Twins`/`The Odyssey`; other raw values (4/6/7) are seen in other
// real adventures but not yet cross-referenced to a name.
data_constants!(LocationCivilization<u32> {
    Greek = 0,
    Trojan = 1,
    Persian = 2,
    Centaur = 3,
    Amazon = 4,
    Egyptian = 5,
    Mayan = 6,
    Phoenician = 7,
    Oceanid = 8,
    Atlantean = 9,
});

// A `ForeignCity`'s diplomatic stance toward the player. The raw `attitude` field also carries a
// `Colony`-valued `3` on the parent city's own entry and on `EnchantedPlace`s, unexplained, but
// those location types have no `relation` field of their own to receive it.
data_constants!(Relationship<u32> {
    Ally = 0,
    Rival = 1,
    Vasal = 2,
});

// A `DistantCity`'s off-map compass direction, read from the same raw `civilization` byte
// `ForeignCity` uses for its nation skin (`LocationCivilization`) - unrelated meanings, just a
// shared raw field, the same way many `EventData` fields are reinterpreted per event type.
data_constants!(WorldDirection<u32> {
    None = 0,
    N = 1,
    Ne = 2,
    E = 3,
    Se = 4,
    S = 5,
    Sw = 6,
    W = 7,
    Nw = 8,
});

// Placement of the location's name label relative to its marker. Confirmed against `The Odyssey`.
data_constants!(LabelPosition<u8> {
    Left = 0,
    Top = 1,
    Right = 2,
    Bottom = 3,
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::adventure::Adventure;
    use std::io::Result;

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            assert_eq!(
                adventure.world_locations,
                vec![
                    WorldLocation {
                        marker_variant: 1,
                        position: (359, 559,),
                        name: EntityName::Custom(String::from("Diapraea"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 0,
                        location_type: WorldLocationType::ParentCity(ParentCity {
                            leader_name: EntityName::Custom(String::from("Diapepes"),),
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (587, 466,),
                        name: EntityName::Custom(String::from("Dharkuum"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 1,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::Custom(String::from("Phoeda"),),
                            active: false,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Bronze, 8,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Selling(ResourceType::Orichalc, 24,),
                                TradedGoods::Buying(ResourceType::Fleece, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (500, 278,),
                        name: EntityName::Custom(String::from("Atlantis"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 2,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Atlon"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 1,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Grape, 12,),
                                TradedGoods::Selling(ResourceType::Olive, 12,),
                                TradedGoods::Selling(ResourceType::OliveOil, 12,),
                                TradedGoods::Buying(ResourceType::Armor, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (553, 211,),
                        name: EntityName::Custom(String::from("Mestonia"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 3,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Mestor"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 12,),
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Wheat, 12,),
                                TradedGoods::Buying(ResourceType::Sculpture, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (512, 141,),
                        name: EntityName::Custom(String::from("Elasippium"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 4,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Elasippus"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fleece, 24,),
                                TradedGoods::Buying(ResourceType::Carrot, 12,),
                                TradedGoods::Buying(ResourceType::Wheat, 12,),
                                TradedGoods::Buying(ResourceType::Armor, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (531, 339,),
                        name: EntityName::Custom(String::from("Gades"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 5,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Gadrius"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::BlackMarble, 12,),
                                TradedGoods::Buying(ResourceType::Meat, 12,),
                                TradedGoods::Buying(ResourceType::Orichalc, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (428, 240,),
                        name: EntityName::Custom(String::from("Evanestus"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 6,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Eggudico"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 12,),
                                TradedGoods::Selling(ResourceType::OliveOil, 24,),
                                TradedGoods::Buying(ResourceType::Fleece, 12,),
                                TradedGoods::Buying(ResourceType::BlackMarble, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (439, 388,),
                        name: EntityName::Custom(String::from("Euaemos"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 7,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Euaemon"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 12,),
                                TradedGoods::Selling(ResourceType::Wine, 12,),
                                TradedGoods::Buying(ResourceType::Meat, 12,),
                                TradedGoods::Buying(ResourceType::Bronze, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (420, 325,),
                        name: EntityName::Custom(String::from("Leucippium"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 8,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Nauklarida"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Selling(ResourceType::Wine, 12,),
                                TradedGoods::Buying(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::Fleece, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (345, 309,),
                        name: EntityName::Custom(String::from("Ampher"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 9,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Ampheres"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 500,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Carrot, 12,),
                                TradedGoods::Buying(ResourceType::Bronze, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (334, 406,),
                        name: EntityName::Custom(String::from("Mneussus"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 10,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Mneusus"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 750,),
                            military_strength: 1,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::BlackMarble, 12,),
                                TradedGoods::Buying(ResourceType::Fleece, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (200, 428,),
                        name: EntityName::Custom(String::from("Autoch"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 11,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Evercles"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Sculpture, 4,),
                            military_strength: 1,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Carrot, 12,),
                                TradedGoods::Buying(ResourceType::Orichalc, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (321, 499,),
                        name: EntityName::Custom(String::from("Azium"),),
                        label_position: LabelPosition::Left,
                        city_number: 12,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Azaes"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 750,),
                            military_strength: 5,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 12,),
                                TradedGoods::Selling(ResourceType::Wine, 12,),
                                TradedGoods::Buying(ResourceType::Armor, 24,),
                                TradedGoods::Buying(ResourceType::Sculpture, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (249, 132,),
                        name: EntityName::Custom(String::from("Zeff"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 13,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Nohu"),),
                            civilization: LocationCivilization::Oceanid,
                            active: false,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fish, 12,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Marble, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (195, 213,),
                        name: EntityName::Custom(String::from("Xilyuf"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 14,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Caffsyemb"),),
                            civilization: LocationCivilization::Oceanid,
                            active: false,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fish, 12,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::BlackMarble, 12,),
                                TradedGoods::Buying(ResourceType::Sculpture, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (390, 165,),
                        name: EntityName::Custom(String::from("Yehm"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 15,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Cymph"),),
                            civilization: LocationCivilization::Oceanid,
                            active: false,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fish, 12,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::BlackMarble, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (597, 311,),
                        name: EntityName::Custom(String::from("Cennia"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 16,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Plawl"),),
                            civilization: LocationCivilization::Centaur,
                            active: true,
                            visible: false,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Buying(ResourceType::Wine, 12,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (396, 432,),
                        name: EntityName::Custom(String::from("Mt Phaesus"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 17,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Spaunch"),),
                            civilization: LocationCivilization::Centaur,
                            active: true,
                            visible: false,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Buying(ResourceType::Wine, 12,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (485, 200,),
                        name: EntityName::Custom(String::from("Mt Geimn"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 18,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Borth"),),
                            civilization: LocationCivilization::Centaur,
                            active: true,
                            visible: false,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Meat, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (644, 394,),
                        name: EntityName::Custom(String::from("Persiopolis"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 20,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(0,),
                            civilization: LocationCivilization::Persian,
                            active: true,
                            visible: false,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (660, 476,),
                        name: EntityName::Custom(String::from("Egyptium"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 21,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Memotep"),),
                            civilization: LocationCivilization::Egyptian,
                            active: true,
                            visible: false,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![],
                        },),
                    },
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            assert_eq!(
                adventure.world_locations,
                vec![
                    WorldLocation {
                        marker_variant: 1,
                        position: (176, 312,),
                        name: EntityName::TextId(27,),
                        label_position: LabelPosition::Top,
                        city_number: 0,
                        location_type: WorldLocationType::ParentCity(ParentCity {
                            leader_name: EntityName::TextId(81,),
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (141, 215,),
                        name: EntityName::TextId(9,),
                        label_position: LabelPosition::Right,
                        city_number: 1,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(12,),
                            active: false,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fleece, 12,),
                            military_strength: 2,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Fleece, 36,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (178, 543,),
                        name: EntityName::TextId(65,),
                        label_position: LabelPosition::Bottom,
                        city_number: 2,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(78,),
                            active: false,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 500,),
                            paid_tribute: (ResourceType::Fish, 8,),
                            military_strength: 2,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Buying(ResourceType::Olive, 36,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (230, 475,),
                        name: EntityName::TextId(79,),
                        label_position: LabelPosition::Bottom,
                        city_number: 3,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(57,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 500,),
                            paid_tribute: (ResourceType::Drachmas, 500,),
                            military_strength: 4,
                            economy_strength: 5,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Selling(ResourceType::Marble, 24,),
                                TradedGoods::Selling(ResourceType::Fleece, 24,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (294, 467,),
                        name: EntityName::TextId(57,),
                        label_position: LabelPosition::Bottom,
                        city_number: 4,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(52,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 5,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Wine, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (545, 195,),
                        name: EntityName::TextId(61,),
                        label_position: LabelPosition::Left,
                        city_number: 5,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(8,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 50,),
                            military_strength: 5,
                            economy_strength: 3,
                            favour: 80,
                            traded_goods: vec![
                                TradedGoods::Buying(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Armor, 24,),
                                TradedGoods::Buying(ResourceType::Wine, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (157, 357,),
                        name: EntityName::TextId(80,),
                        label_position: LabelPosition::Bottom,
                        city_number: 6,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(13,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 4,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Selling(ResourceType::Fish, 24,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (242, 279,),
                        name: EntityName::TextId(81,),
                        label_position: LabelPosition::Bottom,
                        city_number: 7,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(38,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 4,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Buying(ResourceType::Olive, 24,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (682, 154,),
                        name: EntityName::TextId(3,),
                        label_position: LabelPosition::Left,
                        city_number: 8,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(18,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 1000,),
                            paid_tribute: (ResourceType::Drachmas, 1000,),
                            military_strength: 6,
                            economy_strength: 5,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 24,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Buying(ResourceType::Olive, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (542, 119,),
                        name: EntityName::TextId(78,),
                        label_position: LabelPosition::Left,
                        city_number: 9,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(50,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Sculpture, 1,),
                            military_strength: 6,
                            economy_strength: 4,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 36,),
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Grape, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (595, 183,),
                        name: EntityName::TextId(68,),
                        label_position: LabelPosition::Right,
                        city_number: 10,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(73,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Wheat, 10,),
                            military_strength: 5,
                            economy_strength: 5,
                            favour: 30,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (312, 400,),
                        name: EntityName::TextId(40,),
                        label_position: LabelPosition::Top,
                        city_number: 11,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(8,),
                            civilization: LocationCivilization::Greek,
                            active: false,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (314, 605,),
                        name: EntityName::TextId(16,),
                        label_position: LabelPosition::Left,
                        city_number: 12,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(82,),
                            direction: WorldDirection::Se,
                            active: false,
                            visible: true,
                            military_strength: 3,
                            economy_strength: 3,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Fleece, 12,),
                                TradedGoods::Selling(ResourceType::Wine, 12,),
                                TradedGoods::Buying(ResourceType::Bronze, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (153, 596,),
                        name: EntityName::TextId(34,),
                        label_position: LabelPosition::Right,
                        city_number: 13,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(25,),
                            direction: WorldDirection::Sw,
                            active: false,
                            visible: true,
                            military_strength: 3,
                            economy_strength: 3,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                ]
            );
        }

        return Ok(());
    }

    #[test]
    #[ignore]
    fn parse_the_odyssey_v2() -> Result<()> {
        // "The Odyssey V2" is an archived snapshot of `Test`'s first resave of `The Odyssey` into
        // the new settings format (see docs/adventure_model_todo.md's "Cross-version byte check"
        // section) - MapData (and so world_locations) was byte-identical to `parse_the_odyssey`'s
        // source at the time this was captured. `favour` values below have been checked in-game and
        // corrected from the library's raw output; `economy_strength`/`military_strength` and the
        // parent city's own `favour` are NOT fully confirmed - see the TODO doc before trusting
        // those specifically. `Test` itself has since been resaved again with real content changes
        // (not just a format change), so this snapshot is preserved under its own name to keep the
        // original clean comparison pair against `parse_the_odyssey` intact.
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey V2"))?;

            assert_eq!(
                adventure.world_locations,
                vec![
                    WorldLocation {
                        marker_variant: 1,
                        position: (176, 312,),
                        name: EntityName::TextId(27,),
                        label_position: LabelPosition::Top,
                        city_number: 0,
                        location_type: WorldLocationType::ParentCity(ParentCity {
                            leader_name: EntityName::TextId(81,),
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (141, 215,),
                        name: EntityName::TextId(9,),
                        label_position: LabelPosition::Right,
                        city_number: 1,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(12,),
                            active: false,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fleece, 12,),
                            military_strength: 2,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Fleece, 36,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (178, 543,),
                        name: EntityName::TextId(65,),
                        label_position: LabelPosition::Bottom,
                        city_number: 2,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(78,),
                            active: false,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 500,),
                            paid_tribute: (ResourceType::Fish, 8,),
                            military_strength: 2,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Buying(ResourceType::Olive, 36,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (230, 475,),
                        name: EntityName::TextId(79,),
                        label_position: LabelPosition::Bottom,
                        city_number: 3,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(57,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 500,),
                            paid_tribute: (ResourceType::Drachmas, 500,),
                            military_strength: 4,
                            economy_strength: 5,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Selling(ResourceType::Marble, 24,),
                                TradedGoods::Selling(ResourceType::Fleece, 24,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (294, 467,),
                        name: EntityName::TextId(57,),
                        label_position: LabelPosition::Bottom,
                        city_number: 4,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(52,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 5,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Wine, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (545, 195,),
                        name: EntityName::TextId(61,),
                        label_position: LabelPosition::Left,
                        city_number: 5,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(8,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 50,),
                            military_strength: 5,
                            economy_strength: 3,
                            favour: 80,
                            traded_goods: vec![
                                TradedGoods::Buying(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Armor, 24,),
                                TradedGoods::Buying(ResourceType::Wine, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (157, 357,),
                        name: EntityName::TextId(80,),
                        label_position: LabelPosition::Bottom,
                        city_number: 6,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(13,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 4,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Selling(ResourceType::Fish, 24,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (242, 279,),
                        name: EntityName::TextId(81,),
                        label_position: LabelPosition::Bottom,
                        city_number: 7,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(38,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 4,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Buying(ResourceType::Olive, 24,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (682, 154,),
                        name: EntityName::TextId(3,),
                        label_position: LabelPosition::Left,
                        city_number: 8,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(18,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 1000,),
                            paid_tribute: (ResourceType::Drachmas, 1000,),
                            military_strength: 6,
                            economy_strength: 5,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 24,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Buying(ResourceType::Olive, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (542, 119,),
                        name: EntityName::TextId(78,),
                        label_position: LabelPosition::Left,
                        city_number: 9,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(50,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Sculpture, 1,),
                            military_strength: 6,
                            economy_strength: 4,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 36,),
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Grape, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (595, 183,),
                        name: EntityName::TextId(68,),
                        label_position: LabelPosition::Right,
                        city_number: 10,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(73,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Wheat, 10,),
                            military_strength: 5,
                            economy_strength: 5,
                            favour: 30,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (312, 400,),
                        name: EntityName::TextId(40,),
                        label_position: LabelPosition::Top,
                        city_number: 11,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(8,),
                            civilization: LocationCivilization::Greek,
                            active: false,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (314, 605,),
                        name: EntityName::TextId(16,),
                        label_position: LabelPosition::Left,
                        city_number: 12,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(82,),
                            direction: WorldDirection::Se,
                            active: false,
                            visible: true,
                            military_strength: 3,
                            economy_strength: 3,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Fleece, 12,),
                                TradedGoods::Selling(ResourceType::Wine, 12,),
                                TradedGoods::Buying(ResourceType::Bronze, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (153, 596,),
                        name: EntityName::TextId(34,),
                        label_position: LabelPosition::Right,
                        city_number: 13,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(25,),
                            direction: WorldDirection::Sw,
                            active: false,
                            visible: true,
                            military_strength: 3,
                            economy_strength: 3,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                ]
            );
        }

        return Ok(());
    }

    #[test]
    #[ignore]
    fn parse_test_adventure() -> Result<()> {
        // "Test" has been resaved more than once during this investigation (see
        // `parse_the_odyssey_v2` above for the archived first resave, which was still
        // byte-identical `MapData` to `parse_the_odyssey`'s source). This later resave changed real
        // content, not just the container format: `TestP.map` (the parent map) grew and its bytes
        // diverge from the earlier snapshot, while the colony maps did not - so parent-city stats
        // (`favour`, `military_strength`, `economy_strength`, marker sprite) and several other
        // locations' `active`/`visible`/tribute fields have genuinely changed since
        // `parse_the_odyssey_v2` was captured, not just been reinterpreted differently. Values below
        // are the library's current output against *this* resave, not yet confirmed in-game - except
        // location `12`'s `direction`, which this resave's in-game edit deliberately changed from
        // `Se` to `S` (only that one field) to confirm `WorldDirection`'s raw encoding; see
        // `WorldDirection`'s own doc comment.
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/Test"))?;

            assert_eq!(
                adventure.world_locations,
                vec![
                    WorldLocation {
                        marker_variant: 1,
                        position: (176, 312,),
                        name: EntityName::TextId(27,),
                        label_position: LabelPosition::Top,
                        city_number: 0,
                        location_type: WorldLocationType::ParentCity(ParentCity {
                            leader_name: EntityName::TextId(81,),
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (141, 215,),
                        name: EntityName::TextId(9,),
                        label_position: LabelPosition::Right,
                        city_number: 1,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(12,),
                            active: false,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fleece, 12,),
                            military_strength: 2,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Fleece, 36,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (178, 543,),
                        name: EntityName::TextId(65,),
                        label_position: LabelPosition::Bottom,
                        city_number: 2,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(78,),
                            active: false,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 500,),
                            paid_tribute: (ResourceType::Fish, 8,),
                            military_strength: 2,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Buying(ResourceType::Olive, 36,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (230, 475,),
                        name: EntityName::TextId(79,),
                        label_position: LabelPosition::Bottom,
                        city_number: 3,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(57,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 500,),
                            paid_tribute: (ResourceType::Drachmas, 500,),
                            military_strength: 4,
                            economy_strength: 5,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Selling(ResourceType::Marble, 24,),
                                TradedGoods::Selling(ResourceType::Fleece, 24,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (294, 467,),
                        name: EntityName::TextId(57,),
                        label_position: LabelPosition::Bottom,
                        city_number: 4,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(52,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 5,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Wine, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (545, 195,),
                        name: EntityName::TextId(61,),
                        label_position: LabelPosition::Left,
                        city_number: 5,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(8,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 50,),
                            military_strength: 5,
                            economy_strength: 3,
                            favour: 80,
                            traded_goods: vec![
                                TradedGoods::Buying(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Armor, 24,),
                                TradedGoods::Buying(ResourceType::Wine, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (157, 357,),
                        name: EntityName::TextId(80,),
                        label_position: LabelPosition::Bottom,
                        city_number: 6,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(13,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 4,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Selling(ResourceType::Fish, 24,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (242, 279,),
                        name: EntityName::TextId(81,),
                        label_position: LabelPosition::Bottom,
                        city_number: 7,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(38,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 4,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![TradedGoods::Buying(ResourceType::Olive, 24,),],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (682, 154,),
                        name: EntityName::TextId(3,),
                        label_position: LabelPosition::Left,
                        city_number: 8,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(18,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 1000,),
                            paid_tribute: (ResourceType::Drachmas, 1000,),
                            military_strength: 6,
                            economy_strength: 5,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 24,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Buying(ResourceType::Olive, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (542, 119,),
                        name: EntityName::TextId(78,),
                        label_position: LabelPosition::Left,
                        city_number: 9,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(50,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Sculpture, 1,),
                            military_strength: 6,
                            economy_strength: 4,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 36,),
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Grape, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (595, 183,),
                        name: EntityName::TextId(68,),
                        label_position: LabelPosition::Right,
                        city_number: 10,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(73,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Wheat, 10,),
                            military_strength: 5,
                            economy_strength: 5,
                            favour: 30,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (312, 400,),
                        name: EntityName::TextId(40,),
                        label_position: LabelPosition::Top,
                        city_number: 11,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(8,),
                            civilization: LocationCivilization::Greek,
                            active: false,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (314, 605,),
                        name: EntityName::TextId(16,),
                        label_position: LabelPosition::Left,
                        city_number: 12,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(82,),
                            direction: WorldDirection::S,
                            active: false,
                            visible: false,
                            military_strength: 3,
                            economy_strength: 3,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Fleece, 12,),
                                TradedGoods::Selling(ResourceType::Wine, 12,),
                                TradedGoods::Buying(ResourceType::Bronze, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (153, 596,),
                        name: EntityName::TextId(34,),
                        label_position: LabelPosition::Right,
                        city_number: 13,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(25,),
                            direction: WorldDirection::Sw,
                            active: false,
                            visible: false,
                            military_strength: 3,
                            economy_strength: 3,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_the_sinking_of_atlantis() -> Result<()> {
        // Another new-format (`version_2 == 26`) adventure, for coverage beyond `The Youngest Twins`.
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Sinking of Atlantis"))?;

            assert_eq!(
                adventure.world_locations,
                vec![
                    WorldLocation {
                        marker_variant: 1,
                        position: (483, 290,),
                        name: EntityName::Custom(String::from("Atlantis"),),
                        label_position: LabelPosition::Top,
                        city_number: 0,
                        location_type: WorldLocationType::ParentCity(ParentCity {
                            leader_name: EntityName::Custom(String::from("Atlon"),),
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (416, 237,),
                        name: EntityName::Custom(String::from("Evenestus"),),
                        label_position: LabelPosition::Left,
                        city_number: 1,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::Custom(String::from("Eggudico"),),
                            active: true,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Orange, 18,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 24,),
                                TradedGoods::Selling(ResourceType::Orange, 12,),
                                TradedGoods::Buying(ResourceType::Fleece, 24,),
                                TradedGoods::Buying(ResourceType::BlackMarble, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (400, 311,),
                        name: EntityName::Custom(String::from("Leucippium"),),
                        label_position: LabelPosition::Top,
                        city_number: 2,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::Custom(String::from("Nauklarida"),),
                            active: true,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Orichalc, 8,),
                            military_strength: 1,
                            economy_strength: 4,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Orichalc, 24,),
                                TradedGoods::Buying(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::Fleece, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (533, 320,),
                        name: EntityName::Custom(String::from("Gades"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 3,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Gadrius"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::BlackMarble, 16,),
                            military_strength: 4,
                            economy_strength: 5,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::BlackMarble, 12,),
                                TradedGoods::Buying(ResourceType::Meat, 12,),
                                TradedGoods::Buying(ResourceType::Orichalc, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (420, 367,),
                        name: EntityName::Custom(String::from("Euaemos"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 4,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Euaemon"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Wheat, 16,),
                            military_strength: 3,
                            economy_strength: 4,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 24,),
                                TradedGoods::Selling(ResourceType::Wine, 12,),
                                TradedGoods::Buying(ResourceType::Orange, 36,),
                                TradedGoods::Buying(ResourceType::Bronze, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (332, 325,),
                        name: EntityName::Custom(String::from("Ampher"),),
                        label_position: LabelPosition::Left,
                        city_number: 5,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Ampheres"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Armor, 4,),
                            military_strength: 2,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Buying(ResourceType::Bronze, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (329, 401,),
                        name: EntityName::Custom(String::from("Mneussus"),),
                        label_position: LabelPosition::Left,
                        city_number: 6,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Mneuseus"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Bronze, 8,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Selling(ResourceType::Orichalc, 12,),
                                TradedGoods::Buying(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::BlackMarble, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (553, 212,),
                        name: EntityName::Custom(String::from("Mestonia\0Mesto"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 7,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Mestor"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 500,),
                            military_strength: 1,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 12,),
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Wheat, 12,),
                                TradedGoods::Buying(ResourceType::Sculpture, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (487, 134,),
                        name: EntityName::Custom(String::from("Elasippium"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 8,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Elassipus"),),
                            civilization: LocationCivilization::Atlantean,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 1,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Orange, 36,),
                                TradedGoods::Selling(ResourceType::Fleece, 24,),
                                TradedGoods::Buying(ResourceType::Wheat, 12,),
                                TradedGoods::Buying(ResourceType::Armor, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (593, 308,),
                        name: EntityName::Custom(String::from("Cennia"),),
                        label_position: LabelPosition::Right,
                        city_number: 9,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Plawl"),),
                            civilization: LocationCivilization::Centaur,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 2,
                            economy_strength: 3,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Meat, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (496, 192,),
                        name: EntityName::Custom(String::from("Mt Geimm"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 10,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Borth"),),
                            civilization: LocationCivilization::Centaur,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 1,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Meat, 12,),
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (382, 423,),
                        name: EntityName::Custom(String::from("Mt Phaesus"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 11,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Spaunch"),),
                            civilization: LocationCivilization::Centaur,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 250,),
                            military_strength: 1,
                            economy_strength: 1,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Meat, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 36,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (234, 231,),
                        name: EntityName::Custom(String::from("Xilyuf"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 12,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Caffsyemb"),),
                            civilization: LocationCivilization::Oceanid,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 1000,),
                            military_strength: 4,
                            economy_strength: 2,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 36,),
                                TradedGoods::Selling(ResourceType::Orichalc, 12,),
                                TradedGoods::Buying(ResourceType::BlackMarble, 12,),
                                TradedGoods::Buying(ResourceType::Sculpture, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (410, 160,),
                        name: EntityName::Custom(String::from("Yehm"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 13,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Cymph"),),
                            civilization: LocationCivilization::Oceanid,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fish, 12,),
                            military_strength: 2,
                            economy_strength: 1,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 12,),
                                TradedGoods::Buying(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::BlackMarble, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (238, 149,),
                        name: EntityName::Custom(String::from("Zeff"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 14,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::Custom(String::from("Nohu"),),
                            civilization: LocationCivilization::Oceanid,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fish, 12,),
                            military_strength: 2,
                            economy_strength: 1,
                            favour: 50,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::Wood, 12,),
                                TradedGoods::Buying(ResourceType::Marble, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (270, 427,),
                        name: EntityName::Custom(String::from("Crimson Isle"),),
                        label_position: LabelPosition::Bottom,
                        city_number: 16,
                        location_type: WorldLocationType::EnchantedPlace(EnchantedPlace {
                            active: true,
                            visible: true,
                            military_strength: 3,
                        },),
                    },
                ]
            );
        }

        return Ok(());
    }

    #[test]
    fn parse_open_play_sandbox_2() -> Result<()> {
        // Another old-format (`version_2 != 26`) adventure, for coverage beyond `The Odyssey`.
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/Open Play Sandbox 2"))?;

            assert_eq!(
                adventure.world_locations,
                vec![
                    WorldLocation {
                        marker_variant: 1,
                        position: (354, 423,),
                        name: EntityName::TextId(67,),
                        label_position: LabelPosition::Bottom,
                        city_number: 0,
                        location_type: WorldLocationType::ParentCity(ParentCity {
                            leader_name: EntityName::TextId(71,),
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (250, 218,),
                        name: EntityName::TextId(49,),
                        label_position: LabelPosition::Left,
                        city_number: 1,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(47,),
                            active: true,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Fleece, 16,),
                            military_strength: 2,
                            economy_strength: 4,
                            favour: 100,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Meat, 24,),
                                TradedGoods::Selling(ResourceType::Grape, 24,),
                                TradedGoods::Selling(ResourceType::Fleece, 24,),
                                TradedGoods::Buying(ResourceType::Sculpture, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (465, 126,),
                        name: EntityName::TextId(74,),
                        label_position: LabelPosition::Left,
                        city_number: 2,
                        location_type: WorldLocationType::Colony(Colony {
                            leader_name: EntityName::TextId(76,),
                            active: true,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 1000,),
                            military_strength: 4,
                            economy_strength: 1,
                            favour: 100,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Marble, 12,),
                                TradedGoods::Selling(ResourceType::Wine, 24,),
                                TradedGoods::Buying(ResourceType::Meat, 24,),
                                TradedGoods::Buying(ResourceType::Grape, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (307, 384,),
                        name: EntityName::TextId(40,),
                        label_position: LabelPosition::Bottom,
                        city_number: 4,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(8,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Vasal,
                            paying_tribute: true,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Wheat, 16,),
                            military_strength: 3,
                            economy_strength: 3,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 24,),
                                TradedGoods::Selling(ResourceType::Grape, 24,),
                                TradedGoods::Selling(ResourceType::Olive, 24,),
                                TradedGoods::Buying(ResourceType::Armor, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (396, 350,),
                        name: EntityName::TextId(37,),
                        label_position: LabelPosition::Bottom,
                        city_number: 5,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(6,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Wine, 16,),
                            military_strength: 3,
                            economy_strength: 2,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wheat, 24,),
                                TradedGoods::Selling(ResourceType::Wine, 24,),
                                TradedGoods::Buying(ResourceType::Fleece, 12,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (640, 433,),
                        name: EntityName::TextId(39,),
                        label_position: LabelPosition::Left,
                        city_number: 6,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(23,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::Drachmas, 500,),
                            military_strength: 3,
                            economy_strength: 2,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Buying(ResourceType::Fish, 36,),
                                TradedGoods::Buying(ResourceType::Wine, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (176, 138,),
                        name: EntityName::TextId(72,),
                        label_position: LabelPosition::Bottom,
                        city_number: 7,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(21,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::OliveOil, 16,),
                            military_strength: 1,
                            economy_strength: 4,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Olive, 24,),
                                TradedGoods::Selling(ResourceType::OliveOil, 12,),
                                TradedGoods::Buying(ResourceType::Cheese, 36,),
                                TradedGoods::Buying(ResourceType::Armor, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (221, 395,),
                        name: EntityName::TextId(45,),
                        label_position: LabelPosition::Left,
                        city_number: 8,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(4,),
                            civilization: LocationCivilization::Greek,
                            active: true,
                            visible: true,
                            relation: Relationship::Ally,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 100,),
                            paid_tribute: (ResourceType::OliveOil, 16,),
                            military_strength: 3,
                            economy_strength: 4,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fleece, 24,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Selling(ResourceType::OliveOil, 24,),
                                TradedGoods::Buying(ResourceType::Marble, 36,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (581, 177,),
                        name: EntityName::TextId(68,),
                        label_position: LabelPosition::Left,
                        city_number: 9,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(20,),
                            civilization: LocationCivilization::Trojan,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 500,),
                            paid_tribute: (ResourceType::Fish, 24,),
                            military_strength: 3,
                            economy_strength: 2,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 12,),
                                TradedGoods::Selling(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Marble, 12,),
                                TradedGoods::Buying(ResourceType::Fleece, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (300, 191,),
                        name: EntityName::TextId(46,),
                        label_position: LabelPosition::Right,
                        city_number: 10,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(24,),
                            civilization: LocationCivilization::Centaur,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 250,),
                            paid_tribute: (ResourceType::Drachmas, 100,),
                            military_strength: 2,
                            economy_strength: 4,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 24,),
                                TradedGoods::Selling(ResourceType::Armor, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Wine, 12,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (687, 305,),
                        name: EntityName::TextId(54,),
                        label_position: LabelPosition::Left,
                        city_number: 11,
                        location_type: WorldLocationType::ForeignCity(ForeignCity {
                            leader_name: EntityName::TextId(31,),
                            civilization: LocationCivilization::Persian,
                            active: true,
                            visible: true,
                            relation: Relationship::Rival,
                            paying_tribute: false,
                            received_tribute: (ResourceType::Drachmas, 750,),
                            paid_tribute: (ResourceType::Drachmas, 1000,),
                            military_strength: 4,
                            economy_strength: 3,
                            favour: 75,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Wood, 12,),
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Buying(ResourceType::Olive, 36,),
                                TradedGoods::Buying(ResourceType::OliveOil, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (667, 605,),
                        name: EntityName::TextId(13,),
                        label_position: LabelPosition::Left,
                        city_number: 12,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(68,),
                            direction: WorldDirection::Se,
                            active: true,
                            visible: true,
                            military_strength: 1,
                            economy_strength: 5,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Bronze, 12,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Wheat, 24,),
                                TradedGoods::Buying(ResourceType::Grape, 36,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (305, 608,),
                        name: EntityName::TextId(16,),
                        label_position: LabelPosition::Left,
                        city_number: 13,
                        location_type: WorldLocationType::DistantCity(DistantCity {
                            leader_name: EntityName::TextId(53,),
                            direction: WorldDirection::Se,
                            active: true,
                            visible: true,
                            military_strength: 1,
                            economy_strength: 5,
                            traded_goods: vec![
                                TradedGoods::Selling(ResourceType::Fish, 24,),
                                TradedGoods::Selling(ResourceType::Sculpture, 12,),
                                TradedGoods::Buying(ResourceType::Wood, 24,),
                                TradedGoods::Buying(ResourceType::Bronze, 24,),
                            ],
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (619, 115,),
                        name: EntityName::TextId(52,),
                        label_position: LabelPosition::Left,
                        city_number: 14,
                        location_type: WorldLocationType::EnchantedPlace(EnchantedPlace {
                            active: true,
                            visible: true,
                            military_strength: 1,
                        },),
                    },
                    WorldLocation {
                        marker_variant: 1,
                        position: (281, 529,),
                        name: EntityName::TextId(59,),
                        label_position: LabelPosition::Left,
                        city_number: 15,
                        location_type: WorldLocationType::EnchantedPlace(EnchantedPlace {
                            active: true,
                            visible: true,
                            military_strength: 1,
                        },),
                    },
                ]
            );
        }

        return Ok(());
    }
}
