use num_derive::FromPrimitive;

use crate::memory::bank::bank::ReadWriteStatusRegisterId;
use crate::memory::ppu::ciram::CiramSide;

use super::bank::RomRamModeRegisterId;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BankIndex(u16);

impl BankIndex {
    pub const fn from_u8(value: u8) -> BankIndex {
        BankIndex(value as u16)
    }

    pub const fn from_u16(value: u16) -> BankIndex {
        BankIndex(value)
    }

    pub const fn from_i16(value: i16) -> BankIndex {
        BankIndex(value as u16)
    }

    pub fn to_raw(self) -> u16 {
        self.0
    }
}

impl From<u8> for BankIndex {
    fn from(value: u8) -> Self {
        BankIndex(value.into())
    }
}

#[derive(Debug)]
pub struct PrgBankRegisters {
    registers: [BankLocation; 5],
    read_write_statuses: [ReadWriteStatus; 15],
    rom_ram_modes: [MemType; 12],
}

impl PrgBankRegisters {
    pub fn new() -> Self {
        Self {
            registers: [BankLocation::Index(BankIndex(0)); 5],
            read_write_statuses: [ReadWriteStatus::ReadWrite; 15],
            rom_ram_modes: [MemType::WorkRam; 12],
        }
    }

    pub fn registers(&self) -> &[BankLocation; 5] {
        &self.registers
    }

    pub fn read_write_statuses(&self) -> &[ReadWriteStatus; 15] {
        &self.read_write_statuses
    }

    pub fn get(&self, id: PrgBankRegisterId) -> BankLocation {
        self.registers[id as usize]
    }

    pub fn set(&mut self, id: PrgBankRegisterId, bank_index: BankIndex) {
        self.registers[id as usize] = BankLocation::Index(bank_index);
    }

    pub fn set_bits(&mut self, id: PrgBankRegisterId, new_value: u16, mask: u16) {
        let value = self.registers[id as usize].index()
            .unwrap_or_else(|| panic!("bank location at id {id:?} to not be in VRAM"));
        let updated_value = (value.0 & !mask) | (new_value & mask);
        self.registers[id as usize] = BankLocation::Index(BankIndex(updated_value));
    }

    pub fn update(&mut self, id: PrgBankRegisterId, updater: &dyn Fn(u16) -> u16) {
        let value = self.registers[id as usize].index()
            .unwrap_or_else(|| panic!("bank location at id {id:?} to not be in VRAM"));
        self.registers[id as usize] = BankLocation::Index(BankIndex(updater(value.0)));
    }

    pub fn set_to_ciram_side(&mut self, id: PrgBankRegisterId, ciram_side: CiramSide) {
        self.registers[id as usize] = BankLocation::Ciram(ciram_side);
    }

    pub fn read_write_status(&self, id: ReadWriteStatusRegisterId) -> ReadWriteStatus {
        self.read_write_statuses[id as usize]
    }

    pub fn set_read_write_status(&mut self, id: ReadWriteStatusRegisterId, status: ReadWriteStatus) {
        self.read_write_statuses[id as usize] = status;
    }

    pub fn rom_ram_mode(&self, id: RomRamModeRegisterId) -> MemType {
        self.rom_ram_modes[id as usize]
    }

    pub fn set_rom_ram_mode(&mut self, id: RomRamModeRegisterId, rom_ram_mode: MemType) {
        self.rom_ram_modes[id as usize] = rom_ram_mode;
    }
}

#[derive(Clone, Debug)]
pub struct ChrBankRegisters {
    registers: [BankLocation; 18],
    chr_meta_registers: [ChrBankRegisterId; 2],
    read_write_statuses: [ReadWriteStatus; 15],
    rom_ram_modes: [MemType; 12],
}

impl ChrBankRegisters {
    pub fn new() -> Self {
        Self {
            registers: [BankLocation::Index(BankIndex(0)); 18],
            // Meta registers are only used for CHR currently.
            chr_meta_registers: [ChrBankRegisterId::C0, ChrBankRegisterId::C0],
            read_write_statuses: [ReadWriteStatus::ReadWrite; 15],
            rom_ram_modes: [MemType::WorkRam; 12],
        }
    }

    pub fn registers(&self) -> &[BankLocation; 18] {
        &self.registers
    }

    pub fn meta_registers(&self) -> &[ChrBankRegisterId; 2] {
        &self.chr_meta_registers
    }

    pub fn read_write_statuses(&self) -> &[ReadWriteStatus; 15] {
        &self.read_write_statuses
    }

    pub fn get(&self, id: ChrBankRegisterId) -> BankLocation {
        self.registers[id as usize]
    }

    pub fn set(&mut self, id: ChrBankRegisterId, bank_index: BankIndex) {
        self.registers[id as usize] = BankLocation::Index(bank_index);
    }

    pub fn set_bits(&mut self, id: ChrBankRegisterId, new_value: u16, mask: u16) {
        let value = self.registers[id as usize].index()
            .unwrap_or_else(|| panic!("bank location at id {id:?} to not be in VRAM"));
        let updated_value = (value.0 & !mask) | (new_value & mask);
        self.registers[id as usize] = BankLocation::Index(BankIndex(updated_value));
    }

    pub fn update(&mut self, id: ChrBankRegisterId, updater: &dyn Fn(u16) -> u16) {
        let value = self.registers[id as usize].index()
            .unwrap_or_else(|| panic!("bank location at id {id:?} to not be in VRAM"));
        self.registers[id as usize] = BankLocation::Index(BankIndex(updater(value.0)));
    }

    pub fn set_to_ciram_side(&mut self, id: ChrBankRegisterId, ciram_side: CiramSide) {
        self.registers[id as usize] = BankLocation::Ciram(ciram_side);
    }

    pub fn get_from_meta(&self, id: MetaRegisterId) -> BankLocation {
        self.get(self.chr_meta_registers[id as usize])
    }

    pub fn set_meta_chr(&mut self, id: MetaRegisterId, value: ChrBankRegisterId) {
        self.chr_meta_registers[id as usize] = value;
    }

    pub fn read_write_status(&self, id: ReadWriteStatusRegisterId) -> ReadWriteStatus {
        self.read_write_statuses[id as usize]
    }

    pub fn set_read_write_status(&mut self, id: ReadWriteStatusRegisterId, status: ReadWriteStatus) {
        self.read_write_statuses[id as usize] = status;
    }

    pub fn rom_ram_mode(&self, id: RomRamModeRegisterId) -> MemType {
        self.rom_ram_modes[id as usize]
    }

    pub fn set_rom_ram_mode(&mut self, id: RomRamModeRegisterId, rom_ram_mode: MemType) {
        self.rom_ram_modes[id as usize] = rom_ram_mode;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BankLocation {
    Index(BankIndex),
    Ciram(CiramSide),
}

impl BankLocation {
    pub fn index(self) -> Option<BankIndex> {
        if let BankLocation::Index(index) = self {
            Some(index)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum PrgBankRegisterId {
    P0,
    P1,
    P2,
    P3,
    P4,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum ChrBankRegisterId {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
    C12,
}

impl ChrBankRegisterId {
    pub fn to_raw_chr_id(self) -> u8 {
        self as u8
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MetaRegisterId {
    M0,
    M1,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ReadWriteStatus {
    Disabled,
    ReadOnlyZeros,
    ReadOnly,
    ReadWrite,
    WriteOnly,
}

impl ReadWriteStatus {
    pub fn is_readable(self) -> bool {
        // ReadOnlyZeros is excluded since actual memory can't be read.
        matches!(self, ReadWriteStatus::ReadWrite | ReadWriteStatus::ReadOnly)
    }

    pub fn is_writable(self) -> bool {
        matches!(self, ReadWriteStatus::ReadWrite | ReadWriteStatus::WriteOnly)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MemType {
    WorkRam,
    Rom,
    SaveRam,
}
