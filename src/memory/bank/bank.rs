use crate::mapper::CiramSide;
use crate::mapper::NameTableSource;
use crate::mapper::ReadStatus;
use crate::mapper::WriteStatus;
use crate::memory::bank::bank::PrgSourceRegisterId::*;
use crate::memory::bank::bank::ChrSourceRegisterId::*;
use crate::memory::bank::bank_number::{BankNumber, PrgBankRegisters, PrgBankRegisterId, MetaRegisterId};
use crate::memory::bank::bank_number::{BankLocation, MemType};

use super::bank_number::{ChrBankRegisterId, ChrBankRegisters};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PrgSourceProvider {
    Fixed(Option<PrgSource>),
    Switchable(PrgSourceRegisterId),
}

impl PrgSourceProvider {
    const fn is_mapped(self) -> bool {
        !matches!(self, Self::Fixed(None))
    }

    const fn is_switchable(self) -> bool {
        matches!(self, Self::Switchable(_))
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgSource {
    Rom,
    // Work RAM or Save RAM
    RamOrAbsent,
    WorkRamOrRom,
    SaveRam,
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
    RomOrRam,
    Rom,
    WorkRam,
    SaveRam,
    Ciram(CiramSide),
    MapperCustom { page_number: u8 },
}

impl ChrSource {
    pub fn from_name_table_source(name_table_source: NameTableSource) -> (Self, Option<BankNumber>) {
        match name_table_source {
            NameTableSource::Ciram(ciram_side) => (ChrSource::Ciram(ciram_side), None),
            NameTableSource::Rom { bank_number } => (ChrSource::Rom, Some(bank_number)),
            NameTableSource::Ram { bank_number } => (ChrSource::WorkRam, Some(bank_number)),
            NameTableSource::MapperCustom { page_number } => (ChrSource::MapperCustom { page_number }, None),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrgBank {
    bank_number_provider: PrgBankNumberProvider,
    prg_source_provider: PrgSourceProvider,
    read_status_register_id: Option<ReadStatusRegisterId>,
    write_status_register_id: Option<WriteStatusRegisterId>,
}

impl PrgBank {
    pub const ABSENT: Self = Self {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        prg_source_provider: PrgSourceProvider::Fixed(None),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const WORK_RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        prg_source_provider: PrgSourceProvider::Fixed(Some(PrgSource::RamOrAbsent)),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const SAVE_RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        prg_source_provider: PrgSourceProvider::Fixed(Some(PrgSource::SaveRam)),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const ROM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        prg_source_provider: PrgSourceProvider::Fixed(Some(PrgSource::Rom)),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        prg_source_provider: PrgSourceProvider::Fixed(Some(PrgSource::WorkRamOrRom)),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const ROM_RAM: PrgBank = PrgBank {
        bank_number_provider: PrgBankNumberProvider::FIXED_ZERO,
        prg_source_provider: PrgSourceProvider::Switchable(PS0),
        read_status_register_id: None,
        write_status_register_id: None,
    };

    pub const fn fixed_index(mut self, index: i16) -> Self {
        assert!(self.prg_source_provider.is_mapped(), "An ABSENT bank can't be fixed_index.");
        self.bank_number_provider = PrgBankNumberProvider::Fixed(BankNumber::from_i16(index));
        self
    }

    pub const fn switchable(mut self, register_id: PrgBankRegisterId) -> Self {
        assert!(self.prg_source_provider.is_mapped(), "An ABSENT bank can't be switchable.");
        self.bank_number_provider = PrgBankNumberProvider::Switchable(register_id);
        self
    }

    pub const fn read_status(mut self, read_id: ReadStatusRegisterId) -> Self {
        assert!(self.prg_source_provider.is_mapped(), "An ABSENT bank can't have a read status register.");
        self.read_status_register_id = Some(read_id);
        self
    }

    pub const fn write_status(mut self, write_id: WriteStatusRegisterId) -> Self {
        assert!(self.prg_source_provider.is_mapped(), "An ABSENT bank can't have a write status register.");
        self.write_status_register_id = Some(write_id);
        self
    }

    pub const fn read_write_status(mut self, read_id: ReadStatusRegisterId, write_id: WriteStatusRegisterId) -> Self {
        assert!(self.prg_source_provider.is_mapped(), "An ABSENT bank can't have a read or write status register.");
        self.read_status_register_id = Some(read_id);
        self.write_status_register_id = Some(write_id);
        self
    }

    pub const fn rom_ram_register(mut self, id: PrgSourceRegisterId) -> Self {
        assert!(self.prg_source_provider.is_switchable(), "Only ROM_RAM may have a rom ram register.");
        self.prg_source_provider = PrgSourceProvider::Switchable(id);
        self
    }

    pub fn is_rom(self) -> bool {
        matches!(self.prg_source_provider, PrgSourceProvider::Fixed(Some(PrgSource::Rom)) | PrgSourceProvider::Switchable(_))
    }

    pub fn supports_ram(self) -> bool {
        matches!(self.prg_source_provider,
            PrgSourceProvider::Fixed(Some(PrgSource::RamOrAbsent | PrgSource::WorkRamOrRom | PrgSource::SaveRam))
                | PrgSourceProvider::Switchable(_))
    }

    pub fn is_rom_ram(self) -> bool {
        matches!(self.prg_source_provider, PrgSourceProvider::Switchable(_))
    }

    pub fn bank_number(self, regs: &PrgBankRegisters) -> Result<BankNumber, String> {
        if self.prg_source_provider.is_mapped() {
            Ok(self.bank_number_provider.bank_number(regs))
        } else {
            Err(format!("Empty banks {self:?} don't have a bank location."))
        }
    }

    pub const fn bank_register_id(self) -> Option<PrgBankRegisterId> {
        match self.bank_number_provider {
            PrgBankNumberProvider::Fixed(_) => None,
            PrgBankNumberProvider::Switchable(reg_id) => Some(reg_id),
        }
    }

    pub fn read_status_register_id(self) -> Option<ReadStatusRegisterId> {
        self.read_status_register_id
    }

    pub fn write_status_register_id(self) -> Option<WriteStatusRegisterId> {
        self.write_status_register_id
    }

    // FIXME: Use explicit rom_read_status() and ram_read_status() providers, then simplify this accordingly.
    pub fn memory_type(self, regs: &PrgBankRegisters) -> Option<MemType> {
        let prg_source = match self.prg_source_provider {
            PrgSourceProvider::Fixed(prg_source) => prg_source,
            PrgSourceProvider::Switchable(reg_id) => Some(regs.rom_ram_mode(reg_id)),
        }?;

        let read_id = self.read_status_register_id.map_or(ReadStatus::Enabled, |id| regs.read_status(id));
        let write_id = self.write_status_register_id.map_or(WriteStatus::Enabled, |id| regs.write_status(id));

        // There's currently no way to set make the ROM ReadStatus of a RomRam bank switchable.
        if self.is_rom_ram() && (prg_source == PrgSource::Rom || !regs.cartridge_has_ram()) {
            return Some(MemType::Rom(ReadStatus::Enabled));
        }

        match prg_source {
            PrgSource::Rom => Some(MemType::Rom(read_id)),
            PrgSource::SaveRam => Some(MemType::SaveRam(read_id, write_id)),
            PrgSource::RamOrAbsent if regs.cartridge_has_ram() => Some(MemType::WorkRam(read_id, write_id)),
            PrgSource::RamOrAbsent => None,
            PrgSource::WorkRamOrRom if regs.cartridge_has_ram() => Some(MemType::WorkRam(read_id, write_id)),
            PrgSource::WorkRamOrRom => Some(MemType::Rom(read_id)),
        }
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
pub enum ReadStatusRegisterId {
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
    R12,
    R13,
    R14,
    R15,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum WriteStatusRegisterId {
    W0,
    W1,
    W2,
    W3,
    W4,
    W5,
    W6,
    W7,
    W8,
    W9,
    W10,
    W11,
    W12,
    W13,
    W14,
    W15,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgSourceRegisterId {
    PS0,
    PS1,
    PS2,
    PS3,
    PS4,
    PS5,
    PS6,
    PS7,
    PS8,
    PS9,
    PS10,
    PS11,
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
    read_status_register_id: Option<ReadStatusRegisterId>,
    write_status_register_id: Option<WriteStatusRegisterId>,
}

impl ChrBank {
    pub const EMPTY: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(None),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const ROM_OR_RAM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::RomOrRam)),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const ROM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::Rom)),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const RAM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::WorkRam)),
        read_status_register_id: None,
        write_status_register_id: None,
    };
    pub const SWITCHABLE_SOURCE: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Switchable(CS0),
        read_status_register_id: None,
        write_status_register_id: None,
    };

    pub const fn ciram(ciram_side: CiramSide) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::Ciram(ciram_side))),
            read_status_register_id: None,
            write_status_register_id: None,
        }
    }

    pub const fn with_switchable_source(source_reg_id: ChrSourceRegisterId) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Switchable(source_reg_id),
            read_status_register_id: None,
            write_status_register_id: None,
        }
    }

    pub const fn mapper_sourced(page_number: u8) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::MapperCustom { page_number })),
            read_status_register_id: None,
            write_status_register_id: None,
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

    pub const fn register_id(&self, regs: &ChrBankRegisters) -> Option<ChrBankRegisterId> {
        self.bank_number_provider.register_id(regs)
    }

    pub const fn read_status_register_id(&self) -> Option<ReadStatusRegisterId> {
        self.read_status_register_id
    }

    pub const fn write_status_register_id(&self) -> Option<WriteStatusRegisterId> {
        self.write_status_register_id
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

    pub const fn read_status(mut self, read_id: ReadStatusRegisterId) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        self.read_status_register_id = Some(read_id);
        self
    }

    pub const fn write_status(mut self, write_id: WriteStatusRegisterId) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        self.write_status_register_id = Some(write_id);
        self
    }

    pub const fn read_write_status(mut self, read_id: ReadStatusRegisterId, write_id: WriteStatusRegisterId) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        self.read_status_register_id = Some(read_id);
        self.write_status_register_id = Some(write_id);
        self
    }

    const fn set_location(mut self, location: ChrBankNumberProvider) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        assert!(self.read_status_register_id.is_none(), "Location must be set before read status register.");
        assert!(self.write_status_register_id.is_none(), "Location must be set before write status register.");
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