use crate::address::Address;
use crate::cartridge::INes;
use crate::op_code::{Instruction, OpCode, Argument};
use crate::mapper::mapper0::Mapper0;
use crate::memory::Memory;
use crate::status::Status;

const NMI_VECTOR: Address = Address::new(0xFFFA);
const RESET_VECTOR: Address = Address::new(0xFFFC);
const IRQ_VECTOR: Address = Address::new(0xFFFE);

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
    pub fn startup(ines: INes) -> Cpu {
        if ines.mapper_number() != 0 {
            panic!("Only mapper 0 is currently supported.");
        }

        let mut memory = Memory::startup();

        let mapper = Mapper0::new();
        mapper.map(ines, &mut memory)
            .expect("Failed to copy cartridge ROM into CPU memory.");

        let program_counter = memory.address_from_vector(RESET_VECTOR);
        println!("Starting execution at PC=0x{:4X}", program_counter.to_raw());

        Cpu {
            accumulator: 0,
            x_index: 0,
            y_index: 0,
            program_counter,
            status: Status::startup(),
            memory,
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

        println!("Instruction: {:?}", instruction);

        self.program_counter = self.program_counter.advance(instruction.length());

        let op_code = instruction.template.op_code;

        match instruction.argument {
            Argument::Implicit =>
                self.execute_implicit_op_code(op_code),
            Argument::Immediate(value) =>
                self.execute_immediate_op_code(op_code, value),
            Argument::Address(address) => {
                if let Some(jump_address) = self.execute_address_op_code(op_code, address) {
                    self.program_counter = jump_address;
                }
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
            ASL => self.accumulator = self.asl(self.accumulator),
            ROL => self.accumulator = self.rol(self.accumulator),
            LSR => self.accumulator = self.lsr(self.accumulator),
            ROR => self.accumulator = self.ror(self.accumulator),

            LDA => self.accumulator = self.nz(value),
            LDX => self.x_index = self.nz(value),
            LDY => self.y_index = self.nz(value),
            TXA => self.accumulator = self.nz(value),
            TYA => self.accumulator = self.nz(value),
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

            ASL => self.memory[address] = self.asl(self.memory[address]),
            ROL => self.memory[address] = self.rol(self.memory[address]),
            LSR => self.memory[address] = self.lsr(self.memory[address]),
            ROR => self.memory[address] = self.ror(self.memory[address]),

            STA => self.memory[address] = self.accumulator,
            STX => self.memory[address] = self.x_index,
            STY => self.memory[address] = self.y_index,
            DEC => self.memory[address] = self.nz(self.memory[address].wrapping_sub(1)),
            INC => self.memory[address] = self.nz(self.memory[address].wrapping_add(1)),

            BIT => {
                let value = self.memory[address];
                self.status.negative = value & 0b1000_0000 != 0;
                self.status.overflow = value & 0b0100_0000 != 0;
                self.status.zero = value & self.accumulator == 0;
            },

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
        // If the inputs have the same sign, set overflow if the output doesn't.
        self.status.overflow =
            (is_neg(self.accumulator) == is_neg(value)) &&
            (is_neg(self.accumulator) != is_neg(result));
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

    fn asl(&mut self, value: u8) -> u8 {
        self.status.carry = (value >> 7) == 1;
        self.nz(value << 1)
    }

    fn rol(&mut self, value: u8) -> u8 {
        let old_carry = self.status.carry;
        self.status.carry = (value >> 7) == 1;
        let mut result = value << 1;
        if old_carry {
            result |= 1;
        }

        result
    }

    fn ror(&mut self, value: u8) -> u8 {
        let old_carry = self.status.carry;
        self.status.carry = (value & 1) == 1;
        let mut result = value >> 1;
        if old_carry {
            result |= 0b1000_0000;
        }

        result
    }

    fn lsr(&mut self, value: u8) -> u8 {
        self.status.carry = (value & 1) == 1;
        self.nz(value >> 1)
    }

    fn nz(&mut self, value: u8) -> u8 {
        self.status.negative = is_neg(value);
        self.status.zero = value == 0;
        value
    }
}

fn is_neg(value: u8) -> bool {
    (value >> 7) == 1
}
