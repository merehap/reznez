use crate::memory::bank::bank_number::{ReadStatus, WriteStatus};
use crate::memory::read_result::ReadResult;

const KIBIBYTE: usize = 0x400;

pub struct SmallPage {
    _name: String,
    page: [u8; KIBIBYTE],
    read_status: ReadStatus,
    write_status: WriteStatus,
}

impl SmallPage {
    pub fn new(name: String, read_status: ReadStatus, write_status: WriteStatus) -> Self {
        Self {
            _name: name,
            page: [0; KIBIBYTE],
            read_status,
            write_status,
        }
    }

    pub fn peek(&self, index: u16) -> ReadResult {
        match self.read_status {
            ReadStatus::Disabled => ReadResult::OPEN_BUS,
            ReadStatus::ReadOnlyZeros => ReadResult::full(0),
            ReadStatus::Enabled => ReadResult::full(self.page[index as usize]),
        }
    }

    pub fn write(&mut self, index: u16, value: u8) {
        match self.write_status {
            WriteStatus::Disabled => { /* Do nothing. */ }
            WriteStatus::Enabled => self.page[index as usize] = value,
        }
    }

    pub fn set_read_status(&mut self, read_status: ReadStatus) {
        self.read_status = read_status;
    }

    pub fn set_write_status(&mut self, write_status: WriteStatus) {
        self.write_status = write_status;
    }

    pub fn to_raw_ref(&self) -> &[u8; KIBIBYTE] {
        &self.page
    }

    pub fn to_raw_ref_mut(&mut self) -> Option<&mut [u8; KIBIBYTE]> {
        match self.write_status {
            WriteStatus::Disabled => None,
            WriteStatus::Enabled => Some(&mut self.page),
        }
    }
}