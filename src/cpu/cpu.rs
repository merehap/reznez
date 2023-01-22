use log::info;

use crate::cpu::step::*;
use crate::cpu::cycle_action::{CycleAction, From, To, Field};
use crate::cpu::cycle_action_queue::CycleActionQueue;
use crate::cpu::instruction;
use crate::cpu::instruction::{AccessMode, Instruction, OpCode};
use crate::cpu::status;
use crate::cpu::status::Status;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::ports::DmaPort;
use crate::memory::memory::CpuMemory;

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
    next_op_code: Option<(u8, CpuAddress)>,

    cycle_action_queue: CycleActionQueue,
    nmi_status: NmiStatus,

    dma_port: DmaPort,

    cycle: u64,

    current_interrupt_vector: InterruptVector,

    jammed: bool,

    address_bus: CpuAddress,
    data_bus: u8,
    previous_data_bus_value: u8,
    pending_address_low: u8,
    address_carry: i8,

    suppress_program_counter_increment: bool,
    suppress_next_instruction_start: bool,
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
            next_op_code: None,

            cycle_action_queue: CycleActionQueue::new(),
            nmi_status: NmiStatus::Inactive,
            dma_port: memory.ports().dma.clone(),

            // See startup sequence in NES-manual so this isn't hard-coded.
            cycle: 7,

            // The initial value probably doesn't matter.
            current_interrupt_vector: InterruptVector::Reset,

            jammed: false,

            address_bus: CpuAddress::new(0x0000),
            data_bus: 0,
            previous_data_bus_value: 0,
            pending_address_low: 0,
            address_carry: 0,

            suppress_program_counter_increment: false,
            suppress_next_instruction_start: false,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self, memory: &mut CpuMemory) {
        self.status.interrupts_disabled = true;
        self.program_counter = memory.reset_vector();
        self.address_bus = memory.reset_vector();
        self.address_carry = 0;
        self.current_instruction = None;
        self.next_op_code = None;
        self.cycle_action_queue = CycleActionQueue::new();
        self.nmi_status = NmiStatus::Inactive;
        self.cycle = 7;
        self.jammed = false;
        self.suppress_program_counter_increment = false;
        self.suppress_next_instruction_start = false;
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

    pub fn address_bus(&self) -> CpuAddress {
        self.address_bus
    }

    pub fn current_instruction(&self) -> Option<Instruction> {
        self.current_instruction
    }

    pub fn next_op_code(&self) -> Option<(u8, CpuAddress)> {
        self.next_op_code
    }

    pub fn jammed(&self) -> bool {
        self.jammed
    }

    pub fn nmi_pending(&self) -> bool {
        self.nmi_status == NmiStatus::Pending
    }

    pub fn schedule_nmi(&mut self) {
        self.nmi_status = NmiStatus::Pending;
        self.current_interrupt_vector = InterruptVector::Nmi;
    }

    pub fn step(&mut self, memory: &mut CpuMemory) -> Option<Step> {
        if self.jammed {
            return None;
        }

        if self.next_op_code.is_some() {
            self.cycle_action_queue.enqueue_op_code_interpret();
        }

        if self.cycle_action_queue.is_empty() {
            // Get ready to start the next instruction.
            self.cycle_action_queue.enqueue_op_code_read();
        }

        let step = self.cycle_action_queue.dequeue()
            .expect("Ran out of CycleActions!");
        info!(target: "cpustep", "\tPC: {}, Cycle: {}, {:?}", self.program_counter, self.cycle, step);
        self.previous_data_bus_value = self.data_bus;
        match step {
            Step::Read(from, _) => {
                self.address_bus = self.from_address(memory, from);
                self.data_bus = memory.read(self.address_bus).unwrap_or(self.data_bus);
            }
            Step::ReadField(field, from, _) => {
                self.address_bus = self.from_address(memory, from);
                self.data_bus = memory.read(self.address_bus).unwrap_or(self.data_bus);
                self.set_field_value(field);
            }
            Step::Write(to, _) => {
                self.address_bus = self.to_address(memory, to);
                memory.write(self.address_bus, self.data_bus);
            }
            Step::WriteField(field, to, _) => {
                self.address_bus = self.to_address(memory, to);
                self.data_bus = self.field_value(field);
                memory.write(self.address_bus, self.data_bus);
            }
        }

        for &action in step.actions() {
            self.execute_cycle_action(memory, action);
        }

        self.suppress_program_counter_increment = false;

        if self.nmi_status == NmiStatus::Pending {
            info!(target: "cpuoperation", "NMI will start after the current instruction completes.");
            self.nmi_status = NmiStatus::Ready;
        }

        self.cycle += 1;

        Some(step)
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
            IncrementAddressBusLow => { self.address_bus.offset_low(1); }
            IncrementDmaAddress => self.dma_port.increment_current_address(),
            StorePendingAddressLowByte => self.pending_address_low = self.previous_data_bus_value,
            StorePendingAddressLowByteWithXOffset => {
                let carry;
                (self.pending_address_low, carry) =
                    self.previous_data_bus_value.overflowing_add(self.x);
                if carry {
                    self.address_carry = 1;
                }
            }
            StorePendingAddressLowByteWithYOffset => {
                let carry;
                (self.pending_address_low, carry) =
                    self.previous_data_bus_value.overflowing_add(self.y);
                if carry {
                    self.address_carry = 1;
                }
            }

            IncrementStackPointer => memory.stack().increment_stack_pointer(),
            DecrementStackPointer => memory.stack().decrement_stack_pointer(),

            DisableInterrupts => self.status.interrupts_disabled = true,

            CheckNegativeAndZero => {
                self.status.negative = (self.data_bus >> 7) == 1;
                self.status.zero = self.data_bus == 0;
            }

            XOffsetAddressBus => { self.address_bus.offset_low(self.x); }
            YOffsetAddressBus => { self.address_bus.offset_low(self.y); }
            MaybeInsertOopsStep => {
                if self.address_carry != 0 {
                    self.cycle_action_queue.skip_to_front(ADDRESS_BUS_READ_STEP);
                }
            }
            MaybeInsertBranchOopsStep => {
                if self.address_carry != 0 {
                    self.suppress_next_instruction_start = true;
                    self.suppress_program_counter_increment = true;
                    info!(target: "cpuoperation", "\tBranch crossed page boundary, 'Oops' cycle added.");
                    self.cycle_action_queue.skip_to_front(READ_OP_CODE_STEP);
                }
            }
            AddCarryToAddressBus => {
                self.address_bus.offset_high(self.address_carry);
                self.address_carry = 0;
            }
            AddCarryToProgramCounter => {
                if self.address_carry != 0 {
                    self.program_counter.offset_high(self.address_carry);
                    self.address_carry = 0;
                }
            }

            StartNextInstruction => {
                if self.suppress_next_instruction_start {
                    self.suppress_next_instruction_start = false;
                    return;
                }

                if self.dma_port.take_page().is_some() {
                    info!(target: "cpuoperation", "Starting DMA transfer at {}.", self.dma_port.current_address());
                    self.cycle_action_queue.enqueue_dma_transfer(self.cycle);
                    self.suppress_program_counter_increment = true;
                    return;
                }

                match self.nmi_status {
                    NmiStatus::Inactive => {
                        self.next_op_code = Some((self.data_bus, self.address_bus));
                        // If the next instruction is BRK, set the appropriate interrupt vector.
                        if self.data_bus == 0x00 {
                            self.current_interrupt_vector = InterruptVector::Irq;
                        }
                    }
                    NmiStatus::Pending => {
                        self.next_op_code = Some((self.data_bus, self.address_bus));
                    }
                    NmiStatus::Ready => {
                        info!(target: "cpuoperation", "Starting NMI");
                        self.nmi_status = NmiStatus::Active;
                        // NMI has BRK's code point (0x00). TODO: Set the data bus to 0x00?
                        self.next_op_code = Some((0x00, self.address_bus));
                        self.suppress_program_counter_increment = true;
                    }
                    NmiStatus::Active => unreachable!("TODO: Eventually this might set status to Inactive."),
                }
            }

            InterpretOpCode => {
                let (op_code, start_address) = self.next_op_code.take().unwrap();
                if self.nmi_status == NmiStatus::Active {
                    // FIXME: This should happen during the first cycle of the next instruction.
                    self.nmi_status = NmiStatus::Inactive;
                    self.suppress_program_counter_increment = true;
                    self.current_instruction = None;
                    self.cycle_action_queue.enqueue_nmi();
                    return;
                }

                let instruction = instruction::Instruction::from_memory(
                    op_code, start_address, self.x, self.y, memory);
                self.current_instruction = Some(instruction);
                if instruction.template.access_mode == AccessMode::Imp && instruction.template.op_code != OpCode::BRK {
                    self.suppress_program_counter_increment = true;
                }

                self.cycle_action_queue.enqueue_instruction(instruction);
            }
            ExecuteOpCode => {
                let value = self.previous_data_bus_value;
                let access_mode = self.current_instruction.unwrap().template.access_mode;
                let rmw_operand = if access_mode == AccessMode::Imp {
                    &mut self.a
                } else {
                    &mut self.data_bus
                };

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
                    NOP => { /* Do nothing. */ },

                    // Immediate op codes.
                    LDA => self.a = self.nz(value),
                    LDX => self.x = self.nz(value),
                    LDY => self.y = self.nz(value),
                    CMP => self.cmp(value),
                    CPX => self.cpx(value),
                    CPY => self.cpy(value),
                    ORA => self.a = self.nz(self.a | value),
                    AND => self.a = self.nz(self.a & value),
                    EOR => self.a = self.nz(self.a ^ value),
                    ADC => self.a = self.adc(value),
                    SBC => self.a = self.sbc(value),
                    LAX => {
                        self.a = value;
                        self.x = value;
                        self.nz(value);
                    }
                    ANC => {
                        self.a = self.nz(self.a & value);
                        self.status.carry = self.status.negative;
                    }
                    ALR => {
                        self.a = self.nz(self.a & value);
                        Cpu::lsr(&mut self.status, &mut self.a);
                    }
                    ARR => {
                        // TODO: What a mess.
                        let value = (self.a & value) >> 1;
                        self.a = self.nz(value | if self.status.carry {0x80} else {0x00});
                        self.status.carry = self.a & 0x40 != 0;
                        self.status.overflow =
                            ((if self.status.carry {0x01} else {0x00}) ^
                            ((self.a >> 5) & 0x01)) != 0;
                    }
                    AXS => {
                        self.status.carry = self.a & self.x >= value;
                        self.x = self.nz((self.a & self.x).wrapping_sub(value));
                    }

                    BIT => {
                        self.status.negative = value & 0b1000_0000 != 0;
                        self.status.overflow = value & 0b0100_0000 != 0;
                        self.status.zero = value & self.a == 0;
                    }

                    // Write op codes.
                    STA | STX | STY | SAX | SHX | SHY => unreachable!(),


                    // Read-Modify-Write op codes.
                    ASL => Cpu::asl(&mut self.status, rmw_operand),
                    ROL => Cpu::rol(&mut self.status, rmw_operand),
                    LSR => Cpu::lsr(&mut self.status, rmw_operand),
                    ROR => Cpu::ror(&mut self.status, rmw_operand),
                    INC => {
                        self.data_bus = self.data_bus.wrapping_add(1);
                        Cpu::nz_status(&mut self.status, self.data_bus);
                    }
                    DEC => {
                        self.data_bus = self.data_bus.wrapping_sub(1);
                        Cpu::nz_status(&mut self.status, self.data_bus);
                    }
                    SLO => {
                        Cpu::asl(&mut self.status, &mut self.data_bus);
                        self.a |= self.data_bus;
                        self.nz(self.a);
                    }
                    SRE => {
                        Cpu::lsr(&mut self.status, &mut self.data_bus);
                        self.a ^= self.data_bus;
                        self.nz(self.a);
                    }
                    RLA => {
                        Cpu::rol(&mut self.status, &mut self.data_bus);
                        self.a &= self.data_bus;
                        self.nz(self.a);
                    },
                    RRA => {
                        Cpu::ror(&mut self.status, &mut self.data_bus);
                        self.a = self.adc(self.data_bus);
                        self.nz(self.a);
                    }
                    ISC => {
                        self.data_bus = self.data_bus.wrapping_add(1);
                        self.a = self.sbc(self.data_bus);
                    }
                    DCP => {
                        self.data_bus = self.data_bus.wrapping_sub(1);
                        self.cmp(self.data_bus);
                    },

                    // Relative op codes.
                    BPL => if !self.status.negative { self.branch(); }
                    BMI => if self.status.negative { self.branch(); }
                    BVC => if !self.status.overflow { self.branch(); }
                    BVS => if self.status.overflow { self.branch(); }
                    BCC => if !self.status.carry { self.branch(); }
                    BCS => if self.status.carry { self.branch(); }
                    BNE => if !self.status.zero { self.branch(); }
                    BEQ => if self.status.zero { self.branch(); }

                    _ => todo!("{:X?}", self.current_instruction.unwrap()),
                }
            }
        }
    }

    fn from_address(&mut self, memory: &CpuMemory, from: From) -> CpuAddress {
        use self::From::*;
        match from {
            AddressBusTarget => self.address_bus,
            DmaAddressTarget => self.dma_port.current_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget =>
                CpuAddress::from_low_high(self.pending_address_low, self.data_bus),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.data_bus, 0),
            PendingProgramCounterTarget => {
                self.address_bus = CpuAddress::from_low_high(self.pending_address_low, self.data_bus);
                // FIXME: Make this a CycleAction instead so from_address won't have side effects.
                self.program_counter = self.address_bus;
                self.program_counter
            }
            TopOfStack => memory.stack_pointer_address(),
            InterruptVectorLow => match self.current_interrupt_vector {
                InterruptVector::Nmi   => CpuAddress::new(0xFFFA),
                InterruptVector::Reset => CpuAddress::new(0xFFFC),
                InterruptVector::Irq   => CpuAddress::new(0xFFFE),
            }
            InterruptVectorHigh => match self.current_interrupt_vector {
                InterruptVector::Nmi   => CpuAddress::new(0xFFFB),
                InterruptVector::Reset => CpuAddress::new(0xFFFD),
                InterruptVector::Irq   => CpuAddress::new(0xFFFF),
            }
        }
    }

    // A copy of from_address, unfortunately.
    fn to_address(&mut self, memory: &CpuMemory, to: To) -> CpuAddress {
        use self::To::*;
        match to {
            AddressBusTarget => self.address_bus,
            DmaAddressTarget => self.dma_port.current_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget =>
                CpuAddress::from_low_high(self.pending_address_low, self.data_bus),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.data_bus, 0),
            PendingProgramCounterTarget => {
                self.address_bus = CpuAddress::from_low_high(self.pending_address_low, self.data_bus);
                // FIXME: Make this a CycleAction instead so from_address won't have side effects.
                self.program_counter = self.address_bus;
                self.program_counter
            }
            TopOfStack => memory.stack_pointer_address(),
            AddressTarget(address) => address,
        }
    }

    fn field_value(&mut self, field: Field) -> u8 {
        use Field::*;
        match field {
            ProgramCounterLowByte => self.program_counter.low_byte(),
            ProgramCounterHighByte => self.program_counter.high_byte(),
            Accumulator => self.a,
            Status => unreachable!(),
            StatusForInstruction => self.status.to_instruction_byte(),
            StatusForInterrupt => self.status.to_interrupt_byte(),
            OpRegister => match self.current_instruction.unwrap().template.op_code {
                OpCode::STA => self.a,
                OpCode::STX => self.x,
                OpCode::STY => self.y,
                OpCode::SAX => self.a & self.x,
                // FIXME: Calculations should be done as part of an earlier CycleAction.
                OpCode::SHX => {
                    let (low, high) = self.address_bus.to_low_high();
                    self.address_bus = CpuAddress::from_low_high(low, high & self.x);
                    self.x & high.wrapping_add(1)
                }

                // FIXME: Calculations should be done as part of an earlier CycleAction.
                OpCode::SHY => {
                    let (low, high) = self.address_bus.to_low_high();
                    self.address_bus = CpuAddress::from_low_high(low, high & self.y);
                    self.y & high.wrapping_add(1)
                }
                op_code => todo!("{:?}", op_code),
            }
        }
    }

    fn set_field_value(&mut self, field: Field) {
        use Field::*;
        match field {
            ProgramCounterLowByte => unreachable!(),
            ProgramCounterHighByte => {
                self.program_counter = CpuAddress::from_low_high(
                    self.previous_data_bus_value,
                    self.data_bus,
                );
            }

            Accumulator => self.a = self.data_bus,
            Status => self.status = status::Status::from_byte(self.data_bus),
            StatusForInstruction => unreachable!(),
            StatusForInterrupt => unreachable!(),
            OpRegister => panic!(),
        }
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

    fn asl(status: &mut Status, value: &mut u8) {
        status.carry = (*value >> 7) == 1;
        *value <<= 1;
        Cpu::nz_status(status, *value);
    }

    fn rol(status: &mut Status, value: &mut u8) {
        let old_carry = status.carry;
        status.carry = (*value >> 7) == 1;
        *value <<= 1;
        if old_carry {
            *value |= 1;
        }

        Cpu::nz_status(status, *value);
    }

    fn ror(status: &mut Status, value: &mut u8) {
        let old_carry = status.carry;
        status.carry = (*value & 1) == 1;
        *value >>= 1;
        if old_carry {
            *value |= 0b1000_0000;
        }

        Cpu::nz_status(status, *value);
    }

    fn lsr(status: &mut Status, value: &mut u8) {
        status.carry = (*value & 1) == 1;
        *value >>= 1;
        Cpu::nz_status(status, *value);
    }

    // Set or unset the negative (N) and zero (Z) fields based upon "value".
    fn nz(&mut self, value: u8) -> u8 {
        self.status.negative = is_neg(value);
        self.status.zero = value == 0;
        value
    }

    fn nz_status(status: &mut Status, value: u8) {
        status.negative = is_neg(value);
        status.zero = value == 0;
    }

    fn branch(&mut self) {
        self.suppress_program_counter_increment = true;
        self.address_carry = self.program_counter.offset_with_carry(self.previous_data_bus_value as i8);
        self.suppress_next_instruction_start = true;
        info!(target: "cpuoperation", "\tBranch taken, cycle added.");
        self.cycle_action_queue.skip_to_front(BRANCH_TAKEN_STEP);
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

#[derive(PartialEq, Eq)]
enum NmiStatus {
    Inactive,
    Pending,
    Ready,
    Active,
}

#[derive(Debug)]
enum InterruptVector {
    Nmi,
    Reset,
    Irq,
}

#[cfg(test)]
mod tests {
    use crate::cartridge;
    use crate::memory::memory;
    use crate::memory::memory::Memory;
    use crate::util::logger;
    use crate::util::logger::Logger;

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
        logger::init(Logger { log_cpu_operations: true, log_cpu_steps: true }).unwrap();

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
