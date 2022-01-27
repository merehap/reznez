use crate::cpu::address::Address as CpuAddress;
use crate::cpu::memory::{NMI_VECTOR, RESET_VECTOR, IRQ_VECTOR};
use crate::cpu::memory::Memory as CpuMemory;
use crate::cpu::port_access::PortAccess;

use crate::memory::mapper::Mapper;
use crate::memory::ppu_address::PpuAddress;
use crate::memory::ppu_internal_ram::PpuInternalRam;
use crate::memory::stack::Stack;
use crate::ppu::name_table::name_table::NameTable;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::name_table::name_table_number::NameTableNumber;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::pattern_table::PatternTable;

#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]

pub const PALETTE_TABLE_START: PpuAddress = PpuAddress::from_u16(0x3F00);

pub struct Memory {
    mapper: Box<dyn Mapper>,
    cpu_memory: CpuMemory,
    ppu_internal_ram: PpuInternalRam,
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
            ppu_internal_ram: PpuInternalRam::new(name_table_mirroring),
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
        self.mapper.ppu_read(&self.ppu_internal_ram, address)
    }

    #[inline]
    pub fn ppu_write(&mut self, address: PpuAddress, value: u8) {
        self.mapper.ppu_write(&mut self.ppu_internal_ram, address, value)
    }

    #[inline]
    pub fn cpu_memory(&self) -> &CpuMemory {
        &self.cpu_memory
    }

    #[inline]
    pub fn cpu_memory_mut(&mut self) -> &mut CpuMemory {
        &mut self.cpu_memory
    }

    #[inline]
    pub fn stack(&mut self) -> Stack {
        self.cpu_memory.stack()
    }

    #[inline]
    pub fn stack_pointer(&self) -> u8 {
        self.cpu_memory.stack_pointer
    }

    #[inline]
    pub fn stack_pointer_mut(&mut self) -> &mut u8 {
        &mut self.cpu_memory.stack_pointer
    }

    pub fn latch(&self) -> Option<PortAccess> {
        self.cpu_memory.latch()
    }

    pub fn reset_cpu_latch(&mut self) {
        self.cpu_memory.reset_latch()
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

    #[inline]
    pub fn pattern_table(&self) -> PatternTable {
        PatternTable::new(self.mapper.raw_pattern_table())
    }

    #[inline]
    pub fn name_table(&self, number: NameTableNumber) -> NameTable {
        NameTable::new(self.mapper.raw_name_table(&self.ppu_internal_ram, number))
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        PaletteTable::new(self.ppu_internal_ram.palette_ram.to_slice(), &self.system_palette)
    }

    fn address_from_vector(&mut self, mut vector: CpuAddress) -> CpuAddress {
        CpuAddress::from_low_high(
            self.cpu_read(vector),
            self.cpu_read(vector.inc()),
        )
    }
}
