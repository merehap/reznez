use std::fmt;

use crate::memory::bank::bank::{Bank, Location};
use crate::memory::bank::bank_index::{BankRegisters, BankRegisterId};

use crate::memory::ppu::ciram::CiramSide;

use crate::mapper::{BankIndex, ReadWriteStatus, ReadWriteStatusRegisterId};
use crate::memory::bank::bank_index::{BankConfiguration, BankLocation, RomRamMode};
use crate::memory::ppu::chr_memory::AccessOverride;

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
        rom_bank_configuration: Option<BankConfiguration>,
        ram_bank_configuration: Option<BankConfiguration>,
        access_override: Option<AccessOverride>,
    ) -> String {
        match self.bank {
            Bank::Empty => "E".into(),
            // TODO: Add page number when there is more than one Work RAM page.
            Bank::WorkRam(_, _) => "W".into(),
            Bank::SaveRam(..) => "S".into(),
            Bank::ExtendedRam(_) => "X".into(),
            Bank::Rom(location, _) | Bank::Ram(location, _) | Bank::RomRam(location, _, _) =>
                self.resolved_bank_location(registers, location, rom_bank_configuration, ram_bank_configuration, access_override).to_string(),
            Bank::MirrorOf(_) => "M".into(),
        }
    }

    pub fn resolved_bank_index(
        &self,
        registers: &BankRegisters,
        location: Location,
        bank_configuration: BankConfiguration,
    ) -> u16 {
        let stored_bank_index = match location {
            Location::Fixed(bank_index) => bank_index,
            Location::Switchable(register_id) => registers.get(register_id).index().unwrap(),
            Location::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id).index().unwrap(),
        };

        stored_bank_index.to_u16(bank_configuration, self.size())
    }

    pub fn resolved_bank_location(
        &self,
        registers: &BankRegisters,
        location: Location,
        chr_rom_bank_configuration: Option<BankConfiguration>,
        chr_ram_bank_configuration: Option<BankConfiguration>,
        access_override: Option<AccessOverride>,
    ) -> ChrLocation {
        let bank_location: BankLocation = match location {
            Location::Fixed(bank_index) => BankLocation::Index(bank_index),
            Location::Switchable(register_id) => registers.get(register_id),
            Location::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        };

        let is_ram = match access_override {
            None => match self.bank {
                Bank::Rom(..) => false,
                Bank::Ram(..) | Bank::SaveRam(..) => true,
                Bank::RomRam(_, _, rom_ram_mode) => registers.rom_ram_mode(rom_ram_mode) == RomRamMode::Ram,
                _ => panic!("Unsupported bank type for CHR: {:?}", self.bank),
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
            Bank::Rom(location, _) | Bank::Ram(location, _)  | Bank::RomRam(location, _, _) | Bank::WorkRam(location, _) => Ok(location),
            Bank::SaveRam(_) => Ok(Location::Fixed(BankIndex::from_u8(0))),
            Bank::Empty | Bank::ExtendedRam(_) | Bank::MirrorOf(_) =>
                Err(format!("Bank type {:?} does not have a bank location.", self.bank)),
        }
    }

    pub const fn register_id(self) -> Option<BankRegisterId> {
        if let Bank::Rom(Location::Switchable(id), _) | Bank::Ram(Location::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }
    pub fn read_write_status_info(self) -> ReadWriteStatusInfo {
        match self.bank {
            Bank::Ram(_, Some(register_id)) | Bank::RomRam(_, register_id, _) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::ReadOnly },
            Bank::WorkRam(_, Some(register_id)) =>
                ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent: ReadWriteStatus::Disabled },
            // TODO: SaveRam will probably need to support status registers.
            Bank::SaveRam(..) =>
                ReadWriteStatusInfo::Absent,
            Bank::ExtendedRam(Some(register_id)) =>
                ReadWriteStatusInfo::MapperCustom { register_id },
            Bank::Empty | Bank::Rom(..) | Bank::MirrorOf(..) | Bank::Ram(..) | Bank::ExtendedRam(..) | Bank::WorkRam(..) =>
                ReadWriteStatusInfo::Absent,
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
