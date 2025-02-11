use std::fmt;

use crate::memory::bank::bank::{Bank, Location};
use crate::memory::bank::bank_index::{BankIndex, BankRegisters, BankRegisterId};

use crate::memory::ppu::ciram::CiramSide;

use super::bank::bank_index::BankLocation;

// A Window is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy, Debug)]
pub struct Window {
    start: u16,
    end: u16,
    bank: Bank,
}

impl Window {
    pub const fn new(start: u16, end: u16, size: u32, bank: Bank) -> Window {
        assert!(end > start);
        let actual_size = end as u32 - start as u32 + 1;
        assert!(actual_size == size);

        Window { start, end, bank }
    }

    pub fn bank_string(
        &self,
        registers: &BankRegisters,
        bank_size: u16,
        bank_count: u16,
        align_large_layouts: bool,
    ) -> String {
        match self.bank {
            Bank::Empty => "E".into(),
            Bank::WorkRam(_) => "W".into(),
            Bank::ExtendedRam(_) => "X".into(),
            Bank::Rom(location) | Bank::Ram(location, _) =>
                self.resolved_bank_location(
                    registers,
                    location,
                    bank_size,
                    bank_count,
                    align_large_layouts,
                ).to_string(),
            Bank::MirrorOf(_) => "M".into(),
        }
    }

    pub fn resolved_bank_index(
        &self,
        registers: &BankRegisters,
        location: Location,
        bank_size: u16,
        bank_count: u16,
        align_large_layouts: bool,
    ) -> u16 {
        let stored_bank_index = match location {
            Location::Fixed(bank_index) => bank_index,
            Location::Switchable(register_id) => registers.get(register_id).index().unwrap(),
            Location::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id).index().unwrap(),
        };

        self.resolve_bank_index(stored_bank_index, bank_size, bank_count, align_large_layouts)
    }

    pub fn resolved_bank_location(
        &self,
        registers: &BankRegisters,
        location: Location,
        bank_size: u16,
        bank_count: u16,
        align_large_layouts: bool,
    ) -> ChrLocation {
        let bank_location: BankLocation = match location {
            Location::Fixed(bank_index) => BankLocation::Index(bank_index),
            Location::Switchable(register_id) => registers.get(register_id),
            Location::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        };

        match bank_location {
            BankLocation::Index(index) => {
                let bank_index = self.resolve_bank_index(index, bank_size, bank_count, align_large_layouts);
                ChrLocation::BankIndex(bank_index)
            }
            BankLocation::Ciram(ciram_side) => {
                ChrLocation::Ciram(ciram_side)
            }
        }
    }

    fn resolve_bank_index(&self, bank_index: BankIndex, bank_size: u16, bank_count: u16, align_large_layouts: bool) -> u16 {
        let mut resolved_bank_index = bank_index.to_u16(bank_count);
        if align_large_layouts {
            let window_multiple = self.size() / bank_size;
            // Clear low bits for large windows.
            resolved_bank_index &= !(window_multiple - 1);
        }

        resolved_bank_index
    }

    pub const fn start(self) -> u16 {
        self.start
    }

    pub const fn end(self) -> u16 {
        self.end
    }

    pub const fn size(self) -> u16 {
        self.end - self.start + 1
    }

    pub const fn bank(self) -> Bank {
        self.bank
    }

    pub fn location(self) -> Result<Location, String> {
        match self.bank {
            Bank::Rom(location) | Bank::Ram(location, _) => Ok(location),
            Bank::Empty | Bank::WorkRam(_) | Bank::ExtendedRam(_) | Bank::MirrorOf(_) =>
                Err(format!("Bank type {:?} does not have a bank location.", self.bank)),
        }
    }

    pub const fn register_id(self) -> Option<BankRegisterId> {
        if let Bank::Rom(Location::Switchable(id)) | Bank::Ram(Location::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end {
            Some(address - self.start)
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &BankRegisters) -> bool {
        self.bank.is_writable(registers)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrLocation {
    BankIndex(u16),
    Ciram(CiramSide),
}

impl fmt::Display for ChrLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChrLocation::BankIndex(index) => write!(f, "{index}"),
            ChrLocation::Ciram(CiramSide::Left) => write!(f, "LNT"),
            ChrLocation::Ciram(CiramSide::Right) => write!(f, "RNT"),
        }
    }
}