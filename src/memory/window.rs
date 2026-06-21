use std::num::NonZeroU16;

use crate::memory::address_template::address_resolver::AddressResolver;
use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::bank::bank::{PrgSourceRegisterId, ReadStatusRegisterId, WriteStatusRegisterId};
use crate::memory::bank::bank_number::{BankNumber, ChrBankRegisters, PrgBankRegisters, PrgBankRegisterId, MemSpace, ReadStatus, WriteStatus};
use crate::util::unit::{KIBIBYTE, KIBIBYTE_U16};

use super::bank::bank::{ChrBank, ChrBankNumberProvider};
use super::bank::bank_number::ChrBankRegisterId;

// A PrgWindow is a range within the CPU address space.
// If a single bank is not enough to fill the window, then subsequent banks will be included too.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PrgWindow {
    start: PrgWindowStart,
    end: PrgWindowEnd,
    size: PrgWindowSize,
    prg_source_provider: PrgSourceProvider,
    bank_number_provider: PrgBankNumberProvider,
    read_status_register_id: Option<ReadStatusRegisterId>,
    write_status_register_id: Option<WriteStatusRegisterId>,
    rom_address_template: Option<AddressResolver<PrgBankRegisterId>>,
}

impl PrgWindow {
    pub const fn new(start: u16, end: u16, size: u32, prg_source_provider: PrgSourceProvider) -> Self {
        let start = PrgWindowStart::new(start);
        let end = PrgWindowEnd::new(end);
        let size = PrgWindowSize::new(size, start, end);
        Self {
            start,
            end,
            size,
            prg_source_provider,
            bank_number_provider: PrgBankNumberProvider::Fixed(BankNumber::ZERO),
            read_status_register_id: None,
            write_status_register_id: None,
            rom_address_template: None,
        }
    }

    pub const fn start(self) -> u16 {
        self.start.0
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end.0
    }

    pub const fn size(self) -> PrgWindowSize {
        self.size
    }

    pub const fn register_id(self) -> Option<PrgBankRegisterId> {
        self.bank_register_id()
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start.0 <= address && address <= self.end.0.get() {
            Some(address - self.start.0)
        } else {
            None
        }
    }

    pub fn get_rom_address_template(&self, bank_sizes: &BankSizes) -> AddressResolver<PrgBankRegisterId> {
        self.rom_address_template_override()
            .map_or(AddressResolver::prg(self, bank_sizes, 0), |template| template.reduced(bank_sizes))
    }

    pub fn ram_address_template(&self, bank_sizes: &BankSizes, work_ram_start_inner_bank_number: u16) -> AddressResolver<PrgBankRegisterId> {
        AddressResolver::prg(self, bank_sizes, work_ram_start_inner_bank_number)
    }

    pub const fn validate_rom_address_template_width(&self, max_rom_size: u32) {
        if let Some(rom_address_template) = self.rom_address_template_override() {
            let max_width = (max_rom_size - 1).count_ones() as u8;
            let template_width = rom_address_template.total_width();
            let segment_count = rom_address_template.segment_count();
            const_panic::concat_assert!(template_width == max_width,
                "Override ROM Address Template was not the correct bit width. Expected ", max_width, ", Found ", template_width,
                " Segment count: ", segment_count);
        }
    }

    pub const fn fixed_number(mut self, index: i16) -> Self {
        assert!(
            self.prg_source_provider.is_mapped(),
            "An ABSENT bank can't be fixed_index."
        );
        self.bank_number_provider =
            PrgBankNumberProvider::Fixed(BankNumber::from_i16(index));
        self
    }

    pub const fn switchable(mut self, register_id: PrgBankRegisterId) -> Self {
        assert!(
            self.prg_source_provider.is_mapped(),
            "An ABSENT bank can't be switchable."
        );
        self.bank_number_provider = PrgBankNumberProvider::Switchable(register_id);
        self
    }

    pub const fn read_status(mut self, read_id: ReadStatusRegisterId) -> Self {
        assert!(
            self.prg_source_provider.is_mapped(),
            "An ABSENT bank can't have a read status register."
        );
        self.read_status_register_id = Some(read_id);
        self
    }

    pub const fn write_status(mut self, write_id: WriteStatusRegisterId) -> Self {
        assert!(
            self.prg_source_provider.is_mapped(),
            "An ABSENT bank can't have a write status register."
        );
        self.write_status_register_id = Some(write_id);
        self
    }

    pub const fn read_write_status(
        mut self,
        read_id: ReadStatusRegisterId,
        write_id: WriteStatusRegisterId,
    ) -> Self {
        assert!(
            self.prg_source_provider.is_mapped(),
            "An ABSENT bank can't have a read or write status register."
        );
        self.read_status_register_id = Some(read_id);
        self.write_status_register_id = Some(write_id);
        self
    }

    pub const fn rom_ram_register(mut self, id: PrgSourceRegisterId) -> Self {
        assert!(
            self.prg_source_provider.is_switchable(),
            "Only ROM_RAM may have a rom ram register."
        );
        self.prg_source_provider = PrgSourceProvider::Switchable(id);
        self
    }

    pub const fn rom_address_template(mut self, template: &'static str) -> Self {
        assert!(
            self.prg_source_provider.is_mapped(),
            "An ABSENT bank can't have an override ROM address template."
        );
        match AddressResolver::from_formatted(template, 0) {
            Ok(template) => self.rom_address_template = Some(template),
            Err(err) => panic!("{}", err),
        }

        self
    }

    pub const fn source_provider(&self) -> PrgSourceProvider {
        self.prg_source_provider
    }

    pub const fn is_rom(self) -> bool {
        matches!(
            self.prg_source_provider,
            PrgSourceProvider::Fixed(Some(PrgSource::Rom))
                | PrgSourceProvider::Switchable(_)
        )
    }

    pub const fn supports_ram(self) -> bool {
        matches!(
            self.prg_source_provider,
            PrgSourceProvider::Fixed(Some(PrgSource::RamOrAbsent | PrgSource::RamOrRom))
                | PrgSourceProvider::Switchable(_)
        )
    }

    pub fn is_absent(self) -> bool {
        self.prg_source_provider == PrgSourceProvider::Fixed(None)
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

    pub const fn prg_bank_number_provider(self) -> PrgBankNumberProvider {
        self.bank_number_provider
    }

    pub const fn bank_register_id(self) -> Option<PrgBankRegisterId> {
        match self.bank_number_provider {
            PrgBankNumberProvider::Fixed(_) => None,
            PrgBankNumberProvider::Switchable(reg_id) => Some(reg_id),
        }
    }

    pub const fn rom_address_template_override(self) -> Option<AddressResolver<PrgBankRegisterId>> {
        self.rom_address_template
    }


    // FIXME: Use explicit rom_read_status() and ram_read_status() providers, then simplify this accordingly.
    pub fn page_number_space(self, regs: &PrgBankRegisters) -> Option<MemSpace> {
        let prg_source = match self.prg_source_provider {
            PrgSourceProvider::Fixed(prg_source) => prg_source,
            PrgSourceProvider::Switchable(reg_id) => Some(regs.rom_ram_mode(reg_id)),
        }?;

        let read_status = self
            .read_status_register_id
            .map_or(ReadStatus::Enabled, |id| regs.read_status(id));
        let write_status = self
            .write_status_register_id
            .map_or(WriteStatus::Enabled, |id| regs.write_status(id));

        // There's currently no way to set make the ROM ReadStatus of a RomRam bank switchable.
        if self.is_rom_ram() && (prg_source == PrgSource::Rom || !regs.cartridge_has_ram()) {
            return Some(MemSpace::Rom(ReadStatus::Enabled));
        }

        match prg_source {
            PrgSource::RamOrRom | PrgSource::RamOrAbsent if regs.cartridge_has_ram() => {
                Some(MemSpace::Ram(read_status, write_status))
            }
            PrgSource::Rom | PrgSource::RamOrRom => {
                Some(MemSpace::Rom(read_status))
            }
            PrgSource::RamOrAbsent => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgSourceProvider {
    Fixed(Option<PrgSource>),
    Switchable(PrgSourceRegisterId),
}

impl PrgSourceProvider {
    pub const ABSENT:          Self = Self::Fixed(None);
    pub const RAM_OR_ABSENT:   Self = Self::Fixed(Some(PrgSource::RamOrAbsent));
    pub const ROM:             Self = Self::Fixed(Some(PrgSource::Rom));
    pub const WORK_RAM_OR_ROM: Self = Self::Fixed(Some(PrgSource::RamOrRom));
    pub const ROM_RAM:         Self = Self::Switchable(PrgSourceRegisterId::PS0);

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
    RamOrRom,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgBankNumberProvider {
    Fixed(BankNumber),
    Switchable(PrgBankRegisterId),
}

impl PrgBankNumberProvider {
    fn bank_number(self, regs: &PrgBankRegisters) -> BankNumber {
        match self {
            Self::Fixed(bank_number) => bank_number,
            Self::Switchable(register_id) => regs.get(register_id),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChrWindow {
    start: ChrWindowStart,
    end: ChrWindowEnd,
    size: ChrWindowSize,
    bank: ChrBank,
}

impl ChrWindow {
    pub const fn new(start: u16, end: u16, size: u32, bank: ChrBank) -> Self {
        let start = ChrWindowStart::new(start);
        let end = ChrWindowEnd::new(end);
        let size = ChrWindowSize::new(size, start, end);
        Self { start, end, size, bank }
    }

    pub const fn start(self) -> u16 {
        self.start.0
    }

    pub const fn end(self) -> NonZeroU16 {
        self.end.0
    }

    pub const fn size(self) -> ChrWindowSize {
        self.size
    }

    pub const fn bank(self) -> ChrBank {
        self.bank
    }

    pub fn is_in_bounds(self, address: u16) -> bool {
        self.start.0 <= address && address <= self.end.0.get()
    }

    pub fn location(self) -> Result<ChrBankNumberProvider, String> {
        self.bank.location()
    }

    pub const fn register_id(self, regs: &ChrBankRegisters) -> Option<ChrBankRegisterId> {
        self.bank.register_id(regs)
    }

    pub fn offset(self, address: u16) -> Option<u16> {
        if self.start.0 <= address && address <= self.end.0.get() {
            Some(address - self.start.0)
        } else {
            None
        }
    }

    pub fn rom_address_template(&self, bank_sizes: &BankSizes) -> AddressResolver<ChrBankRegisterId> {
        self.bank().rom_address_template_override()
            .map_or(AddressResolver::chr(self.bank.chr_bank_number_provider(), self.size, bank_sizes, 0), |template| template.reduced(bank_sizes))
    }

    pub fn ram_address_template(&self, bank_sizes: &BankSizes, work_ram_start_inner_bank_number: u16) -> AddressResolver<ChrBankRegisterId> {
        AddressResolver::chr(self.bank.chr_bank_number_provider(), self.size, bank_sizes, work_ram_start_inner_bank_number)
    }

    pub const fn validate_rom_address_template_width(&self, max_rom_size: u32) {
        if let Some(rom_address_template) = self.bank().rom_address_template_override() {
            let max_width = (max_rom_size - 1).count_ones() as u8;
            let template_width = rom_address_template.total_width();
            let segment_count = rom_address_template.segment_count();
            const_panic::concat_assert!(template_width == max_width,
                "Override ROM Address Template was not the correct bit width. Expected ", max_width, ", Found ", template_width,
                " Segment count: ", segment_count);
        }
    }
}

const PRG_PAGE_SIZE: u16 = 8 * KIBIBYTE as u16;
const PRG_SUB_PAGE_SIZE: u16 = KIBIBYTE as u16 / 8;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PrgWindowStart(u16);

impl PrgWindowStart {
    const fn new(address: u16) -> Self {
        assert!(
            address >= 0x6000,
            "PrgWindow start address must be equal to or greater than 0x6000."
        );
        assert!(
            address.is_multiple_of(PRG_SUB_PAGE_SIZE),
            "PrgWindow start address must be a multiple of 0x80 (128)."
        );
        Self(address)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PrgWindowEnd(NonZeroU16);

impl PrgWindowEnd {
    const fn new(address: u16) -> Self {
        assert!(
            address > 0x6000,
            "PrgWindow end address must be greater than 0x6000."
        );
        assert!(
            address.wrapping_add(1).is_multiple_of(PRG_SUB_PAGE_SIZE),
            "PrgWindow end address must be a multiple of 0x80 (128), minus 1."
        );
        Self(NonZeroU16::new(address).unwrap())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PrgWindowSize(u16);

impl PrgWindowSize {
    pub const MIN: Self = Self(8 * KIBIBYTE as u16);

    pub const fn from_raw(size: u32) -> Self {
        assert!(
            size >= KIBIBYTE / 8,
            "PrgWindow sizes must be at least 128 (0x80) bytes."
        );
        assert!(
            size <= 32 * KIBIBYTE,
            "PrgWindow sizes must be at most 32 kibibytes."
        );

        let size = size as u16;
        assert!(
            size.is_multiple_of(PRG_SUB_PAGE_SIZE),
            "PrgWindow sizes must be multiples of 128 bytes."
        );

        Self(size)
    }

    const fn new(size: u32, start: PrgWindowStart, end: PrgWindowEnd) -> Self {
        assert!(
            end.0.get() > start.0,
            "PrgWindow end address was less than its start address."
        );
        assert!(
            end.0.get() - start.0 + 1 == size as u16,
            "PrgWindow size was must equal the end address minus the start address, plus one."
        );

        Self::from_raw(size)
    }

    pub const fn page_multiple(self) -> u16 {
        self.0 / PRG_PAGE_SIZE
    }

    pub fn sub_page_multiple(self) -> u8 {
        u8::try_from((self.0 % PRG_PAGE_SIZE) / PRG_SUB_PAGE_SIZE).unwrap()
    }

    pub const fn bit_count(self) -> u8 {
        assert!(self.0.is_power_of_two());
        (self.0 - 1).count_ones() as u8
    }

    pub const fn to_raw(self) -> u16 {
        self.0
    }
}

const CHR_PAGE_SIZE: u16 = KIBIBYTE as u16;

#[derive(Clone, Copy, Debug)]
pub struct ChrWindowStart(u16);

impl ChrWindowStart {
    const fn new(address: u16) -> Self {
        assert!(
            address < 0x4000,
            "ChrWindow start address must be less than 0x4000."
        );
        assert!(
            address.is_multiple_of(CHR_PAGE_SIZE),
            "ChrWindow start address must be a multiple of 0x400."
        );
        Self(address)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChrWindowEnd(NonZeroU16);

impl ChrWindowEnd {
    const fn new(address: u16) -> Self {
        assert!(
            address < 0x4000,
            "ChrWindow end address must be less than 0x4000."
        );
        assert!(
            address.wrapping_add(1).is_multiple_of(CHR_PAGE_SIZE),
            "ChrWindow end address must be a multiple of 0x400, minus 1."
        );
        Self(
            NonZeroU16::new(address)
                .expect("ChrWindow end address to be greater than 0."),
        )
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct ChrWindowSize(u16);

impl ChrWindowSize {
    pub const NAME_TABLE_WINDOW_SIZE: Self = Self(1 * KIBIBYTE_U16);

    const fn new(size: u32, start: ChrWindowStart, end: ChrWindowEnd) -> Self {
        assert!(
            size >= KIBIBYTE,
            "ChrWindow sizes must be at least 1 kibibyte."
        );
        assert!(
            size <= 8 * KIBIBYTE,
            "ChrWindow sizes must be at most 8 kibibytes."
        );
        let size = size as u16;

        assert!(
            end.0.get() > start.0,
            "ChrWindow end address was less than its start address."
        );
        assert!(
            end.0.get() - start.0 + 1 == size,
            "ChrWindow size was must equal the end address minus the start address, plus one."
        );

        Self(size)
    }

    pub fn page_multiple(self) -> u16 {
        self.0 / CHR_PAGE_SIZE
    }

    pub const fn bit_count(self) -> u8 {
        assert!(self.0.is_power_of_two());
        (self.0 - 1).count_ones() as u8
    }

    pub const fn to_raw(self) -> u16 {
        self.0
    }
}
