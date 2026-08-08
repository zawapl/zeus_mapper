use crate::map_helper::map_accessor::MapAccessor;
use crate::map_helper::map_accessor::MapPosition;
use crate::prelude::DataConstant;
use crate::prelude::TerrainTypeFlag;
use crate::prelude::TerrainTypeFlag::*;
use crate::utils::TILE_MAP_IMAGE_INDEX_MASK;

const UNKNOWN_TERRAIN_TEXTURE: u32 = 201;

const T: bool = true;

pub fn convert_to_texture_id(sprite: u32) -> u32 {
    sprite & TILE_MAP_IMAGE_INDEX_MASK
}

// Check if texture is from Poseidon_Loaded.sg3 instead of Zeus_Terrain.sg3
pub fn is_poseidon_texture(sprite: u32) -> bool {
    return sprite > 0x1ffff;
}

pub fn calculate_texture_id<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, is_tropical: bool) -> u32 {
    let disasters = Disaster | Flood | Landslide;
    let original_terrain = map_accessor.terrain_flags_at(pos).0;
    let terrain = original_terrain & !disasters.0;

    if terrain == 0 {
        return calculate_empty(map_accessor, pos) - UNKNOWN_TERRAIN_TEXTURE;
    }

    let leading_terrain_type = 31 - terrain.leading_zeros();
    if let Some(terrain_type_flag) = TerrainTypeFlag::try_resolve(&leading_terrain_type) {
        return match terrain_type_flag {
            Forest => calculate_forest(map_accessor, pos, is_tropical),
            Rock => calculate_rock(map_accessor, pos, 534),
            WaterShallow => calculate_water_shallow(map_accessor, pos),
            Road => calculate_road(map_accessor, pos),
            Scrub => calculate_scrub(map_accessor, pos, is_tropical),
            Slope => calculate_slope(map_accessor, pos),
            AccessRamp => calculate_access_ramp(map_accessor, pos),
            Meadow => calculate_meadow(map_accessor, pos),
            Sand => calculate_sand(map_accessor, pos),
            Marble => calculate_marble(map_accessor, pos, 1531),
            Marshland => calculate_marshland(map_accessor, pos),
            Copper => calculate_rock(map_accessor, pos, 548),
            Silver => calculate_rock(map_accessor, pos, 563),
            HighSlope => calculate_high_slope(map_accessor, pos),
            HalfSlope => calculate_half_slope(map_accessor, pos),
            Earthquake => calculate_earthquake(map_accessor, pos),
            WaterDeep => calculate_water_deep(map_accessor, pos),
            Corner => calculate_slope_corner(map_accessor, pos),
            _ => UNKNOWN_TERRAIN_TEXTURE,
        } - UNKNOWN_TERRAIN_TEXTURE;
    }
    panic!("Should not reach here");
}

fn calculate_empty<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let random = map_accessor.random_at(pos) as u32;
    let tile_size = map_accessor.tile_size_at(pos);

    if tile_size != 0 {
        let surrounded_by_water = all_true(test_neighbours_8(pos, |p| map_accessor.terrain_flags_at(&p).is(WaterShallow)));

        return if !surrounded_by_water {
            476 + (random % 8)
        } else {
            487 + (random % 8)
        };
    }

    return 306 + (random % 8);
}

fn calculate_forest<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, is_tropical: bool) -> u32 {
    let random = map_accessor.random_at(pos) as u32;
    let scrub = map_accessor.scrub_at(pos);

    let is_forest = |pos| {
        let terrain_type = map_accessor.terrain_flags_at(&pos);
        return terrain_type.is_any_of(Forest | Rock | Copper);
    };

    let scrub_offset = if scrub < 34 {
        0
    } else if scrub < 68 {
        32
    } else {
        64
    };

    let tropical_offset = if is_tropical { 1179 - 1060 } else { 0 };

    return if all_true(test_neighbours_24(pos, is_forest)) {
        1203 - tropical_offset + scrub_offset + (random % 8)
    } else if all_true(test_neighbours_16(pos, is_forest)) {
        1195 - tropical_offset + scrub_offset + (random % 8)
    } else if all_true(test_neighbours_8(pos, is_forest)) {
        1187 - tropical_offset + scrub_offset + (random % 8)
    } else {
        1179 - tropical_offset + scrub_offset + (random % 8)
    };
}

fn calculate_rock<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, offset: u32) -> u32 {
    if map_accessor.terrain_flags_at(pos).is(Slope) {
        return calculate_slope(map_accessor, pos);
    }

    if map_accessor.terrain_flags_at(pos).is(Marble) {
        return calculate_marble(map_accessor, pos, 710);
    }

    let offset = if map_accessor.terrain_flags_at(&pos).is_all_of(Silver | Copper) {
        if map_accessor.terrain_flags_at(pos).is(Rock) { 577 } else { 864 }
    } else {
        offset
    };

    let tile_size = map_accessor.tile_size_at(pos) & 0xf;

    return if tile_size == 0 {
        offset + (map_accessor.random_at(pos) as u32 % 8)
    } else {
        let root_offset = map_accessor.root_offset_at(pos) as u16;
        let dx = root_offset & 0b111;
        let dy = (root_offset >> 3) & 0b111;

        let source_pos = MapPosition::from_x_y_untrimmed(pos.x() - dx, pos.y() - dy);

        if tile_size == 1 {
            // 2x2
            offset + 8 + (map_accessor.random_at(&source_pos) as u32) % 4
        } else {
            // 3x3
            offset + 12 + (map_accessor.random_at(&source_pos) as u32) % 2
        }
    };
}

fn calculate_water_shallow<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let [s_n, s_ne, s_e, s_se, s_s, s_sw, s_w, s_nw] = test_neighbours_8(pos, |p| {
        return map_accessor.terrain_flags_at(&p).is_any_of(Sand);
    });

    let any_sand = s_n || s_ne || s_e || s_se || s_s || s_sw || s_w || s_nw;

    let [ws_ne, ws_se, ws_sw, ws_nw] = test_neighbours_4(pos, |p| {
        map_accessor.terrain_flags_at(&p).is_any_of(Sand | WaterDeep | WaterShallow)
    });

    let all_sand = ws_ne && ws_se && ws_sw && ws_nw;

    if any_sand && all_sand {
        return calculate_water_edge(map_accessor, pos, 663, WaterShallow);
    } else if !any_sand {
        return calculate_water_edge(map_accessor, pos, 364, WaterShallow);
    };

    let [w_ne, w_se, w_sw, w_nw] = test_neighbours_4(pos, |p| {
        return map_accessor.terrain_flags_at(&p).is_any_of(WaterShallow);
    });

    if w_nw && w_se && s_s {
        return 869;
    } else if w_nw && w_se && s_w {
        return 870;
    } else if w_ne && w_sw && s_w {
        return 875;
    } else if w_ne && w_sw && s_n {
        return 876;
    } else if w_se && w_nw && s_n {
        return 881;
    } else if w_se && w_nw && s_e {
        return 882;
    } else if w_ne && w_sw && s_e {
        return 887;
    } else if w_ne && w_sw && s_s {
        return 888;
    } else if w_ne && w_nw && s_se {
        return 873;
    } else if w_ne && w_se && s_nw {
        return 874;
    } else if w_ne && w_se && s_sw {
        return 879;
    } else if w_se && w_sw && s_ne {
        return 880;
    } else if w_se && w_sw && s_nw {
        return 885;
    } else if w_sw && w_nw && s_se {
        return 886;
    } else if w_sw && w_nw && s_ne {
        return 891;
    } else if w_ne && w_nw && s_sw {
        return 892;
    } else if w_ne && w_se && s_s {
        return 871;
    } else if w_ne && w_nw && s_w {
        return 872;
    } else if w_se && w_sw && s_w {
        return 877;
    } else if w_ne && w_se && s_n {
        return 878;
    } else if w_sw && w_nw && s_n {
        return 883;
    } else if w_se && w_sw && s_e {
        return 884;
    } else if w_ne && w_nw && s_e {
        return 889;
    } else if w_sw && w_nw && s_s {
        return 890;
    }

    return UNKNOWN_TERRAIN_TEXTURE;
}

fn calculate_road<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let [ne, se, sw, nw] = test_neighbours_4(pos, |p| map_accessor.terrain_flags_at(&p).is(Road));

    return match [ne, se, sw, nw] {
        [T, T, T, T] => 914,
        [T, T, T, _] => 910,
        [_, T, T, T] => 911,
        [T, _, T, T] => 912,
        [T, T, _, T] => 913,
        [_, T, _, T] => 898,
        [T, _, T, _] => 897,
        [T, T, _, _] => 901,
        [_, T, T, _] => 902,
        [_, _, T, T] => 903,
        [T, _, _, T] => 904,
        [T, _, _, _] => 905,
        [_, T, _, _] => 906,
        [_, _, T, _] => 907,
        [_, _, _, T] => 908,
        _ => 909,
    };
}

fn calculate_scrub<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, is_tropical: bool) -> u32 {
    if map_accessor.terrain_flags_at(pos).is(Forest) {
        return calculate_forest(map_accessor, pos, is_tropical);
    }

    if map_accessor.terrain_flags_at(pos).is(Road) {
        return calculate_road(map_accessor, pos);
    }

    if map_accessor.terrain_flags_at(pos).is(WaterShallow) {
        return calculate_water_shallow(map_accessor, pos);
    }

    if map_accessor.terrain_flags_at(pos).is(Rock) {
        return calculate_rock(map_accessor, pos, 534);
    }

    let scrub = map_accessor.scrub_at(pos);

    let [n, ne, e, se, s, sw, w, nw] = test_neighbours_8(pos, |p| {
        return map_accessor.terrain_flags_at(&p).is_any_of(Sand);
    });

    if ne && se {
        return 858;
    } else if se && sw {
        return 860;
    } else if sw && nw {
        return 862;
    } else if nw && ne {
        return 864;
    } else if ne {
        return 857;
    } else if se {
        return 859;
    } else if sw {
        return 861;
    } else if nw {
        return 863;
    } else if e {
        return 865;
    } else if s {
        return 866;
    } else if w {
        return 867;
    } else if n {
        return 868;
    }

    let length = scrub as u32 / 8;

    return if length < 11 {
        202 + length
    } else {
        let neighbour = test_neighbours_8(pos, |p| {
            return map_accessor
                .terrain_flags_at(&p)
                .is_any_of(Forest | Rock | WaterShallow | Road | Slope | Marble | Marshland);
        })
        .iter()
        .any(|x| *x);

        return if neighbour { 238 } else { 262 };
    };
}

fn calculate_slope<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let root_offset = map_accessor.root_offset_at(pos) as u16;
    let tile_size = map_accessor.tile_size_at(pos) & 0xf;

    if tile_size == 0 {
        if root_offset & 1 == 0 {
            let elevation_up = map_accessor.elevation_at(pos) + 1;

            let [ne, se, sw, nw] = test_neighbours_4(pos, |p| map_accessor.elevation_at(&p) == elevation_up);

            if nw {
                return choose4(map_accessor, pos, 1382, 1509);
            } else if ne {
                return choose4(map_accessor, pos, 1384, 1512);
            } else if sw {
                return choose2(map_accessor, pos, 1380, 1520);
            } else if se {
                return choose2(map_accessor, pos, 1386, 1519);
            }
        }

        let elevation = map_accessor.elevation_at(pos);

        if map_accessor.terrain_flags_at(&pos.south_east()).is(HalfSlope)
            && map_accessor.elevation_at(&pos.south_east().south_east()) > elevation
        {
            return calculate_half_slope(map_accessor, &pos.south_east()) + 1;
        } else if map_accessor.terrain_flags_at(&pos.north_east()).is(HalfSlope)
            && map_accessor.elevation_at(&pos.north_east().north_east()) > elevation
        {
            return calculate_half_slope(map_accessor, &pos.north_east()) + 1;
        } else if map_accessor.terrain_flags_at(&pos.south_west()).is(HalfSlope)
            && map_accessor.elevation_at(&pos.south_west().south_west()) > elevation
        {
            return calculate_half_slope(map_accessor, &pos.south_west()) + 1;
        } else if map_accessor.terrain_flags_at(&pos.north_west()).is(HalfSlope)
            && map_accessor.elevation_at(&pos.north_west().north_west()) > elevation
        {
            return calculate_half_slope(map_accessor, &pos.north_west()) + 1;
        }
    } else {
        let dx = root_offset & 0b111;
        let dy = (root_offset >> 3) & 0b111;

        return calculate_all_slope_2x2(map_accessor, &MapPosition::from_x_y_untrimmed(pos.x() - dx, pos.y() - dy));
    }
    return UNKNOWN_TERRAIN_TEXTURE;
}

fn calculate_high_slope<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    if map_accessor.terrain_flags_at(pos).is(WaterShallow) {
        return calculate_water_shallow(map_accessor, pos);
    }

    let elevation_up = map_accessor.elevation_at(pos) + 1;
    let [ne1, se1, sw1, nw1] = test_neighbours_4(pos, |p| map_accessor.elevation_at(&p) == elevation_up);

    let elevation_up2 = map_accessor.elevation_at(pos) + 2;
    let [n2, ne2, e2, se2, s2, sw2, w2, nw2] = test_neighbours_8(pos, |p| map_accessor.elevation_at(&p) == elevation_up2);

    return if w2 && nw2 && sw2 {
        1370
    } else if n2 && nw2 && ne2 {
        1367
    } else if s2 && se2 && sw2 {
        1369
    } else if e2 && ne2 && se2 {
        1368
    } else if s2 && sw1 {
        1371
    } else if w2 && sw1 {
        1372
    } else if w2 && nw1 {
        1373
    } else if n2 && nw1 {
        1374
    } else if n2 && ne1 {
        1375
    } else if e2 && ne1 {
        1376
    } else if e2 && se1 {
        1377
    } else if s2 && se1 {
        1378
    } else if sw2 {
        choose2(map_accessor, pos, 1360, 1530)
    } else if nw2 {
        choose4(map_accessor, pos, 1362, 1495)
    } else if ne2 {
        choose4(map_accessor, pos, 1364, 1498)
    } else if se2 {
        choose2(map_accessor, pos, 1366, 1529)
    } else if s2 {
        choose2(map_accessor, pos, 1359, 1505)
    } else if w2 {
        choose2(map_accessor, pos, 1361, 1506)
    } else if e2 {
        choose2(map_accessor, pos, 1365, 1508)
    } else if n2 {
        choose2(map_accessor, pos, 1363, 1507)
    } else {
        UNKNOWN_TERRAIN_TEXTURE
    };
}

fn calculate_earthquake<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let random = map_accessor.random_at(pos) as u32;

    let [ne, se, sw, nw] = test_neighbours_4(pos, |p| map_accessor.terrain_flags_at(&p).is(Earthquake));

    return match [ne, se, sw, nw] {
        [T, T, T, T] => 473,
        [T, T, T, _] => 469,
        [_, T, T, T] => 470,
        [T, _, T, T] => 471,
        [T, T, _, T] => 472,
        [T, _, T, _] => 444 + (random % 4),
        [_, T, _, T] => 448 + (random % 4),
        [T, T, _, _] => 452 + (random % 2),
        [_, T, T, _] => 454 + (random % 2),
        [_, _, T, T] => 456 + (random % 2),
        [T, _, _, T] => 458 + (random % 2),
        [T, _, _, _] => 460 + (random % 2),
        [_, _, T, _] => 462 + (random % 2),
        [_, T, _, _] => 464 + (random % 2),
        [_, _, _, T] => 466 + (random % 2),
        _ => 468,
    };
}

fn calculate_water_deep<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let random = map_accessor.random_at(pos) as u32;

    let [n, ne, e, se, s, sw, w, nw] = test_neighbours_8(pos, |p| map_accessor.terrain_flags_at(&p).is(WaterDeep));

    // TODO rewrite to check for positives
    let variant = if !ne && !se {
        1
    } else if !se && !sw {
        3
    } else if !nw && !sw {
        5
    } else if !nw && !ne {
        7
    } else if !ne {
        0
    } else if !se {
        2
    } else if !sw {
        4
    } else if !nw {
        6
    } else if !e && !w {
        12
    } else if !n && !s {
        13
    } else if !e {
        8
    } else if !s {
        9
    } else if !w {
        10
    } else if !n {
        11
    } else {
        14
    };

    let random_offset = if variant == 14 { (random % 6) * 15 } else { 0 };

    return 761 + random_offset + variant;
}

fn calculate_slope_corner<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    debug_assert!(map_accessor.terrain_flags_at(&pos).is(Slope));

    if map_accessor.terrain_flags_at(&pos).is(Road) {
        return calculate_road(map_accessor, pos);
    }

    if map_accessor.terrain_flags_at(&pos).is(WaterShallow) {
        return calculate_water_shallow(map_accessor, pos);
    }

    let elevation_up = map_accessor.elevation_at(&pos) + 1;
    let [n, ne, e, se, s, sw, w, nw] = test_neighbours_8(pos, |p| map_accessor.elevation_at(&p) == elevation_up);

    let tile_size = map_accessor.tile_size_at(pos);

    return if nw && ne {
        1387
    } else if ne && se {
        1388
    } else if sw && se {
        1389
    } else if nw && sw {
        1390
    } else if s {
        if tile_size == 33 {
            1525
        } else if tile_size == 1 {
            1411
        } else {
            choose2(map_accessor, pos, 1379, 1521)
        }
    } else if w {
        if tile_size == 33 {
            1526
        } else if tile_size == 1 {
            1413
        } else {
            choose2(map_accessor, pos, 1381, 1522)
        }
    } else if n {
        if tile_size == 33 {
            1527
        } else if tile_size == 1 {
            1415
        } else {
            choose2(map_accessor, pos, 1383, 1523)
        }
    } else if e {
        if tile_size == 33 {
            1528
        } else if tile_size == 1 {
            1417
        } else {
            choose2(map_accessor, pos, 1385, 1524)
        }
    } else {
        UNKNOWN_TERRAIN_TEXTURE
    };
}

fn calculate_access_ramp<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    if map_accessor.terrain_flags_at(pos).is(Road) {
        return calculate_road_access_ramp(map_accessor, pos);
    }

    let elevation_down = map_accessor.elevation_at(pos).wrapping_sub(1);
    let elevation_up = map_accessor.elevation_at(pos) + 1;

    return if map_accessor.elevation_at(&pos.north_west()) == elevation_up {
        1392
    } else if map_accessor.elevation_at(&pos.north_east()) == elevation_up {
        if map_accessor.elevation_at(&pos.south_east()) == elevation_down {
            calculate_slope(map_accessor, pos)
        } else {
            1393
        }
    } else if map_accessor.elevation_at(&pos.south_west()) == elevation_up {
        if map_accessor.elevation_at(&pos.south_east()) == elevation_down {
            calculate_slope(map_accessor, pos)
        } else {
            1391
        }
    } else if map_accessor.elevation_at(&pos.south_east()) == elevation_up {
        if map_accessor.elevation_at(&pos.south_west()) == elevation_down {
            calculate_slope(map_accessor, pos)
        } else {
            1394
        }
    } else {
        UNKNOWN_TERRAIN_TEXTURE
    };
}

fn calculate_road_access_ramp<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let elevation_up = map_accessor.elevation_at(pos) + 1;

    let [ne, se, sw, nw] = test_neighbours_4(pos, |p| map_accessor.elevation_at(&p) == elevation_up);

    return if sw {
        1395
    } else if nw {
        1396
    } else if ne {
        1397
    } else if se {
        1398
    } else {
        UNKNOWN_TERRAIN_TEXTURE
    };
}

fn calculate_meadow<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    if map_accessor.terrain_flags_at(pos).is(Road) {
        return calculate_road(map_accessor, pos);
    }

    let meadow = map_accessor.meadow_at(pos);
    let scrub = map_accessor.scrub_at(pos);
    let scrub_length = if scrub > 0 { (scrub - 1) as u32 / 8 } else { 0 };
    let random = (map_accessor.random_at(pos) % 8) as u32;

    return if meadow < 40 {
        if scrub > 8 { 1299 + scrub_length } else { 1351 + random }
    } else if meadow < 72 {
        1311 + scrub_length
    } else {
        if scrub > 88 {
            1343 + random
        } else if scrub > 8 {
            1323 + scrub_length
        } else {
            1335 + random
        }
    };
}

fn calculate_sand<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let random = (map_accessor.random_at(pos) % 6) as u32;
    return 851 + random;
}

fn calculate_marble<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, offset: u32) -> u32 {
    if map_accessor.terrain_flags_at(pos).is(Meadow) {
        return calculate_meadow(map_accessor, pos);
    }

    let random = map_accessor.random_at(pos) as u32;
    let is_black_marble = map_accessor.terrain_flags_at(pos).is(Copper);

    let [n, ne, e, se, s, sw, w, nw] = test_neighbours_8(pos, test_marble(map_accessor, is_black_marble));

    return match [n, ne, e, se, s, sw, w, nw] {
        [T, T, T, T, T, T, T, T] => offset + calculate_marble2(map_accessor, pos),
        [_, T, T, T, T, T, T, T] => offset + 22,
        [T, T, _, T, T, T, T, T] => offset + 23,
        [T, T, T, T, _, T, T, T] => offset + 24,
        [T, T, T, T, T, T, _, T] => offset + 25,
        [_, _, _, T, T, T, T, T] => offset + 7 + (random % 3),
        [T, T, _, _, _, T, T, T] => offset + 11 + (random % 3),
        [T, T, T, T, _, _, _, T] => offset + 15 + (random % 3),
        [_, T, T, T, T, T, _, _] => offset + 19 + (random % 3),
        [_, _, _, T, T, T, _, _] => offset + 6,
        [_, _, _, _, _, T, T, T] => offset + 10,
        [T, T, _, _, _, _, _, T] => offset + 14,
        [_, T, T, T, _, _, _, _] => offset + 18,
        _ => UNKNOWN_TERRAIN_TEXTURE,
    };
}

fn calculate_marble2<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let random = map_accessor.random_at(pos) as u32;
    let is_black_marble = map_accessor.terrain_flags_at(pos).is(Copper);

    let matching_marble = test_neighbours_16(pos, test_marble(map_accessor, is_black_marble));

    // n, nne, ne, nee, e, see, se, sse, s, ssw, sw, sww, w, nww, nw, nnw
    return match matching_marble {
        [T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T] => calculate_marble3(map_accessor, pos),
        [T, T, T, T, T, T, T, T, _, T, T, T, T, T, T, T] => 67,
        [T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T] => 68,
        [T, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T] => 69,
        [_, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T] => 70,
        [T, T, T, T, T, T, T, T, _, T, T, T, _, T, T, T] => 61,
        [T, T, T, T, _, T, T, T, _, T, T, T, T, T, T, T] => 62,
        [_, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T] => 63,
        [_, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T] => 64,
        [_, T, T, T, T, T, T, T, _, T, T, T, T, T, T, T] => 65,
        [T, T, T, T, _, T, T, T, T, T, T, T, _, T, T, T] => 66,
        [_, T, T, T, _, T, T, T, _, T, T, T, T, T, T, T] => 57,
        [_, T, T, T, _, T, T, T, T, T, T, T, _, T, T, T] => 58,
        [_, T, T, T, T, T, T, T, _, T, T, T, _, T, T, T] => 59,
        [T, T, T, T, _, T, T, T, _, T, T, T, _, T, T, T] => 60,
        [_, T, T, T, _, T, T, T, _, T, T, T, _, T, T, T] => 56,
        [_, _, _, _, _, T, T, T, T, T, T, T, T, T, T, T] => 41 + (random % 3),
        [T, T, T, T, _, _, _, _, _, T, T, T, T, T, T, T] => 45 + (random % 3),
        [T, T, T, T, T, T, T, T, _, _, _, _, _, T, T, T] => 49 + (random % 3),
        [_, T, T, T, T, T, T, T, T, T, T, T, _, _, _, _] => 53 + (random % 3),
        [_, T, T, T, _, T, T, T, T, T, T, T, _, _, _, _] => 72,
        [_, T, T, T, T, T, T, T, _, T, T, T, _, _, _, _] => 73,
        [_, T, T, T, _, T, T, T, _, T, T, T, _, _, _, _] => 74,
        [_, _, _, _, _, T, T, T, _, T, T, T, T, T, T, T] => 75,
        [_, _, _, _, _, T, T, T, T, T, T, T, _, T, T, T] => 76,
        [_, _, _, _, _, T, T, T, _, T, T, T, _, T, T, T] => 77,
        [T, T, T, T, _, _, _, _, _, T, T, T, _, T, T, T] => 78,
        [_, T, T, T, _, _, _, _, _, T, T, T, T, T, T, T] => 79,
        [_, T, T, T, _, _, _, _, _, T, T, T, _, T, T, T] => 80,
        [_, T, T, T, T, T, T, T, _, _, _, _, _, T, T, T] => 81,
        [T, T, T, T, _, T, T, T, _, _, _, _, _, T, T, T] => 82,
        [_, T, T, T, _, T, T, T, _, _, _, _, _, T, T, T] => 83,
        [_, _, _, _, _, T, T, T, T, T, T, T, _, _, _, _] => 40,
        [_, _, _, _, _, _, _, _, _, T, T, T, T, T, T, T] => 44,
        [T, T, T, T, _, _, _, _, _, _, _, _, _, T, T, T] => 48,
        [_, T, T, T, T, T, T, T, _, _, _, _, _, _, _, _] => 52,
        [_, T, T, T, _, T, T, T, _, _, _, _, _, _, _, _] => 86,
        [_, _, _, _, _, T, T, T, _, T, T, T, _, _, _, _] => 87,
        [_, _, _, _, _, _, _, _, _, T, T, T, _, T, T, T] => 88,
        [_, T, T, T, _, _, _, _, _, _, _, _, _, T, T, T] => 89,
        [_, T, T, T, _, _, _, _, _, T, T, T, _, _, _, _] => 90,
        [_, _, _, _, _, T, T, T, _, _, _, _, _, T, T, T] => 91,
        [_, T, T, T, _, _, _, _, _, _, _, _, _, _, _, _] => 84,
        [_, _, _, _, _, T, T, T, _, _, _, _, _, _, _, _] => 85,
        [_, _, _, _, _, _, _, _, _, T, T, T, _, _, _, _] => 92,
        [_, _, _, _, _, _, _, _, _, _, _, _, _, T, T, T] => 93,
        _ => 71,
    };
}

fn calculate_marble3<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let random = map_accessor.random_at(pos) as u32;
    let is_black_marble = map_accessor.terrain_flags_at(pos).is(Copper);

    let matching_marble = test_neighbours_24(pos, test_marble(map_accessor, is_black_marble));

    // n, nnne, nne, ne, nee, neee, e, seee, see, se, sse, ssse, s, sssw, ssw, sw, sww, swww, w, nwww, nww, nw, nnw, nnnw
    return match matching_marble {
        [T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T] => 94 + (random % 6),
        [T, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T] => 127,
        [T, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T] => 128,
        [T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T] => 129,
        [_, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T] => 130,
        [T, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T] => 121,
        [T, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T] => 122,
        [_, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T] => 123,
        [_, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T] => 124,
        [_, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T] => 125,
        [T, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T] => 126,
        [_, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T] => 117,
        [_, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T] => 118,
        [_, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T] => 119,
        [T, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T] => 120,
        [_, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T] => 116,
        [_, _, _, _, _, _, _, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T] => 101 + (random % 3),
        [T, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T, T, T, T, T, T, T] => 105 + (random % 3),
        [T, T, T, T, T, T, T, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T] => 109 + (random % 3),
        [_, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, _, _, _, _, _, _] => 113 + (random % 3),
        [_, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T, _, _, _, _, _, _] => 132,
        [_, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T, _, _, _, _, _, _] => 133,
        [_, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T, _, _, _, _, _, _] => 134,
        [_, _, _, _, _, _, _, T, T, T, T, T, _, T, T, T, T, T, T, T, T, T, T, T] => 135,
        [_, _, _, _, _, _, _, T, T, T, T, T, T, T, T, T, T, T, _, T, T, T, T, T] => 136,
        [_, _, _, _, _, _, _, T, T, T, T, T, _, T, T, T, T, T, _, T, T, T, T, T] => 137,
        [T, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T, _, T, T, T, T, T] => 138,
        [_, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T, T, T, T, T, T, T] => 139,
        [_, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T, _, T, T, T, T, T] => 140,
        [_, T, T, T, T, T, T, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T] => 141,
        [T, T, T, T, T, T, _, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T] => 142,
        [_, T, T, T, T, T, _, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T] => 143,
        [_, _, _, _, _, _, _, T, T, T, T, T, T, T, T, T, T, T, _, _, _, _, _, _] => 100,
        [_, _, _, _, _, _, _, _, _, _, _, _, _, T, T, T, T, T, T, T, T, T, T, T] => 104,
        [T, T, T, T, T, T, _, _, _, _, _, _, _, _, _, _, _, _, _, T, T, T, T, T] => 108,
        [_, T, T, T, T, T, T, T, T, T, T, T, _, _, _, _, _, _, _, _, _, _, _, _] => 112,
        [_, T, T, T, T, T, _, T, T, T, T, T, _, _, _, _, _, _, _, _, _, _, _, _] => 146,
        [_, _, _, _, _, _, _, T, T, T, T, T, _, T, T, T, T, T, _, _, _, _, _, _] => 147,
        [_, _, _, _, _, _, _, _, _, _, _, _, _, T, T, T, T, T, _, T, T, T, T, T] => 148,
        [_, T, T, T, T, T, _, _, _, _, _, _, _, _, _, _, _, _, _, T, T, T, T, T] => 149,
        [_, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T, _, _, _, _, _, _] => 150,
        [_, _, _, _, _, _, _, T, T, T, T, T, _, _, _, _, _, _, _, T, T, T, T, T] => 151,
        [_, T, T, T, T, T, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _] => 144,
        [_, _, _, _, _, _, _, T, T, T, T, T, _, _, _, _, _, _, _, _, _, _, _, _] => 145,
        [_, _, _, _, _, _, _, _, _, _, _, _, _, T, T, T, T, T, _, _, _, _, _, _] => 152,
        [_, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, T, T, T, T, T] => 153,
        _ => 131,
    };
}

fn test_marble<T: MapAccessor>(map_accessor: &T, is_black_marble: bool) -> impl Fn(MapPosition) -> bool {
    return move |pos| {
        let terrain_type_flags = map_accessor.terrain_flags_at(&pos);
        return terrain_type_flags.is(Marble) && terrain_type_flags.is(Copper) == is_black_marble;
    };
}

fn calculate_marshland<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    return calculate_water_edge(map_accessor, pos, 972, Marshland);
}

fn calculate_water_edge<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, offset: u32, terrain_type: TerrainTypeFlag) -> u32 {
    let random = map_accessor.random_at(pos) as u32;

    let [n, ne, e, se, s, sw, w, nw] = test_neighbours_8(pos, |p| map_accessor.terrain_flags_at(&p).is(terrain_type));

    return match [n, ne, e, se, s, sw, w, nw] {
        [T, T, T, T, T, T, T, T] => {
            if terrain_type == WaterShallow {
                663 + (random % 6)
            } else if terrain_type == Marshland {
                964 + (random % 8)
            } else {
                offset + (random % 6)
            }
        }
        [T, T, T, T, T, T, _, T] => offset + 50,
        [_, T, T, T, T, T, T, T] => offset + 51,
        [T, T, _, T, T, T, T, T] => offset + 52,
        [T, T, T, T, _, T, T, T] => offset + 53,
        [_, T, T, T, T, T, _, T] => offset + 54,
        [_, T, _, T, T, T, T, T] => offset + 55,
        [T, T, _, T, _, T, T, T] => offset + 56,
        [T, T, T, T, _, T, _, T] => offset + 57,
        [T, T, _, T, T, T, _, T] => offset + 48,
        [_, T, T, T, _, T, T, T] => offset + 49,
        [T, T, T, T, _, _, _, T] => offset + 8 + (random % 4),
        [_, T, T, T, T, T, _, _] => offset + 12 + (random % 4),
        [_, _, _, T, T, T, T, T] => offset + 16 + (random % 4),
        [T, T, _, _, _, T, T, T] => offset + 20 + (random % 4),
        [_, T, _, T, T, T, _, T] => offset + 58,
        [_, T, _, T, _, T, T, T] => offset + 59,
        [T, T, _, T, _, T, _, T] => offset + 60,
        [_, T, T, T, _, T, _, T] => offset + 61,
        [_, T, _, T, _, T, _, T] => offset + 62,
        [_, T, T, T, _, _, _, T] => offset + 63,
        [T, T, _, T, _, _, _, T] => offset + 64,
        [_, T, _, T, T, T, _, _] => offset + 66,
        [_, T, T, T, _, T, _, _] => offset + 67,
        [_, _, _, T, _, T, T, T] => offset + 69,
        [_, _, _, T, T, T, _, T] => offset + 70,
        [T, T, _, _, _, T, _, T] => offset + 72,
        [_, T, _, _, _, T, T, T] => offset + 73,
        [_, T, _, T, _, _, _, T] => offset + 65,
        [_, T, _, T, _, T, _, _] => offset + 68,
        [_, _, _, T, _, T, _, T] => offset + 71,
        [_, T, _, _, _, T, _, T] => offset + 74,
        [_, T, T, T, _, _, _, _] => offset + 24 + (random % 4),
        [_, _, _, T, T, T, _, _] => offset + 28 + (random % 4),
        [_, _, _, _, _, T, T, T] => offset + 32 + (random % 4),
        [T, T, _, _, _, _, _, T] => offset + 36 + (random % 4),
        [_, _, _, T, _, _, _, T] => offset + 40 + (random % 2),
        [_, T, _, _, _, T, _, _] => offset + 42 + (random % 2),
        [_, T, _, T, _, _, _, _] => offset + 75,
        [_, _, _, T, _, T, _, _] => offset + 76,
        [_, _, _, _, _, T, _, T] => offset + 77,
        [_, T, _, _, _, _, _, T] => offset + 78,
        [_, T, _, _, _, _, _, _] => offset + 44,
        [_, _, _, T, _, _, _, _] => offset + 45,
        [_, _, _, _, _, T, _, _] => offset + 46,
        [_, _, _, _, _, _, _, T] => offset + 47,
        _ => offset + 79,
    };
}

fn calculate_half_slope<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    debug_assert!(map_accessor.terrain_flags_at(&pos).is(Slope));

    if !map_accessor.terrain_flags_at(&pos).is(AccessRamp) && map_accessor.terrain_flags_at(&pos).is(Road) {
        return calculate_road(map_accessor, &pos);
    }

    if map_accessor.terrain_flags_at(&pos).is(HighSlope) {
        return calculate_high_slope(map_accessor, &pos);
    }

    let elevation_up = map_accessor.elevation_at(&pos) + 1;
    let [e_ne, e_se, e_sw, e_nw] = test_neighbours_4(pos, |p| map_accessor.elevation_at(&p) == elevation_up);

    let [t_n, t_ne, t_e, t_se, _t_s, t_sw, t_w, t_nw] = test_neighbours_8(pos, |p| {
        return map_accessor.terrain_flags_at(&p).is(HalfSlope);
    });

    let tile_size = map_accessor.tile_size_at(pos) & 0xf;

    return if tile_size == 0 {
        if e_sw {
            if map_accessor.terrain_flags_at(&pos.north_east()).is(AccessRamp) {
                calculate_half_slope_access_ramp(map_accessor, &pos)
            } else if t_nw && t_se {
                1427
            } else if t_se {
                1459
            } else if t_nw {
                1461
            } else {
                1475
            }
        } else if e_nw {
            if map_accessor.terrain_flags_at(&pos.south_east()).is(AccessRamp) {
                calculate_half_slope_access_ramp(map_accessor, &pos)
            } else if t_ne && t_sw {
                1429
            } else if t_sw {
                1463
            } else if t_ne {
                1465
            } else {
                1477
            }
        } else if e_ne {
            if map_accessor.terrain_flags_at(&pos.south_west()).is(AccessRamp) {
                calculate_half_slope_access_ramp(map_accessor, &pos)
            } else if t_nw && t_se {
                1431
            } else if t_se {
                1467
            } else if t_nw {
                1469
            } else {
                1479
            }
        } else if e_se {
            if map_accessor.terrain_flags_at(&pos.north_west()).is(AccessRamp) {
                calculate_half_slope_access_ramp(map_accessor, &pos)
            } else if t_ne && t_sw {
                1433
            } else if t_ne {
                1471
            } else if t_sw {
                1473
            } else {
                1481
            }
        } else {
            if map_accessor.terrain_flags_at(&pos).is(AccessRamp) {
                calculate_half_slope_access_ramp(map_accessor, &pos)
            } else if t_nw && t_w {
                1464
            } else if t_ne && t_n {
                1470
            } else if t_nw && t_n {
                1466
            } else if t_ne && t_e {
                1468
            } else if t_se {
                1474
            } else if t_sw {
                1476
            } else {
                UNKNOWN_TERRAIN_TEXTURE
            }
        }
    } else {
        calculate_all_slope_2x2(map_accessor, &pos)
    };
}

fn calculate_all_slope_2x2<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let root_offset = map_accessor.root_offset_at(pos);
    let dx = (root_offset & 0b111) as u16;
    let dy = ((root_offset >> 3) & 0b111) as u16;

    if root_offset != 0 {
        return calculate_all_slope_2x2(map_accessor, &MapPosition::from_x_y_untrimmed(pos.x() - dx, pos.y() - dy));
    } else {
        let elevation_up = map_accessor.elevation_at(pos) + 1;
        let [n, ne, e, se, s, sw, w, nw] = test_neighbours_2x2(pos, |p| map_accessor.elevation_at(&p) == elevation_up);

        if nw && n && ne {
            return 1419;
        } else if ne && e && se {
            return 1420;
        } else if se && s && sw {
            return 1421;
        } else if sw && w && nw {
            return 1422;
        } else if sw {
            if !map_accessor.terrain_flags_at(&pos.south_west()).is(HalfSlope) {
                return 1403;
            } else if !map_accessor.terrain_flags_at(&pos.south().south_east()).is(HalfSlope) {
                return 1404;
            }
            let options = [1412, 1685, 1689, 1693];
            let random = map_accessor.random_at(&pos.south()) % 4;
            return options[random as usize];
        } else if nw {
            if !map_accessor.terrain_flags_at(&pos.north_east()).is(HalfSlope) {
                return 1405;
            } else if !map_accessor.terrain_flags_at(&pos.south_west().south_west()).is(HalfSlope) {
                return 1406;
            }
            let options = [1414, 1686, 1690, 1694];
            let random = map_accessor.random_at(&pos.south_west()) % 4;
            return options[random as usize];
        } else if ne {
            if !map_accessor.terrain_flags_at(&pos.north_west()).is(HalfSlope) {
                return 1407;
            } else if !map_accessor.terrain_flags_at(&pos.south_east().south_east()).is(HalfSlope) {
                return 1408;
            }
            let options = [1416, 1687, 1691, 1695];
            let random = map_accessor.random_at(pos) % 4;
            return options[random as usize];
        } else if se {
            if !map_accessor.terrain_flags_at(&pos.south().south_west()).is(HalfSlope) {
                return 1409;
            } else if !map_accessor.terrain_flags_at(&pos.east()).is(HalfSlope) {
                return 1410;
            }
            let options = [1418, 1688, 1692, 1696];
            let random = map_accessor.random_at(&pos.south_east()) % 4;
            return options[random as usize];
        } else if s {
            return choose2(map_accessor, &pos.south(), 1411, 1525);
        } else if w {
            return choose2(map_accessor, &pos.south_west(), 1413, 1526);
        } else if n {
            return choose2(map_accessor, &pos, 1415, 1527);
        } else if e {
            return choose2(map_accessor, &pos.south_east(), 1417, 1528);
        }
        return UNKNOWN_TERRAIN_TEXTURE;
    }
}

fn calculate_half_slope_access_ramp<T: MapAccessor>(map_accessor: &T, pos: &MapPosition) -> u32 {
    let road_offset = if map_accessor.terrain_flags_at(&pos).is(Road) { 8 } else { 0 };

    if (map_accessor.root_offset_at(pos) & 1) == 0 {
        let elevation_up = map_accessor.elevation_at(pos) + 1;
        let [ne, se, sw, nw] = test_neighbours_4(pos, |p| map_accessor.elevation_at(&p) == elevation_up);
        if sw {
            return 1435 + road_offset;
        } else if nw {
            return 1437 + road_offset;
        } else if ne {
            return 1439 + road_offset;
        } else if se {
            return 1441 + road_offset;
        }
    } else {
        for p in [pos.north_west(), pos.north_east(), pos.south_west(), pos.south_east()] {
            let terrain = map_accessor.terrain_flags_at(&p);
            if terrain.is_all_of(HalfSlope | AccessRamp) {
                return calculate_half_slope_access_ramp(map_accessor, &p) + 1;
            }
        }
    }

    return UNKNOWN_TERRAIN_TEXTURE;
}

fn test_neighbours_4<F: Fn(MapPosition) -> bool>(pos: &MapPosition, test: F) -> [bool; 4] {
    return [
        test(pos.north_east()),
        test(pos.south_east()),
        test(pos.south_west()),
        test(pos.north_west()),
    ];
}

fn test_neighbours_8<F: Fn(MapPosition) -> bool>(pos: &MapPosition, test: F) -> [bool; 8] {
    return [
        test(pos.north()),
        test(pos.north_east()),
        test(pos.east()),
        test(pos.south_east()),
        test(pos.south()),
        test(pos.south_west()),
        test(pos.west()),
        test(pos.north_west()),
    ];
}

fn test_neighbours_16<F: Fn(MapPosition) -> bool>(pos: &MapPosition, test: F) -> [bool; 16] {
    return [
        test(pos.north().north()),
        test(pos.north().north_east()),
        test(pos.north_east().north_east()),
        test(pos.east().north_east()),
        test(pos.east().east()),
        test(pos.east().south_east()),
        test(pos.south_east().south_east()),
        test(pos.south().south_east()),
        test(pos.south().south()),
        test(pos.south().south_west()),
        test(pos.south_west().south_west()),
        test(pos.west().south_west()),
        test(pos.west().west()),
        test(pos.west().north_west()),
        test(pos.north_west().north_west()),
        test(pos.north().north_west()),
    ];
}

fn test_neighbours_24<F: Fn(MapPosition) -> bool>(pos: &MapPosition, test: F) -> [bool; 24] {
    return [
        test(pos.north().north().north()),
        test(pos.north().north().north_east()),
        test(pos.north_east().north_east().north()),
        test(pos.north_east().north_east().north_east()),
        test(pos.north_east().north_east().east()),
        test(pos.east().east().north_east()),
        test(pos.east().east().east()),
        test(pos.east().east().south_east()),
        test(pos.south_east().south_east().east()),
        test(pos.south_east().south_east().south_east()),
        test(pos.south_east().south_east().south()),
        test(pos.south().south().south_east()),
        test(pos.south().south().south()),
        test(pos.south().south().south_west()),
        test(pos.south_west().south_west().south()),
        test(pos.south_west().south_west().south_west()),
        test(pos.south_west().south_west().west()),
        test(pos.west().west().south_west()),
        test(pos.west().west().west()),
        test(pos.west().west().north_west()),
        test(pos.north_west().north_west().west()),
        test(pos.north_west().north_west().north_west()),
        test(pos.north_west().north_west().north()),
        test(pos.north().north().north_west()),
    ];
}

fn test_neighbours_2x2<F: Fn(MapPosition) -> bool>(pos: &MapPosition, test: F) -> [bool; 8] {
    return [
        test(pos.north()),
        test(pos.north_east()),
        test(pos.east().south_east()),
        test(pos.south_east().south_east()),
        test(pos.south().south()),
        test(pos.south_west().south_west()),
        test(pos.west().south_west()),
        test(pos.north_west()),
    ];
}

fn all_true<const N: usize>(data: [bool; N]) -> bool {
    return data.iter().all(|x| *x);
}

fn choose2<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, a: u32, b: u32) -> u32 {
    return if (map_accessor.random_at(pos) % 2) == 0 { a } else { b };
}

fn choose4<T: MapAccessor>(map_accessor: &T, pos: &MapPosition, a: u32, b: u32) -> u32 {
    let random = (map_accessor.random_at(pos) % 4) as u32;
    return if random == 0 { a } else { b - 1 + random };
}

#[cfg(test)]
mod tests {
    use crate::map_helper::map_accessor::MapAccessor;
    use crate::map_helper::position_iterator::for_each_valid_position;
    use crate::map_helper::texture_calculator::UNKNOWN_TERRAIN_TEXTURE;
    use crate::map_helper::texture_calculator::calculate_texture_id;
    use crate::prelude::MapData;
    use crate::prelude::MapPosition;
    use crate::prelude::PakData;
    use crate::prelude::TerrainTypeFlag::*;
    use crate::prelude::TerrainTypeFlags;
    use crate::prelude::convert_to_texture_id;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Result;
    use std::path::PathBuf;

    #[ignore] // todo ignore plan terrain as that seems unstable and remove the ignored
    #[test]
    fn test_maps() -> Result<()> {
        let game_root = std::env::var("ZEUS_HOME").expect("ZEUS_HOME env var is not file_data");

        let mut successful_tiles = 0;

        let adventures_folder = fs::read_dir(format!("{}/Adventures", game_root))?;

        let mut files_tested = 0;

        for adventure in adventures_folder {
            let folder_path = adventure?.path();
            if folder_path.is_dir() {
                let files = fs::read_dir(folder_path)?;
                for file in files {
                    let file_path = file?.path();
                    successful_tiles += check_file(file_path)?;
                    files_tested += 1;
                }
            } else {
                successful_tiles += check_file(folder_path)?;
                files_tested += 1;
            }
        }

        assert!(files_tested >= 8);

        println!("{successful_tiles:?} tiles verified");

        return Ok(());
    }

    fn check_file(file_path: PathBuf) -> Result<usize> {
        let mut successful_tiles = 0;

        let extension = file_path.extension().map_or("", |x| x.to_str().unwrap_or(""));
        if extension == "pak" {
            let mut reader = File::open(&file_path).map(BufReader::new)?;
            let pak_data = PakData::read_from(&mut reader)?;
            let zeus_map_file = &pak_data.map_data[0];
            for_each_valid_position((zeus_map_file.scenario_data.map_size - 4) as u16, |pos| {
                if should_validate(zeus_map_file, &pos) {
                    let expected = convert_to_texture_id(zeus_map_file.sprite[pos.index()]);
                    let calculated = calculate_texture_id(zeus_map_file, &pos, zeus_map_file.scenario_data.tropical != 0);
                    if expected != calculated {
                        assert!(
                            false,
                            "Error for file {file_path:?} for {:#?} (success for {successful_tiles} textures)",
                            ErrorTile::new(&zeus_map_file, &pos)
                        );
                    }
                    successful_tiles += 1;
                }
            });

            println!("File {file_path:?} all correct");
        }

        return Ok(successful_tiles);
    }

    fn should_validate(map_data: &MapData, position: &MapPosition) -> bool {
        return !map_data.terrain_flags_at(position).is_any_of(Earthquake);
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct ErrorTile {
        x: u16,
        y: u16,
        expected: u32,
        calculated: u32,
        root_offset: u8,
        terrain_type: TerrainTypeFlags,
        tile_size: u8,
        random: u8,
        meadow: u8,
        scrub: u8,
        tropical: u32,
        neighbour_no: TerrainTypeFlags,
        neighbour_ne: TerrainTypeFlags,
        neighbour_ea: TerrainTypeFlags,
        neighbour_se: TerrainTypeFlags,
        neighbour_so: TerrainTypeFlags,
        neighbour_sw: TerrainTypeFlags,
        neighbour_we: TerrainTypeFlags,
        neighbour_nw: TerrainTypeFlags,
        neighbours_2: Vec<TerrainTypeFlags>,
    }

    impl ErrorTile {
        fn new(map_data: &MapData, position: &MapPosition) -> Self {
            let neighbour_types = [
                position.north(),
                position.north_west(),
                position.west(),
                position.south_west(),
                position.south(),
                position.south_east(),
                position.east(),
                position.north_east(),
            ]
            .map(|pos| map_data.terrain_flags_at(&pos));

            let neighbours_2 = if map_data.terrain_flags_at(position).is_any_of(Forest) {
                let x = position.x();
                let y = position.y();

                vec![
                    (x.wrapping_sub(2), y.wrapping_sub(2)),
                    (x.wrapping_sub(2), y.wrapping_sub(1)),
                    (x.wrapping_sub(2), y + 0),
                    (x.wrapping_sub(2), y + 1),
                    (x.wrapping_sub(2), y + 2),
                    (x.wrapping_sub(1), y.wrapping_sub(2)),
                    (x.wrapping_sub(1), y + 2),
                    (x + 0, y.wrapping_sub(2)),
                    (x + 0, y + 2),
                    (x + 1, y.wrapping_sub(2)),
                    (x + 1, y + 2),
                    (x + 2, y.wrapping_sub(2)),
                    (x + 2, y.wrapping_sub(1)),
                    (x + 2, y + 0),
                    (x + 2, y + 1),
                    (x + 2, y + 2),
                ]
                .iter()
                .map(|&(x, y)| map_data.terrain_flags_at(&MapPosition::from_x_y_untrimmed(x, y)))
                .collect()
            } else {
                vec![]
            };

            return Self {
                x: position.x(),
                y: position.y(),
                expected: convert_to_texture_id(map_data.sprite[position.index()]) + UNKNOWN_TERRAIN_TEXTURE,
                calculated: calculate_texture_id(map_data, &position, map_data.scenario_data.tropical != 0) + UNKNOWN_TERRAIN_TEXTURE,
                root_offset: map_data.root_offset_at(position),
                terrain_type: map_data.terrain_flags_at(position),
                tile_size: map_data.tile_size_at(position),
                random: map_data.random_at(position),
                meadow: map_data.meadow_at(position),
                scrub: map_data.scrub_at(position),
                tropical: map_data.scenario_data.tropical,
                neighbour_no: neighbour_types[0],
                neighbour_ne: neighbour_types[7],
                neighbour_ea: neighbour_types[6],
                neighbour_se: neighbour_types[5],
                neighbour_so: neighbour_types[4],
                neighbour_sw: neighbour_types[3],
                neighbour_we: neighbour_types[2],
                neighbour_nw: neighbour_types[1],
                neighbours_2,
            };
        }
    }
}
