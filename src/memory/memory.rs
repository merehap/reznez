use crate::cpu::address::Address as CpuAddress;
use crate::cpu::memory::{NMI_VECTOR, RESET_VECTOR, IRQ_VECTOR};
use crate::cpu::memory::Memory as CpuMemory;

use crate::memory::mapper::Mapper;

use crate::memory::ppu_address::PpuAddress;
use crate::memory::vram::Vram;
use crate::ppu::name_table::name_table::NameTable;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::name_table::name_table_number::NameTableNumber;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::pattern_table::PatternTable;

const PATTERN_TABLE_START: PpuAddress = PpuAddress::from_u16(0);
const PATTERN_TABLE_SIZE: u16 = 0x2000;

const NAME_TABLE_START: u16 = 0x2000;
const NAME_TABLE_SIZE: u16 = 0x400;
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
const NAME_TABLE_INDEXES: [PpuAddress; 4] =
    [
        PpuAddress::from_u16(NAME_TABLE_START + 0 * NAME_TABLE_SIZE),
        PpuAddress::from_u16(NAME_TABLE_START + 1 * NAME_TABLE_SIZE),
        PpuAddress::from_u16(NAME_TABLE_START + 2 * NAME_TABLE_SIZE),
        PpuAddress::from_u16(NAME_TABLE_START + 3 * NAME_TABLE_SIZE),
    ];

pub const PALETTE_TABLE_START: PpuAddress = PpuAddress::from_u16(0x3F00);
const PALETTE_TABLE_SIZE: u16 = 0x20;

pub struct Memory {
    mapper: Box<dyn Mapper>,
    cpu_memory: CpuMemory,
    vram: Vram,
    system_palette: SystemPalette,
}

impl Memory {
    pub fn new(
        mapper: Box<dyn Mapper>,
        name_table_mirroring: NameTableMirroring,
        system_palette: SystemPalette,
    ) -> Memory {

        Memory {
            mapper,
            cpu_memory: CpuMemory::new(),
            vram: Vram::new(name_table_mirroring),
            system_palette,
        }
    }

    #[inline]
    pub fn cpu_read(&mut self, address: CpuAddress) -> u8 {
        self.mapper.cpu_read(&mut self.cpu_memory, address)
    }

    #[inline]
    pub fn cpu_write(&mut self, address: CpuAddress, value: u8) {
        self.mapper.cpu_write(&mut self.cpu_memory, address, value)
    }

    #[inline]
    pub fn ppu_read(&self, address: PpuAddress) -> u8 {
        self.mapper.ppu_read(&self.vram, address)
    }

    #[inline]
    pub fn ppu_write(&mut self, address: PpuAddress, value: u8) {
        self.mapper.ppu_write(&mut self.vram, address, value)
    }

    #[inline]
    pub fn cpu_memory(&self) -> &CpuMemory {
        &self.cpu_memory
    }

    #[inline]
    pub fn cpu_memory_mut(&mut self) -> &mut CpuMemory {
        &mut self.cpu_memory
    }

    pub fn nmi_vector(&mut self) -> CpuAddress {
        self.address_from_vector(NMI_VECTOR)
    }

    pub fn reset_vector(&mut self) -> CpuAddress {
        self.address_from_vector(RESET_VECTOR)
    }

    pub fn irq_vector(&mut self) -> CpuAddress {
        self.address_from_vector(IRQ_VECTOR)
    }

    fn address_from_vector(&mut self, mut vector: CpuAddress) -> CpuAddress {
        CpuAddress::from_low_high(
            self.cpu_read(vector),
            self.cpu_read(vector.inc()),
        )
    }

    #[inline]
    pub fn pattern_table(&self) -> PatternTable {
        let raw = self.mapper.ppu_slice(
            &self.vram,
            PATTERN_TABLE_START,
            PATTERN_TABLE_START.advance(PATTERN_TABLE_SIZE - 1),
        );
        PatternTable::new(raw.try_into().unwrap())
    }

    #[inline]
    pub fn name_table(&self, number: NameTableNumber) -> NameTable {
        let index = NAME_TABLE_INDEXES[number as usize];
        let raw = self.mapper.ppu_slice(
            &self.vram,
            index,
            index.advance(NAME_TABLE_SIZE - 1),
        );
        NameTable::new(raw.try_into().unwrap())
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        let raw = self.mapper.ppu_slice(
            &self.vram,
            PALETTE_TABLE_START,
            PALETTE_TABLE_START.advance(PALETTE_TABLE_SIZE - 1)
        );
        PaletteTable::new(raw.try_into().unwrap(), &self.system_palette)
    }
}
