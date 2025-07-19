use crate::controller::joypad::Joypad;

pub struct Ports {
    pub joypad1: Joypad,
    pub joypad2: Joypad,
}

impl Ports {
    pub fn new(joypad1: Joypad, joypad2: Joypad) -> Ports {
        Ports { joypad1, joypad2 }
    }

    pub fn change_strobe(&mut self, value: u8) {
        if value & 1 == 1 {
            self.joypad1.strobe_on();
            self.joypad2.strobe_on();
        } else {
            self.joypad1.strobe_off();
            self.joypad2.strobe_off();
        };
    }
}