use crate::constants::data_constant::DataConstant;
use crate::constants::data_constant::data_constants;
use crate::differ::default_differ_impl;

data_constants!(ResourceType<u8>{
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
});

impl ResourceType {
    pub(crate) fn vec_from_data<const N: usize>(data: &[u8; N], new_file_ver: bool) -> Vec<ResourceType> {
        return data
            .iter()
            .filter(|&&id| id != 0)
            .filter_map(|id| ResourceType::try_resolve_for_format(id, new_file_ver))
            .collect();
    }

    pub(crate) fn vec_to_data<const N: usize>(resources: &[ResourceType]) -> [u8; N] {
        let mut data = [0u8; N];

        for (i, resource) in resources.iter().take(data.len()).enumerate() {
            data[i] = resource.value();
        }

        return data;
    }

    /// Resolves a raw resource id, accounting for old-format (`new_file_ver == false`) adventures'
    /// narrower resource list.
    ///
    /// Always writes the new file format - only reading needs to support the old format.
    pub(crate) fn try_resolve_for_format(id: &u8, new_file_ver: bool) -> Option<ResourceType> {
        if new_file_ver {
            return ResourceType::try_resolve(id);
        }

        return ResourceType::try_resolve_old_format(id);
    }

    /// Old-format (`new_file_ver == false`) adventures don't know about `Orange`, `BlackMarble`,
    /// `Orichalc`, or `Chariot` - their resource ids are a step short of the new format's for every
    /// id from the first gap onward, so raw ids `1..=19` resolve to a *different* `ResourceType`
    /// than [`ResourceType::try_resolve`] would give the same byte in a new-format file.
    ///
    /// Confirmed against `The Odyssey`: cross-referencing `WorldLocationData` tribute/trade
    /// resource ids, `MapData.prices`, and `RealEpisodeData.city_resources` against in-game values
    /// showed every raw id from `8` onward reading one `ResourceType` too "early" (e.g. raw `13`
    /// read as `Olive` but meaning `Fleece`) - consistent with `Orange`/`BlackMarble`/`Orichalc`/
    /// `Chariot` simply not existing in the id sequence at all for these adventures, rather than a
    /// constant offset.
    fn try_resolve_old_format(id: &u8) -> Option<ResourceType> {
        return match id {
            1..=7 => ResourceType::try_resolve(id),
            8..=14 => ResourceType::try_resolve(&(id + 1)),
            15..=18 => ResourceType::try_resolve(&(id + 3)),
            19 => ResourceType::try_resolve(&(id + 4)),
            _ => None,
        };
    }

    /// Builds `Adventure.prices` (indexed by [`ResourceType::value`]) from `MapData.prices`.
    ///
    /// Old-format (`new_file_ver == false`) adventures index this array by the same narrower,
    /// gap-free id sequence [`ResourceType::try_resolve_for_format`] resolves - so the raw array is
    /// re-indexed onto the new-format id space here, leaving `Orange`/`BlackMarble`/`Orichalc`/
    /// `Chariot`'s slots at `0` (never populated in an old-format file).
    pub(crate) fn prices_from_data(prices: &[u32], new_file_ver: bool) -> Vec<u32> {
        if new_file_ver {
            return prices.to_vec();
        }

        let mut result = vec![0; prices.len()];
        for (id, &price) in prices.iter().enumerate() {
            if let Some(resource) = ResourceType::try_resolve_old_format(&(id as u8)) {
                let index = resource.value() as usize;
                if index < result.len() {
                    result[index] = price;
                }
            }
        }

        return result;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradedGoods {
    Buying(ResourceType, u8),
    Selling(ResourceType, u8),
}

default_differ_impl!(TradedGoods);

impl TradedGoods {
    /// Builds the sell-then-buy trade list from `RealEpisodeData.city_resources_sold`/
    /// `city_resources_bought` (plain resource-id lists) and `city_resources_quantity`, which is
    /// indexed *by resource id* (not by position in `sold`/`bought`) and holds the raw tier
    /// (`12`/`24`/`36`, i.e. low/medium/high) directly as the amount.
    pub(crate) fn vec_from_data(sold: &[u8], bought: &[u8], quantity: &[u8], new_file_ver: bool) -> Vec<TradedGoods> {
        let mut result = vec![];

        for &id in sold.iter().filter(|&&id| id != 0) {
            if let Some(resource_id) = ResourceType::try_resolve_for_format(&id, new_file_ver) {
                result.push(TradedGoods::Selling(resource_id, quantity.get(id as usize).copied().unwrap_or(0)));
            }
        }
        for &id in bought.iter().filter(|&&id| id != 0) {
            if let Some(resource_id) = ResourceType::try_resolve_for_format(&id, new_file_ver) {
                result.push(TradedGoods::Buying(resource_id, quantity.get(id as usize).copied().unwrap_or(0)));
            }
        }

        return result;
    }

    /// Inverse of [`TradedGoods::vec_from_data`]: splits `goods` back into sold/bought resource-id
    /// lists (capped at `N` entries each, extras dropped) and a quantity table indexed by resource
    /// id.
    pub(crate) fn vec_to_data<const N: usize, const Q: usize>(goods: &[TradedGoods]) -> ([u8; N], [u8; N], [u8; Q]) {
        let mut sold = [0u8; N];
        let mut bought = [0u8; N];
        let mut quantity = [0u8; Q];
        let mut sold_len = 0;
        let mut bought_len = 0;

        for good in goods {
            match good {
                TradedGoods::Buying(resource, amount) if bought_len < bought.len() => {
                    let id = resource.value();
                    bought[bought_len] = id;
                    bought_len += 1;
                    quantity[id as usize] = *amount;
                }
                TradedGoods::Selling(resource, amount) if sold_len < sold.len() => {
                    let id = resource.value();
                    sold[sold_len] = id;
                    sold_len += 1;
                    quantity[id as usize] = *amount;
                }
                _ => {}
            }
        }

        return (sold, bought, quantity);
    }
}
