use crate::apu::apu_registers::ApuRegisters;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_internal_ram::{
    CpuInternalRam, IRQ_VECTOR, NMI_VECTOR, RESET_VECTOR,
};
use crate::memory::cpu::ports::Ports;
use crate::memory::cpu::stack::Stack;
use crate::memory::mapper::Mapper;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ppu_internal_ram::PpuInternalRam;
use crate::ppu::name_table::name_table::NameTable;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::ppu::register::ppu_registers::PpuRegisters;

pub struct Memory {
    mapper: Box<dyn Mapper>,
    cpu_internal_ram: CpuInternalRam,
    ppu_internal_ram: PpuInternalRam,
    ports: Ports,
    ppu_registers: PpuRegisters,
    apu_registers: ApuRegisters,
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
            apu_registers: ApuRegisters::default(),
            system_palette,
        }
    }

    pub fn as_cpu_memory(&mut self) -> CpuMemory {
        CpuMemory { memory: self }
    }

    pub fn as_ppu_memory(&mut self) -> PpuMemory {
        PpuMemory { memory: self }
    }

    pub fn mapper(&self) -> &dyn Mapper {
        &*self.mapper
    }

    pub fn mapper_mut(&mut self) -> &mut dyn Mapper {
        &mut *self.mapper
    }

    pub fn stack_pointer(&self) -> u8 {
        self.cpu_internal_ram.stack_pointer
    }

    #[inline]
    pub fn ppu_regs(&self) -> &PpuRegisters {
        &self.ppu_registers
    }

    pub fn apu_regs(&mut self) -> &mut ApuRegisters {
        &mut self.apu_registers
    }
}

pub struct CpuMemory<'a> {
    memory: &'a mut Memory,
}

impl<'a> CpuMemory<'a> {
    #[inline]
    pub fn peek(&self, address: CpuAddress) -> Option<u8> {
        self.memory.mapper.cpu_peek(
            &self.memory.cpu_internal_ram,
            &self.memory.ppu_internal_ram,
            &self.memory.ports,
            &self.memory.ppu_registers,
            &self.memory.apu_registers,
            address,
        )
    }

    #[inline]
    pub fn read(&mut self, address: CpuAddress) -> Option<u8> {
        self.memory.mapper.cpu_read(
            &self.memory.cpu_internal_ram,
            &self.memory.ppu_internal_ram,
            &mut self.memory.ports,
            &mut self.memory.ppu_registers,
            &mut self.memory.apu_registers,
            address,
        )
    }

    #[inline]
    pub fn write(&mut self, address: CpuAddress, value: u8) {
        self.memory.mapper.cpu_write(
            &mut self.memory.cpu_internal_ram,
            &mut self.memory.ppu_internal_ram,
            &mut self.memory.ports,
            &mut self.memory.ppu_registers,
            &mut self.memory.apu_registers,
            address,
            value,
        );
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

    pub fn stack_pointer_address(&self) -> CpuAddress {
        CpuAddress::from_low_high(self.stack_pointer(), 0x01)
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
            self.read(vector).expect("Read open bus."),
            self.read(vector.inc()).expect("Read open bus."),
        )
    }
}

pub struct PpuMemory<'a> {
    memory: &'a mut Memory,
}

impl<'a> PpuMemory<'a> {
    #[inline]
    pub fn read(&mut self, address: PpuAddress, rendering: bool) -> u8 {
        self.memory
            .mapper
            .ppu_read(&self.memory.ppu_internal_ram, address, rendering)
    }

    #[inline]
    pub fn write(&mut self, address: PpuAddress, value: u8) {
        self.memory
            .mapper
            .ppu_write(&mut self.memory.ppu_internal_ram, address, value);
    }

    pub fn process_end_of_ppu_cycle(&mut self) {
        self.memory
            .mapper
            .on_end_of_ppu_cycle();
    }

    pub fn process_current_ppu_address(&mut self, address: PpuAddress) {
        self.memory
            .mapper
            .process_current_ppu_address(address);
    }

    #[inline]
    pub fn regs(&self) -> &PpuRegisters {
        &self.memory.ppu_registers
    }

    #[inline]
    pub fn regs_mut(&mut self) -> &mut PpuRegisters {
        &mut self.memory.ppu_registers
    }

    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.memory.mapper.name_table_mirroring()
    }

    #[inline]
    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        self.memory.mapper.chr_memory().pattern_table(side)
    }

    #[inline]
    pub fn background_pattern_table(&self) -> PatternTable {
        self.pattern_table(self.regs().background_table_side())
    }

    #[inline]
    pub fn sprite_pattern_table(&self) -> PatternTable {
        self.pattern_table(self.regs().sprite_table_side())
    }

    #[inline]
    pub fn name_table(&self, quadrant: NameTableQuadrant) -> NameTable {
        NameTable::new(
            self.memory
                .mapper
                .raw_name_table(&self.memory.ppu_internal_ram, quadrant),
        )
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        PaletteTable::new(
            self.memory.ppu_internal_ram.palette_ram.to_slice(),
            &self.memory.system_palette,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::memory::test_data;
    use crate::memory::memory::PpuAddress;
    use crate::ppu::register::registers::ctrl::AddressIncrement;

    #[test]
    fn mirrored_3000s() {
        let mut memory = test_data::memory();
        let mut memory = memory.as_ppu_memory();
        let mut address = PpuAddress::from_u16(0x2000);
        let mut value = 1;
        while address < PpuAddress::from_u16(0x2F00) {
            memory.write(address, value);
            let low_value = memory.read(address, false);
            let high_value = memory.read(PpuAddress::from_u16(address.to_u16() + 0x1000), false);
            assert_eq!(low_value, value);
            assert_eq!(low_value, high_value);

            value = value.wrapping_add(1);
            address.advance(AddressIncrement::Right);
        }

        let mut address = PpuAddress::from_u16(0x3000);
        let mut value = 111;
        while address < PpuAddress::from_u16(0x3F00) {
            memory.write(address, value);
            let high_value = memory.read(address, false);
            let low_value = memory.read(PpuAddress::from_u16(address.to_u16() - 0x1000), false);
            assert_eq!(low_value, value);
            assert_eq!(low_value, high_value);

            value = value.wrapping_add(1);
            address.advance(AddressIncrement::Right);
        }
    }
}

#[cfg(test)]
pub mod test_data {
    use crate::cartridge;
    use crate::cartridge::Cartridge;
    use crate::memory::cpu::ports;
    use crate::memory::mappers::mapper000::Mapper000;
    use crate::ppu::palette::system_palette;

    use super::*;

    pub fn memory() -> Memory {
        Memory::new(
            Box::new(Mapper000::new(&cartridge::test_data::cartridge()).unwrap()),
            ports::test_data::ports(),
            system_palette::test_data::system_palette(),
        )
    }

    pub fn memory_with_cartridge(cartridge: &Cartridge) -> Memory {
        let mapper = Box::new(Mapper000::new(cartridge).unwrap());
        Memory::new(
            mapper,
            ports::test_data::ports(),
            system_palette::test_data::system_palette(),
        )
    }
}
