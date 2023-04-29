use std::cell::RefCell;
use std::rc::Rc;

use crate::controller::joypad::Joypad;
use crate::memory::cpu::cpu_address::CpuAddress;

pub struct Ports {
    pub oam_dma: OamDmaPort,
    pub joypad1: Rc<RefCell<Joypad>>,
    pub joypad2: Rc<RefCell<Joypad>>,
}

impl Ports {
    pub fn new(joypad1: Rc<RefCell<Joypad>>, joypad2: Rc<RefCell<Joypad>>) -> Ports {
        Ports { oam_dma: OamDmaPort::new(), joypad1, joypad2 }
    }

    pub fn change_strobe(&mut self, value: u8) {
        if value & 1 == 1 {
            self.joypad1.borrow_mut().strobe_on();
            self.joypad2.borrow_mut().strobe_on();
        } else {
            self.joypad1.borrow_mut().strobe_off();
            self.joypad2.borrow_mut().strobe_off();
        };
    }
}

#[derive(Clone)]
pub struct OamDmaPort {
    page: Rc<RefCell<Option<u8>>>,
    // TODO: Find a way to remove this field.
    current_address: CpuAddress,
}

impl OamDmaPort {
    pub fn new() -> OamDmaPort {
        OamDmaPort {
            page: Rc::new(RefCell::new(None)),
            current_address: CpuAddress::new(0x0000),
        }
    }

    pub fn set_page(&mut self, page: u8) {
        *self.page.borrow_mut() = Some(page);
    }

    pub fn page_present(&self) -> bool {
        self.page.borrow().is_some()
    }

    pub fn take_page(&mut self) -> Option<()> {
        if let Some(port) = self.page.borrow_mut().take() {
            self.current_address = CpuAddress::from_low_high(0x00, port);
            Some(())
        } else {
            None
        }
    }

    pub fn current_address(&self) -> CpuAddress {
        self.current_address
    }

    pub fn increment_current_address(&mut self) {
        self.current_address.inc();
    }
}

#[cfg(test)]
pub mod test_data {
    use super::*;

    pub fn ports() -> Ports {
        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
        Ports::new(joypad1, joypad2)
    }
}
