use crate::utils::read_utils::ReadFrom;
use crate::utils::write_utils::WriteTo;
use my_macros::LogDifferences;
use std::io;
use std::io::Read;
use std::io::Write;

#[derive(Debug, PartialEq, Copy, Clone, Default, LogDifferences)]
pub struct EventData {
    pub id: u16,
    pub event_type: u8,
    pub month: u8,
    pub item_chosen: u16,
    pub first_item: u16,
    pub second_item: u16,
    pub third_item: u16,
    pub amount: u16,
    pub fixed_amount: u16,
    pub min_amount: u16,
    pub max_amount: u16,
    pub time: u16,
    pub fixed_time: u16,
    pub min_time: u16,
    pub max_time: u16,
    pub target: u16,
    pub fixed_target: u16,
    pub min_target: u16,
    pub max_target: u16,
    pub on_success: u16,
    pub on_failure: u16,
    pub flags: u32,
    pub warnings: u16,
    pub time_ctr: u16,
    pub status: u16,
    pub need_msg_res: u16,
    pub triggerer: u16,
    pub god_or_mon_or_warship_id: u16,
    pub mtar1: u16,
    pub mtar2: u16,
    pub mtar3: u16,
    pub magg: u16,
    pub unkown_row: [u16; 9],
    pub trigger_on_1: u16,
    pub trigger_on_2: u16,
    pub eff_on_city: u16,
    pub source: u16,
    pub source_fixed: u16,
    pub source_min: u16,
    pub source_max: u16,
    pub subtype: u16,
    pub prev_amount: u16,
    pub related_to_triggered_evt: u32,
    pub unknown_1: u16,
    pub trig_reason: u16,
    pub unknown_2: u16,
    pub unknown_3: u8,
    pub other_city: u8,
    pub loot_type: u16,
    pub loot_amount: u16,
    pub unknown_4: u8,
    pub ally_city: u8,
    pub ally_strength: u8,
    pub to_strength: u8,
    pub unknown_5: u16,
    pub quest: u8,
    pub tail: u8,
}

impl ReadFrom for EventData {
    fn read_from(reader: &mut impl Read) -> io::Result<Self> {
        return Ok(EventData {
            id: ReadFrom::read_from(reader)?,
            event_type: ReadFrom::read_from(reader)?,
            month: ReadFrom::read_from(reader)?,
            item_chosen: ReadFrom::read_from(reader)?,
            first_item: ReadFrom::read_from(reader)?,
            second_item: ReadFrom::read_from(reader)?,
            third_item: ReadFrom::read_from(reader)?,
            amount: ReadFrom::read_from(reader)?,
            fixed_amount: ReadFrom::read_from(reader)?,
            min_amount: ReadFrom::read_from(reader)?,
            max_amount: ReadFrom::read_from(reader)?,
            time: ReadFrom::read_from(reader)?,
            fixed_time: ReadFrom::read_from(reader)?,
            min_time: ReadFrom::read_from(reader)?,
            max_time: ReadFrom::read_from(reader)?,
            target: ReadFrom::read_from(reader)?,
            fixed_target: ReadFrom::read_from(reader)?,
            min_target: ReadFrom::read_from(reader)?,
            max_target: ReadFrom::read_from(reader)?,
            on_success: ReadFrom::read_from(reader)?,
            on_failure: ReadFrom::read_from(reader)?,
            flags: ReadFrom::read_from(reader)?,
            warnings: ReadFrom::read_from(reader)?,
            time_ctr: ReadFrom::read_from(reader)?,
            status: ReadFrom::read_from(reader)?,
            need_msg_res: ReadFrom::read_from(reader)?,
            triggerer: ReadFrom::read_from(reader)?,
            god_or_mon_or_warship_id: ReadFrom::read_from(reader)?,
            mtar1: ReadFrom::read_from(reader)?,
            mtar2: ReadFrom::read_from(reader)?,
            mtar3: ReadFrom::read_from(reader)?,
            magg: ReadFrom::read_from(reader)?,
            unkown_row: ReadFrom::read_from(reader)?,
            trigger_on_1: ReadFrom::read_from(reader)?,
            trigger_on_2: ReadFrom::read_from(reader)?,
            eff_on_city: ReadFrom::read_from(reader)?,
            source: ReadFrom::read_from(reader)?,
            source_fixed: ReadFrom::read_from(reader)?,
            source_min: ReadFrom::read_from(reader)?,
            source_max: ReadFrom::read_from(reader)?,
            subtype: ReadFrom::read_from(reader)?,
            prev_amount: ReadFrom::read_from(reader)?,
            related_to_triggered_evt: ReadFrom::read_from(reader)?,
            unknown_1: ReadFrom::read_from(reader)?,
            trig_reason: ReadFrom::read_from(reader)?,
            unknown_2: ReadFrom::read_from(reader)?,
            unknown_3: ReadFrom::read_from(reader)?,
            other_city: ReadFrom::read_from(reader)?,
            loot_type: ReadFrom::read_from(reader)?,
            loot_amount: ReadFrom::read_from(reader)?,
            unknown_4: ReadFrom::read_from(reader)?,
            ally_city: ReadFrom::read_from(reader)?,
            ally_strength: ReadFrom::read_from(reader)?,
            to_strength: ReadFrom::read_from(reader)?,
            unknown_5: ReadFrom::read_from(reader)?,
            quest: ReadFrom::read_from(reader)?,
            tail: ReadFrom::read_from(reader)?,
        });
    }
}

impl WriteTo for EventData {
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<usize> {
        let mut bytes = 0;

        bytes += WriteTo::write_to(&self.id, writer)?;
        bytes += WriteTo::write_to(&self.event_type, writer)?;
        bytes += WriteTo::write_to(&self.month, writer)?;
        bytes += WriteTo::write_to(&self.item_chosen, writer)?;
        bytes += WriteTo::write_to(&self.first_item, writer)?;
        bytes += WriteTo::write_to(&self.second_item, writer)?;
        bytes += WriteTo::write_to(&self.third_item, writer)?;
        bytes += WriteTo::write_to(&self.amount, writer)?;
        bytes += WriteTo::write_to(&self.fixed_amount, writer)?;
        bytes += WriteTo::write_to(&self.min_amount, writer)?;
        bytes += WriteTo::write_to(&self.max_amount, writer)?;
        bytes += WriteTo::write_to(&self.time, writer)?;
        bytes += WriteTo::write_to(&self.fixed_time, writer)?;
        bytes += WriteTo::write_to(&self.min_time, writer)?;
        bytes += WriteTo::write_to(&self.max_time, writer)?;
        bytes += WriteTo::write_to(&self.target, writer)?;
        bytes += WriteTo::write_to(&self.fixed_target, writer)?;
        bytes += WriteTo::write_to(&self.min_target, writer)?;
        bytes += WriteTo::write_to(&self.max_target, writer)?;
        bytes += WriteTo::write_to(&self.on_success, writer)?;
        bytes += WriteTo::write_to(&self.on_failure, writer)?;
        bytes += WriteTo::write_to(&self.flags, writer)?;
        bytes += WriteTo::write_to(&self.warnings, writer)?;
        bytes += WriteTo::write_to(&self.time_ctr, writer)?;
        bytes += WriteTo::write_to(&self.status, writer)?;
        bytes += WriteTo::write_to(&self.need_msg_res, writer)?;
        bytes += WriteTo::write_to(&self.triggerer, writer)?;
        bytes += WriteTo::write_to(&self.god_or_mon_or_warship_id, writer)?;
        bytes += WriteTo::write_to(&self.mtar1, writer)?;
        bytes += WriteTo::write_to(&self.mtar2, writer)?;
        bytes += WriteTo::write_to(&self.mtar3, writer)?;
        bytes += WriteTo::write_to(&self.magg, writer)?;
        bytes += WriteTo::write_to(&self.unkown_row, writer)?;
        bytes += WriteTo::write_to(&self.trigger_on_1, writer)?;
        bytes += WriteTo::write_to(&self.trigger_on_2, writer)?;
        bytes += WriteTo::write_to(&self.eff_on_city, writer)?;
        bytes += WriteTo::write_to(&self.source, writer)?;
        bytes += WriteTo::write_to(&self.source_fixed, writer)?;
        bytes += WriteTo::write_to(&self.source_min, writer)?;
        bytes += WriteTo::write_to(&self.source_max, writer)?;
        bytes += WriteTo::write_to(&self.subtype, writer)?;
        bytes += WriteTo::write_to(&self.prev_amount, writer)?;
        bytes += WriteTo::write_to(&self.related_to_triggered_evt, writer)?;
        bytes += WriteTo::write_to(&self.unknown_1, writer)?;
        bytes += WriteTo::write_to(&self.trig_reason, writer)?;
        bytes += WriteTo::write_to(&self.unknown_2, writer)?;
        bytes += WriteTo::write_to(&self.unknown_3, writer)?;
        bytes += WriteTo::write_to(&self.other_city, writer)?;
        bytes += WriteTo::write_to(&self.loot_type, writer)?;
        bytes += WriteTo::write_to(&self.loot_amount, writer)?;
        bytes += WriteTo::write_to(&self.unknown_4, writer)?;
        bytes += WriteTo::write_to(&self.ally_city, writer)?;
        bytes += WriteTo::write_to(&self.ally_strength, writer)?;
        bytes += WriteTo::write_to(&self.to_strength, writer)?;
        bytes += WriteTo::write_to(&self.unknown_5, writer)?;
        bytes += WriteTo::write_to(&self.quest, writer)?;
        bytes += WriteTo::write_to(&self.tail, writer)?;

        return Ok(bytes);
    }
}
