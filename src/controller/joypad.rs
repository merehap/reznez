use std::ops::{Index, IndexMut};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::mapper::ReadResult;

// https://wiki.nesdev.com/w/index.php/Controller_reading_code
#[derive(Clone, Copy)]
pub struct Joypad {
    strobe_mode: StrobeMode,
    selected_button: Option<Button>,
    button_statuses: ButtonStatuses,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            strobe_mode: StrobeMode::On,
            selected_button: Some(Button::A),
            button_statuses: ButtonStatuses::ALL_UNPRESSED,
        }
    }

    // Peek 0x4016 and 0x4017
    pub fn peek_status(&self) -> ReadResult {
        let button_status = self.selected_button
            .map_or(ButtonStatus::Pressed, |b| self.button_statuses[b]);
        let value = match button_status {
            ButtonStatus::Unpressed => 0b0000_0000,
            ButtonStatus::Pressed   => 0b0000_0001,
        };

        ReadResult::partial(value, 0b0000_0111)
    }

    // Read 0x4016 and 0x4017
    pub fn read_status(&mut self) -> ReadResult {
        let status = self.peek_status();
        if self.strobe_mode == StrobeMode::Off {
            // Advance to the next button for the next read.
            self.selected_button = self.selected_button.and_then(Button::next);
        }

        status
    }

    pub fn change_strobe(&mut self, value: u8) {
        if value & 1 == 1 {
            self.strobe_on();
        } else {
            self.strobe_off();
        }
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
    pub const ALL: [Button; 8] =
        [Button::A, Button::B, Button::Select, Button::Start, Button::Up, Button::Down, Button::Left, Button::Right];

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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ButtonStatus {
    Unpressed,
    Pressed,
}
