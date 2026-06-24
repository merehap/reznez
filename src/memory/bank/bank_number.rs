use crate::memory::bank::bank::MemoryPresence;
use crate::memory::regions::ciram::CiramSide;
use crate::memory::register_ids::bank::{ChrBankRegisterId, MetaRegisterId, PrgBankRegisterId};
use crate::memory::register_ids::read_write_status::{ReadStatusRegisterId, WriteStatusRegisterId};
use crate::memory::register_ids::source::{ChrSourceRegisterId, PrgSourceRegisterId};
use crate::memory::window::{ChrSource, PrgSource};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BankNumber(u16);

impl BankNumber {
    pub const ZERO: Self = Self(0);

    pub const fn from_u8(value: u8) -> BankNumber {
        BankNumber(value as u16)
    }

    pub const fn from_u16(value: u16) -> BankNumber {
        BankNumber(value)
    }

    pub const fn from_i16(value: i16) -> BankNumber {
        BankNumber(value as u16)
    }

    pub const fn to_raw(self) -> u16 {
        self.0
    }
}

impl From<u8> for BankNumber {
    fn from(value: u8) -> Self {
        BankNumber(value.into())
    }
}

#[derive(Debug)]
pub struct PrgBankRegisters {
    registers: [BankNumber; 11],
    read_statuses: [ReadStatus; 16],
    write_statuses: [WriteStatus; 16],
    rom_ram_modes: [PrgSource; 12],
    cartridge_has_ram: bool,
    work_ram_start_page_number: u16,
}

impl PrgBankRegisters {
    pub fn new(cartridge_has_ram: bool, work_ram_start_page_number: u16) -> Self {
        Self {
            registers: [BankNumber::ZERO; 11],
            read_statuses: [ReadStatus::Enabled; 16],
            write_statuses: [WriteStatus::Enabled; 16],
            rom_ram_modes: [PrgSource::RamOrRom; 12],
            cartridge_has_ram,
            work_ram_start_page_number,
        }
    }

    pub fn registers(&self) -> &[BankNumber; 11] {
        &self.registers
    }

    pub fn read_statuses(&self) -> &[ReadStatus; 16] {
        &self.read_statuses
    }

    pub fn write_statuses(&self) -> &[WriteStatus; 16] {
        &self.write_statuses
    }

    pub fn cartridge_has_ram(&self) -> bool {
        self.cartridge_has_ram
    }

    pub fn work_ram_start_page_number(&self) -> u16 {
        self.work_ram_start_page_number
    }

    pub fn get(&self, id: PrgBankRegisterId) -> BankNumber {
        self.registers[id as usize]
    }

    pub fn set(&mut self, id: PrgBankRegisterId, bank_number: BankNumber) {
        self.registers[id as usize] = bank_number;
    }

    pub fn reset_registers(&mut self) {
        for register in &mut self.registers {
            *register = BankNumber::ZERO;
        }
    }

    pub fn read_status(&self, id: ReadStatusRegisterId) -> ReadStatus {
        self.read_statuses[id as usize]
    }

    pub fn set_read_status(&mut self, id: ReadStatusRegisterId, status: ReadStatus) {
        self.read_statuses[id as usize] = status;
    }

    pub fn write_status(&self, id: WriteStatusRegisterId) -> WriteStatus {
        self.write_statuses[id as usize]
    }

    pub fn set_write_status(&mut self, id: WriteStatusRegisterId, status: WriteStatus) {
        self.write_statuses[id as usize] = status;
    }

    pub fn rom_ram_mode(&self, id: PrgSourceRegisterId) -> PrgSource {
        self.rom_ram_modes[id as usize]
    }

    pub fn set_rom_ram_mode(&mut self, id: PrgSourceRegisterId, rom_ram_mode: PrgSource) {
        self.rom_ram_modes[id as usize] = rom_ram_mode;
    }
}

#[derive(Clone, Debug)]
pub struct ChrBankRegisters {
    registers: [BankNumber; 16],
    chr_meta_registers: [ChrBankRegisterId; 4],
    read_statuses: [ReadStatus; 15],
    write_statuses: [WriteStatus; 15],
    chr_sources: [ChrSource; 12],
    // TODO: These 4 don't belong here, find the proper place for them.
    cartridge_has_rom: bool,
    cartridge_has_ram: bool,
    pub layout_rom_presence: MemoryPresence,
    pub layout_ram_presence: MemoryPresence,
}

impl ChrBankRegisters {
    pub fn new(
        cartridge_has_rom: bool,
        cartridge_has_ram: bool,
        default_chr_source: ChrSource,
    ) -> Self {
        Self {
            registers: [BankNumber::ZERO; 16],
            // Meta registers are only used for CHR currently.
            chr_meta_registers: [ChrBankRegisterId::C; 4],
            read_statuses: [ReadStatus::Enabled; 15],
            write_statuses: [WriteStatus::Enabled; 15],
            chr_sources: [default_chr_source; 12],
            cartridge_has_rom,
            cartridge_has_ram,
            layout_rom_presence: MemoryPresence::Absent,
            layout_ram_presence: MemoryPresence::Absent,
        }
    }

    pub fn registers(&self) -> &[BankNumber; 16] {
        &self.registers
    }

    pub fn meta_registers(&self) -> &[ChrBankRegisterId; 4] {
        &self.chr_meta_registers
    }

    pub fn read_statuses(&self) -> &[ReadStatus; 15] {
        &self.read_statuses
    }

    pub fn write_statuses(&self) -> &[WriteStatus; 15] {
        &self.write_statuses
    }

    pub fn has_rom(&self) -> bool {
        self.cartridge_has_rom && self.layout_rom_presence != MemoryPresence::Absent
    }

    pub fn has_ram(&self) -> bool {
        self.cartridge_has_ram && self.layout_ram_presence != MemoryPresence::Absent
    }

    pub fn cartridge_has_ram(&self) -> bool {
        self.cartridge_has_ram
    }

    pub fn layout_rom_presence(&self) -> MemoryPresence {
        self.layout_rom_presence
    }

    pub fn layout_ram_presence(&self) -> MemoryPresence {
        self.layout_ram_presence
    }

    pub fn get(&self, id: ChrBankRegisterId) -> BankNumber {
        self.registers[id as usize]
    }

    pub fn set(&mut self, id: ChrBankRegisterId, bank_number: BankNumber) {
        self.registers[id as usize] = bank_number;
    }

    pub fn set_bits(&mut self, id: ChrBankRegisterId, new_value: u16, mask: u16) {
        let value = self.registers[id as usize];
        let updated_value = (value.0 & !mask) | (new_value & mask);
        self.registers[id as usize] = BankNumber(updated_value);
    }

    pub fn update(&mut self, id: ChrBankRegisterId, updater: &dyn Fn(u16) -> u16) {
        let value = self.registers[id as usize];
        self.registers[id as usize] = BankNumber(updater(value.0));
    }

    pub fn set_to_ciram_side(&mut self, id: ChrSourceRegisterId, ciram_side: CiramSide) {
        self.chr_sources[id as usize] = ChrSource::Ciram(ciram_side);
    }

    pub fn get_from_meta(&self, id: MetaRegisterId) -> BankNumber {
        self.get(self.get_register_id_from_meta(id))
    }

    pub fn set_meta_chr(&mut self, id: MetaRegisterId, value: ChrBankRegisterId) {
        self.chr_meta_registers[id as usize] = value;
    }

    pub const fn get_register_id_from_meta(&self, id: MetaRegisterId) -> ChrBankRegisterId {
        self.chr_meta_registers[id as usize]
    }

    pub fn reset_registers(&mut self) {
        for register in &mut self.registers {
            *register = BankNumber::ZERO;
        }

        for register in &mut self.chr_meta_registers {
            *register = ChrBankRegisterId::C;
        }
    }

    pub fn read_status(&self, id: ReadStatusRegisterId) -> ReadStatus {
        self.read_statuses[id as usize]
    }

    pub fn set_read_status(&mut self, id: ReadStatusRegisterId, status: ReadStatus) {
        self.read_statuses[id as usize] = status;
    }

    pub fn write_status(&self, id: WriteStatusRegisterId) -> WriteStatus {
        self.write_statuses[id as usize]
    }

    pub fn set_write_status(&mut self, id: WriteStatusRegisterId, status: WriteStatus) {
        self.write_statuses[id as usize] = status;
    }

    pub fn chr_source(&self, id: ChrSourceRegisterId) -> ChrSource {
        self.chr_sources[id as usize]
    }

    pub fn set_chr_source(&mut self, id: ChrSourceRegisterId, chr_source: ChrSource) {
        self.chr_sources[id as usize] = chr_source;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ReadStatus {
    Disabled,
    Enabled,
    ReadOnlyZeros,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum WriteStatus {
    Disabled,
    Enabled,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgMemTypeStatus {
    Rom(ReadStatus),
    WorkRam(ReadStatus, WriteStatus),
    SaveRam(ReadStatus, WriteStatus),
}

impl PrgMemTypeStatus {
    pub fn is_rom(self) -> bool {
        matches!(self, Self::Rom(_))
    }

    pub fn read_status(self) -> ReadStatus {
        match self {
            Self::Rom(read_status) => read_status,
            Self::WorkRam(read_status, ..) => read_status,
            Self::SaveRam(read_status, ..) => read_status,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum MemSpace {
    Rom(ReadStatus),
    Ram(ReadStatus, WriteStatus),
}
