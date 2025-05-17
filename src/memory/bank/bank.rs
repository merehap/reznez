use crate::memory::bank::bank_index::{BankIndex, PrgBankRegisters, PrgBankRegisterId, MetaRegisterId, ReadWriteStatus};

use crate::memory::bank::bank_index::{BankLocation, MemoryType};

use super::bank_index::{ChrBankRegisterId, ChrBankRegisters};

#[derive(Clone, Copy, Debug)]
pub enum PrgBank {
    Empty,
    WorkRam(PrgBankLocation, Option<ReadWriteStatusRegisterId>),
    Rom(PrgBankLocation, Option<ReadWriteStatusRegisterId>),
    Ram(PrgBankLocation, Option<ReadWriteStatusRegisterId>),
    RomRam(PrgBankLocation, ReadWriteStatusRegisterId, RomRamModeRegisterId),
}

impl PrgBank {
    pub const EMPTY: PrgBank = PrgBank::Empty;
    pub const WORK_RAM: PrgBank = PrgBank::WorkRam(PrgBankLocation::Fixed(BankIndex::from_u8(0)), None);
    pub const ROM: PrgBank = PrgBank::Rom(PrgBankLocation::Fixed(BankIndex::from_u8(0)), None);
    pub const RAM: PrgBank = PrgBank::Ram(PrgBankLocation::Fixed(BankIndex::from_u8(0)), None);
    pub const ROM_RAM: PrgBank = PrgBank::RomRam(PrgBankLocation::Fixed(BankIndex::from_u8(0)), ReadWriteStatusRegisterId::S0, RomRamModeRegisterId::R0);

    pub const fn fixed_index(self, index: i16) -> Self {
        self.set_location(PrgBankLocation::Fixed(BankIndex::from_i16(index)))
    }

    pub const fn switchable(self, register_id: PrgBankRegisterId) -> Self {
        self.set_location(PrgBankLocation::Switchable(register_id))
    }

    pub const fn status_register(self, id: ReadWriteStatusRegisterId) -> Self {
        match self {
            PrgBank::Rom(location, None) => PrgBank::Rom(location, Some(id)),
            PrgBank::WorkRam(location, None) => PrgBank::WorkRam(location, Some(id)),
            PrgBank::Ram(location, None) => PrgBank::Ram(location, Some(id)),
            PrgBank::RomRam(location, _, rom_ram) => PrgBank::RomRam(location, id, rom_ram),
            _ => panic!("Cannot provide a status register here."),
        }
    }

    pub const fn rom_ram_register(self, id: RomRamModeRegisterId) -> Self {
        match self {
            PrgBank::RomRam(location, status, _) => PrgBank::RomRam(location, status, id),
            _ => panic!("Only RomRam supports RomRam registers."),
        }
    }

    pub fn is_rom(self) -> bool {
        matches!(self, PrgBank::Rom(..) | PrgBank::RomRam(..))
    }

    pub fn is_work_ram(self) -> bool {
        matches!(self, PrgBank::WorkRam(..))
    }

    pub fn is_ram(self) -> bool {
        matches!(self, PrgBank::WorkRam(..) | PrgBank::Ram(..) | PrgBank::RomRam(..))
    }

    pub fn location(self) -> Result<PrgBankLocation, String> {
        match self {
            PrgBank::Rom(location, _) | PrgBank::Ram(location, _)  | PrgBank::RomRam(location, _, _) | PrgBank::WorkRam(location, _) => Ok(location),
            PrgBank::Empty =>
                Err(format!("Empty banks {:?} don't have a bank location.", self)),
        }
    }

    pub fn bank_location(self, registers: &PrgBankRegisters) -> Option<BankLocation> {
        if let PrgBank::Rom(location, _) | PrgBank::Ram(location, _) | PrgBank::WorkRam(location, _) = self {
            Some(location.bank_location(registers))
        } else {
            None
        }
    }

    pub fn status_register_id(&self) -> Option<ReadWriteStatusRegisterId> {
        match self {
            PrgBank::Empty => None,
            PrgBank::Rom(_, reg_id) => *reg_id,
            PrgBank::Ram(_, reg_id) | PrgBank::WorkRam(_, reg_id) => *reg_id,
            PrgBank::RomRam(_, reg_id, _) => Some(*reg_id),
        }
    }

    pub fn memory_type(&self, regs: &PrgBankRegisters) -> Option<MemoryType> {
        match self {
            PrgBank::Empty => None,
            PrgBank::Rom(..) => Some(MemoryType::Rom),
            PrgBank::Ram(..) | PrgBank::WorkRam(..) => Some(MemoryType::Ram),
            PrgBank::RomRam(_, _, mode) => Some(regs.rom_ram_mode(*mode)),
        }
    }

    pub fn is_writable(self, registers: &PrgBankRegisters) -> bool {
        match self {
            PrgBank::Empty => false,
            PrgBank::Rom(..) => false,
            // RAM with no status register is always writable.
            PrgBank::Ram(_, None) | PrgBank::WorkRam(_, None) => true,
            PrgBank::RomRam(_, status_register_id, rom_ram_mode) =>
                registers.rom_ram_mode(rom_ram_mode) == MemoryType::Ram &&
                    registers.read_write_status(status_register_id) == ReadWriteStatus::ReadWrite,
            PrgBank::Ram(_, Some(status_register_id)) | PrgBank::WorkRam(_, Some(status_register_id)) =>
                registers.read_write_status(status_register_id) == ReadWriteStatus::ReadWrite,
        }
    }

    pub fn as_rom(self) -> PrgBank {
        match self {
            PrgBank::Rom(loc, status) | PrgBank::Ram(loc, status) =>
                PrgBank::Rom(loc, status),
            // RomRam status registers are for RAM, not ROM.
            PrgBank::RomRam(loc, _, _) =>
                PrgBank::Rom(loc, None),
            PrgBank::Empty | PrgBank::WorkRam(..) =>
                PrgBank::Empty,
        }
    }

    const fn set_location(self, location: PrgBankLocation) -> Self {
        match self {
            PrgBank::Rom(_, None) => PrgBank::Rom(location, None),
            PrgBank::Ram(_, None) => PrgBank::Ram(location, None),
            PrgBank::RomRam(_, status, rom_ram_mode) => PrgBank::RomRam(location, status, rom_ram_mode),
            PrgBank::WorkRam(_, None) => PrgBank::WorkRam(location, None),
            PrgBank::Rom(_, Some(_)) => panic!("ROM location must be set before ROM status register."),
            PrgBank::Ram(_, Some(_)) => panic!("RAM location must be set before RAM status register."),
            PrgBank::WorkRam(_, Some(_)) => panic!("RAM location must be set before RAM status register."),
            _ => panic!("Bank indexes can only be used for ROM or RAM or Work RAM."),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgBankLocation {
    Fixed(BankIndex),
    Switchable(PrgBankRegisterId),
}

impl PrgBankLocation {
    pub fn bank_location(self, regs: &PrgBankRegisters) -> BankLocation {
        match self {
            Self::Fixed(bank_index) => BankLocation::Index(bank_index),
            Self::Switchable(register_id) => regs.get(register_id),
        }
    }

    pub fn bank_index(self, regs: &PrgBankRegisters) -> BankIndex {
        match self {
            Self::Fixed(bank_index) => bank_index,
            Self::Switchable(register_id) => regs.get(register_id).index().unwrap(),
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
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
}

#[derive(Clone, Copy, Debug)]
pub enum ChrBank {
    // TODO: Add configurable writability?
    SaveRam(u32),
    Rom(ChrBankLocation, Option<ReadWriteStatusRegisterId>),
    Ram(ChrBankLocation, Option<ReadWriteStatusRegisterId>),
    RomRam(ChrBankLocation, RomRamModeRegisterId),
}

impl ChrBank {
    pub const ROM: ChrBank = ChrBank::Rom(ChrBankLocation::Fixed(BankIndex::from_u8(0)), None);
    pub const RAM: ChrBank = ChrBank::Ram(ChrBankLocation::Fixed(BankIndex::from_u8(0)), None);
    pub const ROM_RAM: ChrBank = ChrBank::RomRam(ChrBankLocation::Fixed(BankIndex::from_u8(0)), RomRamModeRegisterId::R0);

    pub const fn fixed_index(self, index: i16) -> Self {
        self.set_location(ChrBankLocation::Fixed(BankIndex::from_i16(index)))
    }

    pub const fn switchable(self, register_id: ChrBankRegisterId) -> Self {
        self.set_location(ChrBankLocation::Switchable(register_id))
    }

    pub const fn meta_switchable(self, meta_id: MetaRegisterId) -> Self {
        self.set_location(ChrBankLocation::MetaSwitchable(meta_id))
    }

    pub fn is_rom(self) -> bool {
        matches!(self, ChrBank::Rom(..) | ChrBank::RomRam(..))
    }

    pub fn is_ram(self) -> bool {
        matches!(self, ChrBank::Ram(..) | ChrBank::RomRam(..) | ChrBank::SaveRam(..))
    }

    pub fn is_writable(self, registers: &PrgBankRegisters) -> bool {
        match self {
            Self::Rom(..) => false,
            // RAM with no status register is always writable.
            Self::Ram(_, None) | Self::RomRam(_, _) => true,
            Self::Ram(_, Some(status_register_id)) =>
                registers.read_write_status(status_register_id) == ReadWriteStatus::ReadWrite,
            Self::SaveRam(..) => true,
        }
    }

    pub fn location(self) -> Result<ChrBankLocation, String> {
        match self {
            ChrBank::Rom(location, _) | ChrBank::Ram(location, _) | ChrBank::RomRam(location, _) => Ok(location),
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

    pub const fn rom_ram_register(self, id: RomRamModeRegisterId) -> Self {
        match self {
            ChrBank::RomRam(location, _) => ChrBank::RomRam(location, id),
            _ => panic!("Only RomRam supports RomRam registers."),
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
            Self::RomRam(_, id) => Self::RomRam(location, id),
            _ => todo!(),
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