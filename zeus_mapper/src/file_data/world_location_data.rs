use crate::pkware::explode;
use crate::utils::boxed_array::BoxedArray;
use crate::utils::read_utils::ReadFrom;
use crate::utils::read_utils::read_string_from;
use crate::utils::read_utils::read_vec_from;
use crate::utils::read_utils::to_usize;
use crate::utils::validation::ValidationError;
use crate::utils::validation::ValidationResult;
use crate::utils::validation::validate_expected_constant;
use crate::utils::write_utils::WriteTo;
use crate::utils::write_utils::write_maybe_compressed;
use crate::utils::write_utils::write_string_to;
use my_macros::LogDifferences;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, LogDifferences)]
pub struct WorldLocationData {
    pub exists: u16,
    pub location_type: u8,
    pub name: u8,
    pub slot_index: u8,
    pub unknown_1: BoxedArray<u8, 91>,
    pub trade_quantities: BoxedArray<u8, 37>,
    pub unknown_2: BoxedArray<u8, 55>,
    pub buying: [i8; 8],
    pub selling: [i8; 8],
    pub civilization: u32,
    pub leader_name: u32,
    pub attitude: u32,
    pub economical_strength: u32,
    pub military_strength: u32,
    pub tribute: u32,
    pub tribute_rec_amount: u32,
    pub tribute_pay_amount: u32,
    pub tribute_pay_resource: i16,
    pub tribute_rec_resource: i16,
    pub unknown_3: BoxedArray<u8, 100>,
    pub favour: u8,
    pub constant_1_0x00: BoxedArray<u8, 11>,
    // See `resolve_active` for how this and `active_new` combine. Not read for `ParentCity`. Top
    // 3 bytes confirmed always 0 across every real record sampled, so this is a single
    // little-endian u32 (matching a C `BOOL`/`int` field's natural width) rather than a flag byte
    // plus 3 bytes of padding.
    pub active_old: u32,
    pub favour_or_visibility: u32, // depending on the version it might be one of the other
    pub constant_3_0x00: [u8; 8],
    pub active_new: u32,
    pub visible: u32,
    pub unknown_4: BoxedArray<u8, 100>,
    pub unknown_5: Vec<u8>,
    pub trade_route_visible: u8,
    pub custom_name: String,
    pub custom_leader_name: String,
    pub tail: Vec<u8>,
}

impl Default for WorldLocationData {
    fn default() -> Self {
        return WorldLocationData {
            exists: 0,
            location_type: 0,
            name: 0,
            slot_index: 0,
            unknown_1: Default::default(),
            trade_quantities: Default::default(),
            unknown_2: Default::default(),
            buying: [0; 8],
            selling: [0; 8],
            civilization: 0,
            leader_name: 0,
            attitude: 0,
            economical_strength: 0,
            military_strength: 0,
            tribute: 0,
            tribute_rec_amount: 0,
            tribute_pay_amount: 0,
            tribute_pay_resource: 0,
            tribute_rec_resource: 0,
            unknown_3: Default::default(),
            favour: 0,
            constant_1_0x00: Default::default(),
            active_old: 0,
            favour_or_visibility: 0,
            constant_3_0x00: Default::default(),
            active_new: 0,
            visible: 0,
            unknown_4: Default::default(),
            unknown_5: vec![0; 28],
            trade_route_visible: 4,
            custom_name: String::new(),
            custom_leader_name: String::new(),
            tail: vec![0; 3],
        };
    }
}

impl WorldLocationData {
    pub fn validate(&self) -> ValidationResult {
        validate_expected_constant("constant_1_0x00", self.constant_1_0x00.as_ref(), 0)?;

        // todo check in game if values in higher bytes actually affect how the game interprets this
        if self.active_old > 0xFF {
            return Err(ValidationError::expected_range("active_old", self.active_old, 0, 0xFF));
        }

        validate_expected_constant("constant_3_0x00", &self.constant_3_0x00, 0)?;

        return Ok(());
    }

    pub(crate) fn read_arr_from(reader: &mut impl Read, include_extras: bool) -> io::Result<BoxedArray<Self, 22>> {
        let mut result = Vec::with_capacity(22);

        let compressed_size = i32::read_from(reader)?;

        if compressed_size < 0 {
            for _ in 0..22 {
                result.push(Self::read_from(reader, include_extras)?);
            }
        } else {
            let mut compressed = vec![0; to_usize(compressed_size)?];
            reader.read_exact(&mut compressed)?;
            let mut decompressed_reader = Cursor::new(explode(&compressed)?);

            for _ in 0..22 {
                result.push(Self::read_from(&mut decompressed_reader, include_extras)?);
            }
        }

        return Ok(BoxedArray::from_vec(result));
    }

    fn read_from(reader: &mut impl Read, include_extras: bool) -> io::Result<Self> {
        let unknown_476_length = if include_extras { 28 } else { 0 };
        let strings_length = if include_extras { 32 } else { 0 };
        let tail_length = if include_extras { 3 } else { 0 };

        return Ok(WorldLocationData {
            exists: ReadFrom::read_from(reader)?,
            location_type: ReadFrom::read_from(reader)?,
            name: ReadFrom::read_from(reader)?,
            slot_index: ReadFrom::read_from(reader)?,
            unknown_1: ReadFrom::read_from(reader)?,
            trade_quantities: ReadFrom::read_from(reader)?,
            unknown_2: ReadFrom::read_from(reader)?,
            buying: ReadFrom::read_from(reader)?,
            selling: ReadFrom::read_from(reader)?,
            civilization: ReadFrom::read_from(reader)?,
            leader_name: ReadFrom::read_from(reader)?,
            attitude: ReadFrom::read_from(reader)?,
            economical_strength: ReadFrom::read_from(reader)?,
            military_strength: ReadFrom::read_from(reader)?,
            tribute: ReadFrom::read_from(reader)?,
            tribute_rec_amount: ReadFrom::read_from(reader)?,
            tribute_pay_amount: ReadFrom::read_from(reader)?,
            tribute_pay_resource: ReadFrom::read_from(reader)?,
            tribute_rec_resource: ReadFrom::read_from(reader)?,
            unknown_3: ReadFrom::read_from(reader)?,
            favour: ReadFrom::read_from(reader)?,
            constant_1_0x00: ReadFrom::read_from(reader)?,
            active_old: ReadFrom::read_from(reader)?,
            favour_or_visibility: ReadFrom::read_from(reader)?,
            constant_3_0x00: ReadFrom::read_from(reader)?,
            active_new: ReadFrom::read_from(reader)?,
            visible: ReadFrom::read_from(reader)?,
            unknown_4: ReadFrom::read_from(reader)?,
            unknown_5: read_vec_from(reader, unknown_476_length)?,
            trade_route_visible: if include_extras { ReadFrom::read_from(reader)? } else { 4 },
            custom_name: read_string_from(reader, strings_length)?,
            custom_leader_name: read_string_from(reader, strings_length)?,
            tail: read_vec_from(reader, tail_length)?,
        });
    }
    pub(crate) fn write_arr_to<W: Write, const N: usize>(
        data: &BoxedArray<Self, N>,
        writer: &mut W,
        include_extras: bool,
    ) -> io::Result<usize> {
        let mut uncompressed = Cursor::new(vec![]);

        for world_location_data in data.iter() {
            world_location_data.write_to(&mut uncompressed, include_extras)?;
        }

        return write_maybe_compressed(uncompressed.get_ref(), writer);
    }

    fn write_to<W: Write>(&self, writer: &mut W, include_extras: bool) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.exists, writer)?;
        bytes += WriteTo::write_to(&self.location_type, writer)?;
        bytes += WriteTo::write_to(&self.name, writer)?;
        bytes += WriteTo::write_to(&self.slot_index, writer)?;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.trade_quantities, writer)?;
        bytes += WriteTo::write_to(&self.unknown_2, writer)?;
        bytes += WriteTo::write_to(&self.buying, writer)?;
        bytes += WriteTo::write_to(&self.selling, writer)?;
        bytes += WriteTo::write_to(&self.civilization, writer)?;
        bytes += WriteTo::write_to(&self.leader_name, writer)?;
        bytes += WriteTo::write_to(&self.attitude, writer)?;
        bytes += WriteTo::write_to(&self.economical_strength, writer)?;
        bytes += WriteTo::write_to(&self.military_strength, writer)?;
        bytes += WriteTo::write_to(&self.tribute, writer)?;
        bytes += WriteTo::write_to(&self.tribute_rec_amount, writer)?;
        bytes += WriteTo::write_to(&self.tribute_pay_amount, writer)?;
        bytes += WriteTo::write_to(&self.tribute_pay_resource, writer)?;
        bytes += WriteTo::write_to(&self.tribute_rec_resource, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
        bytes += WriteTo::write_to(&self.favour, writer)?;
        bytes += WriteTo::write_to(&self.constant_1_0x00, writer)?;
        bytes += WriteTo::write_to(&self.active_old, writer)?;
        bytes += WriteTo::write_to(&self.favour_or_visibility, writer)?;
        bytes += WriteTo::write_to(&self.constant_3_0x00, writer)?;
        bytes += WriteTo::write_to(&self.active_new, writer)?;
        bytes += WriteTo::write_to(&self.visible, writer)?;
        bytes += WriteTo::write_to(&self.unknown_4, writer)?;

        if include_extras {
            bytes += WriteTo::write_to(&self.unknown_5, writer)?;
            bytes += WriteTo::write_to(&self.trade_route_visible, writer)?;
            bytes += write_string_to(&self.custom_name, writer, 32)?;
            bytes += write_string_to(&self.custom_leader_name, writer, 32)?;
            bytes += WriteTo::write_to(&self.tail, writer)?;
        }

        return Ok(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::io::Cursor;

    #[test]
    fn read_write_without_extras() -> io::Result<()> {
        let original = base();

        let mut buffer = vec![];

        original.write_to(&mut buffer, false)?;

        let deserialized = WorldLocationData::read_from(&mut Cursor::new(buffer), false)?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    #[test]
    fn read_write_with_extras() -> io::Result<()> {
        let mut original = base();
        original.unknown_5 = vec![24; 28];
        original.trade_route_visible = 25;
        original.custom_name = "Custom Name".to_string();
        original.custom_leader_name = "Custom Leader".to_string();
        original.tail = vec![26; 3];

        let mut buffer = vec![];

        original.write_to(&mut buffer, true)?;

        let deserialized = WorldLocationData::read_from(&mut Cursor::new(buffer), true)?;

        assert_eq!(original, deserialized);

        return Ok(());
    }

    fn base() -> WorldLocationData {
        return WorldLocationData {
            exists: 1,
            location_type: 2,
            name: 3,
            slot_index: 3,
            unknown_1: BoxedArray::from_vec(vec![3; 91]),
            trade_quantities: BoxedArray::from_vec(vec![4; 37]),
            unknown_2: BoxedArray::from_vec(vec![5; 55]),
            buying: [6; 8],
            selling: [7; 8],
            civilization: 8,
            leader_name: 9,
            attitude: 10,
            economical_strength: 11,
            military_strength: 12,
            tribute: 13,
            tribute_rec_amount: 14,
            tribute_pay_amount: 15,
            tribute_pay_resource: 16,
            tribute_rec_resource: 17,
            unknown_3: BoxedArray::from_vec(vec![18; 100]),
            favour: 18,
            constant_1_0x00: BoxedArray::from_vec(vec![18; 11]),
            active_old: 18,
            favour_or_visibility: 19,
            constant_3_0x00: [20; 8],
            active_new: 21,
            visible: 22,
            unknown_4: BoxedArray::from_vec(vec![23; 100]),
            unknown_5: vec![],
            trade_route_visible: 4,
            custom_name: "".to_string(),
            custom_leader_name: "".to_string(),
            tail: vec![],
        };
    }
}
