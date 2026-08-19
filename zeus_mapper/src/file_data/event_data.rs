use crate::constants::data_constant::DataConstant;
use crate::file_data::resource_id::ResourceId;
use crate::utils::read_utils::ReadFrom;
use crate::utils::validation::ValidationError;
use crate::utils::validation::ValidationResult;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, PartialEq, Copy, Clone, LogDifferences)]
pub struct EventData {
    pub id: u16,
    pub event_type: u8,
    pub month: u8,
    pub item_chosen: i16,
    pub first_item: i16,
    pub second_item: i16,
    pub third_item: i16,
    pub amount: u16,
    pub fixed_amount: i16,
    pub min_amount: i16,
    pub max_amount: i16,
    pub time: u16,
    pub fixed_time: i16,
    pub min_time: i16,
    pub max_time: i16,
    pub target: u16,
    pub fixed_target: i16,
    pub min_target: i16,
    pub max_target: i16,
    pub on_success: i16,
    pub on_failure: u16,
    pub flags: u32,
    pub warnings: i16,
    pub time_ctr: u16,
    pub status: u16,
    pub need_msg_res: u16,
    pub triggerer: u16,
    pub god_or_mon_or_warship_id: u16,
    pub mtar1: u16,
    pub mtar2: u16,
    pub mtar3: u16,
    pub magg: u16,
    pub unkown_row: [u16; 9],
    pub trigger_on_1: u16,
    pub trigger_on_2: u16,
    pub eff_on_city: u16,
    pub source: u16,
    pub source_fixed: i16,
    pub source_min: i16,
    pub source_max: i16,
    pub subtype: u16,
    pub prev_amount: u16,
    pub related_to_triggered_evt: u32,
    pub constant_1_0x00: u16,
    pub trig_reason: u16,
    pub constant_2_0x00: u16,
    pub constant_3_0x00: u8,
    pub other_city: u8,
    pub loot_type: u16,
    pub loot_amount: u16,
    pub constant_4_0x00: u8,
    pub ally_city: u8,
    pub ally_strength: u8,
    pub to_strength: u8,
    pub unknown_1: u16,
    pub quest: u8,
    pub tail: u8,
}

impl EventData {
    pub fn validate(&self, new_file_ver: bool) -> ValidationResult {
        if self.constant_1_0x00 != 0 {
            return Err(ValidationError::expected_exactly("constant_1_0x00", self.constant_1_0x00, 0));
        }

        if self.constant_2_0x00 != 0 {
            return Err(ValidationError::expected_exactly("constant_2_0x00", self.constant_2_0x00, 0));
        }

        if self.constant_3_0x00 != 0 {
            return Err(ValidationError::expected_exactly("constant_3_0x00", self.constant_3_0x00, 0));
        }

        if self.constant_4_0x00 != 0 {
            return Err(ValidationError::expected_exactly("constant_4_0x00", self.constant_4_0x00, 0));
        }

        self.validate_month()?;
        EventData::validate_non_negative_or_unset("on_success", self.on_success)?;
        EventData::validate_non_negative("warnings", self.warnings)?;

        return match self.event_type {
            1 => self.validate_request(new_file_ver),
            2 => self.validate_invasion(),
            3 => self.validate_earthquake(),
            4 => self.validate_quest(new_file_ver),
            5 => self.validate_landslide(),
            8 => self.validate_wage_increase(),
            9 => self.validate_wage_decrease(),
            13 => self.validate_demand_increase(new_file_ver),
            14 => self.validate_demand_decrease(new_file_ver),
            15 => self.validate_price_increase(new_file_ver),
            16 => self.validate_price_decrease(new_file_ver),
            19 => self.validate_city_status_change(),
            20 => self.validate_rival_army_change(),
            21 => self.validate_supply_increase(new_file_ver),
            22 => self.validate_supply_decrease(new_file_ver),
            23 => self.validate_gift(new_file_ver),
            24 => self.validate_lava_flow(),
            25 => self.validate_tidal_wave(),
            26 => self.validate_monster_invasion(new_file_ver),
            27 => self.validate_god_invasion(),
            28 => self.validate_sink_land(),
            _ => Err(ValidationError::expected_one_of(
                "event_type",
                self.event_type,
                &[1, 2, 3, 4, 5, 8, 9, 13, 14, 15, 16, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28],
            )),
        };
    }

    fn validate_request(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;

        if self.warnings < 1 {
            return Err(ValidationError::expected_range("warnings", self.warnings, 1, i16::MAX));
        }

        return match self.subtype {
            0 => self.validate_general_goods_request(new_file_ver),
            1 => self.validate_city_under_attack(new_file_ver),
            2 => self.validate_city_attacks_rival(new_file_ver),
            3 => self.validate_festival_request(new_file_ver),
            7 => self.validate_city_terrorized(new_file_ver),
            4 => self.validate_construction_request(new_file_ver),
            5 => self.validate_famine_request(new_file_ver),
            6 => self.validate_financial_request(new_file_ver),
            _ => Err(ValidationError::expected_one_of("subtype", self.subtype, &[0, 1, 2, 3, 4, 5, 6, 7])),
        };
    }

    fn validate_general_goods_request(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_items_one_of(
            &[
                ResourceId::Urchin.value(),
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
                ResourceId::Drachmas.value(),
                ResourceId::Food.value(),
            ],
            new_file_ver,
        )?;

        return self.validate_amount_range();
    }

    fn validate_city_under_attack(&self, new_file_ver: bool) -> ValidationResult {
        if !matches!(self.eff_on_city, 0 | 1 | 2) {
            return Err(ValidationError::expected_one_of("eff_on_city", self.eff_on_city, &[0, 1, 2]));
        }

        return self.validate_item_exact(ResourceId::Troops.value(), new_file_ver);
    }

    fn validate_city_attacks_rival(&self, new_file_ver: bool) -> ValidationResult {
        if !matches!(self.eff_on_city, 0 | 2) {
            return Err(ValidationError::expected_one_of("eff_on_city", self.eff_on_city, &[0, 2]));
        }
        return self.validate_item_exact(ResourceId::Troops.value(), new_file_ver);
    }

    fn validate_festival_request(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_items_one_of(
            &[
                ResourceId::Wood.value(),
                ResourceId::Bronze.value(),
                ResourceId::Marble.value(),
                ResourceId::Grape.value(),
                ResourceId::Olive.value(),
                ResourceId::Fleece.value(),
                ResourceId::Armor.value(),
                ResourceId::Sculpture.value(),
                ResourceId::OliveOil.value(),
                ResourceId::Wine.value(),
                ResourceId::Drachmas.value(),
                ResourceId::Food.value(),
            ],
            new_file_ver,
        )?;

        if self.god_or_mon_or_warship_id > 13 {
            return Err(ValidationError::expected_range(
                "god_or_mon_or_warship_id",
                self.god_or_mon_or_warship_id,
                0,
                13,
            ));
        }

        return self.validate_amount_range();
    }

    fn validate_city_terrorized(&self, new_file_ver: bool) -> ValidationResult {
        if !matches!(self.eff_on_city, 0 | 1) {
            return Err(ValidationError::expected_one_of("eff_on_city", self.eff_on_city, &[0, 1]));
        }

        if !matches!(self.god_or_mon_or_warship_id, 0 | 1 | 2) {
            return Err(ValidationError::expected_one_of(
                "god_or_mon_or_warship_id",
                self.god_or_mon_or_warship_id,
                &[0, 1, 2],
            ));
        }

        return self.validate_item_exact(ResourceId::Troops.value(), new_file_ver);
    }

    fn validate_construction_request(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_items_one_of(
            &[
                ResourceId::Wood.value(),
                ResourceId::Marble.value(),
                ResourceId::Sculpture.value(),
                ResourceId::Drachmas.value(),
            ],
            new_file_ver,
        )?;
        return self.validate_amount_range();
    }

    fn validate_famine_request(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_item_exact(ResourceId::Food.value(), new_file_ver)?;

        return self.validate_amount_range();
    }

    fn validate_financial_request(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_item_exact(ResourceId::Drachmas.value(), new_file_ver)?;

        return self.validate_amount_range();
    }

    fn validate_invasion(&self) -> ValidationResult {
        // todo check warships

        self.validate_time_range()?;
        self.validate_source_range()?;
        self.validate_amount_range()?;

        return self.validate_target_range();
    }

    fn validate_earthquake(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;
        return self.validate_disaster_marker();
    }

    fn validate_quest(&self, new_file_ver: bool) -> ValidationResult {
        if self.subtype > 13 {
            return Err(ValidationError::expected_range("subtype", self.subtype, 0, 13));
        }

        self.validate_time_range()?;
        self.validate_item_exact(ResourceId::Hero.value(), new_file_ver)?;

        return self.validate_target_range();
    }

    fn validate_landslide(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;
        return self.validate_disaster_marker();
    }

    fn validate_wage_increase(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;
        return self.validate_amount_range();
    }

    fn validate_wage_decrease(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;
        return self.validate_amount_range();
    }

    fn validate_demand_increase(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_items_one_of(&ResourceId::tradeable_resource_ids(), new_file_ver)?;
        return self.validate_target_range();
    }

    fn validate_demand_decrease(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_items_one_of(&ResourceId::tradeable_resource_ids(), new_file_ver)?;
        return self.validate_target_range();
    }

    fn validate_price_increase(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_items_one_of(&ResourceId::tradeable_resource_ids(), new_file_ver)?;
        return self.validate_amount_range();
    }

    fn validate_price_decrease(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_items_one_of(&ResourceId::tradeable_resource_ids(), new_file_ver)?;
        return self.validate_amount_range();
    }

    fn validate_city_status_change(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;

        return match self.subtype {
            2 => self.validate_trade_shuts_down(),
            3 => self.validate_trade_opens_up(),
            9 => self.validate_rival_becomes_ally(),
            10 => self.validate_city_becomes_rival(),
            11 => self.validate_city_becomes_vassal(),
            13 => self.validate_god_disaster(),
            14 => self.validate_military_buildup(),
            15 => self.validate_military_decline(),
            16 => self.validate_economic_prosperity(),
            17 => self.validate_economic_decline(),
            18 => self.validate_city_becomes_active(),
            19 => self.validate_city_becomes_inactive(),
            20 => self.validate_city_appears(),
            21 => self.validate_city_disappears(),
            23 => self.validate_rebellion_over(),
            24 => self.validate_city_conquered_by(),
            _ => Err(ValidationError::expected_one_of(
                "subtype",
                self.subtype,
                &[2, 3, 9, 10, 11, 13, 14, 15, 16, 17, 18, 19, 20, 21, 23, 24],
            )),
        };
    }

    fn validate_trade_shuts_down(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_trade_opens_up(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_rival_becomes_ally(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_city_becomes_rival(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_city_becomes_vassal(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_god_disaster(&self) -> ValidationResult {
        if self.god_or_mon_or_warship_id > 13 {
            return Err(ValidationError::expected_range(
                "god_or_mon_or_warship_id",
                self.god_or_mon_or_warship_id,
                0,
                13,
            ));
        }

        // `warnings` doubles as the disaster's duration here - confirmed against every real
        // adventure, none of which leaves it at `0`, same as `validate_request`'s requirement.
        if self.warnings == 0 {
            return Err(ValidationError::expected_range("warnings", self.warnings, 1, i16::MAX));
        }

        return Ok(());
    }

    fn validate_military_buildup(&self) -> ValidationResult {
        return self.validate_amount_range();
    }

    fn validate_military_decline(&self) -> ValidationResult {
        return self.validate_amount_range();
    }

    fn validate_economic_prosperity(&self) -> ValidationResult {
        return self.validate_amount_range();
    }

    fn validate_economic_decline(&self) -> ValidationResult {
        return self.validate_amount_range();
    }

    fn validate_city_becomes_active(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_city_becomes_inactive(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_city_appears(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_city_disappears(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_rebellion_over(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_city_conquered_by(&self) -> ValidationResult {
        return Ok(());
    }

    fn validate_rival_army_change(&self) -> ValidationResult {
        self.validate_time_range()?;
        return self.validate_target_range();
    }

    fn validate_supply_increase(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_items_one_of(&ResourceId::tradeable_resource_ids(), new_file_ver)?;
        return self.validate_target_range();
    }

    fn validate_supply_decrease(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_items_one_of(&ResourceId::tradeable_resource_ids(), new_file_ver)?;
        return self.validate_target_range();
    }

    fn validate_gift(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_items_one_of(
            &[
                ResourceId::Fish.value(),
                ResourceId::Meat.value(),
                ResourceId::Cheese.value(),
                ResourceId::Wheat.value(),
                ResourceId::Wood.value(),
                ResourceId::Bronze.value(),
                ResourceId::Marble.value(),
                ResourceId::Grape.value(),
                ResourceId::Fleece.value(),
                ResourceId::BlackMarble.value(),
                ResourceId::Orichalc.value(),
                ResourceId::Armor.value(),
                ResourceId::Sculpture.value(),
                ResourceId::OliveOil.value(),
                ResourceId::Wine.value(),
                ResourceId::Drachmas.value(),
            ],
            new_file_ver,
        )?;
        self.validate_target_range()?;
        return self.validate_amount_range();
    }

    fn validate_lava_flow(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;
        return self.validate_disaster_marker();
    }

    fn validate_tidal_wave(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;
        return self.validate_disaster_marker();
    }

    fn validate_monster_invasion(&self, new_file_ver: bool) -> ValidationResult {
        return match self.subtype {
            0 => self.validate_monster_in_city(),
            1 => self.validate_monster_unleashed(),
            2 => self.validate_monster_invades(new_file_ver),
            _ => Err(ValidationError::expected_one_of("subtype", self.subtype, &[0, 1, 2])),
        };
    }

    fn validate_monster_in_city(&self) -> ValidationResult {
        // The only subtype with no `Occurrence` of its own (see `event.rs`), so
        // `fixed_time`/`min_time`/`max_time` are meaningless leftovers here.
        return self.validate_items_in_range(0, 2);
    }

    fn validate_monster_unleashed(&self) -> ValidationResult {
        self.validate_time_range()?;
        return self.validate_items_in_range(0, 1);
    }

    fn validate_monster_invades(&self, new_file_ver: bool) -> ValidationResult {
        self.validate_time_range()?;

        if new_file_ver {
            if self.first_item != 2 {
                return Err(ValidationError::expected_exactly("first_item", self.first_item, 2));
            }
        } else {
            if !(0..=2).contains(&self.first_item) {
                return Err(ValidationError::expected_range("first_item", self.first_item, 0, 2));
            }
        }

        if self.second_item >= 0 {
            return Err(ValidationError::expected_exactly("second_item", self.second_item, -1));
        }

        if self.third_item >= 0 {
            return Err(ValidationError::expected_exactly("third_item", self.third_item, -1));
        }

        return Ok(());
    }

    fn validate_god_invasion(&self) -> ValidationResult {
        self.validate_time_range()?;
        return self.validate_items_in_range(0, 13);
    }

    fn validate_sink_land(&self) -> ValidationResult {
        self.validate_time_range()?;
        self.validate_target_range()?;
        self.validate_disaster_marker()?;
        return self.validate_amount_range_bounded(1, 5); // todo check why the editor enforces this limit
    }

    fn validate_target_range(&self) -> ValidationResult {
        EventData::validate_non_negative_or_unset("fixed_target", self.fixed_target)?;
        EventData::validate_non_negative_or_unset("min_target", self.min_target)?;
        EventData::validate_non_negative_or_unset("max_target", self.max_target)?;

        if self.fixed_target >= 0 {
            if self.min_target >= 0 {
                return Err(ValidationError::expected_exactly("min_target", self.min_target, -1));
            }

            if self.max_target >= 0 {
                return Err(ValidationError::expected_exactly("max_target", self.max_target, -1));
            }

            return Ok(());
        }

        if self.min_target < 0 || self.max_target < 0 {
            return Err(ValidationError::expected_range("fixed_target", self.fixed_target, 0, i16::MAX));
        }

        if self.max_target < self.min_target {
            return Err(ValidationError::expected_range(
                "max_target",
                self.max_target,
                self.min_target,
                i16::MAX,
            ));
        }

        return Ok(());
    }

    fn validate_source_range(&self) -> ValidationResult {
        EventData::validate_non_negative_or_unset("source_fixed", self.source_fixed)?;
        EventData::validate_non_negative_or_unset("source_min", self.source_min)?;
        EventData::validate_non_negative_or_unset("source_max", self.source_max)?;

        if self.source_fixed >= 0 {
            if self.source_min >= 0 {
                return Err(ValidationError::expected_exactly("source_min", self.source_min, -1));
            }

            if self.source_max >= 0 {
                return Err(ValidationError::expected_exactly("source_max", self.source_max, -1));
            }

            return Ok(());
        }

        if self.source_min < 0 || self.source_max < 0 {
            return Err(ValidationError::expected_range("source_fixed", self.source_fixed, 0, i16::MAX));
        }

        if self.source_max < self.source_min {
            return Err(ValidationError::expected_range(
                "source_max",
                self.source_max,
                self.source_min,
                i16::MAX,
            ));
        }

        return Ok(());
    }

    fn validate_time_range(&self) -> ValidationResult {
        EventData::validate_non_negative_or_unset("fixed_time", self.fixed_time)?;
        EventData::validate_non_negative_or_unset("min_time", self.min_time)?;
        EventData::validate_non_negative_or_unset("max_time", self.max_time)?;

        if self.fixed_time >= 0 {
            if self.min_time >= 0 {
                return Err(ValidationError::expected_exactly("min_time", self.min_time, -1));
            }

            if self.max_time >= 0 {
                return Err(ValidationError::expected_exactly("max_time", self.max_time, -1));
            }

            return Ok(());
        }

        if self.min_time < 0 || self.max_time < 0 {
            return Err(ValidationError::expected_range("fixed_time", self.fixed_time, 0, i16::MAX));
        }

        if self.max_time < self.min_time {
            return Err(ValidationError::expected_range("max_time", self.max_time, self.min_time, i16::MAX));
        }

        return Ok(());
    }

    fn validate_month(&self) -> ValidationResult {
        if self.month > 11 {
            return Err(ValidationError::expected_range("month", self.month, 0, 11));
        }

        return Ok(());
    }

    fn validate_disaster_marker(&self) -> ValidationResult {
        let marker = if self.fixed_target >= 0 {
            self.fixed_target
        } else {
            self.min_target
        };

        if marker < 1 || marker > 9 {
            return Err(ValidationError::expected_range("fixed_target", marker, 1, 9));
        }

        return Ok(());
    }

    // Check that first item is populated with expected value, others are unpopulated
    fn validate_item_exact(&self, expected: i8, new_file_ver: bool) -> ValidationResult {
        let expected = ResourceId::map_id_to_version(expected, new_file_ver) as i16;
        if self.first_item != expected {
            return Err(ValidationError::expected_exactly("first_item", self.first_item, expected));
        }

        if self.second_item >= 0 {
            return Err(ValidationError::expected_exactly("second_item", self.second_item, -1));
        }

        if self.third_item >= 0 {
            return Err(ValidationError::expected_exactly("third_item", self.third_item, -1));
        }

        return Ok(());
    }

    // Validate first item is populated with expected value, validate other times are expected or empty
    fn validate_items_in_range(&self, min: i16, max: i16) -> ValidationResult {
        if !(min..=max).contains(&self.first_item) {
            return Err(ValidationError::expected_range("first_item", self.first_item, min, max));
        }

        if self.second_item >= 0 && !(min..=max).contains(&self.second_item) {
            return Err(ValidationError::expected_range("second_item", self.second_item, min, max));
        }

        if self.third_item >= 0 && !(min..=max).contains(&self.third_item) {
            return Err(ValidationError::expected_range("third_item", self.third_item, min, max));
        }

        return Ok(());
    }

    // Validate first item is populated with expected value, validate other times are expected or empty
    fn validate_items_one_of(&self, expected: &[i8], new_file_ver: bool) -> ValidationResult {
        let expected = expected
            .iter()
            .map(|&i| ResourceId::map_id_to_version(i, new_file_ver) as i16)
            .filter(|&i| i >= 0)
            .collect::<Vec<_>>();

        if !expected.contains(&self.first_item) {
            return Err(ValidationError::expected_one_of("first_item", self.first_item, expected.as_ref()));
        }

        if self.second_item >= 0 && !expected.contains(&self.second_item) {
            return Err(ValidationError::expected_one_of("second_item", self.second_item, expected.as_ref()));
        }

        if self.third_item >= 0 && !expected.contains(&self.third_item) {
            return Err(ValidationError::expected_one_of("third_item", self.third_item, expected.as_ref()));
        }

        return Ok(());
    }

    fn validate_amount_range(&self) -> ValidationResult {
        return self.validate_amount_range_bounded(1, i16::MAX);
    }

    fn validate_amount_range_bounded(&self, min: i16, max: i16) -> ValidationResult {
        EventData::validate_non_negative_or_unset("fixed_amount", self.fixed_amount)?;
        EventData::validate_non_negative_or_unset("min_amount", self.min_amount)?;
        EventData::validate_non_negative_or_unset("max_amount", self.max_amount)?;

        if self.fixed_amount >= 0 {
            if !(min..=max).contains(&self.fixed_amount) {
                return Err(ValidationError::expected_range("fixed_amount", self.fixed_amount, min, max));
            }

            if self.min_amount >= 0 {
                return Err(ValidationError::expected_exactly("min_amount", self.min_amount, -1));
            }

            if self.max_amount >= 0 {
                return Err(ValidationError::expected_exactly("max_amount", self.max_amount, -1));
            }

            return Ok(());
        }

        if self.min_amount < 0 || self.max_amount < 0 {
            return Err(ValidationError::expected_range("fixed_amount", self.fixed_amount, min, max));
        }

        if self.max_amount < self.min_amount {
            return Err(ValidationError::expected_range("max_amount", self.max_amount, self.min_amount, max));
        }

        if !(min..=max).contains(&self.min_amount) {
            return Err(ValidationError::expected_range("min_amount", self.min_amount, min, max));
        }

        return Ok(());
    }

    fn validate_non_negative_or_unset(name: &'static str, value: i16) -> ValidationResult {
        if value < -1 {
            return Err(ValidationError::expected_range(name, value, -1, i16::MAX));
        }

        return Ok(());
    }

    fn validate_non_negative(name: &'static str, value: i16) -> ValidationResult {
        if value < 0 {
            return Err(ValidationError::expected_range(name, value, 0, i16::MAX));
        }

        return Ok(());
    }
}

impl Default for EventData {
    // Several fields use `u16::MAX`, not `0`, as their "unused" sentinel.
    fn default() -> EventData {
        return EventData {
            id: 0,
            event_type: 0,
            month: 0,
            item_chosen: 0,
            first_item: -1,
            second_item: -1,
            third_item: -1,
            amount: 0,
            fixed_amount: -1,
            min_amount: -1,
            max_amount: -1,
            time: 0,
            fixed_time: -1,
            min_time: -1,
            max_time: -1,
            target: 0,
            fixed_target: -1,
            min_target: -1,
            max_target: -1,
            on_success: -1,
            on_failure: u16::MAX,
            flags: 0,
            warnings: 0,
            time_ctr: 0,
            status: 0,
            need_msg_res: 0,
            triggerer: u16::MAX,
            god_or_mon_or_warship_id: 0,
            mtar1: 0,
            mtar2: 0,
            mtar3: 0,
            magg: 0,
            unkown_row: [0; 9],
            trigger_on_1: u16::MAX,
            trigger_on_2: u16::MAX,
            eff_on_city: 0,
            source: 0,
            source_fixed: -1,
            source_min: -1,
            source_max: -1,
            subtype: 0,
            prev_amount: 0,
            related_to_triggered_evt: 0,
            constant_1_0x00: 0,
            trig_reason: 0,
            constant_2_0x00: 0,
            constant_3_0x00: 0,
            other_city: 0,
            loot_type: 0,
            loot_amount: 0,
            constant_4_0x00: 0,
            ally_city: 0,
            ally_strength: 0,
            to_strength: 0,
            unknown_1: 0,
            quest: 0,
            tail: 0,
        };
    }
}

impl ReadFrom for EventData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(EventData {
            id: ReadFrom::read_from(reader)?,
            event_type: ReadFrom::read_from(reader)?,
            month: ReadFrom::read_from(reader)?,
            item_chosen: ReadFrom::read_from(reader)?,
            first_item: ReadFrom::read_from(reader)?,
            second_item: ReadFrom::read_from(reader)?,
            third_item: ReadFrom::read_from(reader)?,
            amount: ReadFrom::read_from(reader)?,
            fixed_amount: ReadFrom::read_from(reader)?,
            min_amount: ReadFrom::read_from(reader)?,
            max_amount: ReadFrom::read_from(reader)?,
            time: ReadFrom::read_from(reader)?,
            fixed_time: ReadFrom::read_from(reader)?,
            min_time: ReadFrom::read_from(reader)?,
            max_time: ReadFrom::read_from(reader)?,
            target: ReadFrom::read_from(reader)?,
            fixed_target: ReadFrom::read_from(reader)?,
            min_target: ReadFrom::read_from(reader)?,
            max_target: ReadFrom::read_from(reader)?,
            on_success: ReadFrom::read_from(reader)?,
            on_failure: ReadFrom::read_from(reader)?,
            flags: ReadFrom::read_from(reader)?,
            warnings: ReadFrom::read_from(reader)?,
            time_ctr: ReadFrom::read_from(reader)?,
            status: ReadFrom::read_from(reader)?,
            need_msg_res: ReadFrom::read_from(reader)?,
            triggerer: ReadFrom::read_from(reader)?,
            god_or_mon_or_warship_id: ReadFrom::read_from(reader)?,
            mtar1: ReadFrom::read_from(reader)?,
            mtar2: ReadFrom::read_from(reader)?,
            mtar3: ReadFrom::read_from(reader)?,
            magg: ReadFrom::read_from(reader)?,
            unkown_row: ReadFrom::read_from(reader)?,
            trigger_on_1: ReadFrom::read_from(reader)?,
            trigger_on_2: ReadFrom::read_from(reader)?,
            eff_on_city: ReadFrom::read_from(reader)?,
            source: ReadFrom::read_from(reader)?,
            source_fixed: ReadFrom::read_from(reader)?,
            source_min: ReadFrom::read_from(reader)?,
            source_max: ReadFrom::read_from(reader)?,
            subtype: ReadFrom::read_from(reader)?,
            prev_amount: ReadFrom::read_from(reader)?,
            related_to_triggered_evt: ReadFrom::read_from(reader)?,
            constant_1_0x00: ReadFrom::read_from(reader)?,
            trig_reason: ReadFrom::read_from(reader)?,
            constant_2_0x00: ReadFrom::read_from(reader)?,
            constant_3_0x00: ReadFrom::read_from(reader)?,
            other_city: ReadFrom::read_from(reader)?,
            loot_type: ReadFrom::read_from(reader)?,
            loot_amount: ReadFrom::read_from(reader)?,
            constant_4_0x00: ReadFrom::read_from(reader)?,
            ally_city: ReadFrom::read_from(reader)?,
            ally_strength: ReadFrom::read_from(reader)?,
            to_strength: ReadFrom::read_from(reader)?,
            unknown_1: ReadFrom::read_from(reader)?,
            quest: ReadFrom::read_from(reader)?,
            tail: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for EventData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.id, writer)?;
        bytes += WriteTo::write_to(&self.event_type, writer)?;
        bytes += WriteTo::write_to(&self.month, writer)?;
        bytes += WriteTo::write_to(&self.item_chosen, writer)?;
        bytes += WriteTo::write_to(&self.first_item, writer)?;
        bytes += WriteTo::write_to(&self.second_item, writer)?;
        bytes += WriteTo::write_to(&self.third_item, writer)?;
        bytes += WriteTo::write_to(&self.amount, writer)?;
        bytes += WriteTo::write_to(&self.fixed_amount, writer)?;
        bytes += WriteTo::write_to(&self.min_amount, writer)?;
        bytes += WriteTo::write_to(&self.max_amount, writer)?;
        bytes += WriteTo::write_to(&self.time, writer)?;
        bytes += WriteTo::write_to(&self.fixed_time, writer)?;
        bytes += WriteTo::write_to(&self.min_time, writer)?;
        bytes += WriteTo::write_to(&self.max_time, writer)?;
        bytes += WriteTo::write_to(&self.target, writer)?;
        bytes += WriteTo::write_to(&self.fixed_target, writer)?;
        bytes += WriteTo::write_to(&self.min_target, writer)?;
        bytes += WriteTo::write_to(&self.max_target, writer)?;
        bytes += WriteTo::write_to(&self.on_success, writer)?;
        bytes += WriteTo::write_to(&self.on_failure, writer)?;
        bytes += WriteTo::write_to(&self.flags, writer)?;
        bytes += WriteTo::write_to(&self.warnings, writer)?;
        bytes += WriteTo::write_to(&self.time_ctr, writer)?;
        bytes += WriteTo::write_to(&self.status, writer)?;
        bytes += WriteTo::write_to(&self.need_msg_res, writer)?;
        bytes += WriteTo::write_to(&self.triggerer, writer)?;
        bytes += WriteTo::write_to(&self.god_or_mon_or_warship_id, writer)?;
        bytes += WriteTo::write_to(&self.mtar1, writer)?;
        bytes += WriteTo::write_to(&self.mtar2, writer)?;
        bytes += WriteTo::write_to(&self.mtar3, writer)?;
        bytes += WriteTo::write_to(&self.magg, writer)?;
        bytes += WriteTo::write_to(&self.unkown_row, writer)?;
        bytes += WriteTo::write_to(&self.trigger_on_1, writer)?;
        bytes += WriteTo::write_to(&self.trigger_on_2, writer)?;
        bytes += WriteTo::write_to(&self.eff_on_city, writer)?;
        bytes += WriteTo::write_to(&self.source, writer)?;
        bytes += WriteTo::write_to(&self.source_fixed, writer)?;
        bytes += WriteTo::write_to(&self.source_min, writer)?;
        bytes += WriteTo::write_to(&self.source_max, writer)?;
        bytes += WriteTo::write_to(&self.subtype, writer)?;
        bytes += WriteTo::write_to(&self.prev_amount, writer)?;
        bytes += WriteTo::write_to(&self.related_to_triggered_evt, writer)?;
        bytes += WriteTo::write_to(&self.constant_1_0x00, writer)?;
        bytes += WriteTo::write_to(&self.trig_reason, writer)?;
        bytes += WriteTo::write_to(&self.constant_2_0x00, writer)?;
        bytes += WriteTo::write_to(&self.constant_3_0x00, writer)?;
        bytes += WriteTo::write_to(&self.other_city, writer)?;
        bytes += WriteTo::write_to(&self.loot_type, writer)?;
        bytes += WriteTo::write_to(&self.loot_amount, writer)?;
        bytes += WriteTo::write_to(&self.constant_4_0x00, writer)?;
        bytes += WriteTo::write_to(&self.ally_city, writer)?;
        bytes += WriteTo::write_to(&self.ally_strength, writer)?;
        bytes += WriteTo::write_to(&self.to_strength, writer)?;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.quest, writer)?;
        bytes += WriteTo::write_to(&self.tail, writer)?;

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write() -> io::Result<()> {
        let original = EventData {
            id: 1,
            event_type: 2,
            month: 3,
            item_chosen: 4,
            first_item: 5,
            second_item: 6,
            third_item: 7,
            amount: 8,
            fixed_amount: 9,
            min_amount: 10,
            max_amount: 11,
            time: 12,
            fixed_time: 13,
            min_time: 14,
            max_time: 15,
            target: 16,
            fixed_target: 17,
            min_target: 18,
            max_target: 19,
            on_success: 20,
            on_failure: 21,
            flags: 22,
            warnings: 23,
            time_ctr: 24,
            status: 25,
            need_msg_res: 26,
            triggerer: 27,
            god_or_mon_or_warship_id: 28,
            mtar1: 29,
            mtar2: 30,
            mtar3: 31,
            magg: 32,
            unkown_row: [33, 34, 35, 36, 37, 38, 39, 40, 41],
            trigger_on_1: 42,
            trigger_on_2: 43,
            eff_on_city: 44,
            source: 45,
            source_fixed: 46,
            source_min: 47,
            source_max: 48,
            subtype: 49,
            prev_amount: 50,
            related_to_triggered_evt: 51,
            constant_1_0x00: 52,
            trig_reason: 53,
            constant_2_0x00: 54,
            constant_3_0x00: 55,
            other_city: 56,
            loot_type: 57,
            loot_amount: 58,
            constant_4_0x00: 59,
            ally_city: 60,
            ally_strength: 61,
            to_strength: 62,
            unknown_1: 63,
            quest: 64,
            tail: 65,
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = EventData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }
}
