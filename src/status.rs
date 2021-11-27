use crate::util;

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
            negative: false,
            // https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
            overflow: true,
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

    pub fn to_byte(&self) -> u8 {
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

    pub fn to_string(&self) -> String {
        format!(
            "{}{}bb{}{}{}{}",
            if self.negative {'N'} else {'n'},
            if self.overflow {'V'} else {'v'},
            if self.decimal {'D'} else {'d'},
            if self.interrupts_disabled {'I'} else {'i'},
            if self.zero {'Z'} else {'z'},
            if self.carry {'C'} else {'c'},
        )
    }
}
