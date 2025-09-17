use crate::memory::cpu::cpu_address::CpuAddress;

pub struct CpuPinout {
    // AD1 (Audio Pinout: Both pulse waves)
    // AD2 (Audio Pinout: Triangle, Noise, DPCM)
    // RST (CPU Reset)
    // Axx
    pub address_bus: CpuAddress,
    // GND (Ground)
    // Dx
    pub data_bus: u8,
    // CLK (Clock)
    // TST
    // M2 (CPU phase)
    // IRQ
    // NMI
    // R/W (Read/Write signal)
    // OE2 (Controller 2 enable)
    // OE1 (Controller 1 enable)
    // OUT0..OUT2 (Controller outputs)
}

impl CpuPinout {
    pub fn new() -> Self {
        Self {
            address_bus: CpuAddress::ZERO,
            data_bus: 0,
        }
    }
}