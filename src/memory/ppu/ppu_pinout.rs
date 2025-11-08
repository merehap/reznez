use crate::memory::ppu::ppu_address::PpuAddress;
use crate::util::edge_detector::EdgeDetector;

pub struct PpuPinout {
    // cpu_data_bus: u8, // D0-D8
    // cpu_address_bus: u3, // A2-A0
    // /CS
    // EXT: u4,
    // CLK
    // /INT // Connected to the CPU's NMI pin
    // GND
    // VOUT
    // /RST
    // /WR
    // /RD
    address_and_data_bus_detector: EdgeDetector<PpuAddress>,
    // /ALE
    // +5V
}

impl PpuPinout {
    pub fn new() -> Self {
        Self {
            address_and_data_bus_detector: EdgeDetector::any_edge(),
        }
    }

    pub fn address_bus(&self) -> PpuAddress {
        self.address_and_data_bus_detector.current_value()
    }

    #[must_use]
    pub fn set_address_bus(&mut self, addr: PpuAddress) -> bool {
        self.address_and_data_bus_detector.set_value_then_detect(addr)
    }

    pub fn data_bus(&self) -> u8 {
        self.address_and_data_bus_detector.current_value().to_u16() as u8
    }

    #[must_use]
    pub fn set_data_bus(&mut self, data: u8) -> bool {
        let mut value = self.address_and_data_bus_detector.current_value();
        value.set_low_byte(data);
        self.address_and_data_bus_detector.set_value_then_detect(value)
    }
}