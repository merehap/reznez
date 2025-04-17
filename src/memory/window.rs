use std::fmt;
use std::num::NonZeroU16;

use crate::memory::bank::bank::{PrgBank, PrgBankLocation};
use crate::memory::bank::bank_index::{PrgBankRegisters, PrgBankRegisterId};

use crate::memory::ppu::ciram::CiramSide;

use crate::mapper::{BankIndex, ReadWriteStatus, ReadWriteStatusRegisterId};
use crate::memory::bank::bank_index::{BankConfiguration, BankLocation};
use crate::memory::ppu::chr_memory::AccessOverride;

use super::bank::bank::{ChrBank, ChrBankLocation};
use super::bank::bank_index::{ChrBankRegisterId, ChrBankRegisters};

// A Window is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy, Debug)]
pub struct PrgWindow {
    start: u16,
    end: NonZeroU16,
    size: NonZeroU16,
    bank: PrgBank,
}

impl PrgWindow {
    pub const fn new(start: u16, end: u16, size: u32, bank: PrgBank) -> PrgWindow {
        assert!(end > start);
        let actual_size = end - start + 1;

        assert!(size < u16::MAX as u32, "Window size must be small enough to fit inside a u16.");
        let size = NonZeroU16::new(size as u16).expect("Window size to not be zero.");
        assert!(actual_size == size.get());

        let end = NonZeroU16::new(end).expect("Window end index to not be zero.");
        PrgWindow { start, end, size, bank }
    }

    pub fn bank_string(&self, registers: &PrgBankRegisters, rom_bank_configuration: BankConfiguration) -> String {
        match self.bank {
            PrgBank::Empty => "E".into(),
            // TODO: Add page number when there is more than one Work RAM page.
            PrgBank::WorkRam(_, _) => "W".into(),
            PrgBank::SaveRam(..) => "S".into(),
            PrgBank::ExtendedRam(_) => "X".into(),
            PrgBank::Rom(location, _) | PrgBank::Ram(location, _) | PrgBank::RomRam(location, _, _) =>
                self.resolved_bank_index(registers, location, rom_bank_configuration).to_string(),
            PrgBank::MirrorOf(_) => "M".into(),
        }
    }

    pub fn resolved_bank_index(
        &self,
        registers: &PrgBankRegisters,
        location: PrgBankLocation,
        bank_configuration: BankConfiguration,
    ) -> u16 {
        let stored_bank_index = match location {
            PrgBankLocation::Fixed(bank_index) => bank_index,
            PrgBankLocation::Switchable(register_id) => registers.get(register_id).index().unwrap(),
        };

        stored_bank_index.to_u16(bank_configuration, self.size())
    }

    pub const fn start(self) -> u16 {
        self.start
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end
    }

    pub const fn size(self) -> NonZeroU16 {
        self.size
    }

    pub const fn bank(self) -> PrgBank {
        self.bank
    }

    pub fn is_in_bounds(self, address: u16) -> bool {
        self.start <= address && address <= self.end.get()
    }

    pub fn location(self) -> Result<PrgBankLocation, String> {
        match self.bank {
            PrgBank::Rom(location, _) | PrgBank::Ram(location, _)  | PrgBank::RomRam(location, _, _) | PrgBank::WorkRam(location, _) => Ok(location),
            PrgBank::SaveRam(_) => Ok(PrgBankLocation::Fixed(BankIndex::from_u8(0))),
            PrgBank::Empty | PrgBank::ExtendedRam(_) | PrgBank::MirrorOf(_) =>
                Err(format!("Bank type {:?} does not have a bank location.", self.bank)),
        }
    }

    pub const fn register_id(self) -> Option<PrgBankRegisterId> {
        if let PrgBank::Rom(PrgBankLocation::Switchable(id), _) | PrgBank::Ram(PrgBankLocation::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }
    pub fn read_write_status_info(self) -> ReadWriteStatusInfo {
        match self.bank {
            PrgBank::Ram(_, Some(register_id)) | PrgBank::RomRam(_, register_id, _) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::ReadOnly },
            PrgBank::WorkRam(_, Some(register_id)) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::Disabled },
            // TODO: SaveRam will probably need to support status registers.
            PrgBank::SaveRam(..) =>
                ReadWriteStatusInfo::Absent,
            PrgBank::ExtendedRam(Some(register_id)) =>
                ReadWriteStatusInfo::MapperCustom { register_id },
            PrgBank::Empty | PrgBank::Rom(..) | PrgBank::MirrorOf(..) | PrgBank::Ram(..) | PrgBank::ExtendedRam(..) | PrgBank::WorkRam(..) =>
                ReadWriteStatusInfo::Absent,
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end.get() {
            Some(address - self.start)
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &PrgBankRegisters) -> bool {
        self.bank.is_writable(registers)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrLocation {
    RomBankIndex(u16),
    RamBankIndex(u16),
    Ciram(CiramSide),
}

impl fmt::Display for ChrLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChrLocation::RomBankIndex(index) => write!(f, "{index}"),
            ChrLocation::RamBankIndex(index) => write!(f, "W{index}"),
            ChrLocation::Ciram(CiramSide::Left) => write!(f, "LNT"),
            ChrLocation::Ciram(CiramSide::Right) => write!(f, "RNT"),
        }
    }
}

pub enum ReadWriteStatusInfo {
    Absent,
    PossiblyPresent { register_id: ReadWriteStatusRegisterId, status_on_absent: ReadWriteStatus },
    MapperCustom { register_id: ReadWriteStatusRegisterId },
}

#[derive(Clone, Copy, Debug)]
pub struct ChrWindow {
    start: u16,
    end: NonZeroU16,
    size: NonZeroU16,
    bank: ChrBank,
}

impl ChrWindow {
    pub const fn new(start: u16, end: u16, size: u32, bank: ChrBank) -> Self {
        assert!(end > start);
        let actual_size = end - start + 1;

        assert!(size < u16::MAX as u32, "Window size must be small enough to fit inside a u16.");
        let size = NonZeroU16::new(size as u16).expect("Window size to not be zero.");
        assert!(actual_size == size.get());

        let end = NonZeroU16::new(end).expect("Window end index to not be zero.");
        Self { start, end, size, bank }
    }

    pub fn bank_string(
        &self,
        registers: &ChrBankRegisters,
        rom_bank_configuration: Option<BankConfiguration>,
        ram_bank_configuration: Option<BankConfiguration>,
        access_override: Option<AccessOverride>,
    ) -> String {
        match self.bank {
            ChrBank::SaveRam(..) => "S".into(),
            ChrBank::Rom(location, _) | ChrBank::Ram(location, _) =>
                self.resolved_bank_location(registers, location, rom_bank_configuration, ram_bank_configuration, access_override).to_string(),
        }
    }

    pub fn resolved_bank_index(
        &self,
        registers: &ChrBankRegisters,
        location: ChrBankLocation,
        bank_configuration: BankConfiguration,
    ) -> u16 {
        let stored_bank_index = match location {
            ChrBankLocation::Fixed(bank_index) => bank_index,
            ChrBankLocation::Switchable(register_id) => registers.get(register_id).index().unwrap(),
            ChrBankLocation::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id).index().unwrap(),
        };

        stored_bank_index.to_u16(bank_configuration, self.size())
    }

    pub fn resolved_bank_location(
        &self,
        registers: &ChrBankRegisters,
        location: ChrBankLocation,
        chr_rom_bank_configuration: Option<BankConfiguration>,
        chr_ram_bank_configuration: Option<BankConfiguration>,
        access_override: Option<AccessOverride>,
    ) -> ChrLocation {
        let bank_location: BankLocation = match location {
            ChrBankLocation::Fixed(bank_index) => BankLocation::Index(bank_index),
            ChrBankLocation::Switchable(register_id) => registers.get(register_id),
            ChrBankLocation::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        };

        let is_ram = match access_override {
            None => match self.bank {
                ChrBank::Rom(..) => false,
                ChrBank::Ram(..) | ChrBank::SaveRam(..) => true,
            }
            Some(AccessOverride::ForceRom) => false,
            Some(AccessOverride::ForceRam) => true,
        };

        match bank_location {
            BankLocation::Index(index) => {
                if is_ram {
                    let raw_bank_index = index.to_u16(chr_ram_bank_configuration.unwrap(), self.size());
                    ChrLocation::RamBankIndex(raw_bank_index)
                } else {
                    let raw_bank_index = index.to_u16(chr_rom_bank_configuration.unwrap(), self.size());
                    ChrLocation::RomBankIndex(raw_bank_index)
                }
            }
            BankLocation::Ciram(ciram_side) => {
                ChrLocation::Ciram(ciram_side)
            }
        }
    }

    pub const fn start(self) -> u16 {
        self.start
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end
    }

    pub const fn size(self) -> NonZeroU16 {
        self.size
    }

    pub const fn bank(self) -> ChrBank {
        self.bank
    }

    pub fn is_in_bounds(self, address: u16) -> bool {
        self.start <= address && address <= self.end.get()
    }

    pub fn location(self) -> Result<ChrBankLocation, String> {
        match self.bank {
            ChrBank::Rom(location, _) | ChrBank::Ram(location, _) => Ok(location),
            ChrBank::SaveRam(_) => Ok(ChrBankLocation::Fixed(BankIndex::from_u8(0))),
        }
    }

    pub const fn register_id(self) -> Option<ChrBankRegisterId> {
        if let ChrBank::Rom(ChrBankLocation::Switchable(id), _) | ChrBank::Ram(ChrBankLocation::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }
    pub fn read_write_status_info(self) -> ReadWriteStatusInfo {
        match self.bank {
            ChrBank::Ram(_, Some(register_id)) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::ReadOnly },
            // TODO: SaveRam will probably need to support status registers.
            ChrBank::SaveRam(..) =>
                ReadWriteStatusInfo::Absent,
            ChrBank::Rom(..) | ChrBank::Ram(..) =>
                ReadWriteStatusInfo::Absent,
        }
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end.get() {
            Some(address - self.start)
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &PrgBankRegisters) -> bool {
        self.bank.is_writable(registers)
    }
}