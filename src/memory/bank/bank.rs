use crate::memory::bank::bank_index::{BankIndex, BankRegisters, BankRegisterId, MetaRegisterId, RamStatus};

use super::bank_index::{BankLocation, RomRamMode};

#[derive(Clone, Copy, Debug)]
pub enum Bank {
    Empty,
    WorkRam(Location, Option<RamStatusRegisterId>),
    // TODO: Add configurable writability?
    SaveRam(u32),
    ExtendedRam(Option<RamStatusRegisterId>),
    Rom(Location),
    Ram(Location, Option<RamStatusRegisterId>),
    RomRam(Location, RamStatusRegisterId, RomRamModeRegisterId),
    MirrorOf(u16),
}

impl Bank {
    pub const EMPTY: Bank = Bank::Empty;
    pub const WORK_RAM: Bank = Bank::WorkRam(Location::Fixed(BankIndex::from_u8(0)), None);
    pub const EXTENDED_RAM: Bank = Bank::ExtendedRam(None);
    pub const ROM: Bank = Bank::Rom(Location::Fixed(BankIndex::from_u8(0)));
    pub const RAM: Bank = Bank::Ram(Location::Fixed(BankIndex::from_u8(0)), None);
    pub const ROM_RAM: Bank = Bank::RomRam(Location::Fixed(BankIndex::from_u8(0)), RamStatusRegisterId::S0, RomRamModeRegisterId::R0);

    pub const fn fixed_index(self, index: i16) -> Self {
        self.set_location(Location::Fixed(BankIndex::from_i16(index))) }

    pub const fn switchable(self, register_id: BankRegisterId) -> Self {
        self.set_location(Location::Switchable(register_id))
    }

    pub const fn meta_switchable(self, meta_id: MetaRegisterId) -> Self {
        self.set_location(Location::MetaSwitchable(meta_id))
    }

    pub const fn mirror_of(window_address: u16) -> Self {
        Bank::MirrorOf(window_address)
    }

    pub const fn status_register(self, id: RamStatusRegisterId) -> Self {
        match self {
            Bank::WorkRam(location, None) => Bank::WorkRam(location, Some(id)),
            Bank::ExtendedRam(None) => Bank::ExtendedRam(Some(id)),
            Bank::Ram(location, None) => Bank::Ram(location, Some(id)),
            Bank::RomRam(location, _, rom_ram) => Bank::RomRam(location, id, rom_ram),
            _ => panic!("Only RAM and Work RAM support status registers."),
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

    pub fn bank_location(self, registers: &BankRegisters) -> Option<BankLocation> {
        if let Bank::Rom(location) | Bank::Ram(location, _) | Bank::WorkRam(location, _) = self {
            Some(location.bank_location(registers))
        } else {
            None
        }
    }

    pub fn is_writable(self, registers: &BankRegisters) -> bool {
        match self {
            Bank::Empty => false,
            Bank::Rom(_) => false,
            Bank::MirrorOf(_) => todo!("Writability of MirrorOf"),
            // RAM with no status register is always writable.
            Bank::Ram(_, None) | Bank::WorkRam(_, None) | Bank::ExtendedRam(None) => true,
            Bank::RomRam(_, status, rom_ram_mode) =>
                registers.rom_ram_mode(rom_ram_mode) == RomRamMode::Ram && registers.ram_status(status) == RamStatus::ReadWrite,
            Bank::Ram(_, Some(status_register_id)) | Bank::WorkRam(_, Some(status_register_id)) | Bank::ExtendedRam(Some(status_register_id)) =>
                registers.ram_status(status_register_id) == RamStatus::ReadWrite,
            Bank::SaveRam(..) => true,
        }
    }

    const fn set_location(self, location: Location) -> Self {
        match self {
            Bank::Rom(_) => Bank::Rom(location),
            Bank::Ram(_, None) => Bank::Ram(location, None),
            Bank::RomRam(_, status, rom_ram_mode) => Bank::RomRam(location, status, rom_ram_mode),
            Bank::WorkRam(_, None) => Bank::WorkRam(location, None),
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
    MetaSwitchable(MetaRegisterId),
}

impl Location {
    pub fn bank_location(self, registers: &BankRegisters) -> BankLocation {
        match self {
            Self::Fixed(bank_index) => BankLocation::Index(bank_index),
            Self::Switchable(register_id) => registers.get(register_id),
            Self::MetaSwitchable(_) => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum RamStatusRegisterId {
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