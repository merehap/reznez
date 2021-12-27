use std::ops::{Index, IndexMut};

use enum_iterator::IntoEnumIterator;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

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
            strobe_mode: StrobeMode::Off,
            selected_button: None,
            button_statuses: ButtonStatuses::ALL_UNPRESSED,
        }
    }

    pub fn selected_button_status(&mut self) -> ButtonStatus {
        if let Some(selected_button) = self.selected_button {
            self.button_statuses[selected_button]
        } else {
            // After every button has been cycled through, always return Pressed.
            ButtonStatus::Pressed
        }
    }

    pub fn select_next_button(&mut self) {
        if self.strobe_mode == StrobeMode::Off {
            self.selected_button =
                self.selected_button.map(Button::next).flatten();
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, FromPrimitive, IntoEnumIterator)]
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
