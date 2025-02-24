use crate::memory::bank::bank::RamStatusRegisterId;
use crate::memory::ppu::ciram::CiramSide;

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

    pub fn to_u16(self, bank_configuration: BankConfiguration, window_size: u16) -> u16 {
        let mut resolved_bank_index = self.0 % bank_configuration.bank_count;
        if bank_configuration.align_large_layouts {
            let window_multiple = window_size / bank_configuration.bank_size;
            // Clear low bits for large windows.
            resolved_bank_index &= !(window_multiple - 1);
        }

        resolved_bank_index
    }
}

impl From<u8> for BankIndex {
    fn from(value: u8) -> Self {
        BankIndex(value.into())
    }
}

#[derive(Clone, Copy)]
pub struct BankConfiguration {
    bank_size: u16,
    bank_count: u16,
    align_large_layouts: bool,
}

impl BankConfiguration {
    pub fn new(bank_size: u16, bank_count: u16, align_large_layouts: bool) -> Self {
        Self { bank_size, bank_count, align_large_layouts }
    }

    pub fn bank_size(self) -> u16 {
        self.bank_size
    }

    pub fn bank_count(self) -> u16 {
        self.bank_count
    }
}

#[derive(Debug)]
pub struct BankRegisters {
    registers: [BankLocation; 18],
    meta_registers: [BankRegisterId; 2],
    ram_statuses: [RamStatus; 15],
}

impl BankRegisters {
    pub fn new() -> Self {
        Self {
            registers: [BankLocation::Index(BankIndex(0)); 18],
            // Meta registers are only used for CHR currently.
            meta_registers: [BankRegisterId::C0, BankRegisterId::C0],
            ram_statuses: [RamStatus::ReadWrite; 15],
        }
    }

    pub fn get(&self, id: BankRegisterId) -> BankLocation {
        self.registers[id as usize]
    }

    pub fn set(&mut self, id: BankRegisterId, bank_index: BankIndex) {
        self.registers[id as usize] = BankLocation::Index(bank_index);
    }

    pub fn set_bits(&mut self, id: BankRegisterId, new_value: u16, mask: u16) {
        let value = self.registers[id as usize].index()
            .unwrap_or_else(|| panic!("bank location at id {id:?} to not be in VRAM"));
        let updated_value = (value.0 & !mask) | (new_value & mask);
        self.registers[id as usize] = BankLocation::Index(BankIndex(updated_value));
    }

    pub fn update(&mut self, id: BankRegisterId, updater: &dyn Fn(u16) -> u16) {
        let value = self.registers[id as usize].index()
            .unwrap_or_else(|| panic!("bank location at id {id:?} to not be in VRAM"));
        self.registers[id as usize] = BankLocation::Index(BankIndex(updater(value.0)));
    }

    pub fn set_to_ciram_side(&mut self, id: BankRegisterId, ciram_side: CiramSide) {
        self.registers[id as usize] = BankLocation::Ciram(ciram_side);
    }

    pub fn get_from_meta(&self, id: MetaRegisterId) -> BankLocation {
        self.get(self.meta_registers[id as usize])
    }

    pub fn set_meta(&mut self, id: MetaRegisterId, value: BankRegisterId) {
        self.meta_registers[id as usize] = value;
    }

    pub fn ram_status(&self, id: RamStatusRegisterId) -> RamStatus {
        self.ram_statuses[id as usize]
    }

    pub fn set_ram_status(&mut self, id: RamStatusRegisterId, status: RamStatus) {
        self.ram_statuses[id as usize] = status;
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BankRegisterId {
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

    P0,
    P1,
    P2,
    P3,
    P4,
}

impl BankRegisterId {
    pub fn chr_id(id: u16) -> BankRegisterId {
        use BankRegisterId::*;
        match id {
            0 => C0,
            1 => C1,
            2 => C2,
            3 => C3,
            4 => C4,
            5 => C5,
            6 => C6,
            7 => C7,
            8 => C8,
            9 => C9,
            10 => C10,
            11 => C11,
            12 => C12,
            _ => panic!("Bad CHR ID: {id}"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MetaRegisterId {
    M0,
    M1,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RamStatus {
    Disabled,
    ReadOnlyZeros,
    ReadOnly,
    ReadWrite,
    WriteOnly,
}

impl RamStatus {
    pub fn is_writable(self) -> bool {
        matches!(self, RamStatus::ReadWrite | RamStatus::WriteOnly)
    }
}