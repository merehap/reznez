use crate::address::Address;
use crate::memory::Memory;

pub struct Cpu {
    accumulator: u8,
    x_index: u8,
    y_index: u8,
    program_counter: Address,
    stack_pointer: u8,
    status: Status,
    memory: Memory,
}

impl Cpu {
    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn startup() -> Cpu {
        Cpu {
            accumulator: 0,
            x_index: 0,
            y_index: 0,
            // TODO: Verify this value.
            program_counter: Address::new(0),
            stack_pointer: 0xFD,
            status: Status::startup(),
            memory: Memory::startup(),
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self) {
        self.status.interrupts_disabled = true;
        // TODO: APU resets?
    }

}

pub struct Status {
    negative: bool,
    overflow: bool,
    decimal: bool,
    interrupts_disabled: bool,
    zero: bool,
    carry: bool,
}

impl Status {
    fn startup() -> Status {
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
