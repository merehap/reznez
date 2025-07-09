use std::ops::{Index, IndexMut};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

// https://wiki.nesdev.com/w/index.php/Controller_reading_code
#[derive(Clone, Copy)]
pub struct Joypad {
    strobe_mode: StrobeMode,
    selected_button: Option<Button>,
    button_statuses: ButtonStatuses,
    // TODO: Make a Controller trait with a Disconnected implementor instead.
    enabled: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            strobe_mode: StrobeMode::On,
            selected_button: None,
            button_statuses: ButtonStatuses::ALL_UNPRESSED,
            enabled: true,
        }
    }

    pub fn disabled() -> Joypad {
        Joypad {
            strobe_mode: StrobeMode::On,
            selected_button: None,
            button_statuses: ButtonStatuses::ALL_UNPRESSED,
            enabled: false,
        }
    }

    pub fn peek_status(&self) -> ButtonStatus {
        if !self.enabled {
            return ButtonStatus::Unpressed;
        }

        if let Some(selected_button) = self.selected_button {
            self.button_statuses[selected_button]
        } else {
            // The wiki says this should be Pressed after all 8 bits are read, but
            // test_cpu_exec_space_apu.nes fails unless this is Unpressed.
            ButtonStatus::Unpressed
        }
    }

    pub fn read_status(&mut self) -> ButtonStatus {
        let status = self.peek_status();
        if self.strobe_mode == StrobeMode::Off {
            // Advance to the next button for the next read.
            self.selected_button = self.selected_button.and_then(Button::next);
        }

        status
    }

    pub fn strobe_on(&mut self) {
        self.strobe_mode = StrobeMode::On;
        self.selected_button = Some(Button::A);
    }

    pub fn strobe_off(&mut self) {
        self.strobe_mode = StrobeMode::Off;
    }

    pub fn set_button_status(&mut self, button: Button, status: ButtonStatus) {
        self.button_statuses[button] = status;
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum StrobeMode {
    Off,
    On,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, FromPrimitive)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    pub fn next(self) -> Option<Button> {
        if self == Button::Right {
            None
        } else {
            Some(Button::from_usize(self as usize + 1).unwrap())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ButtonStatuses([ButtonStatus; 8]);

impl ButtonStatuses {
    pub const ALL_UNPRESSED: ButtonStatuses =
        ButtonStatuses([ButtonStatus::Unpressed; 8]);
}

impl Index<Button> for ButtonStatuses {
    type Output = ButtonStatus;

    fn index(&self, button: Button) -> &ButtonStatus {
        &self.0[button as usize]
    }
}

impl IndexMut<Button> for ButtonStatuses {
    fn index_mut(&mut self, button: Button) -> &mut ButtonStatus {
        &mut self.0[button as usize]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ButtonStatus {
    Unpressed,
    Pressed,
}
