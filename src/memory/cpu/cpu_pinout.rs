use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::signal_level::SignalLevel;
use crate::util::edge_detector::EdgeDetector;

pub struct CpuPinout {
    // AD1 (Audio Pinout: Both pulse waves)
    // AD2 (Audio Pinout: Triangle, Noise, DPCM)
    // RST (CPU Reset). Triggers when the signal goes high.
    pub reset: EdgeDetector<SignalLevel>,
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
    pub nmi_signal_detector: EdgeDetector<SignalLevel>,
    // R/W (Read/Write signal)
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
            reset: EdgeDetector::target_value(SignalLevel::High),
            address_bus: CpuAddress::ZERO,
            data_bus: 0,
            nmi_signal_detector: EdgeDetector::target_value(SignalLevel::Low),

            mapper_irq_asserted: false,
            frame_irq_asserted: false,
            dmc_irq_asserted: false,
        }
    }

    pub fn irq_asserted(&self) -> bool {
        self.mapper_irq_asserted || self.frame_irq_asserted || self.dmc_irq_asserted
    }

    pub fn mapper_irq_asserted(&self) -> bool {
        self.mapper_irq_asserted
    }

    pub fn acknowledge_mapper_irq(&mut self) {
        self.mapper_irq_asserted = false;
    }

    pub fn assert_mapper_irq(&mut self) {
        self.mapper_irq_asserted = true;
    }

    pub fn frame_irq_asserted(&self) -> bool {
        self.frame_irq_asserted
    }

    pub fn acknowledge_frame_irq(&mut self) {
        self.frame_irq_asserted = false;
    }

    pub fn assert_frame_irq(&mut self) {
        self.frame_irq_asserted = true;
    }

    pub fn dmc_irq_asserted(&self) -> bool {
        self.dmc_irq_asserted
    }

    pub fn acknowledge_dmc_irq(&mut self) {
        self.dmc_irq_asserted = false;
    }

    pub fn assert_dmc_irq(&mut self) {
        self.dmc_irq_asserted = true;
    }
}