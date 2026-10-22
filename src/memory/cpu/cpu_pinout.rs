use crate::bus::Bus;
use crate::cpu::nmi_latch::NmiLatch;
use crate::cpu::rst_latch::RstLatch;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::signal_level::ActiveLow;
use crate::ppu::vblank_interrupt_handler::VBlankInterruptHandler;

pub struct CpuPinout {
    // AD1 (Audio Pinout: Both pulse waves)
    // AD2 (Audio Pinout: Triangle, Noise, DPCM)
    // RST (CPU Reset). A reset is triggered when this signal rises.
    reset: ActiveLow,
    // Axx
    pub address_bus: CpuAddress,
    // GND (Ground)
    // Dx
    pub data_bus: u8,
    // CLK (Clock)
    // TST
    // M2 (CPU phase)
    // IRQ - only available as a method, not a field.
    // NMI. Triggers when the signal goes low.
    nmi: ActiveLow,
    // R/W (Read/Write signal)
    read_write: ReadWrite,
    // OE2 (Controller 2 enable)
    // OE1 (Controller 1 enable)
    // OUT0..OUT2 (Controller outputs)

    mapper_irq_asserted: bool,
    frame_irq_asserted: bool,
    dmc_irq_asserted: bool,
}

impl CpuPinout {
    pub fn new() -> Self {
        Self {
            reset: ActiveLow::Active,
            address_bus: CpuAddress::ZERO,
            data_bus: 0,
            nmi: ActiveLow::Inactive,
            read_write: ReadWrite::Read,

            mapper_irq_asserted: false,
            frame_irq_asserted: false,
            dmc_irq_asserted: false,
        }
    }

    pub fn reset(&self) -> ActiveLow {
        self.reset
    }

    pub fn set_reset(bus: &mut Bus, value: ActiveLow) {
        bus.cpu_pinout.reset = value;
        RstLatch::update(bus);
    }

    pub fn nmi(&self) -> ActiveLow {
        self.nmi
    }

    pub fn set_nmi(bus: &mut Bus, value: ActiveLow) {
        bus.cpu_pinout.nmi = value;
        NmiLatch::update(bus);
    }

    pub fn read_write(&self) -> ReadWrite {
        self.read_write
    }

    pub fn set_read_write(bus: &mut Bus, value: ReadWrite) {
        bus.cpu_pinout.read_write = value;
        bus.ppu_pinout.read_write = value;
        VBlankInterruptHandler::update(bus);
    }

    pub fn irq(&self) -> ActiveLow {
        if self.mapper_irq_asserted || self.frame_irq_asserted || self.dmc_irq_asserted {
            ActiveLow::Active
        } else {
            ActiveLow::Inactive
        }
    }

    pub fn mapper_irq_asserted(&self) -> bool {
        self.mapper_irq_asserted
    }

    pub fn acknowledge_mapper_irq(bus: &mut Bus) {
        bus.cpu_pinout.mapper_irq_asserted = false;
    }

    pub fn assert_mapper_irq(bus: &mut Bus) {
        bus.cpu_pinout.mapper_irq_asserted = true;
    }

    pub fn frame_irq_asserted(&self) -> bool {
        self.frame_irq_asserted
    }

    pub fn acknowledge_frame_irq(bus: &mut Bus) {
        bus.cpu_pinout.frame_irq_asserted = false;
    }

    pub fn assert_frame_irq(bus: &mut Bus) {
        bus.cpu_pinout.frame_irq_asserted = true;
    }

    pub fn dmc_irq_asserted(&self) -> bool {
        self.dmc_irq_asserted
    }

    pub fn acknowledge_dmc_irq(bus: &mut Bus) {
        bus.cpu_pinout.dmc_irq_asserted = false;
    }

    pub fn assert_dmc_irq(bus: &mut Bus) {
        bus.cpu_pinout.dmc_irq_asserted = true;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ReadWrite {
    Write,
    Read,
}