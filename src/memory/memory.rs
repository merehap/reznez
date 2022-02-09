use crate::memory::cpu_address::CpuAddress;
use crate::memory::cpu_internal_ram::{CpuInternalRam, NMI_VECTOR, RESET_VECTOR, IRQ_VECTOR};
use crate::memory::mapper::Mapper;
use crate::memory::ports::Ports;
use crate::memory::ppu_address::PpuAddress;
use crate::memory::ppu_internal_ram::PpuInternalRam;
use crate::memory::stack::Stack;
use crate::ppu::name_table::name_table::NameTable;
use crate::ppu::name_table::name_table_number::NameTableNumber;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::ppu::register::ppu_registers::PpuRegisters;

pub const PALETTE_TABLE_START: PpuAddress = PpuAddress::from_u16(0x3F00);

pub struct Memory {
    mapper: Box<dyn Mapper>,
    cpu_internal_ram: CpuInternalRam,
    ppu_internal_ram: PpuInternalRam,
    ports: Ports,
    ppu_registers: PpuRegisters,
    system_palette: SystemPalette,
}

impl Memory {
    pub fn new(
        mapper: Box<dyn Mapper>,
        ports: Ports,
        system_palette: SystemPalette,
    ) -> Memory {
        Memory {
            mapper,
            cpu_internal_ram: CpuInternalRam::new(),
            ppu_internal_ram: PpuInternalRam::new(),
            ports,
            ppu_registers: PpuRegisters::new(),
            system_palette,
        }
    }

    pub fn as_cpu_memory(&mut self) -> CpuMemory {
        CpuMemory {memory: self}
    }

    pub fn as_ppu_memory(&mut self) -> PpuMemory {
        PpuMemory {memory: self}
    }

    pub fn stack_pointer(&self) -> u8 {
        self.cpu_internal_ram.stack_pointer
    }
}

pub struct CpuMemory<'a> {
    memory: &'a mut Memory,
}

impl <'a> CpuMemory<'a> {
    #[inline]
    pub fn cpu_read(&mut self, address: CpuAddress) -> u8 {
        self.memory.mapper.cpu_read(
            &self.memory.cpu_internal_ram,
            &mut self.memory.ports,
            &mut self.memory.ppu_registers,
            address,
        )
    }

    #[inline]
    pub fn cpu_write(&mut self, address: CpuAddress, value: u8) {
        self.memory.mapper.cpu_write(
            &mut self.memory.cpu_internal_ram,
            &mut self.memory.ports,
            &mut self.memory.ppu_registers,
            address,
            value,
        )
    }

    pub fn ports(&self) -> &Ports {
        &self.memory.ports
    }

    #[inline]
    pub fn stack(&mut self) -> Stack {
        self.memory.cpu_internal_ram.stack()
    }

    #[inline]
    pub fn stack_pointer(&self) -> u8 {
        self.memory.cpu_internal_ram.stack_pointer
    }

    #[inline]
    pub fn stack_pointer_mut(&mut self) -> &mut u8 {
        &mut self.memory.cpu_internal_ram.stack_pointer
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
}

pub struct PpuMemory<'a> {
    memory: &'a mut Memory,
}

impl <'a> PpuMemory<'a> {
    #[inline]
    pub fn ppu_read(&self, address: PpuAddress) -> u8 {
        self.memory.mapper.ppu_read(&self.memory.ppu_internal_ram, address)
    }

    #[inline]
    pub fn ppu_write(&mut self, address: PpuAddress, value: u8) {
        self.memory.mapper.ppu_write(&mut self.memory.ppu_internal_ram, address, value)
    }

    #[inline]
    pub fn ppu_registers(&self) -> &PpuRegisters {
        &self.memory.ppu_registers
    }

    #[inline]
    pub fn ppu_registers_mut(&mut self) -> &mut PpuRegisters {
        &mut self.memory.ppu_registers
    }

    #[inline]
    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        PatternTable::new(self.memory.mapper.raw_pattern_table(side))
    }

    #[inline]
    pub fn name_table(&self, number: NameTableNumber) -> NameTable {
        NameTable::new(self.memory.mapper.raw_name_table(&self.memory.ppu_internal_ram, number))
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        PaletteTable::new(self.memory.ppu_internal_ram.palette_ram.to_slice(), &self.memory.system_palette)
    }
}
