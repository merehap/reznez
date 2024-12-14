use log::info;

use crate::apu::apu_registers::CycleParity;
use crate::cpu::cpu_mode::{CpuMode, CpuModeState};
use crate::cpu::cycle_action::{CycleAction, From, To, Field};
use crate::cpu::instruction::{Instruction, AccessMode, OpCode};
use crate::cpu::status;
use crate::cpu::status::Status;
use crate::cpu::step::*;
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

    mode_state: CpuModeState,
    // TODO: Remove this. Only test code uses this.
    next_op_code: Option<(u8, CpuAddress)>,

    nmi_status: NmiStatus,
    irq_status: IrqStatus,
    reset_status: ResetStatus,

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

            mode_state: CpuModeState::startup(),
            next_op_code: None,

            nmi_status: NmiStatus::Inactive,
            irq_status: IrqStatus::Inactive,
            reset_status: ResetStatus::Active,
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
    pub fn reset(&mut self) {
        info!(target: "cpuflowcontrol", "System reset will start after current instruction.");
        self.reset_status = ResetStatus::Ready;

        self.mode_state = CpuModeState::startup();

        self.address_carry = 0;
        self.next_op_code = None;
        self.nmi_status = NmiStatus::Inactive;
        self.irq_status = IrqStatus::Inactive;
        self.current_interrupt_vector = None;
        self.jammed = false;
        self.suppress_program_counter_increment = false;
        self.suppress_next_instruction_start = false;
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
        self.mode_state.current_instruction()
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

    pub fn nmi_status(&self) -> NmiStatus {
        self.nmi_status
    }

    pub fn irq_status(&self) -> IrqStatus {
        self.irq_status
    }

    pub fn nmi_pending(&self) -> bool {
        self.nmi_status == NmiStatus::Pending
    }

    pub fn schedule_nmi(&mut self) {
        info!(target: "cpuflowcontrol", "NMI pending in CPU.");
        self.nmi_status = NmiStatus::Pending;
    }

    pub fn interrupt_sequence_active(&self) -> bool {
        self.nmi_status == NmiStatus::Active
            || self.irq_status == IrqStatus::Active
            || self.reset_status == ResetStatus::Active
    }

    pub fn next_instruction_starting(&self) -> bool {
        self.next_op_code.is_some()
            && !self.interrupt_sequence_active()
            && !self.suppress_next_instruction_start
            && !self.jammed
    }

    pub fn next_op_code_and_address(&self) -> Option<(u8, CpuAddress)> {
        if self.interrupt_sequence_active() || self.suppress_next_instruction_start || self.jammed {
            None
        } else {
            self.next_op_code
        }
    }

    pub fn step(&mut self, memory: &mut CpuMemory, cycle_parity: CycleParity, irq_pending: bool) -> Option<Step> {
        if self.jammed {
            return None;
        }

        let step = self.mode_state.current_step();
        info!(target: "cpustep", "\tPC: {}, Cycle: {}, {:?}", self.program_counter, memory.cpu_cycle(), step);
        self.previous_data_bus_value = self.data_bus;
        match step {
            Step::Read(from, _) => {
                self.address_bus = self.lookup_from_address(memory, from);
                self.data_bus = memory.read(self.address_bus).resolve(self.data_bus);
            }
            Step::ReadField(field, from, _) => {
                self.address_bus = self.lookup_from_address(memory, from);
                self.data_bus = memory.read(self.address_bus).resolve(self.data_bus);
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

        let mut dma_address = None;
        if step.is_read() && let Some(address) = memory.take_dmc_dma_pending_address() {
            info!(target: "cpuflowcontrol", "Reading DMC DMA byte at {address}.");
            dma_address = Some(address);
            //self.mode_state.dmc_dma();
        }

        if step.is_read() && self.oam_dma_port.take_page().is_some() {
            // TODO: Strip out unused CycleActions.
            info!(target: "cpuflowcontrol", "Starting OAM DMA transfer at {}.",
                self.oam_dma_port.current_address());
            self.mode_state.oam_dma_pending();
        } else {
            for &action in step.actions() {
                self.execute_cycle_action(memory, action, irq_pending);
            }

            self.suppress_program_counter_increment = false;

            if let Some(dma_address) = dma_address {
                let new_sample_buffer = memory.read(dma_address).unwrap_or(self.data_bus);
                memory.set_dmc_sample_buffer(new_sample_buffer);
            }
        }

        memory.process_end_of_cpu_cycle();
        self.mode_state.step(cycle_parity);
        Some(step)
    }

    fn execute_cycle_action(
        &mut self,
        memory: &mut CpuMemory,
        action: CycleAction,
        irq_pending: bool,
    ) {
        match action {
            CycleAction::StartNextInstruction => {
                if self.suppress_next_instruction_start {
                    self.suppress_next_instruction_start = false;
                    return;
                }

                if self.reset_status == ResetStatus::Ready {
                    info!(target: "cpuflowcontrol", "Starting system reset");
                    self.reset_status = ResetStatus::Active;
                    self.data_bus = 0x00;
                    self.next_op_code = Some((0x00, self.address_bus));
                    self.mode_state.set_next_mode(CpuMode::Reset);
                    self.mode_state.clear_current_instruction();
                } else if self.nmi_status == NmiStatus::Ready {
                    info!(target: "cpuflowcontrol", "Starting NMI");
                    self.nmi_status = NmiStatus::Active;
                    self.data_bus = 0x00;
                    self.mode_state.set_next_mode(CpuMode::InterruptSequence);
                    self.mode_state.clear_current_instruction();
                } else if self.irq_status == IrqStatus::Ready && self.nmi_status == NmiStatus::Inactive {
                    info!(target: "cpuflowcontrol", "Starting IRQ");
                    self.irq_status = IrqStatus::Active;
                    self.data_bus = 0x00;
                    self.mode_state.set_next_mode(CpuMode::InterruptSequence);
                    self.mode_state.clear_current_instruction();
                } else {
                    self.mode_state.instruction(Instruction::from_code_point(self.data_bus));
                }

                self.next_op_code = Some((self.data_bus, self.address_bus));
            }

            CycleAction::InterpretOpCode => {
                if !self.interrupt_sequence_active() {
                    self.next_op_code = None;
                }
            }
            CycleAction::ExecuteOpCode => {
                let value = self.previous_data_bus_value;
                let access_mode = self.current_instruction().unwrap().access_mode();
                let rmw_operand = if access_mode == AccessMode::Imp {
                    &mut self.a
                } else {
                    &mut self.data_bus
                };

                use OpCode::*;
                match self.mode_state.current_instruction().unwrap().op_code() {
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
                    NOP => { /* Do nothing. */ }

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

                    _ => todo!("{:X?}", self.current_instruction().unwrap()),
                }
            }

            CycleAction::IncrementProgramCounter => {
                if !self.suppress_program_counter_increment && !self.interrupt_sequence_active() {
                    self.program_counter.inc();
                }
            }
            // TODO: Make sure this isn't supposed to wrap within the same page.
            CycleAction::IncrementAddressBus => { self.address_bus.inc(); }
            CycleAction::IncrementAddressBusLow => { self.address_bus.offset_low(1); }
            CycleAction::IncrementDmaAddress => self.oam_dma_port.increment_current_address(),
            CycleAction::StorePendingAddressLowByte => self.pending_address_low = self.previous_data_bus_value,
            CycleAction::StorePendingAddressLowByteWithXOffset => {
                let carry;
                (self.pending_address_low, carry) =
                    self.previous_data_bus_value.overflowing_add(self.x);
                if carry {
                    self.address_carry = 1;
                }
            }
            CycleAction::StorePendingAddressLowByteWithYOffset => {
                let carry;
                (self.pending_address_low, carry) =
                    self.previous_data_bus_value.overflowing_add(self.y);
                if carry {
                    self.address_carry = 1;
                }
            }

            CycleAction::IncrementStackPointer => memory.stack().increment_stack_pointer(),
            CycleAction::DecrementStackPointer => memory.stack().decrement_stack_pointer(),

            CycleAction::DisableInterrupts => self.status.interrupts_disabled = true,
            CycleAction::SetInterruptVector => {
                self.current_interrupt_vector =
                    if self.reset_status != ResetStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to RESET.");
                        Some(InterruptVector::Reset)
                    } else if self.nmi_status != NmiStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to NMI.");
                        Some(InterruptVector::Nmi)
                    } else if self.irq_status != IrqStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to IRQ.");
                        Some(InterruptVector::Irq)
                    } else if let Some(instruction) = self.current_instruction() && instruction.op_code() == OpCode::BRK {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to BRK.");
                        Some(InterruptVector::Irq)
                    } else {
                        None
                    };
                // We no longer need to track interrupt statuses now that the vector is set.
                self.nmi_status = NmiStatus::Inactive;
                self.irq_status = IrqStatus::Inactive;
                self.reset_status = ResetStatus::Inactive;
                // HACK: This should only be done after an instruction has completed. Branching
                // currently prevents that in some cases unfortunately.
                self.next_op_code = None;
            }
            CycleAction::ClearInterruptVector => self.current_interrupt_vector = None,
            CycleAction::PollInterrupts => {
                if self.nmi_status == NmiStatus::Pending {
                    info!(target: "cpuflowcontrol", "NMI will start after the current instruction completes.");
                    self.nmi_status = NmiStatus::Ready;
                } else if irq_pending && !self.status.interrupts_disabled {
                    info!(target: "cpuflowcontrol", "IRQ will start after the current instruction completes.");
                    self.irq_status = IrqStatus::Ready;
                }
            }

            CycleAction::CheckNegativeAndZero => {
                self.status.negative = (self.data_bus >> 7) == 1;
                self.status.zero = self.data_bus == 0;
            }

            CycleAction::XOffsetAddressBus => { self.address_bus.offset_low(self.x); }
            CycleAction::YOffsetAddressBus => { self.address_bus.offset_low(self.y); }
            CycleAction::MaybeInsertOopsStep => {
                if self.address_carry != 0 {
                    self.mode_state.oops();
                }
            }
            CycleAction::MaybeInsertBranchOopsStep => {
                if self.address_carry != 0 {
                    self.suppress_next_instruction_start = true;
                    self.suppress_program_counter_increment = true;
                    self.mode_state.set_next_mode(CpuMode::BranchOops);
                }
            }

            CycleAction::CopyAddressToPC => {
                self.program_counter = self.address_bus;
            }
            CycleAction::AddCarryToAddressBus => {
                self.address_bus.offset_high(self.address_carry);
                self.address_carry = 0;
            }
            CycleAction::AddCarryToProgramCounter => {
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
            Status => {
                if self.interrupt_sequence_active() {
                    self.status.to_interrupt_byte()
                } else {
                    self.status.to_instruction_byte()
                }
            }
            OpRegister => match self.current_instruction().unwrap().op_code() {
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
        self.mode_state.set_next_mode(CpuMode::BranchTaken);
    }
}

fn is_neg(value: u8) -> bool {
    (value >> 7) == 1
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum NmiStatus {
    #[default]
    Inactive,
    Pending,
    Ready,
    Active,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum IrqStatus {
    #[default]
    Inactive,
    Ready,
    Active,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ResetStatus {
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
