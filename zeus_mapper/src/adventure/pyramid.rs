use crate::adventure::God;
use crate::differ::default_differ_impl;
use crate::prelude::DataConstant;
use crate::prelude::PyramidData;
use my_macros::LogDifferences;

#[derive(LogDifferences, PartialEq, Debug)]
pub struct PyramidSetting {
    pub pyramid_type: PyramidConfig,
    pub allowed: bool,
}

impl PyramidSetting {
    /// Builds the offered pyramid/shrine options for one episode from its settings-level
    /// `MythologyData.pyramids` slots.
    ///
    /// Slots with `pyramid_type == 0` are unused padding (the file always carries a fixed 6-slot
    /// array regardless of how many are actually offered) and are skipped, as are any slots whose
    /// `pyramid_type`/`deity` don't resolve to a known configuration.
    pub(crate) fn vec_from_data(pyramids: &[PyramidData]) -> Vec<PyramidSetting> {
        return pyramids
            .iter()
            .filter(|pyramid| pyramid.pyramid_type != 0)
            .filter_map(|pyramid| {
                PyramidConfig::from_raw_fields(pyramid.pyramid_type, pyramid.deity, pyramid.coloration).map(|pyramid_type| PyramidSetting {
                    pyramid_type,
                    allowed: true,
                })
            })
            .collect();
    }

    /// Inverse of [`PyramidSetting::vec_from_data`]: encodes up to 6 settings into a fixed 6-slot
    /// `MythologyData.pyramids` row, padding any remaining slots with a zeroed (unused)
    /// `PyramidData`, alongside the populated count for `MythologyData.max_pyramids`.
    ///
    /// `allowed` isn't encoded - no raw field for it has been identified yet, see `PyramidSetting`.
    pub(crate) fn vec_to_data(settings: &[PyramidSetting]) -> (Vec<PyramidData>, u32) {
        let mut result = vec![PyramidData::default(); 6];

        for (slot, setting) in settings.iter().take(result.len()).enumerate() {
            let (pyramid_type, deity, coloration) = setting.pyramid_type.to_raw_fields();
            result[slot] = PyramidData {
                pyramid_type,
                deity,
                coloration,
            };
        }

        return (result, settings.len().min(6) as u32);
    }
}

/// A specific, fully-placed pyramid: which structure plus its per-tile color/god configuration.
/// Distinct from `PyramidType`, the flat "which pyramid" id an `EpisodeGoal::Pyramid` goal
/// targets - a goal only names a structure, it doesn't (and, being a single raw `u32` in the file
/// format, can't) carry per-tile color or shrine-god placement data.
#[derive(PartialEq, Debug)]
pub enum PyramidConfig {
    ModestPyramid([PyramidColor; 2]),
    Pyramid([PyramidColor; 3]),
    GreatPyramid([PyramidColor; 4]),
    MajesticPyramid([PyramidColor; 5]),
    SmallMonumentToTheSky([PyramidColor; 3]),
    MonumentToTheSky([PyramidColor; 3]),
    GrantMonumentToTheSky([PyramidColor; 4]),
    MinorShrine(God, [PyramidColor; 2]),
    Shrine(God, [PyramidColor; 3]),
    MajorShrine(God, [PyramidColor; 3]),
    PyramidOfThePantheon([PyramidColor; 3]),
    AltarOfOlympus([PyramidColor; 4]),
    TempleOfOlympus([PyramidColor; 3]),
    ObservatoryKosmika([PyramidColor; 3]),
    MuseumAtlantika([PyramidColor; 2]),
}

default_differ_impl!(PyramidConfig);

impl PyramidConfig {
    /// Decodes one `PyramidData` slot's raw `pyramid_type`/`deity`/`coloration` fields.
    ///
    /// **Assumptions**: `pyramid_type` values `100..=114` were found to correspond 1:1 and in
    /// order to this enum's fifteen variants (confirmed by cross-referencing every observed
    /// `pyramid_type` across the full `Adventures` library against the tile-color count implied by
    /// `coloration`'s low bits - see below); `111` (`AltarOfOlympus`) has not been observed in real
    /// data but fills the one gap in that otherwise-unbroken sequence. `deity` only carries a
    /// meaningful value for the three shrine variants.
    fn from_raw_fields(pyramid_type: u32, deity: u32, coloration: u32) -> Option<PyramidConfig> {
        return match pyramid_type {
            100 => Some(PyramidConfig::ModestPyramid(colors(coloration))),
            101 => Some(PyramidConfig::Pyramid(colors(coloration))),
            102 => Some(PyramidConfig::GreatPyramid(colors(coloration))),
            103 => Some(PyramidConfig::MajesticPyramid(colors(coloration))),
            104 => Some(PyramidConfig::SmallMonumentToTheSky(colors(coloration))),
            105 => Some(PyramidConfig::MonumentToTheSky(colors(coloration))),
            106 => Some(PyramidConfig::GrantMonumentToTheSky(colors(coloration))),
            107 => God::try_resolve(&deity).map(|god| PyramidConfig::MinorShrine(god, colors(coloration))),
            108 => God::try_resolve(&deity).map(|god| PyramidConfig::Shrine(god, colors(coloration))),
            109 => God::try_resolve(&deity).map(|god| PyramidConfig::MajorShrine(god, colors(coloration))),
            110 => Some(PyramidConfig::PyramidOfThePantheon(colors(coloration))),
            111 => Some(PyramidConfig::AltarOfOlympus(colors(coloration))),
            112 => Some(PyramidConfig::TempleOfOlympus(colors(coloration))),
            113 => Some(PyramidConfig::ObservatoryKosmika(colors(coloration))),
            114 => Some(PyramidConfig::MuseumAtlantika(colors(coloration))),
            _ => None,
        };
    }

    /// Inverse of [`PyramidConfig::from_raw_fields`]: returns `(pyramid_type, deity, coloration)`.
    ///
    /// `deity` is `1` (the constant observed on every non-shrine slot in real data) for variants
    /// that don't carry a `God`.
    fn to_raw_fields(&self) -> (u32, u32, u32) {
        return match self {
            PyramidConfig::ModestPyramid(colors) => (100, 1, pack_colors(colors)),
            PyramidConfig::Pyramid(colors) => (101, 1, pack_colors(colors)),
            PyramidConfig::GreatPyramid(colors) => (102, 1, pack_colors(colors)),
            PyramidConfig::MajesticPyramid(colors) => (103, 1, pack_colors(colors)),
            PyramidConfig::SmallMonumentToTheSky(colors) => (104, 1, pack_colors(colors)),
            PyramidConfig::MonumentToTheSky(colors) => (105, 1, pack_colors(colors)),
            PyramidConfig::GrantMonumentToTheSky(colors) => (106, 1, pack_colors(colors)),
            PyramidConfig::MinorShrine(god, colors) => (107, god.value(), pack_colors(colors)),
            PyramidConfig::Shrine(god, colors) => (108, god.value(), pack_colors(colors)),
            PyramidConfig::MajorShrine(god, colors) => (109, god.value(), pack_colors(colors)),
            PyramidConfig::PyramidOfThePantheon(colors) => (110, 1, pack_colors(colors)),
            PyramidConfig::AltarOfOlympus(colors) => (111, 1, pack_colors(colors)),
            PyramidConfig::TempleOfOlympus(colors) => (112, 1, pack_colors(colors)),
            PyramidConfig::ObservatoryKosmika(colors) => (113, 1, pack_colors(colors)),
            PyramidConfig::MuseumAtlantika(colors) => (114, 1, pack_colors(colors)),
        };
    }
}

/// Inverse of [`colors`]: packs a `[PyramidColor]` slice back into `coloration`'s low bits.
///
/// The always-higher, not-yet-understood bits of `coloration` are left unset.
fn pack_colors(colors: &[PyramidColor]) -> u32 {
    let mut coloration = 0u32;

    for (i, color) in colors.iter().enumerate() {
        if *color == PyramidColor::Black {
            coloration |= 1 << i;
        }
    }

    return coloration;
}

/// Unpacks a per-tile `[PyramidColor; N]` array from `coloration`'s low `N` bits (bit `i` is tile
/// `i`, `0` = white, `1` = black); confirmed against six known-color placements across two real
/// adventures.
///
/// The remaining, always-higher bits of `coloration` vary independently of tile count and aren't
/// understood yet.
fn colors<const N: usize>(coloration: u32) -> [PyramidColor; N] {
    return std::array::from_fn(|i| {
        if coloration & (1 << i) != 0 {
            PyramidColor::Black
        } else {
            PyramidColor::White
        }
    });
}

#[derive(PartialEq, Debug)]
pub enum PyramidColor {
    White,
    Black,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adventure::adventure::Adventure;
    use std::io::Result;

    #[test]
    fn parse_the_youngest_twins() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Youngest Twins"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(
                episode_1.pyramid_settings,
                vec![
                    PyramidSetting {
                        pyramid_type: PyramidConfig::ModestPyramid([PyramidColor::White, PyramidColor::White]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::Pyramid([PyramidColor::Black, PyramidColor::Black, PyramidColor::Black]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::GreatPyramid([
                            PyramidColor::White,
                            PyramidColor::White,
                            PyramidColor::White,
                            PyramidColor::White
                        ]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::MajesticPyramid([
                            PyramidColor::Black,
                            PyramidColor::Black,
                            PyramidColor::Black,
                            PyramidColor::Black,
                            PyramidColor::Black
                        ]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::Shrine(God::Artemis, [PyramidColor::White, PyramidColor::Black, PyramidColor::White]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::MajorShrine(
                            God::Athena,
                            [PyramidColor::White, PyramidColor::Black, PyramidColor::White]
                        ),
                        allowed: true
                    }
                ]
            );

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(
                episode_2.pyramid_settings,
                vec![
                    PyramidSetting {
                        pyramid_type: PyramidConfig::ModestPyramid([PyramidColor::White, PyramidColor::White]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::Pyramid([PyramidColor::Black, PyramidColor::Black, PyramidColor::Black]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::GreatPyramid([
                            PyramidColor::White,
                            PyramidColor::White,
                            PyramidColor::White,
                            PyramidColor::White
                        ]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::MajesticPyramid([
                            PyramidColor::Black,
                            PyramidColor::Black,
                            PyramidColor::Black,
                            PyramidColor::Black,
                            PyramidColor::Black
                        ]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::Shrine(God::Artemis, [PyramidColor::White, PyramidColor::Black, PyramidColor::White]),
                        allowed: true
                    },
                    PyramidSetting {
                        pyramid_type: PyramidConfig::MajorShrine(
                            God::Athena,
                            [PyramidColor::White, PyramidColor::Black, PyramidColor::White]
                        ),
                        allowed: true
                    }
                ]
            );

            let colony_episode = adventure.colony_episodes.get(0).expect("Colony episode");
            assert_eq!(colony_episode.pyramid_settings, vec![]);
        }

        return Ok(());
    }

    #[test]
    fn parse_the_odyssey() -> Result<()> {
        if let Ok(game_root) = std::env::var("ZEUS_HOME") {
            let adventure = Adventure::read_from(format!("{game_root}/Adventures/The Odyssey"))?;

            let episode_1 = adventure.parent_episodes.get(0).expect("Episode 1");
            assert_eq!(episode_1.pyramid_settings, vec![]);

            let episode_2 = adventure.parent_episodes.get(1).expect("Episode 2");
            assert_eq!(episode_2.pyramid_settings, vec![]);

            let episode_3 = adventure.parent_episodes.get(2).expect("Episode 3");
            assert_eq!(episode_3.pyramid_settings, vec![]);

            let episode_4 = adventure.parent_episodes.get(3).expect("Episode 4");
            assert_eq!(episode_4.pyramid_settings, vec![]);

            let episode_5 = adventure.parent_episodes.get(4).expect("Episode 5");
            assert_eq!(episode_5.pyramid_settings, vec![]);

            let colony_episode_1 = adventure.colony_episodes.get(0).expect("Colony episode 1");
            assert_eq!(colony_episode_1.pyramid_settings, vec![]);

            let colony_episode_2 = adventure.colony_episodes.get(1).expect("Colony episode 2");
            assert_eq!(colony_episode_2.pyramid_settings, vec![]);
        }

        return Ok(());
    }
}
