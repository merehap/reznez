use std::fmt;

use crate::util;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Status {
    pub negative: bool,
    pub overflow: bool,
    pub decimal: bool,
    pub interrupts_disabled: bool,
    pub zero: bool,
    pub carry: bool,
}

impl Status {
    pub fn startup() -> Status {
        Status {
            // https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
            negative: false,
            overflow: false,
            decimal: false,
            interrupts_disabled: true,
            zero: false,
            carry: false,
        }
    }

    pub fn from_byte(value: u8) -> Status {
        let mut status = Status::startup();
        [ status.negative
        , status.overflow
        , _
        , _
        , status.decimal
        , status.interrupts_disabled
        , status.zero
        , status.carry,
        ] = util::unpack_bools(value);

        status
    }

    pub fn to_register_byte(self) -> u8 {
        util::pack_bools([
            self.negative,
            self.overflow,
            false,
            false,
            self.decimal,
            self.interrupts_disabled,
            self.zero,
            self.carry,
        ])
    }

    pub fn to_instruction_byte(self) -> u8 {
        util::pack_bools([
            self.negative,
            self.overflow,
            true,
            true,
            self.decimal,
            self.interrupts_disabled,
            self.zero,
            self.carry,
        ])
    }

    pub fn to_interrupt_byte(self) -> u8 {
        util::pack_bools([
            self.negative,
            self.overflow,
            true,
            false,
            self.decimal,
            self.interrupts_disabled,
            self.zero,
            self.carry,
        ])
    }
}

impl fmt::Display for Status {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        write!(f,
            "{}{}__{}{}{}{}",
            if self.negative {'N'} else {'_'},
            if self.overflow {'V'} else {'_'},
            if self.decimal {'D'} else {'_'},
            if self.interrupts_disabled {'I'} else {'_'},
            if self.zero {'Z'} else {'_'},
            if self.carry {'C'} else {'_'},
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_SET: Status = Status {
        negative: true,
        overflow: true,
        decimal: true,
        interrupts_disabled: true,
        zero: true,
        carry: true,
    };

    const NONE_SET: Status = Status {
        negative: false,
        overflow: false,
        decimal: false,
        interrupts_disabled: false,
        zero: false,
        carry: false,
    };

    const MIXED_SET: Status = Status {
        negative: true,
        overflow: false,
        decimal: true,
        interrupts_disabled: false,
        zero: false,
        carry: true,
    };

    #[test]
    fn all_set_to_string() {
        assert_eq!(ALL_SET.to_string(), "NV__DIZC");
    }

    #[test]
    fn all_set_to_byte() {
        assert_eq!(ALL_SET.to_instruction_byte(), 0b1111_1111);
    }

    #[test]
    fn all_set_round_trip() {
        assert_eq!(Status::from_byte(ALL_SET.to_instruction_byte()), ALL_SET);
    }

    #[test]
    fn none_set_to_string() {
        assert_eq!(NONE_SET.to_string(), "________");
    }

    #[test]
    fn none_set_to_byte() {
        assert_eq!(NONE_SET.to_instruction_byte(), 0b0011_0000);
    }

    #[test]
    fn none_set_round_trip() {
        assert_eq!(Status::from_byte(NONE_SET.to_instruction_byte()), NONE_SET);
    }

    #[test]
    fn mixed_set_to_string() {
        assert_eq!(MIXED_SET.to_string(), "N___D__C");
    }

    #[test]
    fn mixed_set_round_trip() {
        assert_eq!(Status::from_byte(MIXED_SET.to_instruction_byte()), MIXED_SET);
    }

    #[test]
    fn mixed_set_to_byte() {
        assert_eq!(MIXED_SET.to_instruction_byte(), 0b1011_1001);
    }
}
