use log::{info, error};

use crate::cpu::step::*;
use crate::cpu::cycle_action::{CycleAction, From, To};
use crate::cpu::cycle_action_queue::CycleActionQueue;
use crate::cpu::instruction::{AccessMode, Argument, Instruction, OpCode};
use crate::cpu::status::Status;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::ports::DmaPort;
use crate::memory::memory::CpuMemory;

const OAM_DATA_ADDRESS: CpuAddress = CpuAddress::new(0x2004);

pub struct Cpu {
    // Accumulator
    a: u8,
    // X Index
    x: u8,
    // Y Index
    y: u8,
    program_counter: CpuAddress,
    status: Status,

    current_instruction: Option<Instruction>,
    next_instruction: Option<(Instruction, CpuAddress)>,

    cycle_action_queue: CycleActionQueue,
    nmi_pending: bool,

    dma_port: DmaPort,

    cycle: u64,

    jammed: bool,

    address_bus: CpuAddress,
    data_bus: u8,
    previous_data_bus_value: u8,
    pending_address_low: u8,

    suppress_program_counter_increment: bool,
}

impl Cpu {
    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn new(
        memory: &mut CpuMemory,
        program_counter_source: ProgramCounterSource,
    ) -> Cpu {
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

            current_instruction: None,
            next_instruction: None,

            cycle_action_queue: CycleActionQueue::new(),
            nmi_pending: false,
            dma_port: memory.ports().dma.clone(),

            // See startup sequence in NES-manual so this isn't hard-coded.
            cycle: 7,

            jammed: false,

            address_bus: CpuAddress::new(0x0000),
            data_bus: 0,
            previous_data_bus_value: 0,
            pending_address_low: 0,

            suppress_program_counter_increment: false,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self, memory: &mut CpuMemory) {
        self.status.interrupts_disabled = true;
        self.program_counter = memory.reset_vector();
        self.address_bus = memory.reset_vector();
        self.current_instruction = None;
        self.next_instruction = None;
        self.cycle_action_queue = CycleActionQueue::new();
        self.nmi_pending = false;
        self.cycle = 7;
        self.jammed = false;
        // TODO: APU resets?
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

    pub fn program_counter(&self) -> CpuAddress {
        self.program_counter
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn jammed(&self) -> bool {
        self.jammed
    }

    pub fn nmi_pending(&self) -> bool {
        self.nmi_pending
    }

    pub fn schedule_nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn step(&mut self, memory: &mut CpuMemory) -> StepResult {
        if self.jammed {
            return StepResult::Nop;
        }

        if self.dma_port.take_page().is_some() {
            self.cycle_action_queue.enqueue_dma_transfer(self.cycle);
        }

        if self.cycle_action_queue.is_empty() {
            // Get ready to start the next instruction.
            self.cycle_action_queue.enqueue_instruction_fetch();
        }

        if self.nmi_pending {
            info!(target: "cpu", "Enqueueing NMI at cycle {}. {} cycle(s) until start.",
                self.cycle,
                self.cycle_action_queue.len(),
            );
            self.cycle_action_queue.enqueue_nmi();
            self.nmi_pending = false;
        }

        let step = self.cycle_action_queue.dequeue()
            .expect("Ran out of CycleActions!");
        self.copy_data(memory, step.from(), step.to());
        for &action in step.actions() {
            self.execute_cycle_action(memory, action);
        }

        self.suppress_program_counter_increment = false;

        self.cycle += 1;

        if matches!(step.to(), To::Instruction) {
            let (instruction, program_counter) = self.next_instruction.unwrap();
            StepResult::Instruction(instruction, program_counter)
        } else {
            StepResult::Nop
        }
    }

    fn execute_cycle_action(&mut self, memory: &mut CpuMemory, action: CycleAction) {
        use CycleAction::*;
        match action {
            IncrementProgramCounter => {
                if !self.suppress_program_counter_increment {
                    self.program_counter.inc();
                }
            }
            // TODO: Make sure this isn't supposed to wrap within the same page.
            IncrementAddressBus => { self.address_bus.inc(); }
            IncrementAddressBusLow => self.address_bus.inc_low(),
            SetAddressBusToOamDmaStart => self.address_bus = self.dma_port.start_address(),
            StorePendingAddressLowByte => self.pending_address_low = self.previous_data_bus_value,

            IncrementStackPointer => memory.stack().increment_stack_pointer(),
            DecrementStackPointer => memory.stack().decrement_stack_pointer(),

            DisableInterrupts => self.status.interrupts_disabled = true,

            CheckNegativeAndZero => {
                self.status.negative = (self.data_bus >> 7) == 1;
                self.status.zero = self.data_bus == 0;
             }

            InterpretOpCode => {
                let instruction = self.next_instruction.take().unwrap().0;
                self.current_instruction = Some(instruction);
                if instruction.template.access_mode == AccessMode::Imp && instruction.template.op_code != OpCode::BRK {
                    self.suppress_program_counter_increment = true;
                }

                if instruction.template.access_mode == AccessMode::Rel {
                    self.execute_cycle_action(memory, Instruction);
                } else if instruction.template.cycle_count as u8 == 2 {
                    self.execute_cycle_action(memory, ExecuteOpCode);
                } else {
                    self.cycle_action_queue.enqueue_instruction(instruction);
                }
            }
            Instruction => {
                let instr = self.current_instruction.unwrap();
                match self.execute_instruction(memory, instr) {
                    InstructionResult::Success {branch_taken, oops} if branch_taken || oops => {
                        self.cycle_action_queue.skip_to_front(NOP_STEP);
                        if branch_taken && oops {
                            self.cycle_action_queue.skip_to_front(NOP_STEP);
                        }
                    }
                    InstructionResult::Success {..} => {},
                    InstructionResult::Jam => {
                        self.jammed = true;
                        error!("CPU JAMMED! Instruction code point: ${:02X}", instr.template.code_point);
                    }
                }
            }

            ExecuteOpCode => {
                use OpCode::*;
                match self.current_instruction.unwrap().template.op_code {
                    // Implicit (and Accumulator) op codes.
                    INX => self.x = self.nz(self.x.wrapping_add(1)),
                    INY => self.y = self.nz(self.y.wrapping_add(1)),
                    DEX => self.x = self.nz(self.x.wrapping_sub(1)),
                    DEY => self.y = self.nz(self.y.wrapping_sub(1)),
                    TAX => self.x = self.nz(self.a),
                    TAY => self.y = self.nz(self.a),
                    TSX => self.x = self.nz(memory.stack_pointer()),
                    TXS => *memory.stack_pointer_mut() = self.x,
                    TXA => self.a = self.nz(self.x),
                    TYA => self.a = self.nz(self.y),
                    CLC => self.status.carry = false,
                    SEC => self.status.carry = true,
                    CLD => self.status.decimal = false,
                    SED => self.status.decimal = true,
                    CLI => self.status.interrupts_disabled = false,
                    SEI => self.status.interrupts_disabled = true,
                    CLV => self.status.overflow = false,
                    ASL => self.a = self.asl(self.a),
                    ROL => self.a = self.rol(self.a),
                    LSR => self.a = self.lsr(self.a),
                    ROR => self.a = self.ror(self.a),
                    NOP => { /* Do nothing. */ },

                    // Immediate op codes.
                    LDA => self.a = self.nz(self.data_bus),
                    LDX => self.x = self.nz(self.data_bus),
                    LDY => self.y = self.nz(self.data_bus),
                    CMP => self.cmp(self.data_bus),
                    CPX => self.cpx(self.data_bus),
                    CPY => self.cpy(self.data_bus),
                    ORA => self.a = self.nz(self.a | self.data_bus),
                    AND => self.a = self.nz(self.a & self.data_bus),
                    EOR => self.a = self.nz(self.a ^ self.data_bus),
                    ADC => self.a = self.adc(self.data_bus),
                    SBC => self.a = self.sbc(self.data_bus),
                    LAX => {
                        self.a = self.data_bus;
                        self.x = self.data_bus;
                        self.nz(self.data_bus);
                    }
                    ANC => {
                        self.a = self.nz(self.a & self.data_bus);
                        self.status.carry = self.status.negative;
                    }
                    ALR => {
                        self.a = self.nz(self.a & self.data_bus);
                        self.a = self.lsr(self.a);
                    }
                    ARR => {
                        // TODO: What a mess.
                        let value = (self.a & self.data_bus) >> 1;
                        self.a = self.nz(value | if self.status.carry {0x80} else {0x00});
                        self.status.carry = self.a & 0x40 != 0;
                        self.status.overflow =
                            ((if self.status.carry {0x01} else {0x00}) ^
                            ((self.a >> 5) & 0x01)) != 0;
                    }
                    AXS => {
                        self.status.carry = self.a & self.x >= self.data_bus;
                        self.x = self.nz((self.a & self.x).wrapping_sub(self.data_bus));
                    }

                    op_code => todo!("{:?}", op_code),
                }
            }
        }
    }

    // Note that most source/destination combos are invalid.
    // In particular, there is no way to directly copy from one memory location to another.
    fn copy_data(&mut self, memory: &mut CpuMemory, source: From, destination: To) {
        self.previous_data_bus_value = self.data_bus;

        self.data_bus = match source {
            From::DataBus => self.data_bus,
            From::AddressBusTarget => {
                memory.read(self.address_bus)
            },
            From::ProgramCounterTarget => {
                self.address_bus = self.program_counter;
                memory.read(self.address_bus)
            },
            From::PendingAddressTarget => {
                self.address_bus = CpuAddress::from_low_high(self.pending_address_low, self.data_bus);
                memory.read(self.address_bus)
            }
            From::PendingProgramCounterTarget => {
                self.address_bus = CpuAddress::from_low_high(self.pending_address_low, self.data_bus);
                self.program_counter = self.address_bus;
                memory.read(self.address_bus)
            }
            From::TopOfStack => {
                self.address_bus = memory.stack_pointer_address();
                memory.read(self.address_bus)
            }
            From::AddressTarget(address) => {
                self.address_bus = address;
                memory.read(self.address_bus)
            }
            From::ProgramCounterLowByte => self.program_counter.low_byte(),
            From::ProgramCounterHighByte => self.program_counter.high_byte(),
            From::Accumulator => self.a,
            From::StatusForInstruction => self.status.to_instruction_byte(),
            From::StatusForInterrupt => self.status.to_interrupt_byte(),
        };

        match destination {
            To::DataBus => { /* The data bus was already copied to regardless of source. */ },
            To::TopOfStack => {
                self.address_bus = memory.stack_pointer_address();
                memory.write(self.address_bus, self.data_bus);
            }
            // TODO: Rename.
            To::ProgramCounterHighByte => {
                self.program_counter = CpuAddress::from_low_high(
                    self.previous_data_bus_value,
                    self.data_bus,
                );
            }
            To::OamData => {
                // The only write that doesn't use/change the address bus?
                memory.write(OAM_DATA_ADDRESS, self.data_bus);
            }
            To::Accumulator => self.a = self.data_bus,
            To::Status => self.status = Status::from_byte(self.data_bus),

            To::Instruction => {
                let instruction = Instruction::from_memory(
                    self.address_bus, self.x, self.y, memory);
                self.next_instruction = Some((instruction, self.address_bus));
            }
        }
    }

    #[rustfmt::skip]
    fn execute_instruction(
        &mut self,
        memory: &mut CpuMemory,
        instruction: Instruction,
    ) -> InstructionResult {
        use OpCode::*;
        use Argument::*;

        self.program_counter = self.program_counter.advance(instruction.length() - 2);

        let mut branch_taken = false;
        let mut oops = false;
        if instruction.should_add_oops_cycle() {
            info!(target: "cpu", "'Oops' cycle added.");
            oops = true;
        }

        match (instruction.template.op_code, instruction.argument) {
            (INX, Imp) => unreachable!(),
            (INY, Imp) => unreachable!(),
            (DEX, Imp) => unreachable!(),
            (DEY, Imp) => unreachable!(),
            (TAX, Imp) => unreachable!(),
            (TAY, Imp) => unreachable!(),
            (TSX, Imp) => unreachable!(),
            (TXS, Imp) => unreachable!(),
            (TXA, Imp) => unreachable!(),
            (TYA, Imp) => unreachable!(),
            (PHA, Imp) => unreachable!(),
            (PHP, Imp) => unreachable!(),
            (PLA, Imp) => unreachable!(),
            (PLP, Imp) => unreachable!(),
            (CLC, Imp) => unreachable!(),
            (SEC, Imp) => unreachable!(),
            (CLD, Imp) => unreachable!(),
            (SED, Imp) => unreachable!(),
            (CLI, Imp) => unreachable!(),
            (SEI, Imp) => unreachable!(),
            (CLV, Imp) => unreachable!(),
            (BRK, Imp) => unreachable!(),
            (RTI, Imp) => unreachable!(),
            (RTS, Imp) => unreachable!(),

            (STA, Addr(addr)) => memory.write(addr, self.a),
            (STX, Addr(addr)) => memory.write(addr, self.x),
            (STY, Addr(addr)) => memory.write(addr, self.y),
            (DEC, Addr(addr)) => {
                let value = memory.read(addr).wrapping_sub(1);
                memory.write(addr, value);
                self.nz(value);
            }
            (INC, Addr(addr)) => {
                let value = memory.read(addr).wrapping_add(1);
                memory.write(addr, value);
                self.nz(value);
            }
            (BPL, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(!self.status.negative, addr),
            (BMI, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(self.status.negative, addr),
            (BVC, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(!self.status.overflow, addr),
            (BVS, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(self.status.overflow, addr),
            (BCC, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(!self.status.carry, addr),
            (BCS, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(self.status.carry, addr),
            (BNE, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(!self.status.zero, addr),
            (BEQ, Addr(addr)) =>
                (branch_taken, oops) = self.maybe_branch(self.status.zero, addr),
            (JSR, Addr(_addr)) => unreachable!(),
            (JMP, Addr(_addr)) => unreachable!(),

            (BIT, Addr(addr)) => {
                let val = memory.read(addr);
                self.status.negative = val & 0b1000_0000 != 0;
                self.status.overflow = val & 0b0100_0000 != 0;
                self.status.zero = val & self.a == 0;
            }

            (LDA, Imm(_val)) => unreachable!(),
            (LDX, Imm(_val)) => unreachable!(),
            (LDY, Imm(_val)) => unreachable!(),
            (CMP, Imm(_val)) => unreachable!(),
            (CPX, Imm(_val)) => unreachable!(),
            (CPY, Imm(_val)) => unreachable!(),
            (ORA, Imm(_val)) => unreachable!(),
            (AND, Imm(_val)) => unreachable!(),
            (EOR, Imm(_val)) => unreachable!(),
            (ADC, Imm(_val)) => unreachable!(),
            (SBC, Imm(_val)) => unreachable!(),

            (LDA, Addr(addr)) => {let val = memory.read(addr); self.a = self.nz(val)},
            (LDX, Addr(addr)) => {let val = memory.read(addr); self.x = self.nz(val)},
            (LDY, Addr(addr)) => {let val = memory.read(addr); self.y = self.nz(val)},
            (CMP, Addr(addr)) => {let val = memory.read(addr); self.cmp(val)},
            (CPX, Addr(addr)) => {let val = memory.read(addr); self.cpx(val)},
            (CPY, Addr(addr)) => {let val = memory.read(addr); self.cpy(val)},
            (ORA, Addr(addr)) => {let val = memory.read(addr); self.a = self.nz(self.a | val)},
            (AND, Addr(addr)) => {let val = memory.read(addr); self.a = self.nz(self.a & val)},
            (EOR, Addr(addr)) => {let val = memory.read(addr); self.a = self.nz(self.a ^ val)},
            (ADC, Addr(addr)) => {let val = memory.read(addr); self.a = self.adc(val)},
            (SBC, Addr(addr)) => {let val = memory.read(addr); self.a = self.sbc(val)},

            (LAX, Imm(_val)) => unreachable!(),
            (LAX, Addr(addr)) => {
                let val = memory.read(addr);
                self.a = val;
                self.x = val;
                self.nz(val);
            }

            (ASL, Imp) => unreachable!(),
            (ASL, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.asl(value);
                memory.write(addr, value);
            }
            (ROL, Imp) => unreachable!(),
            (ROL, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.rol(value);
                memory.write(addr, value);
            }
            (LSR, Imp) => unreachable!(),
            (LSR, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.lsr(value);
                memory.write(addr, value);
            }
            (ROR, Imp) => unreachable!(),
            (ROR, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.ror(value);
                memory.write(addr, value);
            }

            // Undocumented op codes.
            (SLO, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.asl(value);
                memory.write(addr, value);
                self.a |= value;
                self.nz(self.a);
            }
            (RLA, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.rol(value);
                memory.write(addr, value);
                self.a &= value;
                self.nz(self.a);
            }
            (SRE, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.lsr(value);
                memory.write(addr, value);
                self.a ^= value;
                self.nz(self.a);
            }
            (RRA, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.ror(value);
                memory.write(addr, value);
                self.a = self.adc(value);
                self.nz(self.a);
            }
            (SAX, Addr(addr)) => memory.write(addr, self.a & self.x),
            (DCP, Addr(addr)) => {
                let value = memory.read(addr).wrapping_sub(1);
                memory.write(addr, value);
                self.cmp(value);
            }
            (ISC, Addr(addr)) => {
                let value = memory.read(addr).wrapping_add(1);
                memory.write(addr, value);
                self.a = self.sbc(value);
            }

            (ANC, Imm(_val)) => unreachable!(),
            (ALR, Imm(_val)) => unreachable!(),
            (ARR, Imm(_val)) => unreachable!(),
            (XAA, _) => unimplemented!(),
            (AXS, Imm(_val)) => unreachable!(),
            (AHX, _) => unimplemented!(),
            (SHY, Addr(addr)) => {
                let (low, high) = addr.to_low_high();
                let value = self.y & high.wrapping_add(1);
                let addr = CpuAddress::from_low_high(low, high & self.y);
                memory.write(addr, value);
            }
            (SHX, Addr(addr)) => {
                let (low, high) = addr.to_low_high();
                let value = self.x & high.wrapping_add(1);
                let addr = CpuAddress::from_low_high(low, high & self.x);
                memory.write(addr, value);
            }
            (TAS, _) => unimplemented!(),
            (LAS, _) => unimplemented!(),

            (NOP, _) => {}
            (JAM, _) => return InstructionResult::Jam,
            (op_code, arg) =>
                unreachable!(
                    "Argument type {:?} is invalid for the {:?} opcode.",
                    arg,
                    op_code,
                    ),
        }

        InstructionResult::Success { branch_taken, oops }
    }

    fn adc(&mut self, value: u8) -> u8 {
        let carry = if self.status.carry { 1 } else { 0 };
        let result = (u16::from(self.a)) + (u16::from(value)) + carry;
        self.status.carry = result > 0xFF;
        let result = self.nz(result as u8);
        // If the inputs have the same sign, set overflow if the output doesn't.
        self.status.overflow =
            (is_neg(self.a) == is_neg(value)) && (is_neg(self.a) != is_neg(result));
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

    fn maybe_branch(
        &mut self,
        take_branch: bool,
        destination: CpuAddress,
    ) -> (bool, bool) {
        if !take_branch {
            return (false, false);
        }

        self.suppress_program_counter_increment = true;

        info!(target: "cpu", "Branch taken, cycle added.");

        let oops = self.program_counter.offset(1).page() != destination.page();
        if oops {
            info!(target: "cpu", "Branch crossed page boundary, 'Oops' cycle added.");
        }

        self.program_counter = destination;

        (take_branch, oops)
    }
}

fn is_neg(value: u8) -> bool {
    (value >> 7) == 1
}

#[derive(Clone, Copy)]
pub enum ProgramCounterSource {
    ResetVector,
    Override(CpuAddress),
}

#[derive(Clone, Copy)]
pub enum StepResult {
    Nop,
    Instruction(Instruction, CpuAddress),
}

impl StepResult {
    pub fn to_instruction_and_program_counter(self) -> Option<(Instruction, CpuAddress)> {
        if let StepResult::Instruction(instruction, program_counter) = self {
            Some((instruction, program_counter))
        } else {
            None
        }
    }
}

enum InstructionResult {
    Jam,
    Success {
        branch_taken: bool,
        oops: bool,
    },
}

#[cfg(test)]
mod tests {
    use crate::cartridge;
    use crate::memory::memory;
    use crate::memory::memory::Memory;

    use super::*;

    /*
    #[test]
    fn nmi_during_instruction() {
        let nmi_vector = CpuAddress::new(0xC000);
        let reset_vector = CpuAddress::new(0x8000);
        let mut mem = memory_with_nop_cartridge(nmi_vector, reset_vector);
        let mut cpu = Cpu::new(
            &mut mem.as_cpu_memory(),
            ProgramCounterSource::ResetVector,
        );

        // No instruction loaded yet.
        assert_eq!(0xFD, mem.stack_pointer());

        // Execute first cycle of the first instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector, cpu.program_counter());

        cpu.schedule_nmi();
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector, cpu.program_counter());

        // Execute final cycle of the first instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        // Execute first cycle of the second instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        // Execute final cycle of the second instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(2), cpu.program_counter());


        // Execute the seven cycles of the NMI subroutine.
        for _ in 0..6 {
            cpu.step(&mut mem.as_cpu_memory());
            assert_eq!(reset_vector.advance(2), cpu.program_counter());
        }

        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFA, mem.stack_pointer());
        assert_eq!(nmi_vector, cpu.program_counter());
    }
    */

    #[test]
    fn nmi_after_instruction() {
        let nmi_vector = CpuAddress::new(0xC000);
        let reset_vector = CpuAddress::new(0x8000);
        let mut mem = memory_with_nop_cartridge(nmi_vector, reset_vector);
        let mut cpu =
            Cpu::new(&mut mem.as_cpu_memory(), ProgramCounterSource::ResetVector);

        // No instruction loaded yet.
        assert_eq!(0xFD, mem.stack_pointer());

        // Execute first cycle of the first instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        // Execute final cycle of the first instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        cpu.schedule_nmi();
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        // Execute first cycle of the second instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(2), cpu.program_counter());

        // Execute final cycle of the second instruction.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(2), cpu.program_counter());

        // Execute the seven cycles of the NMI subroutine.
        for _ in 0..6 {
            cpu.step(&mut mem.as_cpu_memory());
        }

        assert_eq!(reset_vector.advance(2), cpu.program_counter());

        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFA, mem.stack_pointer());
        assert_eq!(nmi_vector, cpu.program_counter());
    }

    #[test]
    fn nmi_scheduled_before_branching() {}

    #[test]
    fn nmi_scheduled_before_oops() {}

    #[test]
    fn nmi_scheduled_before_branching_oops() {}

    fn memory_with_nop_cartridge(
        nmi_vector: CpuAddress,
        reset_vector: CpuAddress,
    ) -> Memory {
        let irq_vector = CpuAddress::new(0xF000);
        // Providing no data results in a program filled with NOPs (0xEA).
        let cartridge = cartridge::test_data::cartridge_with_prg_rom(
            [Vec::new(), Vec::new()],
            nmi_vector,
            reset_vector,
            irq_vector,
        );

        memory::test_data::memory_with_cartridge(&cartridge)
    }
}
