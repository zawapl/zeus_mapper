use crate::prelude::MAX_MAP_SIZE;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MapPosition {
    x: u16,
    y: u16,
    index: usize,
}

impl MapPosition {
    pub fn from_x_y(x: u16, y: u16) -> MapPosition {
        let max = MAX_MAP_SIZE as u16;

        debug_assert!(x < max && y < max);

        let index = x as usize + y as usize * MAX_MAP_SIZE as usize;

        return MapPosition { x, y, index };
    }

    pub fn from_x_y_untrimmed(x: u16, y: u16) -> MapPosition {
        let x = x % MAX_MAP_SIZE as u16;
        let y = y % MAX_MAP_SIZE as u16;

        let index = x as usize + y as usize * MAX_MAP_SIZE as usize;

        return MapPosition { x, y, index };
    }

    pub fn from_index(index: usize) -> MapPosition {
        let x = (index % MAX_MAP_SIZE as usize) as u16;
        let y = (index / MAX_MAP_SIZE as usize) as u16;

        let max = MAX_MAP_SIZE as u16;

        debug_assert!(x < max && y < max);

        return MapPosition { x, y, index };
    }

    pub fn x(&self) -> u16 {
        return self.x;
    }

    pub fn y(&self) -> u16 {
        return self.y;
    }

    pub fn index(&self) -> usize {
        return self.index;
    }

    pub fn north(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(wrapping_dec(self.x()), wrapping_dec(self.y()));
    }

    pub fn north_east(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(self.x(), wrapping_dec(self.y()));
    }

    pub fn east(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(wrapping_inc(self.x()), wrapping_dec(self.y()));
    }

    pub fn south_east(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(wrapping_inc(self.x()), self.y());
    }

    pub fn south(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(wrapping_inc(self.x()), wrapping_inc(self.y()));
    }

    pub fn south_west(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(self.x(), wrapping_inc(self.y()));
    }

    pub fn west(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(wrapping_dec(self.x()), wrapping_inc(self.y()));
    }

    pub fn north_west(&self) -> MapPosition {
        return MapPosition::from_x_y_untrimmed(wrapping_dec(self.x()), self.y());
    }
}

fn wrapping_inc(val: u16) -> u16 {
    return val.wrapping_add(1);
}

fn wrapping_dec(val: u16) -> u16 {
    return val.wrapping_sub(1);
}
