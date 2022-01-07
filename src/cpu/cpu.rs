use log::{info, log_enabled};
use log::Level::Info;

use crate::cpu::address::Address;
use crate::cpu::instruction::{Instruction, OpCode, Argument};
use crate::cpu::memory::Memory;
use crate::cpu::status::Status;
use crate::cpu::dma_transfer::{DmaTransfer, DmaTransferState};

pub struct Cpu {
    // Accumulator
    a: u8,
    // X Index
    x: u8,
    // Y Index
    y: u8,
    program_counter: Address,
    status: Status,
    pub memory: Memory,

    nmi_pending: bool,
    dma_transfer: DmaTransfer,

    current_instruction_remaining_cycles: u8,
    cycle: u64,
}

impl Cpu {
    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn new(memory: Memory, program_counter_source: ProgramCounterSource) -> Cpu {
        use ProgramCounterSource::*;
        let program_counter = match program_counter_source {
            ResetVector => memory.reset_vector(),
            Override(address) => address,
        };

        info!("Starting execution at PC={}", program_counter);
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            program_counter,
            status: Status::startup(),
            memory,

            nmi_pending: false,
            dma_transfer: DmaTransfer::inactive(),

            current_instruction_remaining_cycles: 0,
            // Unclear why this is the case.
            cycle: 7,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self) {
        self.status.interrupts_disabled = true;
        self.program_counter = self.memory.reset_vector();
        self.cycle = 7;
        // TODO: APU resets?
    }

    pub fn state_string(&self) -> String {
        let nesting = "";
        format!("{:010} PC:{}, A:{:02X}, X:{:02X}, Y:{:02X}, P:{:02X}, S:{:02X}, {} {}",
            self.cycle,
            self.program_counter,
            self.a,
            self.x,
            self.y,
            self.status.to_register_byte(),
            self.stack_pointer(),
            self.status,
            nesting,
        )
    }

    pub fn accumulator(&self) -> u8 {
        self.a
    }

    pub fn x_index(&self) -> u8 {
        self.x
    }

    pub fn y_index(&self) -> u8 {
        self.y
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

    pub fn nmi_pending(&self) -> bool {
        self.nmi_pending
    }

    pub fn schedule_nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn initiate_dma_transfer(&mut self, memory_page: u8, size: u16) {
        self.dma_transfer = DmaTransfer::new(memory_page, size, self.cycle);
    }

    pub fn step(&mut self) -> StepResult {
        self.cycle += 1;
        self.memory.reset_latch();

        // Normal CPU operation is suspended while the DMA transfer completes.
        match self.dma_transfer.step() {
            DmaTransferState::Finished =>
                {/* No transfer in progress. Continue to normal CPU step.*/},
            DmaTransferState::Write(address) =>
                return StepResult::DmaWrite(self.memory.read(address)),
            _ =>
                return StepResult::Nop,
        }

        if self.current_instruction_remaining_cycles != 0 {
            self.current_instruction_remaining_cycles -= 1;
            return StepResult::Nop;
        }

        if self.nmi_pending {
            self.nmi();
            self.nmi_pending = false;
        }

        let instruction = Instruction::from_memory(
            self.program_counter,
            self.x,
            self.y,
            &mut self.memory,
        );

        if log_enabled!(target: "cpu", Info) {
            info!(target: "cpu", "{} | {}", self.state_string(), instruction);
        }

        let cycle_count = self.execute_instruction(instruction);
        self.current_instruction_remaining_cycles = cycle_count - 1;

        StepResult::InstructionComplete(instruction)
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> u8 {
        self.program_counter = self.program_counter.advance(instruction.length());

        let mut cycle_count = instruction.template.cycle_count as u8;
        if instruction.should_add_oops_cycle() {
            info!(target: "cpu", "'Oops' cycle added.");
            cycle_count += 1;
        }

        use OpCode::*;
        use Argument::*;
        match (instruction.template.op_code, instruction.argument) {
            (INX, Imp) => self.x = self.nz(self.x.wrapping_add(1)),
            (INY, Imp) => self.y = self.nz(self.y.wrapping_add(1)),
            (DEX, Imp) => self.x = self.nz(self.x.wrapping_sub(1)),
            (DEY, Imp) => self.y = self.nz(self.y.wrapping_sub(1)),
            (TAX, Imp) => self.x = self.nz(self.a),
            (TAY, Imp) => self.y = self.nz(self.a),
            (TSX, Imp) => self.x = self.nz(self.memory.stack_pointer),
            (TXS, Imp) => self.memory.stack_pointer = self.x,
            (TXA, Imp) => self.a = self.nz(self.x),
            (TYA, Imp) => self.a = self.nz(self.y),
            (PHA, Imp) => self.memory.push_to_stack(self.a),
            (PHP, Imp) => self.memory.push_to_stack(self.status.to_instruction_byte()),
            (PLA, Imp) => {
                self.a = self.memory.pop_from_stack();
                self.nz(self.a);
            },
            (PLP, Imp) => self.status = Status::from_byte(self.memory.pop_from_stack()),
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
                self.memory.push_address_to_stack(self.program_counter);
                self.memory.push_to_stack(self.status.to_instruction_byte());
                self.status.interrupts_disabled = true;
                self.program_counter = self.memory.irq_vector();
            },
            (RTI, Imp) => {
                self.status = Status::from_byte(self.memory.pop_from_stack());
                self.program_counter = self.memory.pop_address_from_stack();
            },
            (RTS, Imp) => self.program_counter = self.memory.pop_address_from_stack().advance(1),

            (STA, Addr(addr, _)) => self.memory.write(addr, self.a),
            (STX, Addr(addr, _)) => self.memory.write(addr, self.x),
            (STY, Addr(addr, _)) => self.memory.write(addr, self.y),
            (DEC, Addr(addr, _)) => {
                let value = self.memory.read(addr).wrapping_sub(1);
                self.memory.write(addr, value);
                self.nz(value);
            },
            (INC, Addr(addr, _)) => {
                let value = self.memory.read(addr).wrapping_add(1);
                self.memory.write(addr, value);
                self.nz(value);
            },
            (BPL, Addr(addr, _)) =>
                if !self.status.negative {cycle_count += self.take_branch(addr);},
            (BMI, Addr(addr, _)) =>
                if self.status.negative {cycle_count += self.take_branch(addr);},
            (BVC, Addr(addr, _)) =>
                if !self.status.overflow {cycle_count += self.take_branch(addr);},
            (BVS, Addr(addr, _)) =>
                if self.status.overflow {cycle_count += self.take_branch(addr);},
            (BCC, Addr(addr, _)) =>
                if !self.status.carry {cycle_count += self.take_branch(addr);},
            (BCS, Addr(addr, _)) =>
                if self.status.carry {cycle_count += self.take_branch(addr);},
            (BNE, Addr(addr, _)) =>
                if !self.status.zero {cycle_count += self.take_branch(addr);},
            (BEQ, Addr(addr, _)) =>
                if self.status.zero {cycle_count += self.take_branch(addr);},
            (JSR, Addr(addr, _)) => {
                // Push the address one previous for some reason.
                self.memory.push_address_to_stack(self.program_counter.offset(-1));
                self.program_counter = addr;
            },
            (JMP, Addr(addr, _)) => self.program_counter = addr,

            (BIT, Addr(_, val)) => {
                self.status.negative = val & 0b1000_0000 != 0;
                self.status.overflow = val & 0b0100_0000 != 0;
                self.status.zero = val & self.a == 0;
            },

            (LDA, Imm(val) | Addr(_, val)) => self.a = self.nz(val),
            (LDX, Imm(val) | Addr(_, val)) => self.x = self.nz(val),
            (LDY, Imm(val) | Addr(_, val)) => self.y = self.nz(val),
            (CMP, Imm(val) | Addr(_, val)) => self.cmp(val),
            (CPX, Imm(val) | Addr(_, val)) => self.cpx(val),
            (CPY, Imm(val) | Addr(_, val)) => self.cpy(val),
            (ORA, Imm(val) | Addr(_, val)) => self.a = self.nz(self.a | val),
            (AND, Imm(val) | Addr(_, val)) => self.a = self.nz(self.a & val),
            (EOR, Imm(val) | Addr(_, val)) => self.a = self.nz(self.a ^ val),
            (ADC, Imm(val) | Addr(_, val)) => self.a = self.adc(val),
            (SBC, Imm(val) | Addr(_, val)) => self.a = self.sbc(val),
            (LAX, Imm(val) | Addr(_, val)) => {
                self.a = val;
                self.x = val;
                self.nz(val);
            },

            (ASL, Imp) => self.a = self.asl(self.a),
            (ASL, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.asl(value);
                self.memory.write(addr, value);
            },
            (ROL, Imp) => self.a = self.rol(self.a),
            (ROL, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.rol(value);
                self.memory.write(addr, value);
            },
            (LSR, Imp) => self.a = self.lsr(self.a),
            (LSR, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.lsr(value);
                self.memory.write(addr, value);
            },
            (ROR, Imp) => self.a = self.ror(self.a),
            (ROR, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.ror(value);
                self.memory.write(addr, value);
            },

            // Undocumented op codes.
            (SLO, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.asl(value);
                self.memory.write(addr, value);
                self.a |= value;
                self.nz(self.a);
            },
            (RLA, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.rol(value);
                self.memory.write(addr, value);
                self.a &= value;
                self.nz(self.a);
            },
            (SRE, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.lsr(value);
                self.memory.write(addr, value);
                self.a ^= value;
                self.nz(self.a);
            },
            (RRA, Addr(addr, _)) => {
                let value = self.memory.read(addr);
                let value = self.ror(value);
                self.memory.write(addr, value);
                self.a = self.adc(value);
                self.nz(self.a);
            },
            (SAX, Addr(addr, _)) => self.memory.write(addr, self.a & self.x),
            (DCP, Addr(addr, _)) => {
                let value = self.memory.read(addr).wrapping_sub(1);
                self.memory.write(addr, value);
                self.cmp(value);
            },
            (ISC, Addr(addr, _)) => {
                let value = self.memory.read(addr).wrapping_add(1);
                self.memory.write(addr, value);
                self.a = self.sbc(value);
            },

            (ANC, Imm(val)) => {
                self.a = self.nz(self.a & val);
                self.status.carry = self.status.negative;
            },
            (ALR, Imm(val)) => {
                self.a = self.nz(self.a & val);
                self.a = self.lsr(self.a);
            },
            (ARR, Imm(val)) => {
                // TODO: What a mess.
                let value = (self.a & val) >> 1;
                self.a = self.nz(value | if self.status.carry {0x80} else {0x00});
                self.status.carry = self.a & 0x40 != 0;
                self.status.overflow =
                    ((if self.status.carry {0x01} else {0x00}) ^
                    ((self.a >> 5) & 0x01)) != 0;
            },
            (XAA, _) => unimplemented!(),
            (AXS, Imm(val)) => {
                self.status.carry = self.a & self.x >= val;
                self.x = self.nz((self.a & self.x).wrapping_sub(val));
            },
            (AHX, _) => unimplemented!(),
            (SHY, Addr(addr, _)) => {
                let (low, high) = addr.to_low_high();
                let value = self.y & high.wrapping_add(1);
                let addr = Address::from_low_high(low, high & self.y);
                self.memory.write(addr, value);
            },
            (SHX, Addr(addr, _)) => {
                let (low, high) = addr.to_low_high();
                let value = self.x & high.wrapping_add(1);
                let addr = Address::from_low_high(low, high & self.x);
                self.memory.write(addr, value);
            },
            (TAS, _) => unimplemented!(),
            (LAS, _) => unimplemented!(),

            (NOP, _) => {},
            (JAM, _) => panic!("JAM instruction encountered!"),
            (op_code, arg) =>
                unreachable!(
                    "Argument type {:?} is invalid for the {:?} opcode.",
                    arg,
                    op_code,
                    ),
        }

        cycle_count as u8
    }

    fn adc(&mut self, value: u8) -> u8 {
        let carry = if self.status.carry {1} else {0};
        let result = (u16::from(self.a)) + (u16::from(value)) + carry;
        self.status.carry = result > 0xFF;
        let result = self.nz(result as u8);
        // If the inputs have the same sign, set overflow if the output doesn't.
        self.status.overflow =
            (is_neg(self.a) == is_neg(value)) &&
            (is_neg(self.a) != is_neg(result));
        result
    }

    fn sbc(&mut self, value: u8) -> u8 {
        self.adc(value ^ 0xFF)
    }

    fn cmp(&mut self, value: u8) {
        self.nz(self.a.wrapping_sub(value));
        self.status.carry = self.a >= value;
    }

    fn cpx(&mut self, value: u8) {
        self.nz(self.x.wrapping_sub(value));
        self.status.carry = self.x >= value;
    }

    fn cpy(&mut self, value: u8) {
        self.nz(self.y.wrapping_sub(value));
        self.status.carry = self.y >= value;
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

    // Set or unset the negative (N) and zero (Z) fields based upon "value".
    fn nz(&mut self, value: u8) -> u8 {
        self.status.negative = is_neg(value);
        self.status.zero = value == 0;
        value
    }

    fn take_branch(&mut self, destination: Address) -> u8 {
        info!(target: "cpu", "Branch taken, cycle added.");
        let mut cycle_count = 1;

        if self.program_counter.page() != destination.page() {
            info!(target: "cpu", "Branch crossed page boundary, 'Oops' cycle added.");
            cycle_count += 1;
        }

        self.program_counter = destination;

        cycle_count
    }

    // TODO: Account for how many cycles an NMI takes.
    fn nmi(&mut self) {
        info!(target: "cpu", "Executing NMI.");
        self.memory.push_address_to_stack(self.program_counter);
        self.memory.push_to_stack(self.status.to_interrupt_byte());
        self.program_counter = self.memory.nmi_vector();
    }
}

fn is_neg(value: u8) -> bool {
    (value >> 7) == 1
}

#[derive(Clone, Copy)]
pub enum ProgramCounterSource {
    ResetVector,
    Override(Address),
}

pub enum StepResult {
    Nop,
    InstructionComplete(Instruction),
    DmaWrite(u8),
}
