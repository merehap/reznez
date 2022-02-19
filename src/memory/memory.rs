use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_internal_ram::{CpuInternalRam, NMI_VECTOR, RESET_VECTOR, IRQ_VECTOR};
use crate::memory::cpu::ports::Ports;
use crate::memory::cpu::stack::Stack;
use crate::memory::mapper::Mapper;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ppu_internal_ram::PpuInternalRam;
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
    pub fn read(&mut self, address: CpuAddress) -> u8 {
        self.memory.mapper.cpu_read(
            &self.memory.cpu_internal_ram,
            &mut self.memory.ports,
            &mut self.memory.ppu_registers,
            address,
        )
    }

    #[inline]
    pub fn write(&mut self, address: CpuAddress, value: u8) {
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
            self.read(vector),
            self.read(vector.inc()),
        )
    }
}

pub struct PpuMemory<'a> {
    memory: &'a mut Memory,
}

impl <'a> PpuMemory<'a> {
    #[inline]
    pub fn read(&self, address: PpuAddress) -> u8 {
        self.memory.mapper.ppu_read(&self.memory.ppu_internal_ram, address)
    }

    #[inline]
    pub fn write(&mut self, address: PpuAddress, value: u8) {
        self.memory.mapper.ppu_write(&mut self.memory.ppu_internal_ram, address, value)
    }

    #[inline]
    pub fn regs(&self) -> &PpuRegisters {
        &self.memory.ppu_registers
    }

    #[inline]
    pub fn regs_mut(&mut self) -> &mut PpuRegisters {
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

#[cfg(test)]
pub mod test_data {
    use crate::cartridge;
    use crate::cartridge::Cartridge;
    use crate::memory::cpu::ports;
    use crate::memory::mappers::mapper0::Mapper0;
    use crate::ppu::palette::system_palette;

    use super::*;

    pub fn memory() -> Memory {
        Memory::new(
            Box::new(Mapper0::new(cartridge::test_data::cartridge()).unwrap()),
            ports::test_data::ports(),
            system_palette::test_data::system_palette(),
        )
    }

    pub fn memory_with_cartridge(cartridge: Cartridge) -> Memory {
        let mapper = Box::new(Mapper0::new(cartridge).unwrap());
        Memory::new(
            mapper,
            ports::test_data::ports(),
            system_palette::test_data::system_palette(),
        )
    }
}
