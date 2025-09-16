use crate::apu::apu_registers::ApuRegisters;
use crate::cpu::dmc_dma::DmcDma;
use crate::cpu::oam_dma::OamDma;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_internal_ram::CpuInternalRam;
use crate::memory::cpu::ports::Ports;
use crate::memory::cpu::stack::Stack;
use crate::mapper::MapperParams;
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ciram::Ciram;
use crate::ppu::clock::Clock;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::sprite::oam::Oam;
use crate::util::signal_detector::{EdgeDetector, SignalLevel};

pub const NMI_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFA);
pub const NMI_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFB);
pub const RESET_VECTOR_LOW: CpuAddress   = CpuAddress::new(0xFFFC);
pub const RESET_VECTOR_HIGH: CpuAddress  = CpuAddress::new(0xFFFD);
pub const IRQ_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFE);
pub const IRQ_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFF);

pub struct Memory {
    pub mapper_params: MapperParams,
    pub cpu_internal_ram: CpuInternalRam,
    pub ciram: Ciram,
    pub palette_ram: PaletteRam,
    pub oam: Oam,
    pub ports: Ports,
    pub nmi_signal_detector: EdgeDetector,
    pub ppu_regs: PpuRegisters,
    pub apu_regs: ApuRegisters,
    system_palette: SystemPalette,
    pub dmc_dma: DmcDma,
    pub oam_dma: OamDma,
    pub cpu_address_bus: CpuAddress,
    pub oam_dma_address_bus: CpuAddress,
    pub dmc_dma_address_bus: CpuAddress,
    pub cpu_data_bus: u8,
    cpu_cycle: i64,
}

impl Memory {
    pub fn new(
        mapper_params: MapperParams,
        ports: Ports,
        ppu_clock: Clock,
        system_palette: SystemPalette,
    ) -> Memory {
        Memory {
            mapper_params,
            cpu_internal_ram: CpuInternalRam::new(),
            ciram: Ciram::new(),
            palette_ram: PaletteRam::new(),
            oam: Oam::new(),
            ports,
            nmi_signal_detector: EdgeDetector::new(),
            ppu_regs: PpuRegisters::new(ppu_clock),
            apu_regs: ApuRegisters::new(),
            system_palette,
            dmc_dma: DmcDma::IDLE,
            oam_dma: OamDma::IDLE,
            cpu_address_bus: CpuAddress::ZERO,
            oam_dma_address_bus: CpuAddress::ZERO,
            dmc_dma_address_bus: CpuAddress::ZERO,
            cpu_data_bus: 0,
            cpu_cycle: 0,
        }
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

    pub fn apu_regs_and_dmc_dma_mut(&mut self) -> (&mut ApuRegisters, &mut DmcDma) {
        (&mut self.apu_regs, &mut self.dmc_dma)
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

    pub fn irq_line_level(&mut self) -> SignalLevel {
        let irq_line_low =
            self.apu_regs().frame_irq_pending()
            || self.apu_regs().dmc_irq_pending()
            || self.mapper_params().irq_pending();

        if irq_line_low {
            SignalLevel::Low
        } else {
            SignalLevel::High
        }
    }

    pub fn address_bus(&self, address_bus_type: AddressBusType) -> CpuAddress {
        match address_bus_type {
            AddressBusType::Cpu => self.cpu_address_bus,
            AddressBusType::OamDma => self.oam_dma_address_bus,
            AddressBusType::DmcDma => self.dmc_dma_address_bus,
        }
    }

    pub fn set_address_bus(&mut self, address_bus_type: AddressBusType, address: CpuAddress) {
        match address_bus_type {
            AddressBusType::Cpu => self.cpu_address_bus = address,
            AddressBusType::OamDma => self.oam_dma_address_bus = address,
            AddressBusType::DmcDma => self.dmc_dma_address_bus = address,
        }
    }

    pub fn dmc_dma_address(&self) -> CpuAddress {
        self.apu_regs.dmc.dma_sample_address()
    }

    pub fn set_dmc_sample_buffer(&mut self, value: u8) {
        self.apu_regs.dmc.set_sample_buffer(value);
    }

    #[inline]
    pub fn cpu_stack<'a>(&'a mut self) -> Stack<'a> {
        self.cpu_internal_ram.stack()
    }

    #[inline]
    pub fn cpu_stack_pointer_mut(&mut self) -> &mut u8 {
        &mut self.cpu_internal_ram.stack_pointer
    }

    pub fn cpu_stack_pointer_address(&self) -> CpuAddress {
        CpuAddress::from_low_high(self.stack_pointer(), 0x01)
    }

    pub fn increment_cpu_cycle(&mut self) {
        self.cpu_cycle += 1;
    }

    pub fn set_cpu_cycle(&mut self, cycle: i64) {
        self.cpu_cycle = cycle;
    }

    pub fn chr_rom_bank_count(&self) -> u16 {
        self.mapper_params.chr_memory.rom_bank_count()
    }

    pub fn chr_ram_bank_count(&self) -> u16 {
        self.mapper_params.chr_memory.ram_bank_count()
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        PaletteTable::new(
            self.palette_ram.to_slice(),
            &self.system_palette,
            self.ppu_regs.mask(),
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressBusType {
    Cpu,
    OamDma,
    DmcDma,
}