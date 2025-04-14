use crate::memory::bank::bank_index::{BankIndex, BankRegisters, BankRegisterId, MetaRegisterId, ReadWriteStatus};

use crate::memory::bank::bank_index::{BankLocation, RomRamMode};

use super::bank_index::{ChrBankRegisterId, ChrBankRegisters};

#[derive(Clone, Copy, Debug)]
pub enum Bank {
    Empty,
    WorkRam(Location, Option<ReadWriteStatusRegisterId>),
    // TODO: Add configurable writability?
    SaveRam(u32),
    ExtendedRam(Option<ReadWriteStatusRegisterId>),
    Rom(Location, Option<ReadWriteStatusRegisterId>),
    Ram(Location, Option<ReadWriteStatusRegisterId>),
    RomRam(Location, ReadWriteStatusRegisterId, RomRamModeRegisterId),
    MirrorOf(u16),
}

impl Bank {
    pub const EMPTY: Bank = Bank::Empty;
    pub const WORK_RAM: Bank = Bank::WorkRam(Location::Fixed(BankIndex::from_u8(0)), None);
    pub const EXTENDED_RAM: Bank = Bank::ExtendedRam(None);
    pub const ROM: Bank = Bank::Rom(Location::Fixed(BankIndex::from_u8(0)), None);
    pub const RAM: Bank = Bank::Ram(Location::Fixed(BankIndex::from_u8(0)), None);
    pub const ROM_RAM: Bank = Bank::RomRam(Location::Fixed(BankIndex::from_u8(0)), ReadWriteStatusRegisterId::S0, RomRamModeRegisterId::R0);

    pub const fn fixed_index(self, index: i16) -> Self {
        self.set_location(Location::Fixed(BankIndex::from_i16(index)))
    }

    pub const fn switchable(self, register_id: BankRegisterId) -> Self {
        self.set_location(Location::Switchable(register_id))
    }

    pub const fn mirror_of(window_address: u16) -> Self {
        Bank::MirrorOf(window_address)
    }

    pub const fn status_register(self, id: ReadWriteStatusRegisterId) -> Self {
        match self {
            Bank::Rom(location, None) => Bank::Rom(location, Some(id)),
            Bank::WorkRam(location, None) => Bank::WorkRam(location, Some(id)),
            Bank::ExtendedRam(None) => Bank::ExtendedRam(Some(id)),
            Bank::Ram(location, None) => Bank::Ram(location, Some(id)),
            Bank::RomRam(location, _, rom_ram) => Bank::RomRam(location, id, rom_ram),
            _ => panic!("Cannot provide a status register here."),
        }
    }

    pub const fn rom_ram_register(self, id: RomRamModeRegisterId) -> Self {
        match self {
            Bank::RomRam(location, status, _) => Bank::RomRam(location, status, id),
            _ => panic!("Only RomRam supports RomRam registers."),
        }
    }

    pub fn is_work_ram(self) -> bool {
        matches!(self, Bank::WorkRam(..))
    }

    pub fn is_prg_ram(self) -> bool {
        matches!(self, Bank::WorkRam(..) | Bank::Ram(..) | Bank::RomRam(..))
    }

    pub fn location(self) -> Result<Location, String> {
        match self {
            Bank::Rom(location, _) | Bank::Ram(location, _)  | Bank::RomRam(location, _, _) | Bank::WorkRam(location, _) => Ok(location),
            Bank::SaveRam(_) => Ok(Location::Fixed(BankIndex::from_u8(0))),
            Bank::Empty | Bank::ExtendedRam(_) | Bank::MirrorOf(_) =>
                Err(format!("Bank type {:?} does not have a bank location.", self)),
        }
    }

    pub fn bank_location(self, registers: &BankRegisters) -> Option<BankLocation> {
        if let Bank::Rom(location, _) | Bank::Ram(location, _) | Bank::WorkRam(location, _) = self {
            Some(location.bank_location(registers))
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &BankRegisters) -> bool {
        match self {
            Bank::Empty => false,
            Bank::Rom(..) => false,
            Bank::MirrorOf(_) => todo!("Writability of MirrorOf"),
            // RAM with no status register is always writable.
            Bank::Ram(_, None) | Bank::WorkRam(_, None) | Bank::ExtendedRam(None) => true,
            Bank::RomRam(_, status, rom_ram_mode) =>
                registers.rom_ram_mode(rom_ram_mode) == RomRamMode::Ram && registers.read_write_status(status) == ReadWriteStatus::ReadWrite,
            Bank::Ram(_, Some(status_register_id)) | Bank::WorkRam(_, Some(status_register_id)) | Bank::ExtendedRam(Some(status_register_id)) =>
                registers.read_write_status(status_register_id) == ReadWriteStatus::ReadWrite,
            Bank::SaveRam(..) => true,
        }
    }

    const fn set_location(self, location: Location) -> Self {
        match self {
            Bank::Rom(_, None) => Bank::Rom(location, None),
            Bank::Ram(_, None) => Bank::Ram(location, None),
            Bank::RomRam(_, status, rom_ram_mode) => Bank::RomRam(location, status, rom_ram_mode),
            Bank::WorkRam(_, None) => Bank::WorkRam(location, None),
            Bank::Rom(_, Some(_)) => panic!("ROM location must be set before ROM status register."),
            Bank::Ram(_, Some(_)) => panic!("RAM location must be set before RAM status register."),
            Bank::WorkRam(_, Some(_)) => panic!("RAM location must be set before RAM status register."),
            _ => panic!("Bank indexes can only be used for ROM or RAM or Work RAM."),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Location {
    Fixed(BankIndex),
    Switchable(BankRegisterId),
}

impl Location {
    pub fn bank_location(self, registers: &BankRegisters) -> BankLocation {
        match self {
            Self::Fixed(bank_index) => BankLocation::Index(bank_index),
            Self::Switchable(register_id) => registers.get(register_id),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum ReadWriteStatusRegisterId {
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    S12,
    S13,
    S14,
    S15,
}

#[derive(Clone, Copy, Debug)]
pub enum RomRamModeRegisterId {
    R0,
    R1,
    R2,
}

#[derive(Clone, Copy, Debug)]
pub enum ChrBank {
    // TODO: Add configurable writability?
    SaveRam(u32),
    Rom(ChrBankLocation, Option<ReadWriteStatusRegisterId>),
    Ram(ChrBankLocation, Option<ReadWriteStatusRegisterId>),
}

impl ChrBank {
    pub const ROM: ChrBank = ChrBank::Rom(ChrBankLocation::Fixed(BankIndex::from_u8(0)), None);
    pub const RAM: ChrBank = ChrBank::Ram(ChrBankLocation::Fixed(BankIndex::from_u8(0)), None);

    pub const fn fixed_index(self, index: i16) -> Self {
        self.set_location(ChrBankLocation::Fixed(BankIndex::from_i16(index)))
    }

    pub const fn switchable(self, register_id: ChrBankRegisterId) -> Self {
        self.set_location(ChrBankLocation::Switchable(register_id))
    }

    pub const fn meta_switchable(self, meta_id: MetaRegisterId) -> Self {
        self.set_location(ChrBankLocation::MetaSwitchable(meta_id))
    }

    pub fn is_writable(self, registers: &BankRegisters) -> bool {
        match self {
            Self::Rom(..) => false,
            // RAM with no status register is always writable.
            Self::Ram(_, None) => true,
            Self::Ram(_, Some(status_register_id)) =>
                registers.read_write_status(status_register_id) == ReadWriteStatus::ReadWrite,
            Self::SaveRam(..) => true,
        }
    }

    pub fn location(self) -> Result<ChrBankLocation, String> {
        match self {
            ChrBank::Rom(location, _) | ChrBank::Ram(location, _) => Ok(location),
            ChrBank::SaveRam(_) => Ok(ChrBankLocation::Fixed(BankIndex::from_u8(0))),
        }
    }

    pub fn bank_location(self, registers: &ChrBankRegisters) -> Option<BankLocation> {
        if let ChrBank::Rom(location, _) | ChrBank::Ram(location, _) = self {
            Some(location.bank_location(registers))
        } else {
            None
        }
    }

    pub const fn status_register(self, id: ReadWriteStatusRegisterId) -> Self {
        match self {
            ChrBank::Rom(location, None) => ChrBank::Rom(location, Some(id)),
            ChrBank::Ram(location, None) => ChrBank::Ram(location, Some(id)),
            _ => panic!("Cannot provide a status register here."),
        }
    }

    pub fn as_rom(self) -> ChrBank {
        if let ChrBank::Rom(location, status_register) | ChrBank::Ram(location, status_register) = self {
            ChrBank::Rom(location, status_register)
        } else {
            panic!("Only RAM can be converted into ROM.");
        }
    }

    pub fn as_ram(self) -> ChrBank {
        if let ChrBank::Rom(location, status_register) | ChrBank::Ram(location, status_register) = self {
            ChrBank::Ram(location, status_register)
        } else {
            panic!("Only ROM can be converted into RAM.");
        }
    }

    const fn set_location(self, location: ChrBankLocation) -> Self {
        match self {
            Self::Rom(_, None) => Self::Rom(location, None),
            Self::Ram(_, None) => Self::Ram(location, None),
            Self::Rom(_, Some(_)) => panic!("ROM location must be set before ROM status register."),
            Self::Ram(_, Some(_)) => panic!("RAM location must be set before RAM status register."),
            _ => panic!("Bank indexes can only be used for ROM or RAM or Work RAM."),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrBankLocation {
    Fixed(BankIndex),
    Switchable(ChrBankRegisterId),
    MetaSwitchable(MetaRegisterId),
}

impl ChrBankLocation {
    pub fn bank_location(self, registers: &ChrBankRegisters) -> BankLocation {
        match self {
            Self::Fixed(bank_index) => BankLocation::Index(bank_index),
            Self::Switchable(register_id) => registers.get(register_id),
            Self::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        }
    }
}