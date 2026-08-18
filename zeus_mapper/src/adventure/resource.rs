use crate::constants::data_constant::DataConstant;
use crate::differ::default_differ_impl;
pub use crate::file_data::resource_id::ResourceId as ResourceType;

impl ResourceType {
    pub(crate) fn vec_from_data<const N: usize>(data: &[i8; N], new_file_ver: bool) -> Vec<ResourceType> {
        return data
            .iter()
            .filter(|&&id| id != 0)
            .filter_map(|id| ResourceType::try_resolve_for_format(id, new_file_ver))
            .collect();
    }

    pub(crate) fn vec_to_data<const N: usize>(resources: &[ResourceType]) -> [i8; N] {
        let mut data = [0i8; N];

        for (i, resource) in resources.iter().take(data.len()).enumerate() {
            data[i] = resource.value();
        }

        return data;
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
            if let Some(resource) = ResourceType::try_resolve_for_format(&(id as i8), new_file_ver) {
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
    pub(crate) fn vec_from_data(sold: &[i8], bought: &[i8], quantity: &[u8], new_file_ver: bool) -> Vec<TradedGoods> {
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
    pub(crate) fn vec_to_data<const N: usize, const Q: usize>(goods: &[TradedGoods]) -> ([i8; N], [i8; N], [u8; Q]) {
        let mut sold = [0i8; N];
        let mut bought = [0i8; N];
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
