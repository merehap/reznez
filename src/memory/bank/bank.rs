use crate::mapper::CiramSide;
use crate::mapper::NameTableSource;
use crate::memory::bank::bank::RomRamModeRegisterId::*;
use crate::memory::bank::bank::ChrSourceRegisterId::*;
use crate::memory::bank::bank_number::{BankNumber, PrgBankRegisters, PrgBankRegisterId, MetaRegisterId, ReadWriteStatus};
use crate::memory::bank::bank_number::{BankLocation, MemType};

use super::bank_number::{ChrBankRegisterId, ChrBankRegisters};

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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ChrSourceProvider {
    Fixed(Option<ChrSource>),
    Switchable(ChrSourceRegisterId),
}

impl ChrSourceProvider {
    const fn is_mapped(self) -> bool {
        !matches!(self, Self::Fixed(None))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrSource {
    Rom,
    WorkRam,
    SaveRam,
    Ciram(CiramSide),
    ExtendedRam,
    FillModeTile,
}

impl ChrSource {
    pub fn from_name_table_source(name_table_source: NameTableSource) -> (Self, Option<BankNumber>) {
        match name_table_source {
            NameTableSource::Ram { bank_number } => (ChrSource::WorkRam, Some(bank_number)),
            NameTableSource::Ciram(ciram_side) => (ChrSource::Ciram(ciram_side), None),
            NameTableSource::ExtendedRam => (ChrSource::ExtendedRam, None),
            NameTableSource::FillModeTile => (ChrSource::FillModeTile, None),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrgBank {
    bank_number_provider: PrgBankNumberProvider,
    mem_type_provider: MemTypeProvider,
    missing_ram_fallback_mem_type: Option<MemType>,
    read_write_status_provider: ReadWriteStatusProvider,
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrSourceRegisterId {
    CS0,
    CS1,
    CS2,
    CS3,
    CS4,
    CS5,
    CS6,
    CS7,

    // Name Table Top Left
    NT0,
    // Name Table Top Right
    NT1,
    // Name Table Bottom Left
    NT2,
    // Name Table Bottom Right
    NT3,
}

impl ChrSourceRegisterId {
    pub const ALL_NAME_TABLE_SOURCE_IDS: [Self; 4] =
        [ChrSourceRegisterId::NT0, ChrSourceRegisterId::NT1, ChrSourceRegisterId::NT2, ChrSourceRegisterId::NT3];
}

#[derive(Clone, Copy, Debug)]
pub struct ChrBank {
    bank_number_provider: ChrBankNumberProvider,
    chr_source_provider: ChrSourceProvider,
    missing_ram_fallback_mem_type: Option<MemType>,
    read_write_status_provider: ReadWriteStatusProvider,
}

impl ChrBank {
    pub const EMPTY: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(None),
        missing_ram_fallback_mem_type: None,
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::Disabled),
    };
    pub const ROM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::Rom)),
        missing_ram_fallback_mem_type: Some(MemType::Rom),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadOnly),
    };
    pub const RAM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::WorkRam)),
        missing_ram_fallback_mem_type: Some(MemType::Rom),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
    };
    pub const EXT_RAM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::ExtendedRam)),
        // FIXME: HACK
        missing_ram_fallback_mem_type: Some(MemType::WorkRam),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
    };
    pub const FILL_MODE_TILE: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::FillModeTile)),
        // FIXME: HACK
        missing_ram_fallback_mem_type: Some(MemType::WorkRam),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadOnly),
    };
    pub const ROM_RAM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Switchable(CS0),
        missing_ram_fallback_mem_type: Some(MemType::Rom),
        read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
    };

    pub const fn ciram(ciram_side: CiramSide) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::Ciram(ciram_side))),
            missing_ram_fallback_mem_type: Some(MemType::Rom),
            read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
        }
    }

    pub const fn from_name_table_source(name_table_source: NameTableSource) -> Self {
        match name_table_source {
            NameTableSource::Ciram(ciram_side) => Self::ciram(ciram_side),
            NameTableSource::ExtendedRam => Self::EXT_RAM,
            NameTableSource::FillModeTile => Self::FILL_MODE_TILE,
            NameTableSource::Ram { bank_number } => Self::RAM.fixed_index(bank_number.to_raw() as i16),
        }
    }

    pub const fn with_switchable_source(source_reg_id: ChrSourceRegisterId) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Switchable(source_reg_id),
            // FIXME: HACK
            missing_ram_fallback_mem_type: Some(MemType::WorkRam),
            // FIXME: HACK
            read_write_status_provider: ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite),
        }
    }

    pub const fn fixed_index(self, index: i16) -> Self {
        self.set_location(ChrBankNumberProvider::Fixed(BankNumber::from_i16(index)))
    }

    pub const fn switchable(self, register_id: ChrBankRegisterId) -> Self {
        self.set_location(ChrBankNumberProvider::Switchable(register_id))
    }

    pub const fn meta_switchable(self, meta_id: MetaRegisterId) -> Self {
        self.set_location(ChrBankNumberProvider::MetaSwitchable(meta_id))
    }

    pub fn current_chr_source(self, regs: &ChrBankRegisters) -> Option<ChrSource> {
        match self.chr_source_provider {
            ChrSourceProvider::Fixed(mem_type) => mem_type,
            ChrSourceProvider::Switchable(reg_id) => Some(regs.chr_source(reg_id)),
        }
    }

    pub fn is_rom(self) -> bool {
        matches!(self.chr_source_provider, ChrSourceProvider::Fixed(Some(ChrSource::Rom)) | ChrSourceProvider::Switchable(_))
    }

    pub fn is_ram(self) -> bool {
        matches!(self.chr_source_provider,
            ChrSourceProvider::Fixed(Some(ChrSource::WorkRam | ChrSource::SaveRam)) | ChrSourceProvider::Switchable(_))
    }

    pub fn missing_ram_fallback_mem_type(&self) -> Option<MemType> {
        self.missing_ram_fallback_mem_type
    }

    pub const fn register_id(&self, regs: &ChrBankRegisters) -> Option<ChrBankRegisterId> {
        self.bank_number_provider.register_id(regs)
    }

    pub const fn read_write_status_register_id(&self) -> Option<ReadWriteStatusRegisterId> {
        match self.read_write_status_provider {
            ReadWriteStatusProvider::Fixed(_) => None,
            ReadWriteStatusProvider::Switchable(reg_id) => Some(reg_id),
        }
    }

    pub fn read_write_status(self, regs: &ChrBankRegisters) -> ReadWriteStatus {
        match self.read_write_status_provider {
            ReadWriteStatusProvider::Fixed(status) => status,
            ReadWriteStatusProvider::Switchable(reg_id) => regs.read_write_status(reg_id),
        }
    }

    pub fn is_writable(self, regs: &ChrBankRegisters) -> bool {
        let status = match self.read_write_status_provider {
            ReadWriteStatusProvider::Fixed(status) => status,
            ReadWriteStatusProvider::Switchable(reg_id) => regs.read_write_status(reg_id),
        };

        status.is_writable()
    }

    pub fn location(self) -> Result<ChrBankNumberProvider, String> {
        if self.chr_source_provider.is_mapped() {
            Ok(self.bank_number_provider)
        } else {
            Err("EMPTY banks don't have a location".to_owned())
        }
    }

    pub fn bank_location(self, regs: &ChrBankRegisters) -> Option<BankLocation> {
        self.location().ok().map(|provider| BankLocation::Index(provider.bank_location(regs)))
    }

    pub fn bank_number(self, regs: &ChrBankRegisters) -> Option<BankNumber> {
        self.location().ok().map(|provider| provider.bank_location(regs))
    }

    pub const fn chr_source(mut self, id: ChrSourceRegisterId) -> Self {
        assert!(matches!(self.chr_source_provider, ChrSourceProvider::Switchable(_)));
        self.chr_source_provider = ChrSourceProvider::Switchable(id);
        self
    }

    pub const fn status_register(mut self, id: ReadWriteStatusRegisterId) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        self.read_write_status_provider = ReadWriteStatusProvider::Switchable(id);
        self
    }

    pub fn as_rom(mut self) -> ChrBank {
        match self.missing_ram_fallback_mem_type {
            None => self = Self::EMPTY,
            Some(MemType::Rom) => {
                self.chr_source_provider = ChrSourceProvider::Fixed(Some(ChrSource::Rom));
                self.read_write_status_provider = ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadOnly);
            }
            // FIXME: HACK. Use MemType::WorkRam to signal that there this ChrSource is unaffected by the RAM availability.
            Some(MemType::WorkRam) => { /* Do nothing. */ }
            Some(MemType::SaveRam) => panic!("Non-sensical fallback mem type."),
        }

        self
    }

    pub fn as_work_ram(mut self) -> ChrBank {
        if self.chr_source_provider.is_mapped() {
            self.chr_source_provider = ChrSourceProvider::Fixed(Some(ChrSource::WorkRam));
            self.read_write_status_provider = ReadWriteStatusProvider::Fixed(ReadWriteStatus::ReadWrite);
        }

        self
    }

    const fn set_location(mut self, location: ChrBankNumberProvider) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        assert!(matches!(self.read_write_status_provider, ReadWriteStatusProvider::Fixed(_)),
            "Location must be set before ROM status register.");
        self.bank_number_provider = location;
        self
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChrBankNumberProvider {
    Fixed(BankNumber),
    Switchable(ChrBankRegisterId),
    MetaSwitchable(MetaRegisterId),
}

impl ChrBankNumberProvider {
    const FIXED_ZERO: Self = Self::Fixed(BankNumber::ZERO);

    pub fn bank_location(self, registers: &ChrBankRegisters) -> BankNumber {
        match self {
            Self::Fixed(bank_number) => bank_number,
            Self::Switchable(register_id) => registers.get(register_id),
            Self::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        }
    }

    pub const fn register_id(self, registers: &ChrBankRegisters) -> Option<ChrBankRegisterId> {
        match self {
            Self::Fixed(_) => None,
            Self::Switchable(register_id) => Some(register_id),
            Self::MetaSwitchable(meta_id) => Some(registers.get_register_id_from_meta(meta_id)),
        }
    }
}