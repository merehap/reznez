use strum_macros::Display;

use crate::mapper::CiramSide;
use crate::mapper::NameTableSource;
use crate::memory::address_template::address_resolver::AddressResolver;
use crate::memory::bank::bank::ChrSourceRegisterId::*;
use crate::memory::bank::bank_number::{BankNumber, MetaRegisterId};

use super::bank_number::{ChrBankRegisterId, ChrBankRegisters};

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
    Ciram(CiramSide),
    MapperCustom { page_id: u8 },
}

impl ChrSource {
    pub fn from_name_table_source(name_table_source: NameTableSource) -> (Self, Option<BankNumber>) {
        match name_table_source {
            NameTableSource::Ciram(ciram_side) => (ChrSource::Ciram(ciram_side), None),
            NameTableSource::Rom { bank_number } => (ChrSource::Rom, Some(bank_number)),
            NameTableSource::Ram { bank_number } => (ChrSource::WorkRam, Some(bank_number)),
            NameTableSource::MapperCustom { page_id } => (ChrSource::MapperCustom { page_id }, None),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum ReadStatusRegisterId {
    RS0,
    RS1,
    RS2,
    RS3,
    RS4,
    RS5,
    RS6,
    RS7,
    RS8,
    RS9,
    RS10,
    RS11,
    RS12,
    RS13,
    RS14,
    RS15,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum WriteStatusRegisterId {
    WS0,
    WS1,
    WS2,
    WS3,
    WS4,
    WS5,
    WS6,
    WS7,
    WS8,
    WS9,
    WS10,
    WS11,
    WS12,
    WS13,
    WS14,
    WS15,
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
    NTS0,
    // Name Table Top Right
    NTS1,
    // Name Table Bottom Left
    NTS2,
    // Name Table Bottom Right
    NTS3,
}

impl ChrSourceRegisterId {
    pub const ALL_NAME_TABLE_SOURCE_IDS: [Self; 4] = [
        ChrSourceRegisterId::NTS0,
        ChrSourceRegisterId::NTS1,
        ChrSourceRegisterId::NTS2,
        ChrSourceRegisterId::NTS3,
    ];
}

#[derive(Clone, Copy, Debug)]
pub struct ChrBank {
    bank_number_provider: ChrBankNumberProvider,
    chr_source_provider: ChrSourceProvider,
    read_status_register_id: Option<ReadStatusRegisterId>,
    write_status_register_id: Option<WriteStatusRegisterId>,
    rom_address_template: Option<AddressResolver<ChrBankRegisterId>>,
}

impl ChrBank {
    pub const EMPTY: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(None),
        read_status_register_id: None,
        write_status_register_id: None,
        rom_address_template: None,
    };
    pub const ROM_OR_RAM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::RomOrRam)),
        read_status_register_id: None,
        write_status_register_id: None,
        rom_address_template: None,
    };
    pub const ROM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::Rom)),
        read_status_register_id: None,
        write_status_register_id: None,
        rom_address_template: None,
    };
    pub const RAM: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::WorkRam)),
        read_status_register_id: None,
        write_status_register_id: None,
        rom_address_template: None,
    };
    pub const SWITCHABLE_SOURCE: Self = Self {
        bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
        chr_source_provider: ChrSourceProvider::Switchable(CS0),
        read_status_register_id: None,
        write_status_register_id: None,
        rom_address_template: None,
    };

    pub const fn ciram(ciram_side: CiramSide) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Fixed(Some(ChrSource::Ciram(
                ciram_side,
            ))),
            read_status_register_id: None,
            write_status_register_id: None,
            rom_address_template: None,
        }
    }

    pub const fn with_switchable_source(source_reg_id: ChrSourceRegisterId) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Switchable(source_reg_id),
            read_status_register_id: None,
            write_status_register_id: None,
            rom_address_template: None,
        }
    }

    pub const fn mapper_sourced(page_id: u8) -> Self {
        Self {
            bank_number_provider: ChrBankNumberProvider::FIXED_ZERO,
            chr_source_provider: ChrSourceProvider::Fixed(Some(
                ChrSource::MapperCustom { page_id },
            )),
            read_status_register_id: None,
            write_status_register_id: None,
            rom_address_template: None,
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

    pub const fn rom_address_template(mut self, template: &'static str) -> Self {
        assert!(
            self.chr_source_provider.is_mapped(),
            "An ABSENT bank can't have an override ROM address template."
        );
        match AddressResolver::from_formatted(template, 0) {
            Ok(template) => self.rom_address_template = Some(template),
            Err(err) => panic!("{}", err),
        }

        self
    }

    pub fn current_chr_source(self, regs: &ChrBankRegisters) -> Option<ChrSource> {
        match self.chr_source_provider {
            ChrSourceProvider::Fixed(chr_source) => chr_source,
            ChrSourceProvider::Switchable(reg_id) => Some(regs.chr_source(reg_id)),
        }
    }

    pub const fn rom_presence(self) -> MemoryPresence {
        match self.chr_source_provider {
            ChrSourceProvider::Fixed(Some(ChrSource::Rom)) => MemoryPresence::Required,
            ChrSourceProvider::Switchable(_) | ChrSourceProvider::Fixed(Some(ChrSource::RomOrRam)) => MemoryPresence::Supported,
            ChrSourceProvider::Fixed(_) => MemoryPresence::Absent,
        }
    }

    pub const fn ram_presence(self) -> MemoryPresence {
        match self.chr_source_provider {
            ChrSourceProvider::Fixed(Some(ChrSource::WorkRam)) => MemoryPresence::Required,
            ChrSourceProvider::Switchable(_) | ChrSourceProvider::Fixed(Some(ChrSource::RomOrRam)) => MemoryPresence::Supported,
            ChrSourceProvider::Fixed(_) => MemoryPresence::Absent,
        }
    }

    pub const fn register_id(
        &self,
        regs: &ChrBankRegisters,
    ) -> Option<ChrBankRegisterId> {
        self.bank_number_provider.register_id(regs)
    }

    pub const fn read_status_register_id(&self) -> Option<ReadStatusRegisterId> {
        self.read_status_register_id
    }

    pub const fn write_status_register_id(&self) -> Option<WriteStatusRegisterId> {
        self.write_status_register_id
    }

    pub const fn rom_address_template_override(self) -> Option<AddressResolver<ChrBankRegisterId>> {
        self.rom_address_template
    }

    pub fn location(self) -> Result<ChrBankNumberProvider, String> {
        if self.chr_source_provider.is_mapped() {
            Ok(self.bank_number_provider)
        } else {
            Err("EMPTY banks don't have a location".to_owned())
        }
    }

    pub const fn chr_bank_number_provider(self) -> ChrBankNumberProvider {
        self.bank_number_provider
    }

    pub fn bank_location(self, regs: &ChrBankRegisters) -> Option<BankNumber> {
        self.location()
            .ok()
            .map(|provider| provider.bank_number(regs))
    }

    pub fn bank_number(self, regs: &ChrBankRegisters) -> Option<BankNumber> {
        self.location()
            .ok()
            .map(|provider| provider.bank_number(regs))
    }

    pub const fn chr_source(mut self, id: ChrSourceRegisterId) -> Self {
        assert!(matches!(
            self.chr_source_provider,
            ChrSourceProvider::Switchable(_)
        ));
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

    pub const fn read_write_status(
        mut self,
        read_id: ReadStatusRegisterId,
        write_id: WriteStatusRegisterId,
    ) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        self.read_status_register_id = Some(read_id);
        self.write_status_register_id = Some(write_id);
        self
    }

    const fn set_location(mut self, location: ChrBankNumberProvider) -> Self {
        assert!(self.chr_source_provider.is_mapped());
        assert!(
            self.read_status_register_id.is_none(),
            "Location must be set before read status register."
        );
        assert!(
            self.write_status_register_id.is_none(),
            "Location must be set before write status register."
        );
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

    pub fn bank_number(self, registers: &ChrBankRegisters) -> BankNumber {
        match self {
            Self::Fixed(bank_number) => bank_number,
            Self::Switchable(register_id) => registers.get(register_id),
            Self::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id),
        }
    }

    pub const fn register_id(
        self,
        registers: &ChrBankRegisters,
    ) -> Option<ChrBankRegisterId> {
        match self {
            Self::Fixed(_) => None,
            Self::Switchable(register_id) => Some(register_id),
            Self::MetaSwitchable(meta_id) => {
                Some(registers.get_register_id_from_meta(meta_id))
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Display)]
pub enum MemoryPresence {
    Absent,
    Supported,
    Required,
}