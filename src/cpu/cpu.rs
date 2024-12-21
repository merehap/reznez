use log::info;

use crate::apu::apu_registers::CycleParity;
use crate::cpu::cpu_mode::CpuModeState;
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

    nmi_status: NmiStatus,
    irq_status: IrqStatus,
    reset_status: ResetStatus,

    oam_dma_port: OamDmaPort,

    current_interrupt_vector: Option<InterruptVector>,

    address_bus: CpuAddress,
    data_bus: u8,
    pending_address_low: u8,
    pending_address_high: u8,
    computed_address: CpuAddress,
    address_carry: i8,
    argument: u8,
}

impl Cpu {
    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn new(memory: &mut CpuMemory, starting_cycle: i64) -> Cpu {
        memory.set_cpu_cycle(starting_cycle);

        Cpu {
            a: 0,
            x: 0,
            y: 0,
            // The RESET sequence will set this properly.
            program_counter: CpuAddress::ZERO,
            status: Status::startup(),

            mode_state: CpuModeState::startup(),

            nmi_status: NmiStatus::Inactive,
            irq_status: IrqStatus::Inactive,
            reset_status: ResetStatus::Active,

            oam_dma_port: memory.ports().oam_dma.clone(),

            // The initial value probably doesn't matter.
            current_interrupt_vector: None,

            address_bus: CpuAddress::ZERO,
            data_bus: 0,
            pending_address_low: 0,
            pending_address_high: 0,
            computed_address: CpuAddress::ZERO,
            address_carry: 0,
            argument: 0,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self) {
        info!(target: "cpuflowcontrol", "System reset will start after current instruction.");
        self.reset_status = ResetStatus::Ready;

        self.mode_state = CpuModeState::startup();

        self.address_carry = 0;
        self.nmi_status = NmiStatus::Inactive;
        self.irq_status = IrqStatus::Inactive;
        self.current_interrupt_vector = None;
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

    pub fn mode_state(&self) -> &CpuModeState {
        &self.mode_state
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

    pub fn step(&mut self, memory: &mut CpuMemory, cycle_parity: CycleParity, irq_pending: bool) -> Option<Step> {
        self.mode_state.clear_new_instruction();
        if self.mode_state.is_jammed() {
            return None;
        }

        let original_program_counter = self.program_counter;

        let mut step = self.mode_state.current_step();
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

        let start_new_instruction = step.has_start_new_instruction();
        if step.is_read() && memory.take_dmc_dma_pending() {
            info!(target: "cpuflowcontrol", "Reading DMC DMA byte at {}.", memory.dmc_dma_address());
            self.mode_state.dmc_dma();
            step = step.with_actions_removed();
        } else if step.is_read() && self.oam_dma_port.take_page().is_some() {
            info!(target: "cpuflowcontrol", "Starting OAM DMA transfer at {}.",
                self.oam_dma_port.current_address());
            self.mode_state.oam_dma();
            step = step.with_actions_removed();
        } else {
            for &action in step.actions() {
                self.execute_cycle_action(memory, action, irq_pending);
            }
        }

        info!(target: "cpustep", "\tPC: {}, Cycle: {}, {:?}", original_program_counter, memory.cpu_cycle(), step);

        if start_new_instruction {
            self.mode_state.set_current_instruction_with_address(
                Instruction::from_code_point(self.data_bus),
                self.address_bus,
            )
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
                if self.mode_state.should_suppress_next_instruction_start() {
                    return;
                }

                if self.reset_status == ResetStatus::Ready {
                    info!(target: "cpuflowcontrol", "Starting system reset");
                    self.reset_status = ResetStatus::Active;
                    self.data_bus = 0x00;
                    self.mode_state.reset();
                } else if self.nmi_status == NmiStatus::Ready {
                    info!(target: "cpuflowcontrol", "Starting NMI");
                    self.nmi_status = NmiStatus::Active;
                    self.data_bus = 0x00;
                    self.mode_state.interrupt_sequence();
                } else if self.irq_status == IrqStatus::Ready && self.nmi_status == NmiStatus::Inactive {
                    info!(target: "cpuflowcontrol", "Starting IRQ");
                    self.irq_status = IrqStatus::Active;
                    self.data_bus = 0x00;
                    self.mode_state.interrupt_sequence();
                } else {
                    self.mode_state.instruction(Instruction::from_code_point(self.data_bus));
                }
            }

            CycleAction::InterpretOpCode => {}
            CycleAction::ExecuteOpCode => {
                let access_mode = self.mode_state.current_instruction().unwrap().access_mode();
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
                    PLA => self.a = self.nz(self.argument),
                    PLP => self.status = Status::from_byte(self.argument),
                    CLC => self.status.carry = false,
                    SEC => self.status.carry = true,
                    CLD => self.status.decimal = false,
                    SED => self.status.decimal = true,
                    CLI => self.status.interrupts_disabled = false,
                    SEI => self.status.interrupts_disabled = true,
                    CLV => self.status.overflow = false,
                    NOP => { /* Do nothing. */ }

                    // Immediate op codes.
                    LDA => self.a = self.nz(self.argument),
                    LDX => self.x = self.nz(self.argument),
                    LDY => self.y = self.nz(self.argument),
                    CMP => self.cmp(self.argument),
                    CPX => self.cpx(self.argument),
                    CPY => self.cpy(self.argument),
                    ORA => self.a = self.nz(self.a | self.argument),
                    AND => self.a = self.nz(self.a & self.argument),
                    EOR => self.a = self.nz(self.a ^ self.argument),
                    ADC => self.a = self.adc(self.argument),
                    SBC => self.a = self.sbc(self.argument),
                    LAX => {
                        self.a = self.argument;
                        self.x = self.argument;
                        self.nz(self.argument);
                    }
                    ANC => {
                        self.a = self.nz(self.a & self.argument);
                        self.status.carry = self.status.negative;
                    }
                    ALR => {
                        self.a = self.nz(self.a & self.argument);
                        Cpu::lsr(&mut self.status, &mut self.a);
                    }
                    ARR => {
                        // TODO: What a mess.
                        let value = (self.a & self.argument) >> 1;
                        self.a = self.nz(value | if self.status.carry {0x80} else {0x00});
                        self.status.carry = self.a & 0x40 != 0;
                        self.status.overflow =
                            (u8::from(self.status.carry) ^ ((self.a >> 5) & 0x01)) != 0;
                    }
                    AXS => {
                        self.status.carry = self.a & self.x >= self.argument;
                        self.x = self.nz((self.a & self.x).wrapping_sub(self.argument));
                    }

                    BIT => {
                        self.status.negative = self.argument & 0b1000_0000 != 0;
                        self.status.overflow = self.argument & 0b0100_0000 != 0;
                        self.status.zero = self.argument & self.a == 0;
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

                    JAM => self.mode_state.jammed(),

                    _ => todo!("{:X?}", self.mode_state.current_instruction().unwrap()),
                }
            }

            CycleAction::IncrementProgramCounter => {
                // FIXME : Rather than suppressing this here, this CycleAction should have been
                // stripped out earlier.
                if !self.mode_state.should_suppress_next_instruction_start() && !self.mode_state.is_interrupt_sequence_active() {
                    self.program_counter.inc();
                }
            }
            // TODO: Make sure this isn't supposed to wrap within the same page.
            CycleAction::IncrementAddress => self.computed_address = self.address_bus.inc(),
            CycleAction::IncrementAddressLow => self.computed_address = self.address_bus.offset_low(1).0,
            CycleAction::IncrementOamDmaAddress => self.oam_dma_port.increment_current_address(),

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
                    } else if let Some(instruction) = self.mode_state.current_instruction() && instruction.op_code() == OpCode::BRK {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to BRK.");
                        Some(InterruptVector::Irq)
                    } else {
                        None
                    };
                // We no longer need to track interrupt statuses now that the vector is set.
                self.nmi_status = NmiStatus::Inactive;
                self.irq_status = IrqStatus::Inactive;
                self.reset_status = ResetStatus::Inactive;
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

            CycleAction::SetDmcSampleBuffer => memory.set_dmc_sample_buffer(self.data_bus),

            CycleAction::CheckNegativeAndZero => {
                self.status.negative = (self.data_bus >> 7) == 1;
                self.status.zero = self.data_bus == 0;
            }

            CycleAction::XOffsetPendingAddressLow => {
                let carry;
                (self.pending_address_low, carry) =
                    self.pending_address_low.overflowing_add(self.x);
                if carry {
                    self.address_carry = 1;
                }
            }
            CycleAction::YOffsetPendingAddressLow => {
                let carry;
                (self.pending_address_low, carry) =
                    self.pending_address_low.overflowing_add(self.y);
                if carry {
                    self.address_carry = 1;
                }
            }
            CycleAction::XOffsetAddress => self.computed_address = self.address_bus.offset_low(self.x).0,
            CycleAction::YOffsetAddress => self.computed_address = self.address_bus.offset_low(self.y).0,
            CycleAction::MaybeInsertOopsStep => {
                if self.address_carry != 0 {
                    self.mode_state.oops();
                }
            }
            CycleAction::MaybeInsertBranchOopsStep => {
                if self.address_carry != 0 {
                    self.mode_state.branch_oops();
                }
            }

            CycleAction::CopyAddressToPC => {
                self.program_counter = self.address_bus;
            }
            CycleAction::AddCarryToAddress => {
                self.computed_address = self.address_bus.offset_high(self.address_carry);
                self.address_carry = 0;
            }
            CycleAction::AddCarryToProgramCounter => {
                if self.address_carry != 0 {
                    self.program_counter = self.program_counter.offset_high(self.address_carry);
                    self.address_carry = 0;
                }
            }
        }
    }

    fn lookup_from_address(&self, memory: &CpuMemory, from: From) -> CpuAddress {
        use self::From::*;
        match from {
            AddressBusTarget => self.address_bus,
            OamDmaAddressTarget => self.oam_dma_port.current_address(),
            DmcDmaAddressTarget => memory.dmc_dma_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget => CpuAddress::from_low_high(self.pending_address_low, self.pending_address_high),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.pending_address_low, 0),
            ComputedTarget => self.computed_address,
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

    fn lookup_to_address(&self, memory: &CpuMemory, to: To) -> CpuAddress {
        use self::To::*;
        match to {
            AddressBusTarget => self.address_bus,
            OamDmaAddressTarget => self.oam_dma_port.current_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget =>
                CpuAddress::from_low_high(self.pending_address_low, self.pending_address_high),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.pending_address_low, 0),
            ComputedTarget => self.computed_address,
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
                if self.mode_state.is_interrupt_sequence_active() {
                    self.status.to_interrupt_byte()
                } else {
                    self.status.to_instruction_byte()
                }
            }
            Argument => self.argument,
            PendingAddressLow => self.pending_address_low,
            PendingAddressHigh => self.pending_address_high,
            OpRegister => match self.mode_state.current_instruction().unwrap().op_code() {
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
                    self.argument,
                    self.data_bus,
                );
            }

            Accumulator => self.a = self.data_bus,
            Status => self.status = status::Status::from_byte(self.data_bus),
            Argument => self.argument = self.data_bus,
            PendingAddressLow => self.pending_address_low = self.data_bus,
            PendingAddressHigh => self.pending_address_high = self.data_bus,
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
        (self.program_counter, self.address_carry) = self.program_counter.offset_with_carry(self.argument as i8);
        self.mode_state.branch_taken();
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
