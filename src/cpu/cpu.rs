use log::info;

use crate::cpu::step::*;
use crate::cpu::cycle_action::{CycleAction, From, To, Field};
use crate::cpu::step_queue::StepQueue;
use crate::cpu::instruction::{Instruction, AccessMode, OpCode};
use crate::cpu::status;
use crate::cpu::status::Status;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::ports::OamDmaPort;
use crate::memory::memory::{CpuMemory,
    IRQ_VECTOR_LOW, IRQ_VECTOR_HIGH,
    RESET_VECTOR_LOW, RESET_VECTOR_HIGH,
    NMI_VECTOR_LOW, NMI_VECTOR_HIGH,
};

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
    // TODO: Remove this. Only test code uses this.
    next_op_code: Option<(u8, CpuAddress)>,

    step_queue: StepQueue,
    nmi_status: NmiStatus,
    irq_status: IrqStatus,
    reset_pending: bool,

    oam_dma_port: OamDmaPort,

    current_interrupt_vector: Option<InterruptVector>,

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
    pub fn new(memory: &mut CpuMemory, starting_cycle: i64) -> Cpu {
        memory.set_cpu_cycle(starting_cycle);

        Cpu {
            a: 0,
            x: 0,
            y: 0,
            // The Start sequence will set this properly.
            program_counter: CpuAddress::new(0x0000),
            status: Status::startup(),

            current_instruction: None,
            next_op_code: None,

            step_queue: StepQueue::new(),
            nmi_status: NmiStatus::Inactive,
            irq_status: IrqStatus::Inactive,
            reset_pending: true,
            oam_dma_port: memory.ports().oam_dma.clone(),

            // The initial value probably doesn't matter.
            current_interrupt_vector: None,

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
        self.address_bus = memory.reset_vector();
        self.address_carry = 0;
        self.current_instruction = None;
        self.next_op_code = None;
        self.step_queue = StepQueue::new();
        self.nmi_status = NmiStatus::Inactive;
        self.irq_status = IrqStatus::Inactive;
        self.reset_pending = true;
        memory.set_cpu_cycle(6);
        self.current_interrupt_vector = None;
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

    pub fn oam_dma_pending(&self) -> bool {
        self.oam_dma_port.page_present()
    }

    pub fn nmi_pending(&self) -> bool {
        self.nmi_status == NmiStatus::Pending
    }

    pub fn schedule_nmi(&mut self) {
        info!(target: "cpuflowcontrol", "NMI pending in CPU.");
        self.nmi_status = NmiStatus::Pending;
    }

    pub fn interrupt_active(&self) -> bool {
        self.nmi_status == NmiStatus::Active || self.irq_status == IrqStatus::Active
    }

    pub fn next_instruction_starting(&self) -> bool {
        self.next_op_code.is_some()
            && !self.interrupt_active()
            && !self.suppress_next_instruction_start
            && !self.jammed
    }

    pub fn address_for_next_step(&self, memory: &CpuMemory) -> CpuAddress {
        self.step_queue.peek()
            .map_or(
                self.program_counter,
                |step| {
                    match step {
                        Step::Read(from, _) | Step::ReadField(_, from, _) =>
                            self.lookup_from_address(memory, from),
                        Step::Write(to, _) | Step::WriteField(_, to, _) =>
                            self.lookup_to_address(memory, to),
                    }
                })
    }

    pub fn step(&mut self, memory: &mut CpuMemory, irq_pending: bool) -> Option<Step> {
        if self.jammed {
            return None;
        }

        {
            let cycle = memory.cpu_cycle();
            memory.set_cpu_cycle(cycle + 1);
        }

        if self.step_queue.is_empty() {
            // Get ready to start the next instruction.
            self.step_queue.enqueue_op_code_read();
        }

        let step = self.step_queue.dequeue()
            .expect("Ran out of CycleActions!");
        info!(target: "cpustep", "\tPC: {}, Cycle: {}, {:?}", self.program_counter, memory.cpu_cycle(), step);
        self.previous_data_bus_value = self.data_bus;
        match step {
            Step::Read(from, _) => {
                self.address_bus = self.lookup_from_address(memory, from);
                self.data_bus = memory.read(self.address_bus).unwrap_or(self.data_bus);
            }
            Step::ReadField(field, from, _) => {
                self.address_bus = self.lookup_from_address(memory, from);
                self.data_bus = memory.read(self.address_bus).unwrap_or(self.data_bus);
                self.set_field_value(field);
            }
            Step::Write(to, _) => {
                self.address_bus = self.lookup_to_address(memory, to);
                memory.write(self.address_bus, self.data_bus);
            }
            Step::WriteField(field, to, _) => {
                self.address_bus = self.lookup_to_address(memory, to);
                self.data_bus = self.field_value(field);
                memory.write(self.address_bus, self.data_bus);
            }
        }

        for &action in step.actions() {
            self.execute_cycle_action(memory, action, irq_pending);
        }

        self.suppress_program_counter_increment = false;

        if step.has_start_new_instruction() && !self.suppress_next_instruction_start {
            if self.interrupt_active() {
                self.current_instruction = None;
            } else if let Some((next_op_code, _)) = self.next_op_code {
                self.current_instruction = Some(Instruction::from_code_point(next_op_code));
            }
        }

        memory.process_end_of_cpu_cycle();

        Some(step)
    }

    fn execute_cycle_action(&mut self, memory: &mut CpuMemory, action: CycleAction, irq_pending: bool) {
        use CycleAction::*;
        match action {
            StartNextInstruction => {
                if self.suppress_next_instruction_start {
                    self.suppress_next_instruction_start = false;
                    return;
                }

                if let Some(address) = memory.take_dmc_dma_pending_address() {
                    info!(target: "cpuflowcontrol", "Reading DMC DMA byte at {} on next cycle.", address);
                    let new_sample_buffer = memory.read(address).unwrap_or(self.data_bus);
                    memory.set_dmc_sample_buffer(new_sample_buffer);
                    self.suppress_program_counter_increment = true;
                    return;
                }

                if self.oam_dma_port.take_page().is_some() {
                    info!(target: "cpuflowcontrol", "Starting OAM DMA transfer at {}.",
                        self.oam_dma_port.current_address());
                    self.step_queue.enqueue_oam_dma_transfer(memory.cpu_cycle());
                    self.suppress_program_counter_increment = true;
                    return;
                }

                match self.nmi_status {
                    NmiStatus::Inactive if self.irq_status == IrqStatus::Ready => {
                        info!(target: "cpuflowcontrol", "Starting IRQ");
                        self.irq_status = IrqStatus::Active;
                        // IRQ has BRK's code point (0x00). TODO: Set the data bus to 0x00?
                        self.next_op_code = Some((0x00, self.address_bus));
                        self.suppress_program_counter_increment = true;
                        self.step_queue.enqueue_irq();
                    }
                    NmiStatus::Inactive | NmiStatus::Pending => {
                        self.next_op_code = Some((self.data_bus, self.address_bus));
                        let instruction = Instruction::from_code_point(self.data_bus);
                        self.step_queue.enqueue_instruction(instruction.code_point());
                    }
                    NmiStatus::Ready => {
                        info!(target: "cpuflowcontrol", "Starting NMI");
                        self.nmi_status = NmiStatus::Active;
                        // NMI has BRK's code point (0x00). TODO: Set the data bus to 0x00?
                        self.next_op_code = Some((0x00, self.address_bus));
                        self.suppress_program_counter_increment = true;
                        self.step_queue.enqueue_nmi();
                    }
                    NmiStatus::Active => unreachable!(),
                }
            }

            InterpretOpCode => {
                self.next_op_code = None;
            }
            ExecuteOpCode => {
                let value = self.previous_data_bus_value;
                let access_mode = self.current_instruction.unwrap().access_mode();
                let rmw_operand = if access_mode == AccessMode::Imp {
                    &mut self.a
                } else {
                    &mut self.data_bus
                };

                use OpCode::*;
                match self.current_instruction.unwrap().op_code() {
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
                    PLA => self.a = self.nz(value),
                    PLP => self.status = Status::from_byte(value),
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
                            (u8::from(self.status.carry) ^ ((self.a >> 5) & 0x01)) != 0;
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
                    }

                    TAS => {
                        let sp = self.x & self.a;
                        *memory.stack_pointer_mut() = sp;
                        // TODO: Implement this write properly.
                        //let value = (u16::from(sp) & ((self.address_bus.to_raw() >> 8) + 1)) as u8;
                        //memory.write(self.address_bus, value);
                    }
                    LAS => {
                        let value = memory.read(self.address_bus).unwrap_or(self.data_bus)
                            & memory.stack_pointer();
                        self.a = value;
                        self.x = value;
                        *memory.stack_pointer_mut() = value;
                    }

                    AHX => {
                        // TODO: Implement properly.
                        /*
                        let high_inc = self.address_bus.high_byte().wrapping_add(1);
                        let value = self.a & self.x & high_inc;
                        // TODO: Consolidate this write into the standardized location.
                        memory.write(self.address_bus, value);
                        */
                    }

                    XAA => {
                        // TODO: Implement properly.
                        //self.a = self.nz(self.x & value);
                    }

                    // Relative op codes.
                    BPL => if !self.status.negative { self.branch(); }
                    BMI => if self.status.negative { self.branch(); }
                    BVC => if !self.status.overflow { self.branch(); }
                    BVS => if self.status.overflow { self.branch(); }
                    BCC => if !self.status.carry { self.branch(); }
                    BCS => if self.status.carry { self.branch(); }
                    BNE => if !self.status.zero { self.branch(); }
                    BEQ => if self.status.zero { self.branch(); }

                    JAM => self.jammed = true,

                    _ => todo!("{:X?}", self.current_instruction.unwrap()),
                }
            }

            IncrementProgramCounter => {
                if !self.suppress_program_counter_increment {
                    self.program_counter.inc();
                }
            }
            // TODO: Make sure this isn't supposed to wrap within the same page.
            IncrementAddressBus => { self.address_bus.inc(); }
            IncrementAddressBusLow => { self.address_bus.offset_low(1); }
            IncrementDmaAddress => self.oam_dma_port.increment_current_address(),
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
            SetInterruptVector => {
                self.current_interrupt_vector =
                    if self.reset_pending {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to RESET.");
                        Some(InterruptVector::Reset)
                    } else if self.nmi_status != NmiStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to NMI.");
                        Some(InterruptVector::Nmi)
                    } else if self.irq_status != IrqStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to IRQ.");
                        Some(InterruptVector::Irq)
                    } else if let Some(instruction) = self.current_instruction && instruction.op_code() == OpCode::BRK {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to BRK.");
                        Some(InterruptVector::Irq)
                    } else {
                        None
                    };
                // We no longer need to track interrupt statuses now that the vector is set.
                self.nmi_status = NmiStatus::Inactive;
                self.irq_status = IrqStatus::Inactive;
                self.reset_pending = false;
                // HACK: This should only be done after an instruction has completed. Branching
                // currently prevents that in some cases unfortunately.
                self.next_op_code = None;
            }
            ClearInterruptVector => self.current_interrupt_vector = None,
            PollInterrupts => {
                if self.nmi_status == NmiStatus::Pending {
                    info!(target: "cpuflowcontrol", "NMI will start after the current instruction completes.");
                    self.nmi_status = NmiStatus::Ready;
                } else if irq_pending && !self.status.interrupts_disabled {
                    info!(target: "cpuflowcontrol", "IRQ will start after the current instruction completes.");
                    self.irq_status = IrqStatus::Ready;
                }
            }

            CheckNegativeAndZero => {
                self.status.negative = (self.data_bus >> 7) == 1;
                self.status.zero = self.data_bus == 0;
            }

            XOffsetAddressBus => { self.address_bus.offset_low(self.x); }
            YOffsetAddressBus => { self.address_bus.offset_low(self.y); }
            MaybeInsertOopsStep => {
                if self.address_carry != 0 {
                    self.step_queue.skip_to_front(ADDRESS_BUS_READ_STEP);
                }
            }
            MaybeInsertBranchOopsStep => {
                if self.address_carry != 0 {
                    self.suppress_next_instruction_start = true;
                    self.suppress_program_counter_increment = true;
                    self.step_queue.skip_to_front(READ_OP_CODE_STEP);
                }
            }

            CopyAddressToPC => {
                self.program_counter = self.address_bus;
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
        }
    }

    fn lookup_from_address(&self, memory: &CpuMemory, from: From) -> CpuAddress {
        use self::From::*;
        match from {
            AddressBusTarget => self.address_bus,
            DmaAddressTarget => self.oam_dma_port.current_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget =>
                CpuAddress::from_low_high(self.pending_address_low, self.data_bus),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.data_bus, 0),
            TopOfStack => memory.stack_pointer_address(),
            InterruptVectorLow => match self.current_interrupt_vector.unwrap() {
                InterruptVector::Nmi => NMI_VECTOR_LOW,
                InterruptVector::Reset => RESET_VECTOR_LOW,
                InterruptVector::Irq => IRQ_VECTOR_LOW,
            }
            InterruptVectorHigh => match self.current_interrupt_vector.unwrap() {
                InterruptVector::Nmi => NMI_VECTOR_HIGH,
                InterruptVector::Reset => RESET_VECTOR_HIGH,
                InterruptVector::Irq => IRQ_VECTOR_HIGH,
            }
        }
    }

    // A copy of from_address, unfortunately.
    fn lookup_to_address(&self, memory: &CpuMemory, to: To) -> CpuAddress {
        use self::To::*;
        match to {
            AddressBusTarget => self.address_bus,
            DmaAddressTarget => self.oam_dma_port.current_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget =>
                CpuAddress::from_low_high(self.pending_address_low, self.data_bus),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.data_bus, 0),
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
            OpRegister => match self.current_instruction.unwrap().op_code() {
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
        let carry = u16::from(self.status.carry);
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
        self.step_queue.skip_to_front(BRANCH_TAKEN_STEP);
    }
}

fn is_neg(value: u8) -> bool {
    (value >> 7) == 1
}

#[derive(PartialEq, Eq, Debug)]
enum NmiStatus {
    Inactive,
    Pending,
    Ready,
    Active,
}

#[derive(PartialEq, Eq, Debug)]
enum IrqStatus {
    Inactive,
    Ready,
    Active,
}

#[derive(Clone, Copy, Debug)]
enum InterruptVector {
    Nmi,
    Reset,
    Irq,
}

#[cfg(test)]
mod tests {
    use crate::cartridge::cartridge;
    use crate::memory::memory;
    use crate::memory::memory::Memory;
    use crate::logging::logger;
    use crate::logging::logger::Logger;

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
        logger::init(Logger {
            log_cpu_instructions: true,
            log_cpu_flow_control: true,
            log_cpu_steps: true,
            log_ppu_stages: false,
            log_ppu_flags: false,
            log_ppu_steps: false,
            log_apu_cycles: false,
            log_apu_events: false,
            log_oam_addr: false,
        }).unwrap();

        let nmi_vector = CpuAddress::new(0xC000);
        let reset_vector = CpuAddress::new(0x8000);
        let mut mem = memory_with_nop_cartridge(nmi_vector, reset_vector);
        let mut cpu = Cpu::new(&mut mem.as_cpu_memory(), 0);

        // Skip through the start sequence.
        for _ in 0..7 {
            cpu.step(&mut mem.as_cpu_memory(), false);
        }

        // No instruction loaded yet.
        assert_eq!(0xFD, mem.stack_pointer());

        // Execute first cycle of the first instruction.
        cpu.step(&mut mem.as_cpu_memory(), false);
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        // Execute final cycle of the first instruction.
        cpu.step(&mut mem.as_cpu_memory(), false);
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        cpu.schedule_nmi();
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        // Execute first cycle of the second instruction.
        cpu.step(&mut mem.as_cpu_memory(), false);
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(2), cpu.program_counter());

        // Execute final cycle of the second instruction.
        cpu.step(&mut mem.as_cpu_memory(), false);
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(2), cpu.program_counter());

        // Execute the seven cycles of the NMI subroutine.
        for _ in 0..6 {
            cpu.step(&mut mem.as_cpu_memory(), false);
        }

        assert_eq!(reset_vector.advance(2), cpu.program_counter());

        cpu.step(&mut mem.as_cpu_memory(), false);
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
            vec![0xEA; 32 * crate::util::unit::KIBIBYTE],
            nmi_vector,
            reset_vector,
            irq_vector,
        );

        memory::test_data::memory_with_cartridge(&cartridge)
    }
}
