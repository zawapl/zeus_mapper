use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::to_usize;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Default, LogDifferences)]
pub struct ManifestData {
    pub compressed: u32,
    pub address: u32,
    pub size: u32,
    pub count: u32,
    pub unknown_1: u32,
}

impl ManifestData {
    pub fn default_map_manifest() -> BoxedArray<ManifestData, 300> {
        return map_manifest_template(321);
    }

    pub fn default_sav_manifest() -> BoxedArray<ManifestData, 300> {
        return sav_manifest_template(241, 139);
    }

    pub fn validate_map_manifest(manifest: &BoxedArray<ManifestData, 300>, version_1: u32) -> Result<(), String> {
        let expected = map_manifest_template(version_1);
        return validate_against_template(manifest, &expected);
    }

    pub fn validate_sav_manifest(manifest: &BoxedArray<ManifestData, 300>, version_1: u32, version_2: u32) -> Result<(), String> {
        let expected = sav_manifest_template(version_1, version_2);
        return validate_against_template(manifest, &expected);
    }

    fn set(&mut self, compressed: bool, size: u32, count: u32) {
        self.compressed = if compressed { 1 } else { 0 };
        self.size = size;
        self.count = count;
    }
}

impl ReadFrom for ManifestData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(ManifestData {
            compressed: ReadFrom::read_from(reader)?,
            address: ReadFrom::read_from(reader)?,
            size: ReadFrom::read_from(reader)?,
            count: ReadFrom::read_from(reader)?,
            unknown_1: ReadFrom::read_from(reader)?,
        });
    }
}

pub fn read_segment(reader: &mut impl Read, manifest_element: &ManifestData) -> io::Result<Vec<Vec<u8>>> {
    let mut result = Vec::with_capacity(to_usize(manifest_element.count)?);

    for _ in 0..manifest_element.count {
        let mut row = vec![0; to_usize(manifest_element.size)?];
        reader.read_exact(row.as_mut_slice())?;
        result.push(row);
    }

    return Ok(result);
}

impl WriteTo for ManifestData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;
        bytes += WriteTo::write_to(&self.compressed, writer)?;
        bytes += WriteTo::write_to(&self.address, writer)?;
        bytes += WriteTo::write_to(&self.size, writer)?;
        bytes += WriteTo::write_to(&self.count, writer)?;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        return Ok(bytes);
    }
}

fn map_manifest_template(version_1: u32) -> BoxedArray<ManifestData, 300> {
    let new_format = (318..=321).contains(&version_1);
    let new_version = version_1 >= 241;

    let mut manifest = [ManifestData::default(); 300];

    manifest[0].set(false, 4, 1);
    manifest[1].set(false, 4, 1);
    manifest[2].set(false, 4, 1);
    manifest[3].set(false, 20, 300);
    manifest[4].set(true, 4, 51984);
    manifest[5].set(true, 1, 51984);
    manifest[6].set(true, 4, 51984);
    manifest[7].set(true, 1, 51984);
    manifest[8].set(true, 1, 51984);
    manifest[9].set(true, 1, 51984);
    manifest[10].set(false, 4, 1);
    manifest[11].set(false, 4, 1);
    manifest[12].set(false, 4, 1);
    manifest[13].set(false, 4, 1);
    manifest[14].set(false, 2032, 1);
    manifest[15].set(true, 1, 51984);
    manifest[16].set(false, 18628, 1);
    manifest[17].set(true, if new_format { 72 } else { 40 }, 200);
    manifest[18].set(true, 324, 232);
    manifest[19].set(true, 1, 51984);
    manifest[20].set(true, 36, 1);
    manifest[21].set(false, 4, 36);
    manifest[22].set(true, 1, 51984);
    manifest[23].set(true, 1, 51984);
    manifest[24].set(true, 1, 51984);
    manifest[25].set(true, if new_format { 572 } else { 476 }, 22);
    manifest[26].set(false, 4, 1);
    manifest[27].set(false, if new_version { 52 } else { 40 }, if new_format { 14 } else { 12 });
    manifest[28].set(false, if new_format { 300 } else { 224 }, 1);
    manifest[29].set(false, 76, 6);
    manifest[30].set(false, 4, 1);
    manifest[31].set(false, 4, 1);
    manifest[32].set(false, 36, 10);

    return BoxedArray(Box::new(manifest));
}

fn sav_manifest_template(version_1: u32, version_2: u32) -> BoxedArray<ManifestData, 300> {
    let is_poseidon = version_2 == 139;
    let new_version = version_1 >= 241;

    let mut manifest = [ManifestData::default(); 300];

    manifest[0].set(false, 4, 1);
    manifest[1].set(false, 4, 1);
    manifest[2].set(false, 20, 300);
    manifest[3].set(false, 1, 1);
    manifest[4].set(true, 19184, 1);
    manifest[5].set(false, 4, 1);
    manifest[6].set(false, 4, 1);
    manifest[7].set(true, if is_poseidon { 572 } else { 476 }, 22);
    manifest[8].set(false, 4, 36);
    manifest[9].set(false, 2032, 1);
    manifest[10].set(true, if is_poseidon { 72 } else { 40 }, 200);
    manifest[11].set(false, 1, 1);
    manifest[12].set(false, 4, 1);
    manifest[13].set(false, 124, 150);
    manifest[14].set(false, 4, 1);
    manifest[15].set(true, 1, 51984);
    manifest[16].set(true, 2, 51984);
    manifest[17].set(true, 4, 51984);
    manifest[18].set(true, 1, 51984);
    manifest[19].set(true, 2, 51984);
    manifest[20].set(true, 1, 51984);
    manifest[21].set(true, 1, 51984);
    manifest[22].set(false, 1, 51984);
    manifest[23].set(true, 1, 51984);
    manifest[24].set(true, 1, 51984);
    manifest[25].set(true, 2, 51984);
    manifest[26].set(true, 1, 51984);
    manifest[27].set(true, 1, 51984);
    manifest[28].set(true, 388, 2000);
    manifest[29].set(true, 2, 1000);
    manifest[30].set(true, 1, 500000);
    manifest[31].set(true, 156, 100);
    manifest[32].set(false, 4, 1);
    manifest[33].set(false, if new_version { 1 } else { 4 }, 1);
    manifest[34].set(false, 32, 2);
    manifest[35].set(true, 280, 4000);
    manifest[36].set(false, 4, 1);
    manifest[37].set(false, 4, 1);
    manifest[38].set(false, 4, 1);
    manifest[39].set(false, 4, 1);
    manifest[40].set(false, 4, 1);
    manifest[41].set(false, 4, 1);
    manifest[42].set(false, 4, 1);
    manifest[43].set(false, 4, 1);
    manifest[44].set(false, 4, 1);
    manifest[45].set(false, 4, 1);
    manifest[46].set(false, 4, 1);
    manifest[47].set(false, 4, 1);
    manifest[48].set(false, 4, 1);
    manifest[49].set(false, 4, 1);
    manifest[50].set(false, 4, 1);
    manifest[51].set(false, 4, 36);
    manifest[52].set(false, 4, 36);
    manifest[53].set(false, 4, 1);
    manifest[54].set(true, 60, 1000);
    manifest[55].set(false, 4, 1);
    manifest[56].set(false, 4, 1);
    manifest[57].set(false, 4, 1);
    manifest[58].set(false, 1, 10);
    manifest[59].set(false, 4, 20);
    manifest[60].set(false, 4, 20);
    manifest[61].set(false, 4, 1);
    manifest[62].set(false, 4, 1);
    manifest[63].set(false, 4, 1);
    manifest[64].set(false, 4, 1);
    manifest[65].set(false, 4, 1);
    manifest[66].set(false, 4, 1);
    manifest[67].set(false, 128, 70);
    manifest[68].set(false, 4, 1);
    manifest[69].set(false, 88, 100);
    manifest[70].set(false, 4, 1);
    manifest[71].set(true, 2, 500);
    manifest[72].set(true, 2, 500);
    manifest[73].set(true, 2, 4000);
    manifest[74].set(false, 4, 1);
    manifest[75].set(false, 4, 1);
    manifest[76].set(false, 4, 1);
    manifest[77].set(false, 4, 1);
    manifest[78].set(false, 4, 1);
    manifest[79].set(false, 4, 1);
    manifest[80].set(false, 4, 1);
    manifest[81].set(false, 268, 200);
    manifest[82].set(false, 4, 1);
    manifest[83].set(false, 4, 1);
    manifest[84].set(false, 4, 1);
    manifest[85].set(false, 4, 1);
    manifest[86].set(false, 4, 1);
    manifest[87].set(false, 4, 1);
    manifest[88].set(false, 4, 1);
    manifest[89].set(false, 4, 1);
    manifest[90].set(false, 4, 1);
    manifest[91].set(false, 4, 1);
    manifest[92].set(false, 2, 1);
    manifest[93].set(false, 4, 1);
    manifest[94].set(false, 4, 1);
    manifest[95].set(false, 65, 1);
    manifest[96].set(false, 8, 4);
    manifest[97].set(false, 4, 1);
    manifest[98].set(false, 4, 1);
    manifest[99].set(false, 1, 51984);
    manifest[100].set(false, 4, 4);
    manifest[101].set(true, 324, 232);
    manifest[102].set(false, 1, 51984);
    manifest[103].set(false, 32, 1);
    manifest[104].set(true, 36, 1);
    manifest[105].set(true, 4, 51984);
    manifest[106].set(false, 1, 26);
    manifest[107].set(false, 1, 13);
    manifest[108].set(true, 1, 51984);
    manifest[109].set(true, 1, 51984);
    manifest[110].set(false, 4, 1);
    manifest[111].set(true, 1, 51984);
    manifest[112].set(true, 1, 51984);
    manifest[113].set(true, 4, if is_poseidon { 367 } else { 334 });
    manifest[114].set(false, 4, 1);
    manifest[115].set(false, 28, 8);
    manifest[116].set(false, 1, if is_poseidon { 75 } else { 40 });
    manifest[117].set(false, 4, 1);
    manifest[118].set(false, 1, 1);
    manifest[119].set(false, 4, 900);
    manifest[120].set(false, if new_version { 52 } else { 40 }, if is_poseidon { 14 } else { 12 });
    manifest[121].set(false, 8, 32);
    manifest[122].set(false, if is_poseidon { 300 } else { 224 }, 1);
    manifest[123].set(false, if is_poseidon { 32 } else { 12 }, if is_poseidon { 18 } else { 14 });
    manifest[124].set(false, 28, if is_poseidon { 8 } else { 6 });
    manifest[125].set(false, 200, 1);
    manifest[126].set(false, 1, if is_poseidon { 221 } else { 199 });
    manifest[127].set(false, 1, 17);
    manifest[128].set(false, 76, 6);
    manifest[129].set(false, 4, 1);
    manifest[130].set(false, 1, if is_poseidon { 75 } else { 40 });
    manifest[131].set(false, 4, 1);
    manifest[132].set(false, 112, 4);
    manifest[133].set(false, 4, 1);
    manifest[134].set(false, 16, 2000);
    manifest[135].set(false, 4, 1);
    manifest[136].set(false, 36, 10);
    manifest[137].set(false, 1, 1);
    manifest[138].set(false, if is_poseidon { 4 } else { 0 }, if is_poseidon { 1 } else { 0 });

    return BoxedArray(Box::new(manifest));
}

fn validate_against_template(manifest: &BoxedArray<ManifestData, 300>, expected: &BoxedArray<ManifestData, 300>) -> Result<(), String> {
    for (i, (entry, expected_entry)) in manifest.iter().zip(expected.iter()).enumerate() {
        if (entry.compressed > 0 && expected_entry.compressed == 0)
            || (entry.size != expected_entry.size)
            || (entry.count != expected_entry.count)
        {
            return Err(format!(
                "manifest[{i}] is compressed={}, size={}, count={} - expected compressed={}, size={}, count={}",
                entry.compressed, entry.size, entry.count, expected_entry.compressed, expected_entry.size, expected_entry.count
            ));
        }
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write() -> io::Result<()> {
        let original = ManifestData {
            compressed: 1,
            address: 2,
            size: 3,
            count: 4,
            unknown_1: 5,
        };

        let mut buffer = vec![];

        original.write_to(&mut buffer)?;

        let deserialized = ManifestData::read_from(&mut Cursor::new(buffer))?;

        assert_eq!(original, deserialized);

        return Ok(());
    }
}
