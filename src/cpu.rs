use crate::address::Address;
use crate::op_code::{Instruction, OpCode, Argument};
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

    pub fn step(&mut self) {
        let instruction = Instruction::from_memory(
            self.program_counter,
            self.accumulator,
            self.x_index,
            self.y_index,
            &self.memory,
        );

        let op_code = instruction.template.op_code;

        match instruction.argument {
            Argument::None =>
                self.execute_no_argument_op_code(op_code),
            Argument::Value(value) =>
                self.execute_value_argument_op_code(op_code, value),
            Argument::FlowControl(address) =>
                self.execute_flow_control_op_code(op_code, address),
        }
    }

    fn execute_no_argument_op_code(&mut self, op_code: OpCode) {
        use OpCode::*;
        match op_code {
            DEX => unimplemented!(),
            DEY => unimplemented!(),
            TAX => unimplemented!(),
            TAY => unimplemented!(),
            TSX => unimplemented!(),
            TXS => unimplemented!(),
            PHA => unimplemented!(),
            PLP => unimplemented!(),
            PHP => unimplemented!(),
            BRK => unimplemented!(),
            RTI => unimplemented!(),
            RTS => unimplemented!(),
            CLC => unimplemented!(),
            SEC => unimplemented!(),
            CLD => unimplemented!(),
            SED => unimplemented!(),
            CLI => unimplemented!(),
            SEI => unimplemented!(),
            CLV => unimplemented!(),
            NOP => unimplemented!(),
            JAM => panic!("JAM instruction encountered!"),
            _ => unreachable!("OpCode {:?} must take no arguments.", op_code),
        }
    }

    fn execute_value_argument_op_code(&mut self, op_code: OpCode, value: u8) {
        use OpCode::*;
        match op_code {
            ORA => unimplemented!(),
            AND => unimplemented!(),
            EOR => unimplemented!(),
            ADC => unimplemented!(),
            SBC => unimplemented!(),
            CMP => unimplemented!(),
            CPX => unimplemented!(),
            CPY => unimplemented!(),
            DEC => unimplemented!(),
            INC => unimplemented!(),
            ASL => unimplemented!(),
            ROL => unimplemented!(),
            LSR => unimplemented!(),
            LDA => unimplemented!(),
            STA => unimplemented!(),
            LDX => unimplemented!(),
            STX => unimplemented!(),
            LDY => unimplemented!(),
            STY => unimplemented!(),
            TXA => unimplemented!(),
            TYA => unimplemented!(),
            PLA => unimplemented!(),
            BIT => unimplemented!(),
            _ => unreachable!("OpCode {:?} must take a value argument.", op_code),
        }
    }

    fn execute_flow_control_op_code(&mut self, op_code: OpCode, address: Address) {
        use OpCode::*;
        match op_code {
            BPL => unimplemented!(),
            BMI => unimplemented!(),
            BVC => unimplemented!(),
            BVS => unimplemented!(),
            BCC => unimplemented!(),
            BCS => unimplemented!(),
            BNE => unimplemented!(),
            BEQ => unimplemented!(),
            JSR => unimplemented!(),
            JMP => unimplemented!(),
            BPL => unimplemented!(),
            _ => unreachable!("OpCode {:?} must take an address argument.", op_code),
        }
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
