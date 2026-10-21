use crate::bus::Bus;
use crate::memory::cpu::cpu_pinout::ReadWrite;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::signal_level::{ActiveHigh, ActiveLow, SignalLevel, nor};
use crate::util::edge_detector::EdgeDetector;

pub struct PpuPinout {
    // R/W, from the CPU pinout
    pub read_write: ReadWrite,
    // cpu_data_bus: u8, // D0-D8
    // cpu_address_bus: u3, // A2-A0
    pub selected_register: PpuRegisterType,
    // /CS or /DBE. Active low.
    data_bus_enabled: ActiveLow,
    // EXT: u4,
    // CLK
    pub pclk: SignalLevel,
    // /INT // Connected to the CPU's NMI pin
    // GND
    // VOUT
    // /RST
    // /WR
    pub ppu_data_write: bool,
    // /RD
    pub fetch_cadence_read: bool,
    pub ppu_data_read: bool,
    // A13-A8, AD7-AD0
    address_and_data_bus_detector: EdgeDetector<PpuAddress>,
    // /ALE
    pub fetch_cadence_ale: bool,
    pub ppu_data_read_ale: bool,
    pub ppu_data_write_ale: bool,
    // +5V

    address_latch: u8,
}

impl PpuPinout {
    pub fn new() -> Self {
        Self {
            read_write: ReadWrite::Read,
            selected_register: PpuRegisterType::Ctrl,
            data_bus_enabled: ActiveLow::Inactive,
            pclk: SignalLevel::High, // FIXME: This needs to be determined by the schedule.
            ppu_data_write: false,
            fetch_cadence_read: false,
            ppu_data_read: false,
            address_and_data_bus_detector: EdgeDetector::any_edge(),
            address_latch: 0,
            fetch_cadence_ale: false,
            ppu_data_read_ale: false,
            ppu_data_write_ale: false,
        }
    }

    pub fn data_bus_enabled(&self) -> ActiveLow {
        self.data_bus_enabled
    }

    pub fn set_data_bus_enabled(bus: &mut Bus, value: ActiveLow) {
        bus.ppu_pinout.data_bus_enabled = value;
    }

    pub fn read(&self) -> bool {
        self.fetch_cadence_read || self.ppu_data_read
    }

    pub fn address(&self) -> PpuAddress {
        let full_bus_value = self.address_and_data_bus_detector.current_value().to_u16();
        PpuAddress::from_u16((full_bus_value & 0xFF00) | u16::from(self.address_latch))
    }

    #[must_use]
    pub fn set_address_bus(&mut self, addr: PpuAddress) -> bool {
        // During the entire VRAM address is output on the PPU address pins and
        // the lower eight bits stored in an external octal latch
        self.address_latch = addr.to_u16() as u8;
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

    pub fn ale(&self) -> bool {
        self.fetch_cadence_ale || self.ppu_data_read_ale || self.ppu_data_write_ale
    }

    pub fn r2_enabled(&self) -> ActiveHigh {
        nor(self.r2(), *self.data_bus_enabled).into()
    }

    // Active Low
    pub fn r2(&self) -> SignalLevel {
        if self.read_write == ReadWrite::Read && self.selected_register == PpuRegisterType::Status {
            SignalLevel::Low
        } else {
            SignalLevel::High
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PpuRegisterType {
    Ctrl,    // 0x2000
    Mask,    // 0x2001
    Status,  // 0x2002
    OamAddr, // 0x2003
    OamData, // 0x2004
    Scroll,  // 0x2005
    PpuAddr, // 0x2006
    PpuData, // 0x2007
}