use regex::Regex;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::str::FromStr;

#[derive(Default)]
pub struct BuildingModels {
    pub building_models: Vec<BuildingModel>,
    pub house_models: Vec<HouseModel>,
}

#[derive(Default)]
pub struct BuildingModel {
    pub id: u8,
    pub label: String,
    pub cost: u16,
    pub desirability: i8,
    pub desirability_step: u8,
    pub desirability_step_size: i8,
    pub max_desirability_range: u8,
    pub employee_count: u8,
    pub fire_risk_increment: u16,
    pub damage_risk_increment: u16,
    pub resource_used: u8,
    pub risk_reducer: u8,
    pub comment: String,
}

#[derive(Default)]
pub struct HouseModel {
    pub label: String,
    pub desirability_devolve: i8,
    pub desirability_evolve: i8,
    pub culture_evolve: u8,
    pub water_evolve: u8,
    pub stadium_evolve: u8,
    pub soldiers_percentage: u8,
    pub horses_max: u8,
    pub horses_evolve: u8,
    pub food_evolve: u8,
    pub fleece_evolve: u8,
    pub olive_oil_evolve: u8,
    pub wine_evolve: u8,
    pub armor_evolve: u8,
    pub armor_max: u8,
    pub crime_risk_inc: i8,
    pub crime_risk_base: u8,
    pub _unused_17: u8,
    pub population_capacity: u8,
    pub tax_rate_multiplier: u8,
    pub _unused_20: u8,
    pub disease_risk_inc: i8,
}

impl BuildingModels {
    pub fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        let building_model_pattern =
            Regex::new(r"^(\d+),(.+),\{,(\d+),(-?\d+),(\d+),(-?\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),},([a-zA-Z ]*),").unwrap();

        let house_model_pattern = Regex::new(concat!(
            r"^(.+),\{,(-?\d+),(-?\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(\d+),",
            r"(-?\d+),(\d+),(\d+),(\d+),(\d+),(\d+),(-?\d+),",
        ))
        .unwrap();

        let mut building_models = Vec::new();
        let mut house_models = Vec::new();

        for line in BufReader::new(reader).lines() {
            let line_str = line?;
            if let Some(captures) = building_model_pattern.captures(line_str.as_str()) {
                building_models.push(BuildingModel {
                    id: parse_field(&captures[1])?,
                    label: captures[2].to_string(),
                    cost: parse_field(&captures[3])?,
                    desirability: parse_field(&captures[4])?,
                    desirability_step: parse_field(&captures[5])?,
                    desirability_step_size: parse_field(&captures[6])?,
                    max_desirability_range: parse_field(&captures[7])?,
                    employee_count: parse_field(&captures[8])?,
                    fire_risk_increment: parse_field(&captures[9])?,
                    damage_risk_increment: parse_field(&captures[10])?,
                    resource_used: parse_field(&captures[11])?,
                    risk_reducer: parse_field(&captures[12])?,
                    comment: captures[13].to_string(),
                });
            } else if let Some(captures) = house_model_pattern.captures(line_str.as_str()) {
                house_models.push(HouseModel {
                    label: captures[1].to_string(),
                    desirability_devolve: parse_field(&captures[2])?,
                    desirability_evolve: parse_field(&captures[3])?,
                    culture_evolve: parse_field(&captures[4])?,
                    water_evolve: parse_field(&captures[5])?,
                    stadium_evolve: parse_field(&captures[6])?,
                    soldiers_percentage: parse_field(&captures[7])?,
                    horses_max: parse_field(&captures[8])?,
                    horses_evolve: parse_field(&captures[9])?,
                    food_evolve: parse_field(&captures[10])?,
                    fleece_evolve: parse_field(&captures[11])?,
                    olive_oil_evolve: parse_field(&captures[12])?,
                    wine_evolve: parse_field(&captures[13])?,
                    armor_evolve: parse_field(&captures[14])?,
                    armor_max: parse_field(&captures[15])?,
                    crime_risk_inc: parse_field(&captures[16])?,
                    crime_risk_base: parse_field(&captures[17])?,
                    _unused_17: parse_field(&captures[18])?,
                    population_capacity: parse_field(&captures[19])?,
                    tax_rate_multiplier: parse_field(&captures[20])?,
                    _unused_20: parse_field(&captures[21])?,
                    disease_risk_inc: parse_field(&captures[22])?,
                });
            }
        }

        building_models.shrink_to_fit();
        house_models.shrink_to_fit();

        return Ok(BuildingModels {
            building_models,
            house_models,
        });
    }
}

fn parse_field<T: FromStr>(field: &str) -> io::Result<T>
where
    T::Err: std::fmt::Display,
{
    return field
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("failed to parse '{}': {}", field, e)));
}

#[cfg(test)]
mod tests {
    use crate::model::building_model::BuildingModels;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;

    #[test]
    fn test_odyssey() -> Result<()> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not set");

        let mut reader = File::open(format!("{}/Model/Zeus_Model_VeryEasy.txt", game_root)).map(BufReader::new)?;
        let building_models = BuildingModels::read_from(&mut reader)?;

        assert_eq!(building_models.building_models.len(), 221);
        assert_eq!(building_models.building_models[16].label, "BUILD_GAZEBO");
        assert_eq!(building_models.building_models[106].comment, "Grand Monument to the Sky");

        assert_eq!(building_models.house_models.len(), 13);
        assert_eq!(building_models.house_models[12].label, "Elite 4: Estate");
        assert_eq!(building_models.house_models[12].tax_rate_multiplier, 22);

        return Ok(());
    }
}
