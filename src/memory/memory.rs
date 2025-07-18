use crate::apu::apu_registers::ApuRegisters;
use crate::cpu::dmc_dma::DmcDma;
use crate::cpu::oam_dma::OamDma;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_internal_ram::CpuInternalRam;
use crate::memory::cpu::ports::Ports;
use crate::memory::cpu::stack::Stack;
use crate::mapper;
use crate::mapper::{Mapper, MapperParams};
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ciram::Ciram;
use crate::memory::read_result::ReadResult;
use crate::ppu::clock::Clock;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::sprite::oam::Oam;

use super::ppu::chr_memory::PpuPeek;

pub const NMI_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFA);
pub const NMI_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFB);
pub const RESET_VECTOR_LOW: CpuAddress   = CpuAddress::new(0xFFFC);
pub const RESET_VECTOR_HIGH: CpuAddress  = CpuAddress::new(0xFFFD);
pub const IRQ_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFE);
pub const IRQ_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFF);

pub struct Memory {
    pub mapper: Box<dyn Mapper>,
    pub mapper_params: MapperParams,
    pub cpu_internal_ram: CpuInternalRam,
    pub ciram: Ciram,
    pub palette_ram: PaletteRam,
    pub oam: Oam,
    pub ports: Ports,
    nmi_line_level: SignalLevel,
    pub ppu_regs: PpuRegisters,
    pub apu_regs: ApuRegisters,
    system_palette: SystemPalette,
    pub dmc_dma: DmcDma,
    pub oam_dma: OamDma,
    cpu_data_bus: u8,
    cpu_cycle: i64,
}

impl Memory {
    pub fn new(
        mapper: Box<dyn Mapper>, 
        mapper_params: MapperParams,
        ports: Ports,
        ppu_clock: Clock,
        system_palette: SystemPalette,
    ) -> Memory {
        Memory {
            mapper,
            mapper_params,
            cpu_internal_ram: CpuInternalRam::new(),
            ciram: Ciram::new(),
            palette_ram: PaletteRam::new(),
            oam: Oam::new(),
            ports,
            nmi_line_level: SignalLevel::High,
            ppu_regs: PpuRegisters::new(ppu_clock),
            apu_regs: ApuRegisters::new(),
            system_palette,
            dmc_dma: DmcDma::IDLE,
            oam_dma: OamDma::IDLE,
            cpu_data_bus: 0,
            cpu_cycle: 0,
        }
    }

    pub fn as_cpu_memory<'a>(&'a mut self) -> CpuMemory<'a> {
        CpuMemory { memory: self }
    }

    pub fn as_ppu_memory<'a>(&'a mut self) -> PpuMemory<'a> {
        PpuMemory { memory: self }
    }

    pub fn mapper(&self) -> &dyn Mapper {
        &*self.mapper
    }

    pub fn mapper_mut(&mut self) -> &mut dyn Mapper {
        &mut *self.mapper
    }

    pub fn mapper_params(&self) -> &MapperParams {
        &self.mapper_params
    }

    pub fn mapper_params_mut(&mut self) -> &mut MapperParams {
        &mut self.mapper_params
    }

    pub fn stack_pointer(&self) -> u8 {
        self.cpu_internal_ram.stack_pointer
    }

    #[inline]
    pub fn ppu_regs(&self) -> &PpuRegisters {
        &self.ppu_regs
    }

    #[inline]
    pub fn ppu_regs_mut(&mut self) -> &mut PpuRegisters {
        &mut self.ppu_regs
    }

    pub fn apu_regs(&self) -> &ApuRegisters {
        &self.apu_regs
    }

    pub fn apu_regs_mut(&mut self) -> &mut ApuRegisters {
        &mut self.apu_regs
    }

    pub fn ciram(&self) -> &Ciram {
        &self.ciram
    }

    pub fn ciram_mut(&mut self) -> &mut Ciram {
        &mut self.ciram
    }

    pub fn dmc_dma(&self) -> &DmcDma {
        &self.dmc_dma
    }

    pub fn dmc_dma_mut(&mut self) -> &mut DmcDma {
        &mut self.dmc_dma
    }

    pub fn apu_regs_and_dmc_dma_mut(&mut self) -> (&mut ApuRegisters, &mut DmcDma) {
        (&mut self.apu_regs, &mut self.dmc_dma)
    }

    pub fn oam_dma(&self) -> &OamDma {
        &self.oam_dma
    }

    pub fn ports(&self) -> &Ports {
        &self.ports
    }

    pub fn ports_mut(&mut self) -> &mut Ports {
        &mut self.ports
    }

    pub fn cpu_cycle(&self) -> i64 {
        self.cpu_cycle
    }

    pub fn cpu_internal_ram(&self) -> &CpuInternalRam {
        &self.cpu_internal_ram
    }

    pub fn palette_ram(&self) -> &PaletteRam {
        &self.palette_ram
    }

    pub fn palette_ram_mut(&mut self) -> &mut PaletteRam {
        &mut self.palette_ram
    }

    pub fn oam(&self) -> &Oam {
        &self.oam
    }

    pub fn oam_mut(&mut self) -> &mut Oam {
        &mut self.oam
    }

    #[inline]
    pub fn cpu_peek(&self, address: CpuAddress) -> u8 {
        self.mapper.cpu_peek(self, address).resolve(self.cpu_data_bus)
    }

    pub fn maybe_cpu_peek(&self, address: CpuAddress) -> ReadResult {
        self.mapper.cpu_peek(self, address)
    }

    #[inline]
    pub fn cpu_write(&mut self, address: CpuAddress) {
        let bus_value = self.cpu_data_bus;
        mapper::cpu_write(self, address, bus_value);
    }
}

pub struct CpuMemory<'a> {
    memory: &'a mut Memory,
}

impl CpuMemory<'_> {
    #[inline]
    pub fn read(&mut self, address: CpuAddress) {
        self.memory.cpu_data_bus = self.memory.mapper.cpu_read(
            &mut self.memory.mapper_params,
            &self.memory.cpu_internal_ram,
            &self.memory.ciram,
            &self.memory.palette_ram,
            &self.memory.oam,
            &mut self.memory.ports,
            &mut self.memory.ppu_regs,
            &mut self.memory.apu_regs,
            address,
        ).resolve(self.memory.cpu_data_bus);
        self.memory.mapper.on_cpu_read(&mut self.memory.mapper_params, address, self.memory.cpu_data_bus);
    }

    pub fn ports(&self) -> &Ports {
        &self.memory.ports
    }

    pub fn nmi_line_level(&mut self) -> SignalLevel {
        self.memory.nmi_line_level
    }

    pub fn irq_line_level(&mut self) -> SignalLevel {
        let irq_line_low =
            self.memory.apu_regs().frame_irq_pending()
            || self.memory.apu_regs().dmc_irq_pending()
            || self.memory.mapper_params().irq_pending();

        if irq_line_low {
            SignalLevel::Low
        } else {
            SignalLevel::High
        }
    }

    pub fn dmc_dma_address(&self) -> CpuAddress {
        self.memory.apu_regs.dmc.dma_sample_address()
    }

    pub fn dmc_dma(&mut self) -> &DmcDma {
        &self.memory.dmc_dma
    }

    pub fn dmc_dma_mut(&mut self) -> &mut DmcDma {
        &mut self.memory.dmc_dma
    }

    pub fn oam_dma(&self) -> &OamDma {
        &self.memory.oam_dma
    }

    pub fn oam_dma_mut(&mut self) -> &mut OamDma {
        &mut self.memory.oam_dma
    }

    pub fn set_dmc_sample_buffer(&mut self, value: u8) {
        self.memory.apu_regs.dmc.set_sample_buffer(value);
    }

    #[inline]
    pub fn stack<'a>(&'a mut self) -> Stack<'a> {
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
        self.address_from_vector(NMI_VECTOR_LOW)
    }

    pub fn reset_vector(&mut self) -> CpuAddress {
        self.address_from_vector(RESET_VECTOR_LOW)
    }

    pub fn irq_vector(&mut self) -> CpuAddress {
        self.address_from_vector(IRQ_VECTOR_LOW)
    }

    pub fn data_bus(&self) -> u8 {
        self.memory.cpu_data_bus
    }

    pub fn data_bus_mut(&mut self) -> &mut u8 {
        &mut self.memory.cpu_data_bus
    }

    pub fn cpu_cycle(&self) -> i64 {
        self.memory.cpu_cycle
    }

    pub fn increment_cpu_cycle(&mut self) {
        self.memory.cpu_cycle += 1;
    }

    pub fn set_cpu_cycle(&mut self, cycle: i64) {
        self.memory.cpu_cycle = cycle;
    }

    pub fn process_end_of_cpu_cycle(&mut self) {
        self.memory.mapper.on_end_of_cpu_cycle(&mut self.memory.mapper_params, self.memory.cpu_cycle);
    }

    pub fn memory(&self) -> &Memory {
        self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        self.memory
    }

    fn address_from_vector(&mut self, mut vector: CpuAddress) -> CpuAddress {
        CpuAddress::from_low_high(
            self.memory.cpu_peek(vector),
            self.memory.cpu_peek(vector.inc()),
        )
    }
}

pub struct PpuMemory<'a> {
    memory: &'a mut Memory,
}

impl PpuMemory<'_> {
    #[inline]
    pub fn read(&mut self, address: PpuAddress) -> PpuPeek {
        self.memory.mapper.ppu_read(
            &mut self.memory.mapper_params, &self.memory.ciram, &self.memory.palette_ram, address, true)
    }

    #[inline]
    pub fn write(&mut self, address: PpuAddress, value: u8) {
        self.memory.mapper.ppu_write(
            &mut self.memory.mapper_params, &mut self.memory.ciram, &mut self.memory.palette_ram, address, value);
    }

    pub fn set_nmi_line_level(&mut self, level: SignalLevel) {
        self.memory.nmi_line_level = level;
    }

    pub fn oam(&self) -> &Oam {
        &self.memory.oam
    }

    pub fn oam_mut(&mut self) -> &mut Oam {
        &mut self.memory.oam
    }

    pub fn trigger_ppu_address_change(&mut self, address: PpuAddress) {
        self.memory.mapper.on_ppu_address_change(&mut self.memory.mapper_params, address);
    }

    pub fn process_end_of_ppu_cycle(&mut self) {
        self.memory.mapper.on_end_of_ppu_cycle();
    }

    pub fn memory(&self) -> &Memory {
        self.memory
    }

    #[inline]
    pub fn regs(&self) -> &PpuRegisters {
        &self.memory.ppu_regs
    }

    #[inline]
    pub fn regs_mut(&mut self) -> &mut PpuRegisters {
        &mut self.memory.ppu_regs
    }

    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.memory.mapper_params.name_table_mirroring()
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        PaletteTable::new(
            self.memory.palette_ram.to_slice(),
            &self.memory.system_palette,
            self.memory.ppu_regs.mask(),
        )
    }

    pub fn rom_bank_count(&self) -> u16 {
        self.memory.mapper_params.chr_memory.rom_bank_count()
    }

    pub fn ram_bank_count(&self) -> u16 {
        self.memory.mapper_params.chr_memory.ram_bank_count()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SignalLevel {
    High,
    Low,
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
            let low_value = memory.read(address).value();
            let high_value = memory.read(PpuAddress::from_u16(address.to_u16() + 0x1000)).value();
            assert_eq!(low_value, value);
            assert_eq!(low_value, high_value);

            value = value.wrapping_add(1);
            address.advance(AddressIncrement::Right);
        }

        let mut address = PpuAddress::from_u16(0x3000);
        let mut value = 111;
        while address < PpuAddress::from_u16(0x3F00) {
            memory.write(address, value);
            let high_value = memory.read(address).value();
            let low_value = memory.read(PpuAddress::from_u16(address.to_u16() - 0x1000)).value();
            assert_eq!(low_value, value);
            assert_eq!(low_value, high_value);

            value = value.wrapping_add(1);
            address.advance(AddressIncrement::Right);
        }
    }
}

#[cfg(test)]
pub mod test_data {
    use crate::cartridge::cartridge;
    use crate::cartridge::cartridge::Cartridge;
    use crate::memory::cpu::ports;
    use crate::mappers::mapper000::Mapper000;
    use crate::ppu::palette::system_palette;

    use super::*;

    pub fn memory() -> Memory {
        memory_with_cartridge(&cartridge::test_data::cartridge())
    }

    pub fn memory_with_cartridge(cartridge: &Cartridge) -> Memory {
        let mapper = Mapper000;
        let mapper_params = mapper.layout().make_mapper_params(cartridge);
        Memory::new(
            Box::new(mapper),
            mapper_params,
            ports::test_data::ports(),
            Clock::mesen_compatible(),
            system_palette::test_data::system_palette(),
        )
    }
}