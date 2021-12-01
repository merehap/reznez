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
    pub memory: Memory,

    current_instruction_remaining_cycles: u8,
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

            current_instruction_remaining_cycles: 0,
            // Unclear why this is the case.
            cycle: 7,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self) {
        self.status.interrupts_disabled = true;
        self.program_counter = self.memory.address_from_vector(RESET_VECTOR);
        self.cycle = 7;
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
            self.status,
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

    pub fn step(&mut self) -> Option<Instruction> {
        self.cycle += 1;

        if self.current_instruction_remaining_cycles != 0 {
            self.current_instruction_remaining_cycles -= 1;
            return None;
        }

        let instruction = Instruction::from_memory(
            self.program_counter,
            self.x_index,
            self.y_index,
            &self.memory,
        );
        println!("{} | {}", self.state_string(), instruction);

        let cycle_count = self.execute_instruction(instruction);
        self.current_instruction_remaining_cycles = cycle_count - 1;

        Some(instruction)
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> u8 {
        self.program_counter = self.program_counter.advance(instruction.length());

        let mut cycle_count = instruction.template.cycle_count as u8;
        if instruction.should_add_oops_cycle() {
            println!("'Oops' cycle added.");
            cycle_count += 1;
        }

        use OpCode::*;
        use Argument::*;
        match (instruction.template.op_code, instruction.argument) {
            (INX, Imp) => self.x_index = self.nz(self.x_index.wrapping_add(1)),
            (INY, Imp) => self.y_index = self.nz(self.y_index.wrapping_add(1)),
            (DEX, Imp) => self.x_index = self.nz(self.x_index.wrapping_sub(1)),
            (DEY, Imp) => self.y_index = self.nz(self.y_index.wrapping_sub(1)),
            (TAX, Imp) => self.x_index = self.nz(self.accumulator),
            (TAY, Imp) => self.y_index = self.nz(self.accumulator),
            (TSX, Imp) => self.x_index = self.nz(self.memory.stack_pointer),
            (TXS, Imp) => self.memory.stack_pointer = self.x_index,
            (TXA, Imp) => self.accumulator = self.nz(self.x_index),
            (TYA, Imp) => self.accumulator = self.nz(self.y_index),
            (PHA, Imp) => self.memory.push(self.accumulator),
            (PHP, Imp) => self.memory.push(self.status.to_byte()),
            (PLA, Imp) => {
                self.accumulator = self.memory.pop();
                self.nz(self.accumulator);
            },
            (PLP, Imp) => self.status = Status::from_byte(self.memory.pop()),
            (CLC, Imp) => self.status.carry = false,
            (SEC, Imp) => self.status.carry = true,
            (CLD, Imp) => self.status.decimal = false,
            (SED, Imp) => self.status.decimal = true,
            (CLI, Imp) => self.status.interrupts_disabled = false,
            (SEI, Imp) => self.status.interrupts_disabled = true,
            (CLV, Imp) => self.status.overflow = false,
            (BRK, Imp) => {
                // Not sure why we need to increment here.
                self.program_counter.inc();
                self.memory.push_address(self.program_counter);
                self.memory.push(self.status.to_byte());
                self.status.interrupts_disabled = true;
                self.program_counter = self.memory.address_from_vector(IRQ_VECTOR);
            },
            (RTI, Imp) => {
                self.status = Status::from_byte(self.memory.pop());
                self.program_counter = self.memory.pop_address();
            },
            (RTS, Imp) => self.program_counter = self.memory.pop_address().advance(1),

            (STA, Addr(addr, _)) => self.memory[addr] = self.accumulator,
            (STX, Addr(addr, _)) => self.memory[addr] = self.x_index,
            (STY, Addr(addr, _)) => self.memory[addr] = self.y_index,
            (DEC, Addr(addr, _)) => self.memory[addr] = self.nz(self.memory[addr].wrapping_sub(1)),
            (INC, Addr(addr, _)) => self.memory[addr] = self.nz(self.memory[addr].wrapping_add(1)),
            (BPL, Addr(addr, _)) => if !self.status.negative {cycle_count += self.take_branch(addr);},
            (BMI, Addr(addr, _)) => if self.status.negative {cycle_count += self.take_branch(addr);},
            (BVC, Addr(addr, _)) => if !self.status.overflow {cycle_count += self.take_branch(addr);},
            (BVS, Addr(addr, _)) => if self.status.overflow {cycle_count += self.take_branch(addr);},
            (BCC, Addr(addr, _)) => if !self.status.carry {cycle_count += self.take_branch(addr);},
            (BCS, Addr(addr, _)) => if self.status.carry {cycle_count += self.take_branch(addr);},
            (BNE, Addr(addr, _)) => if !self.status.zero {cycle_count += self.take_branch(addr);},
            (BEQ, Addr(addr, _)) => if self.status.zero {cycle_count += self.take_branch(addr);},
            (JSR, Addr(addr, _)) => {
                // Push the address one previous for some reason.
                self.memory.push_address(self.program_counter.offset(-1));
                self.program_counter = addr;
            },
            (JMP, Addr(addr, _)) => self.program_counter = addr,

            (BIT, Addr(_, val)) => {
                self.status.negative = val & 0b1000_0000 != 0;
                self.status.overflow = val & 0b0100_0000 != 0;
                self.status.zero = val & self.accumulator == 0;
            },

            (LDA, Imm(val) | Addr(_, val)) => self.accumulator = self.nz(val),
            (LDX, Imm(val) | Addr(_, val)) => self.x_index = self.nz(val),
            (LDY, Imm(val) | Addr(_, val)) => self.y_index = self.nz(val),
            (CMP, Imm(val) | Addr(_, val)) => self.cmp(val),
            (CPX, Imm(val) | Addr(_, val)) => self.cpx(val),
            (CPY, Imm(val) | Addr(_, val)) => self.cpy(val),
            (ORA, Imm(val) | Addr(_, val)) => self.accumulator = self.nz(self.accumulator | val),
            (AND, Imm(val) | Addr(_, val)) => self.accumulator = self.nz(self.accumulator & val),
            (EOR, Imm(val) | Addr(_, val)) => self.accumulator = self.nz(self.accumulator ^ val),
            (ADC, Imm(val) | Addr(_, val)) => self.accumulator = self.adc(val),
            (SBC, Imm(val) | Addr(_, val)) => {
                self.accumulator = self.subtract_from_accumulator(val);
                self.status.carry = !self.status.negative;
            },

            (ASL, Imp)           => self.accumulator  = self.asl(self.accumulator),
            (ASL, Addr(addr, _)) => self.memory[addr] = self.asl(self.memory[addr]),
            (ROL, Imp)           => self.accumulator  = self.rol(self.accumulator),
            (ROL, Addr(addr, _)) => self.memory[addr] = self.rol(self.memory[addr]),
            (LSR, Imp)           => self.accumulator  = self.lsr(self.accumulator),
            (LSR, Addr(addr, _)) => self.memory[addr] = self.lsr(self.memory[addr]),
            (ROR, Imp)           => self.accumulator  = self.ror(self.accumulator),
            (ROR, Addr(addr, _)) => self.memory[addr] = self.ror(self.memory[addr]),

            // Undocumented op codes.
            (SLO, Addr(addr, _)) => {
                self.memory[addr] = self.asl(self.memory[addr]);
                self.accumulator |= self.memory[addr];
                self.nz(self.accumulator);
            },
            (RLA, Addr(addr, _)) => {
                self.memory[addr] = self.rol(self.memory[addr]);
                self.accumulator &= self.memory[addr];
                self.nz(self.accumulator);
            },
            (SRE, Addr(addr, _)) => {
                self.memory[addr] = self.lsr(self.memory[addr]);
                self.accumulator ^= self.memory[addr];
                self.nz(self.accumulator);
            },
            (RRA, Addr(addr, _)) => {
                self.memory[addr] = self.ror(self.memory[addr]);
                self.accumulator = self.adc(self.memory[addr]);
                self.nz(self.accumulator);
            },
            (SAX, Addr(addr, _)) => self.memory[addr] = self.accumulator & self.x_index,
            (LAX, Addr(addr, _)) => {
                self.accumulator = self.memory[addr];
                self.x_index = self.memory[addr];
                self.nz(self.memory[addr]);
            },
            (DCP, Addr(addr, _)) => {
                self.memory[addr] = self.memory[addr].wrapping_sub(1);
                self.cmp(self.memory[addr]);
            },
            (ISC, Addr(addr, _)) => {
                self.memory[addr] = self.memory[addr].wrapping_add(1);
                self.accumulator = self.subtract_from_accumulator(self.memory[addr]);
            },

            // Mostly unstable codes.
            (ANC, _) => unimplemented!(),
            (ALR, _) => unimplemented!(),
            (ARR, _) => unimplemented!(),
            (XAA, _) => unimplemented!(),
            (AXS, _) => unimplemented!(),
            (AHX, _) => unimplemented!(),
            (SHY, _) => unimplemented!(),
            (SHX, _) => unimplemented!(),
            (TAS, _) => unimplemented!(),
            (LAS, _) => unimplemented!(),

            (NOP, _) => {},
            (JAM, _) => panic!("JAM instruction encountered!"),
            (op_code, arg) => unreachable!("Argument type {:?} is invalid for the {:?} opcode.", arg, op_code),
        }

        cycle_count as u8
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

    fn take_branch(&mut self, destination: Address) -> u8 {
        println!("Branch taken, cycle added.");
        let mut cycle_count = 1;

        if self.program_counter.page() != destination.page() {
            println!("Branch crossed page boundary, 'Oops' cycle added.");
            cycle_count += 1;
        }

        self.program_counter = destination;

        cycle_count
    }
}

fn is_neg(value: u8) -> bool {
    (value >> 7) == 1
}
