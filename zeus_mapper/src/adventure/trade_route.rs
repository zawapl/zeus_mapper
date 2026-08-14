use crate::prelude::BoxedArray;
use crate::prelude::TradeRouteData;
use crate::prelude::TradeRoutePointData;
use my_macros::LogDifferences;

#[derive(LogDifferences, PartialEq, Debug)]
pub struct TradeRoute {
    pub slot: usize,
    pub destination: u16,
    pub route_number: u16,
    pub distance: u32,
    pub route_type: u8,
    pub points: Vec<(u16, u16)>,
}

impl TradeRoute {
    fn from_data(slot: usize, route: &TradeRouteData) -> TradeRoute {
        let distance = u32::from_le_bytes([route.distance[0], route.distance[1], route.distance[2], route.distance[3]]);
        let destination = u16::from_le_bytes([route.distance[8], route.distance[9]]);
        let route_number = u16::from_le_bytes([route.distance[10], route.distance[11]]);

        let points = route
            .points
            .iter()
            .take(route.points_count as usize)
            .map(|point| (point.x, point.y))
            .collect();

        return TradeRoute {
            slot,
            destination,
            route_number,
            distance,
            route_type: route.route_type,
            points,
        };
    }

    fn to_data(&self) -> TradeRouteData {
        let mut distance = [0u8; 12];
        distance[0..4].copy_from_slice(&self.distance.to_le_bytes());
        distance[8..10].copy_from_slice(&self.destination.to_le_bytes());
        distance[10..12].copy_from_slice(&self.route_number.to_le_bytes());

        let points: Vec<TradeRoutePointData> = self
            .points
            .iter()
            .take(50)
            .map(|&(x, y)| TradeRoutePointData { x, y, unknown_1: 0 })
            .collect();

        return TradeRouteData {
            constant_1_0x05: 5,
            points: BoxedArray::from_vec(points),
            distance,
            route_type: self.route_type,
            points_count: self.points.len().min(50) as u8,
            exists: 1,
            constant_2_0x00: 0,
        };
    }

    pub(crate) fn vec_from_data(routes: &BoxedArray<TradeRouteData, 232>) -> Vec<TradeRoute> {
        return routes
            .iter()
            .enumerate()
            .filter(|(_, route)| route.exists != 0)
            .map(|(slot, route)| TradeRoute::from_data(slot, route))
            .collect();
    }

    pub(crate) fn vec_to_data(routes: &[TradeRoute]) -> BoxedArray<TradeRouteData, 232> {
        let mut data = vec![TradeRouteData::default(); 232];

        for route in routes {
            if route.slot < data.len() {
                data[route.slot] = route.to_data();
            }
        }

        return BoxedArray::from_vec(data);
    }
}
