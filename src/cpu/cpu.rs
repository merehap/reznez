use crate::cpu::address::Address;
use crate::cpu::instruction::{Instruction, OpCode, Argument};
use crate::cpu::memory::Memory;
use crate::cpu::status::Status;

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

    cycle: u64,
}

impl Cpu {
    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn startup(memory: Memory) -> Cpu {
        let program_counter = memory.address_from_vector(RESET_VECTOR);
        Cpu::with_program_counter(memory, program_counter)
    }

    pub fn with_program_counter(memory: Memory, program_counter: Address) -> Cpu {
        println!("Starting execution at PC=0x{:4X}", program_counter.to_raw());
        Cpu {
            accumulator: 0,
            x_index: 0,
            y_index: 0,
            program_counter,
            status: Status::startup(),
            memory,

            // Unclear why this is the case.
            cycle: 7,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self) {
        self.status.interrupts_disabled = true;
        // TODO: APU resets?
    }

    pub fn state_string(&self) -> String {
        let nesting = "";
        format!("{:010} PC:{}, A:0x{:02X}, X:0x{:02X}, Y:0x{:02X}, P:0x{:02X}, S:0x{:02X}, {} {}",
            self.cycle,
            self.program_counter,
            self.accumulator,
            self.x_index,
            self.y_index,
            self.status.to_byte(),
            self.stack_pointer(),
            self.status.to_string(),
            nesting,
        )
    }

    pub fn accumulator(&self) -> u8 {
        self.accumulator
    }

    pub fn x_index(&self) -> u8 {
        self.x_index
    }

    pub fn y_index(&self) -> u8 {
        self.y_index
    }

    pub fn program_counter(&self) -> Address {
        self.program_counter
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn stack_pointer(&self) -> u8 {
        self.memory.stack_pointer
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn step(&mut self) -> Instruction {
        let instruction = Instruction::from_memory(
            self.program_counter,
            self.x_index,
            self.y_index,
            &self.memory,
        );

        println!("{} | {}", self.state_string(), instruction);

        self.program_counter = self.program_counter.advance(instruction.length());

        let op_code = instruction.template.op_code;

        match instruction.argument {
            Argument::Implicit =>
                self.execute_implicit_op_code(op_code),
            Argument::Immediate(value) =>
                self.execute_immediate_op_code(op_code, value),
            Argument::Address(address) => {
                if let (Some(jump_address), branch_taken, branch_crossed_page_boundary) =
                        self.execute_address_op_code(op_code, address) {
                    self.program_counter = jump_address;
                    if branch_taken {
                        println!("Branch taken, cycle added.");
                        self.cycle += 1;
                    }

                    if branch_crossed_page_boundary {
                        println!("Branch crossed page boundary, cycle added.");
                        self.cycle += 1;
                    }
                }
            },
        }

        self.cycle += instruction.template.cycle_count as u64;
        if instruction.should_add_oops_cycle() {
            println!("'Oops' cycle added.");
            self.cycle += 1;
        }

        instruction
    }

    fn execute_implicit_op_code(&mut self, op_code: OpCode) {
        use OpCode::*;
        match op_code {
            INX => self.x_index = self.nz(self.x_index.wrapping_add(1)),
            INY => self.y_index = self.nz(self.y_index.wrapping_add(1)),
            DEX => self.x_index = self.nz(self.x_index.wrapping_sub(1)),
            DEY => self.y_index = self.nz(self.y_index.wrapping_sub(1)),
            TAX => self.x_index = self.nz(self.accumulator),
            TAY => self.y_index = self.nz(self.accumulator),
            TSX => self.x_index = self.nz(self.memory.stack_pointer),
            TXS => self.memory.stack_pointer = self.x_index,
            TXA => self.accumulator = self.nz(self.x_index),
            TYA => self.accumulator = self.nz(self.y_index),
            PHA => self.memory.push(self.accumulator),
            PHP => self.memory.push(self.status.to_byte()),
            PLA => {
                self.accumulator = self.memory.pop();
                self.nz(self.accumulator);
            },
            PLP => self.status = Status::from_byte(self.memory.pop()),
            BRK => {
                // Not sure why we need to increment here.
                self.program_counter.inc();
                self.memory.push_address(self.program_counter);
                self.memory.push(self.status.to_byte());
                self.status.interrupts_disabled = true;
                self.program_counter = self.memory.address_from_vector(IRQ_VECTOR);
            },
            RTI => {
                self.status = Status::from_byte(self.memory.pop());
                self.program_counter = self.memory.pop_address();
            },
            RTS => self.program_counter = self.memory.pop_address().advance(1),
            CLC => self.status.carry = false,
            SEC => self.status.carry = true,
            CLD => self.status.decimal = false,
            SED => self.status.decimal = true,
            CLI => self.status.interrupts_disabled = false,
            SEI => self.status.interrupts_disabled = true,
            CLV => self.status.overflow = false,

            ASL => self.accumulator = self.asl(self.accumulator),
            ROL => self.accumulator = self.rol(self.accumulator),
            LSR => self.accumulator = self.lsr(self.accumulator),
            ROR => self.accumulator = self.ror(self.accumulator),

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
            SBC => {
                self.accumulator = self.subtract_from_accumulator(value);
                self.status.carry = !self.status.negative;
            },
            CMP => self.cmp(value),
            CPX => self.cpx(value),
            CPY => self.cpy(value),

            LDA => self.accumulator = self.nz(value),
            LDX => self.x_index = self.nz(value),
            LDY => self.y_index = self.nz(value),

            // A NOP that takes an argument, but ignores it.
            NOP => {},

            _ => unreachable!("OpCode {:?} must take a value argument.", op_code),
        }
    }

    fn execute_address_op_code(
        &mut self,
        op_code: OpCode,
        address: Address,
    ) -> (Option<Address>, bool, bool) {

        let mut jump_address = None;
        let mut branch_taken = false;
        let value = self.memory[address];

        use OpCode::*;
        match op_code {
            ORA => self.accumulator = self.ora(value),
            AND => self.accumulator = self.and(value),
            EOR => self.accumulator = self.eor(value),
            ADC => self.accumulator = self.adc(value),
            SBC => {
                self.accumulator = self.subtract_from_accumulator(value);
                self.status.carry = !self.status.negative;
            },
            CMP => self.cmp(value),
            CPX => self.cpx(value),
            CPY => self.cpy(value),

            ASL => self.memory[address] = self.asl(value),
            ROL => self.memory[address] = self.rol(value),
            LSR => self.memory[address] = self.lsr(value),
            ROR => self.memory[address] = self.ror(value),

            STA => self.memory[address] = self.accumulator,
            STX => self.memory[address] = self.x_index,
            STY => self.memory[address] = self.y_index,
            DEC => self.memory[address] = self.nz(value.wrapping_sub(1)),
            INC => self.memory[address] = self.nz(value.wrapping_add(1)),

            LDA => self.accumulator = self.nz(value),
            LDX => self.x_index = self.nz(value),
            LDY => self.y_index = self.nz(value),

            BIT => {
                self.status.negative = value & 0b1000_0000 != 0;
                self.status.overflow = value & 0b0100_0000 != 0;
                self.status.zero = value & self.accumulator == 0;
            },

            BPL => if !self.status.negative {branch_taken = true},
            BMI => if self.status.negative {branch_taken = true},
            BVC => if !self.status.overflow {branch_taken = true},
            BVS => if self.status.overflow {branch_taken = true},
            BCC => if !self.status.carry {branch_taken = true},
            BCS => if self.status.carry {branch_taken = true},
            BNE => if !self.status.zero {branch_taken = true},
            BEQ => if self.status.zero {branch_taken = true},
            JSR => {
                // Push the address one previous for some reason.
                self.memory.push_address(self.program_counter.offset(-1));
                jump_address = Some(address);
            },
            JMP => jump_address = Some(address),


            // Undocumented op codes.
            SLO => {
                self.memory[address] = self.asl(value);
                self.accumulator |= self.memory[address];
                self.nz(self.accumulator);
            },
            RLA => {
                self.memory[address] = self.rol(value);
                self.accumulator &= self.memory[address];
                self.nz(self.accumulator);
            },
            SRE => {
                self.memory[address] = self.lsr(value);
                self.accumulator ^= self.memory[address];
                self.nz(self.accumulator);
            },
            RRA => {
                self.memory[address] = self.ror(value);
                self.accumulator = self.adc(self.memory[address]);
                self.nz(self.accumulator);
            },
            SAX => self.memory[address] = self.accumulator & self.x_index,
            LAX => {
                self.accumulator = value;
                self.x_index = value;
                self.nz(value);
            },
            DCP => {
                self.memory[address] = value.wrapping_sub(1);
                self.cmp(self.memory[address]);
            },
            ISC => {
                self.memory[address] = value.wrapping_add(1);
                self.accumulator = self.subtract_from_accumulator(self.memory[address]);
            },

            // A NOP that takes an address, but ignores it.
            NOP => {},

            // Mostly unstable codes.
            ANC => unimplemented!(),
            ALR => unimplemented!(),
            ARR => unimplemented!(),
            XAA => unimplemented!(),
            AXS => unimplemented!(),
            AHX => unimplemented!(),
            SHY => unimplemented!(),
            SHX => unimplemented!(),
            TAS => unimplemented!(),
            LAS => unimplemented!(),

            _ => unreachable!("OpCode {:?} must take an address argument.", op_code),
        }

        let mut branch_crossed_page_boundary = false;
        if branch_taken {
            println!("Branch taken! PC: {}, Jump: {}", self.program_counter, address);
            jump_address = Some(address);
            branch_crossed_page_boundary = self.program_counter.page() != address.page();
        }

        (jump_address, branch_taken, branch_crossed_page_boundary)
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

    fn subtract_from_accumulator(&mut self, value: u8) -> u8 {
        let carry = if self.status.carry {0} else {1};
        // Convert u8s to possibly negative values before widening them.
        let result = (self.accumulator as i8) as i16 - (value as i8) as i16 - carry;
        self.status.overflow = result < -128 || result > 127;
        //self.status.carry = !is_neg(result as u8);
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
        self.nz(self.y_index.wrapping_sub(value));
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

        self.nz(result)
    }

    fn ror(&mut self, value: u8) -> u8 {
        let old_carry = self.status.carry;
        self.status.carry = (value & 1) == 1;
        let mut result = value >> 1;
        if old_carry {
            result |= 0b1000_0000;
        }

        self.nz(result)
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
