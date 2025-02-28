use std::cell::RefCell;
use std::rc::Rc;

use crate::controller::joypad::Joypad;

pub struct Ports {
    pub joypad1: Rc<RefCell<Joypad>>,
    pub joypad2: Rc<RefCell<Joypad>>,
}

impl Ports {
    pub fn new(joypad1: Rc<RefCell<Joypad>>, joypad2: Rc<RefCell<Joypad>>) -> Ports {
        Ports { joypad1, joypad2 }
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

#[cfg(test)]
pub mod test_data {
    use super::*;

    pub fn ports() -> Ports {
        let joypad1 = Rc::new(RefCell::new(Joypad::new()));
        let joypad2 = Rc::new(RefCell::new(Joypad::new()));
        Ports::new(joypad1, joypad2)
    }
}
