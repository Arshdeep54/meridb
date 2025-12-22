use serde::{Deserialize, Serialize};
use sql::ast::ASTValue;

use crate::{record::serialize_record_for_page, types::Column};

use super::record::Record;
use std::collections::HashMap;

pub const PAGE_SIZE: usize = 8192; // 8KB page size
pub const HEADER_LEN: usize = 18; // see PageHeader::write_into
pub const SLOT_LEN: usize = 5; // off u16 + len u16 + flags u8

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub id: u32,
    pub records: HashMap<u64, Record>,
    pub next_page: Option<u32>,
    pub prev_page: Option<u32>,
    pub free_space: usize,
}

impl Page {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            records: HashMap::new(),
            next_page: None,
            prev_page: None,
            free_space: PAGE_SIZE,
        }
    }

    pub fn insert_record(&mut self, record: Record) -> Result<(), String> {
        let record_size = self.calculate_record_size(&record);

        if self.free_space < record_size {
            return Err("Not enough space in page".to_string());
        }

        self.records.insert(record.id, record);
        self.free_space -= record_size;
        Ok(())
    }

    pub fn delete_record(&mut self, record_id: u64) -> Option<Record> {
        if let Some(record) = self.records.remove(&record_id) {
            self.free_space += self.calculate_record_size(&record);
            Some(record)
        } else {
            None
        }
    }

    pub fn get_record(&self, record_id: u64) -> Option<&Record> {
        self.records.get(&record_id)
    }

    pub fn get_record_mut(&mut self, record_id: u64) -> Option<&mut Record> {
        self.records.get_mut(&record_id)
    }

    fn calculate_record_size(&self, record: &Record) -> usize {
        // Simplified size calculation
        // In a real implementation, this would need to account for actual serialized size
        std::mem::size_of::<u64>() + // record id
        record.data.len() * (std::mem::size_of::<String>() + std::mem::size_of::<ASTValue>()) +
        std::mem::size_of::<u64>() // timestamp
    }

    pub fn is_full(&self, required_space: usize) -> bool {
        self.free_space < required_space
    }
}

struct PageHeader {
    magic: [u8; 4],      // b"HPG0"
    version: u32,        // 1
    page_id: u32,        // your in-memory page id
    record_count: u16,   // number of slots
    free_space_off: u16, // offset where free space starts
    flags: u16,          // reserved
}

impl PageHeader {
    fn new(page_id: u32) -> Self {
        Self {
            magic: *b"HPG0",
            version: 1,
            page_id,
            record_count: 0,
            free_space_off: 0,
            flags: 0,
        }
    }

    fn write_into(&self, buf: &mut [u8]) {
        // layout: magic[4] | version u32 | page_id u32 | record_count u16 | free_space_off u16 | flags u16
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..8].copy_from_slice(&self.version.to_le_bytes());
        buf[8..12].copy_from_slice(&self.page_id.to_le_bytes());
        buf[12..14].copy_from_slice(&self.record_count.to_le_bytes());
        buf[14..16].copy_from_slice(&self.free_space_off.to_le_bytes());
        buf[16..18].copy_from_slice(&self.flags.to_le_bytes());
    }
}

// Slot directory entry (end of page, growing downward)
#[derive(Debug, Clone, Copy)]
struct Slot {
    off: u16,
    len: u16,
    flags: u8, // 0 = visible, 1 = tombstone (reserved)
}

impl Slot {
    fn write_into(&self, buf: &mut [u8]) {
        buf[0..2].copy_from_slice(&self.off.to_le_bytes());
        buf[2..4].copy_from_slice(&self.len.to_le_bytes());
        buf[4] = self.flags;
    }
}

impl Page {
    // Serialize the current in-memory page (records) into a fixed 8 KiB page with a heap layout.
    // Pack records sequentially into the payload and write a slot directory at the end.
    // Assumes self.records are already sized to fit (your free_space tracking should guarantee).
    pub fn to_bytes(&self, columns: &[Column]) -> Result<[u8; PAGE_SIZE], String> {
        let mut page = [0u8; PAGE_SIZE];

        let mut hdr = PageHeader::new(self.id);

        let mut ids: Vec<u64> = self.records.keys().copied().collect();
        ids.sort_unstable();

        let mut payload_off = HEADER_LEN;
        let mut slot_dir_end = PAGE_SIZE;
        let mut slots_written = 0usize;

        for rid in ids {
            let rec = self.records.get(&rid).expect("record disappeared");

            let payload = serialize_record_for_page(rec, columns)
                .map_err(|e| format!("serialize record {} failed: {}", rid, e))?;
            let plen = payload.len();

            let need = plen + SLOT_LEN;
            if payload_off + need > slot_dir_end {
                return Err(format!(
                    "page {} overflow while writing record {}; need {}, have {}",
                    self.id,
                    rid,
                    need,
                    slot_dir_end.saturating_sub(payload_off)
                ));
            }

            let start = payload_off;
            let end = start + plen;
            page[start..end].copy_from_slice(&payload);
            payload_off = end;

            slot_dir_end -= SLOT_LEN;
            let slot = Slot {
                off: start as u16,
                len: plen as u16,
                flags: 0,
            };
            slot.write_into(&mut page[slot_dir_end..slot_dir_end + SLOT_LEN]);

            slots_written += 1;
        }

        hdr.record_count = u16::try_from(slots_written).unwrap_or(u16::MAX);
        hdr.free_space_off = payload_off as u16;

        hdr.write_into(&mut page[0..HEADER_LEN]);

        Ok(page)
    }
}

#[derive(Clone, Copy)]
struct ReadHeader {
    record_count: u16,
}

pub fn iter_slots(buf: &[u8]) -> Result<impl Iterator<Item = (u16, u16, u8)> + '_, String> {
    let hdr = read_header(buf)?;
    let rc = hdr.record_count as usize;
    let slot_dir_start = PAGE_SIZE
        .checked_sub(rc * SLOT_LEN)
        .ok_or("slot calc overflow")?;
    if buf.len() < PAGE_SIZE {
        return Err("page buffer too small".into());
    }
    if slot_dir_start < HEADER_LEN {
        return Err("corrupt page (slot_dir overlaps header)".into());
    }

    Ok((0..rc).map(move |i| {
        let off = slot_dir_start + i * SLOT_LEN;
        let o = u16::from_le_bytes(buf[off..off + 2].try_into().unwrap());
        let l = u16::from_le_bytes(buf[off + 2..off + 4].try_into().unwrap());
        let flags = buf[off + 4];
        (o, l, flags)
    }))
}

fn read_header(buf: &[u8]) -> Result<ReadHeader, String> {
    if buf.len() < HEADER_LEN {
        return Err("page too small".into());
    }
    if &buf[0..4] != b"HPG0" {
        return Err("bad page magic".into());
    }
    let rc = u16::from_le_bytes(buf[12..14].try_into().unwrap());
    Ok(ReadHeader { record_count: rc })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_record(id: u64) -> Record {
        let mut record = Record::new(id);
        record.set_value("name", ASTValue::String("Test".to_string()));
        record.set_value("age", ASTValue::Int(25));
        record
    }

    #[test]
    fn test_page_creation() {
        let page = Page::new(1);
        assert_eq!(page.id, 1);
        assert!(page.records.is_empty());
        assert_eq!(page.free_space, PAGE_SIZE);
    }

    #[test]
    fn test_record_operations() {
        let mut page = Page::new(1);
        let record = create_test_record(1);

        // Test insert
        assert!(page.insert_record(record.clone()).is_ok());

        // Test get
        let retrieved_record = page.get_record(1);
        assert!(retrieved_record.is_some());

        // Test delete
        let deleted_record = page.delete_record(1);
        assert!(deleted_record.is_some());
        assert!(page.get_record(1).is_none());
    }

    #[test]
    fn test_page_space_management() {
        let mut page = Page::new(1);
        let initial_space = page.free_space;

        // Insert a record
        let record = create_test_record(1);
        page.insert_record(record).unwrap();

        // Verify space was reduced
        assert!(page.free_space < initial_space);

        // Delete the record
        page.delete_record(1);

        // Verify space was restored
        assert_eq!(page.free_space, initial_space);
    }
}
