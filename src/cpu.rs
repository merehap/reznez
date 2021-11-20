use crate::address::Address;
use crate::op_code::{Instruction, OpCode, Argument};
use crate::memory::Memory;

pub struct Cpu {
    accumulator: u8,
    x_index: u8,
    y_index: u8,
    program_counter: Address,
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
            status: Status::startup(),
            memory: Memory::startup(),
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self) {
        self.status.interrupts_disabled = true;
        // TODO: APU resets?
    }

    pub fn step(&mut self) {
        let instruction = Instruction::from_memory(
            self.program_counter,
            self.x_index,
            self.y_index,
            &self.memory,
        );

        self.program_counter = self.program_counter.advance(instruction.length());

        let op_code = instruction.template.op_code;

        match instruction.argument {
            Argument::Implicit =>
                self.execute_implicit_op_code(op_code),
            Argument::Immediate(value) =>
                self.execute_immediate_op_code(op_code, value),
            Argument::Address(address) => {
                let _jump_address = self.execute_address_op_code(op_code, address);
            },
        }
    }

    fn execute_implicit_op_code(&mut self, op_code: OpCode) {
        use OpCode::*;
        match op_code {
            DEX => self.x_index = self.nz(self.x_index.wrapping_sub(1)),
            DEY => self.y_index = self.nz(self.y_index.wrapping_sub(1)),
            TAX => self.x_index = self.nz(self.accumulator),
            TAY => self.y_index = self.nz(self.accumulator),
            TSX => self.x_index = self.nz(self.memory.stack_pointer),
            TXA => self.accumulator = self.nz(self.x_index),
            TXS => self.memory.stack_pointer = self.x_index,
            PHA => self.memory.push(self.accumulator),
            PHP => self.memory.push(self.status.to_byte()),
            PLA => self.accumulator = self.memory.pop(),
            PLP => self.status = Status::from_byte(self.memory.pop()),
            BRK => unimplemented!(),
            RTI => unimplemented!(),
            RTS => self.program_counter = self.memory.pop_address().advance(1),
            CLC => self.status.carry = false,
            SEC => self.status.carry = true,
            CLD => self.status.decimal = false,
            SED => self.status.decimal = true,
            CLI => self.status.interrupts_disabled = false,
            SEI => self.status.interrupts_disabled = true,
            CLV => self.status.overflow = false,
            NOP => {},

            JAM => panic!("JAM instruction encountered!"),
            _ => unreachable!("OpCode {:?} must take no arguments.", op_code),
        }
    }

    fn execute_immediate_op_code(&mut self, op_code: OpCode, value: u8) {
        use OpCode::*;
        match op_code {
            ORA => self.accumulator = self.ora(value),
            AND => self.accumulator = self.and(value),
            EOR => self.accumulator = self.eor(value),
            ADC => self.accumulator = self.adc(value),
            SBC => self.accumulator = self.sbc(value),
            CMP => self.cmp(value),
            CPX => self.cpx(value),
            CPY => self.cpy(value),
            ASL => {
                self.status.carry = (value >> 7) == 1;
                self.accumulator = self.nz(value << 1);
            },
            ROL => {
                let old_carry = self.status.carry;
                self.status.carry = (value >> 7) == 1;
                self.accumulator = value << 1;
                if old_carry {
                    self.accumulator |= 1;
                }
            },
            LSR => {
                self.status.carry = (value & 1) == 1;
                self.accumulator = self.nz(value >> 1);
            },
            ROR => {
                let old_carry = self.status.carry;
                self.status.carry = (value & 1) == 1;
                self.accumulator = value >> 1;
                if old_carry {
                    self.accumulator |= 0b1000_0000;
                }
            },
            LDA => self.accumulator = self.nz(value),
            LDX => self.x_index = self.nz(value),
            LDY => self.y_index = self.nz(value),
            TXA => self.accumulator = self.nz(value),
            TYA => self.accumulator = self.nz(value),
            BIT => {
                self.status.negative = value & 0b1000_0000 != 0;
                self.status.overflow = value & 0b0100_0000 != 0;
                self.status.zero = value & self.accumulator == 0;
            },
            _ => unreachable!("OpCode {:?} must take a value argument.", op_code),
        }
    }

    fn execute_address_op_code(&mut self, op_code: OpCode, address: Address) -> Option<Address> {
        let mut jump_address = None;

        use OpCode::*;
        match op_code {
            ORA => self.memory[address] = self.ora(self.memory[address]),
            AND => self.memory[address] = self.and(self.memory[address]),
            EOR => self.memory[address] = self.eor(self.memory[address]),
            ADC => self.memory[address] = self.adc(self.memory[address]),
            SBC => self.memory[address] = self.sbc(self.memory[address]),
            CMP => self.cmp(self.memory[address]),
            CPX => self.cpx(self.memory[address]),
            CPY => self.cpy(self.memory[address]),

            STA => self.memory[address] = self.accumulator,
            STX => self.memory[address] = self.x_index,
            STY => self.memory[address] = self.y_index,
            DEC => self.memory[address] = self.nz(self.memory[address].wrapping_sub(1)),
            INC => self.memory[address] = self.nz(self.memory[address].wrapping_add(1)),
            BPL => if !self.status.negative {jump_address = Some(address)},
            BMI => if self.status.negative {jump_address = Some(address)},
            BVC => if !self.status.overflow {jump_address = Some(address)},
            BVS => if self.status.overflow {jump_address = Some(address)},
            BCC => if !self.status.carry {jump_address = Some(address)},
            BCS => if self.status.carry {jump_address = Some(address)},
            BNE => if !self.status.zero {jump_address = Some(address)},
            BEQ => if self.status.zero {jump_address = Some(address)},
            JSR => {
                // Push the address one previous for some reason.
                self.memory.push_address(self.program_counter.offset(-1));
                jump_address = Some(address);
            },
            JMP => jump_address = Some(address),
            _ => unreachable!("OpCode {:?} must take an address argument.", op_code),
        }

        jump_address
    }

    fn ora(&mut self, value: u8) -> u8 {
        self.nz(self.accumulator | value)
    }

    fn and(&mut self, value: u8) -> u8 {
        self.nz(self.accumulator & value)
    }

    fn eor(&mut self, value: u8) -> u8 {
        self.nz(self.accumulator ^ value)
    }

    fn adc(&mut self, value: u8) -> u8 {
        let carry = if self.status.carry {1} else {0};
        let result =
            (self.accumulator as u16) +
            (value as u16) +
            (carry as u16);
        self.status.carry = result > 0xFF;
        let result = self.nz(result as u8);
        self.status.overflow =
            (is_pos(self.accumulator) == is_pos(value)) &&
            (is_pos(self.accumulator) == is_pos(result));
        result
    }

    fn sbc(&mut self, value: u8) -> u8 {
        let carry = if self.status.carry {0} else {1};
        // Convert u8s to possibly negative values before widening them.
        let result = (self.accumulator as i8) as i16 - (value as i8) as i16 - carry;
        self.status.overflow = result < -128 || result > 127;
        // TODO: Is carry supposed to be set? The spec says so.
        self.nz(result as u8)
    }

    fn cmp(&mut self, value: u8) {
        self.nz(self.accumulator.wrapping_sub(value));
        self.status.carry = self.accumulator >= value;
    }

    fn cpx(&mut self, value: u8) {
        self.nz(self.x_index.wrapping_sub(value));
        self.status.carry = self.x_index >= value;
    }

    fn cpy(&mut self, value: u8) {
        self.nz(self.x_index.wrapping_sub(value));
        self.status.carry = self.y_index >= value;
    }

    fn nz(&mut self, value: u8) -> u8 {
        self.status.negative = (value as i8) < 0;
        self.status.zero = value == 0;
        value
    }
}

fn is_pos(value: u8) -> bool {
    (value >> 7) == 0
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

    fn from_byte(value: u8) -> Status {
        let mut status = Status::startup();
        [ status.negative
        , status.overflow
        , _
        , _
        , status.decimal
        , status.interrupts_disabled
        , status.zero
        , status.carry,
        ] = unpack_bools(value);

        status
    }

    fn to_byte(&self) -> u8 {
        pack_bools([
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

    fn to_string(&self) -> String {
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

fn pack_bools(bools: [bool; 8]) -> u8 {
    let mut result = 0;
    for i in 0..8 {
        if bools[7 - i as usize] {
            result += 1 << i;
        }
    }

    result
}

fn unpack_bools(value: u8) -> [bool; 8] {
    let mut bools = [false; 8];

    for i in 0..8 {
        bools[i] = (value & 0b1000_0000) != 0;
    }

    bools
}
