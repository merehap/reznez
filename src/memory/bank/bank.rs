use crate::memory::bank::bank::RomRamModeRegisterId::*;
use crate::memory::bank::bank_number::{BankNumber, PrgBankRegisters, PrgBankRegisterId, MetaRegisterId, ReadWriteStatus};
use crate::memory::bank::bank_number::{BankLocation, MemType};

use super::bank_number::{ChrBankRegisterId, ChrBankRegisters};

#[derive(Clone, Copy, Debug)]
pub struct PrgBank {
    bank_number_provider: PrgBankNumberProvider,
    mem_type_provider: MemTypeProvider,
    missing_ram_fallback_mem_type: Option<MemType>,
    read_write_status_provider: ReadWriteStatusProvider,
}

#[derive(Clone, Copy, Debug)]
enum ReadWriteStatusProvider {
    Fixed(ReadWriteStatus),
    Switchable(ReadWriteStatusRegisterId),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum MemTypeProvider {
    Fixed(Option<MemType>),
    Switchable(RomRamModeRegisterId),
}

impl MemTypeProvider {
    const fn is_mapped(self) -> bool {
        !matches!(self, Self::Fixed(None))
    }

    const fn is_switchable(self) -> bool {
        matches!(self, Self::Switchable(_))
    }
}

impl PrgBank {
    pub const EMPTY: Self = Self {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        mem_type_provider: MemTypeProvider::Fixed(None),
        missing_ram_fallback_mem_type: None,
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::Disabled),
    };
    pub const WORK_RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        mem_type_provider: MemTypeProvider::Fixed(Some(MemType::WorkRam)),
        missing_ram_fallback_mem_type: None,
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
    };
    pub const SAVE_RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        mem_type_provider: MemTypeProvider::Fixed(Some(MemType::SaveRam)),
        missing_ram_fallback_mem_type: None,
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
    };
    pub const ROM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        mem_type_provider: MemTypeProvider::Fixed(Some(MemType::Rom)),
        missing_ram_fallback_mem_type: Some(MemType::Rom),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadOnly),
    };
    pub const RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        mem_type_provider: MemTypeProvider::Fixed(Some(MemType::WorkRam)),
        missing_ram_fallback_mem_type: Some(MemType::Rom),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
    };
    pub const ROM_RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        mem_type_provider: MemTypeProvider::Switchable(R0),
        missing_ram_fallback_mem_type: Some(MemType::Rom),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
    };

    pub const fn fixed_index(mut self, index: i16) -> Self {
        assert!(self.mem_type_provider.is_mapped(), "An EMPTY bank can't be fixed_index.");
        self.bank_number_provider = PrgBankNumberProvider::Fixed(BankNumber::from_i16(index));
        self
    }

    pub const fn switchable(mut self, register_id: PrgBankRegisterId) -> Self {
        assert!(self.mem_type_provider.is_mapped(), "An EMPTY bank can't be switchable.");
        self.bank_number_provider = PrgBankNumberProvider::Switchable(register_id);
        self
    }

    pub const fn status_register(mut self, id: ReadWriteStatusRegisterId) -> Self {
        assert!(self.mem_type_provider.is_mapped(), "An EMPTY bank can't have a status register.");
        self.read_write_status_provider = ReadWriteStatusProvider::Switchable(id);
        self
    }

    pub const fn rom_ram_register(mut self, id: RomRamModeRegisterId) -> Self {
        assert!(self.mem_type_provider.is_switchable(), "Only ROM_RAM may have a rom ram register.");
        self.mem_type_provider = MemTypeProvider::Switchable(id);
        self
    }

    pub fn is_rom(self) -> bool {
        matches!(self.mem_type_provider, MemTypeProvider::Fixed(Some(MemType::Rom)) | MemTypeProvider::Switchable(_))
    }

    pub fn is_ram(self) -> bool {
        matches!(self.mem_type_provider,
            MemTypeProvider::Fixed(Some(MemType::WorkRam) | Some(MemType::SaveRam)) | MemTypeProvider::Switchable(_))
    }

    pub fn is_rom_ram(self) -> bool {
        matches!(self.mem_type_provider, MemTypeProvider::Switchable(_))
    }

    pub fn bank_number(self, regs: &PrgBankRegisters) -> Result<BankNumber, String> {
        if self.mem_type_provider.is_mapped() {
            Ok(self.bank_number_provider.bank_number(regs))
        } else {
            Err(format!("Empty banks {self:?} don't have a bank location."))
        }
    }

    pub const fn bank_register_id(&self) -> Option<PrgBankRegisterId> {
        match self.bank_number_provider {
            PrgBankNumberProvider::Fixed(_) => None,
            PrgBankNumberProvider::Switchable(reg_id) => Some(reg_id),
        }
    }

    pub fn status_register_id(&self) -> Option<ReadWriteStatusRegisterId> {
        match self.read_write_status_provider {
            ReadWriteStatusProvider::Fixed(_) => None,
            ReadWriteStatusProvider::Switchable(id) => Some(id),
        }
    }

    pub fn memory_type(&self, regs: &PrgBankRegisters) -> Option<MemType> {
        match self.mem_type_provider {
            MemTypeProvider::Fixed(mem_type) => mem_type,
            MemTypeProvider::Switchable(reg_id) => Some(regs.rom_ram_mode(reg_id)),
        }
    }

    pub fn missing_ram_fallback_mem_type(&self) -> Option<MemType> {
        self.missing_ram_fallback_mem_type
    }

    pub fn is_writable(self, regs: &PrgBankRegisters) -> bool {
        let status = match self.read_write_status_provider {
            ReadWriteStatusProvider::Fixed(status) => status,
            ReadWriteStatusProvider::Switchable(reg_id) => regs.read_write_status(reg_id),
        };

        status.is_writable()
    }

    pub fn as_rom(mut self) -> PrgBank {
        match self.missing_ram_fallback_mem_type {
            None => self = Self::EMPTY,
            Some(MemType::Rom) => {
                self.mem_type_provider = MemTypeProvider::Fixed(Some(MemType::Rom));
                self.read_write_status_provider = ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadOnly);
            }
            Some(_) => panic!("Non-sensical fallback mem type."),
        }

        self
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PrgBankNumberProvider {
    Fixed(BankNumber),
    Switchable(PrgBankRegisterId),
}

impl PrgBankNumberProvider {
    const FIXED_ZERO: Self = Self::Fixed(BankNumber::ZERO);

    fn bank_number(self, regs: &PrgBankRegisters) -> BankNumber {
        match self {
            Self::Fixed(bank_number) => bank_number,
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
    RomRam(ChrBankLocation, Option<ReadWriteStatusRegisterId>, RomRamModeRegisterId),
}

impl ChrBank {
    pub const ROM: ChrBank = ChrBank::Rom(ChrBankLocation::Fixed(BankNumber::from_u8(0)), None);
    pub const RAM: ChrBank = ChrBank::Ram(ChrBankLocation::Fixed(BankNumber::from_u8(0)), None);
    pub const ROM_RAM: ChrBank = ChrBank::RomRam(ChrBankLocation::Fixed(BankNumber::from_u8(0)), None, RomRamModeRegisterId::R0);

    pub const fn fixed_index(self, index: i16) -> Self {
        self.set_location(ChrBankLocation::Fixed(BankNumber::from_i16(index)))
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
            Self::Ram(_, None) | Self::RomRam(_, None, _) => true,
            Self::Ram(_, Some(status_register_id)) | Self::RomRam(_, Some(status_register_id), _) =>
                registers.read_write_status(status_register_id) == ReadWriteStatus::ReadWrite,
            Self::SaveRam(..) => true,
        }
    }

    pub fn location(self) -> Result<ChrBankLocation, String> {
        match self {
            ChrBank::Rom(location, _) | ChrBank::Ram(location, _) | ChrBank::RomRam(location, ..) => Ok(location),
            ChrBank::SaveRam(_) => Ok(ChrBankLocation::Fixed(BankNumber::from_u8(0))),
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
            ChrBank::RomRam(location, status, _) => ChrBank::RomRam(location, status, id),
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
        if let ChrBank::Rom(location, status_register) | ChrBank::Ram(location, status_register) | ChrBank::RomRam(location, status_register, _)= self {
            ChrBank::Rom(location, status_register)
        } else {
            panic!("Only RAM can be converted into ROM. Tried to convert {self:?}");
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
            Self::RomRam(_, status, id) => Self::RomRam(location, status, id),
            _ => todo!(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrBankLocation {
    Fixed(BankNumber),
    Switchable(ChrBankRegisterId),
    MetaSwitchable(MetaRegisterId),
}

impl ChrBankLocation {
    pub fn bank_location(self, registers: &ChrBankRegisters) -> BankLocation {
        match self {
            Self::Fixed(bank_number) => BankLocation::Index(bank_number),
            Self::Switchable(register_id) => registers.get(register_id),
            Self::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        }
    }
}