use crate::memory::cpu::cpu_address::CpuAddress;
use crate::util::signal_detector::{EdgeDetector, SignalLevel};

pub struct CpuPinout {
    // AD1 (Audio Pinout: Both pulse waves)
    // AD2 (Audio Pinout: Triangle, Noise, DPCM)
    // RST (CPU Reset)
    pub reset: EdgeDetector<SignalLevel, {SignalLevel::High}>,
    // Axx
    pub address_bus: CpuAddress,
    // GND (Ground)
    // Dx
    pub data_bus: u8,
    // CLK (Clock)
    // TST
    // M2 (CPU phase)
    // IRQ - only available as a method, not a field.
    // NMI
    pub nmi_signal_detector: EdgeDetector<SignalLevel, {SignalLevel::Low}>,
    // R/W (Read/Write signal)
    // OE2 (Controller 2 enable)
    // OE1 (Controller 1 enable)
    // OUT0..OUT2 (Controller outputs)

    mapper_irq_pending: bool,
    frame_irq_pending: bool,
    dmc_irq_pending: bool,
}

impl CpuPinout {
    pub fn new() -> Self {
        Self {
            reset: EdgeDetector::new(),
            address_bus: CpuAddress::ZERO,
            data_bus: 0,
            nmi_signal_detector: EdgeDetector::new(),

            mapper_irq_pending: false,
            frame_irq_pending: false,
            dmc_irq_pending: false,
        }
    }

    pub fn irq_pending(&self) -> bool {
        self.mapper_irq_pending || self.frame_irq_pending || self.dmc_irq_pending
    }

    pub fn mapper_irq_pending(&self) -> bool {
        self.mapper_irq_pending
    }

    pub fn clear_mapper_irq_pending(&mut self) {
        self.mapper_irq_pending = false;
    }

    pub fn set_mapper_irq_pending(&mut self) {
        self.mapper_irq_pending = true;
    }

    pub fn frame_irq_pending(&self) -> bool {
        self.frame_irq_pending
    }

    pub fn clear_frame_irq_pending(&mut self) {
        self.frame_irq_pending = false;
    }

    pub fn set_frame_irq_pending(&mut self) {
        self.frame_irq_pending = true;
    }

    pub fn dmc_irq_pending(&self) -> bool {
        self.dmc_irq_pending
    }

    pub fn clear_dmc_irq_pending(&mut self) {
        self.dmc_irq_pending = false;
    }

    pub fn set_dmc_irq_pending(&mut self) {
        self.dmc_irq_pending = true;
    }
}