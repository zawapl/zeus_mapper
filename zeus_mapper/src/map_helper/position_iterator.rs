use crate::prelude::MAX_MAP_SIZE;
use crate::prelude::MapPosition;

pub fn for_each_valid_position<F: FnMut(MapPosition)>(map_size: u16, mut apply: F) {
    let max_map_size = MAX_MAP_SIZE as u16;
    let y_start = (max_map_size - map_size) / 2;
    let middle = max_map_size / 2;

    for y in y_start..(max_map_size - y_start) {
        let x_start = if y < middle { middle - y - 1 } else { y - middle };

        for x in (y_start + x_start)..(max_map_size - y_start - x_start) {
            apply(MapPosition::from_x_y(x, y));
        }
    }
}

pub fn for_each_position<F: FnMut(MapPosition)>(mut apply: F) {
    for i in 0..51984 {
        apply(MapPosition::from_index(i));
    }
}
