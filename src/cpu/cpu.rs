use std::marker::ConstParamTy;

use log::{info, log_enabled};
use log::Level::Info;

use crate::apu::apu_registers::CycleParity;
use crate::config::CpuStepFormatting;
use crate::cpu::cpu_mode::{CpuModeState, InterruptType};
use crate::cpu::dmc_dma::DmcDmaAction;
use crate::cpu::step_action::{StepAction, From, To, Field};
use crate::cpu::instruction::{Instruction, AccessMode, OpCode};
use crate::cpu::oam_dma::OamDmaAction;
use crate::cpu::status;
use crate::cpu::status::Status;
use crate::cpu::step::*;
use crate::mapper::Mapper;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::memory::{AddressBusType, Memory, IRQ_VECTOR_HIGH, IRQ_VECTOR_LOW, NMI_VECTOR_HIGH, NMI_VECTOR_LOW, RESET_VECTOR_HIGH, RESET_VECTOR_LOW};
use crate::memory::signal_level::SignalLevel;

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

    current_interrupt_vector: Option<InterruptType>,

    pending_address_low: u8,
    pending_address_high: u8,
    computed_address: CpuAddress,
    address_carry: i8,
    operand: u8,

    step_formatting: CpuStepFormatting,
}

impl Cpu {
    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn new(memory: &mut Memory, starting_cycle: i64, step_formatting: CpuStepFormatting) -> Cpu {
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

            // The initial value probably doesn't matter.
            current_interrupt_vector: None,

            pending_address_low: 0,
            pending_address_high: 0,
            computed_address: CpuAddress::ZERO,
            address_carry: 0,
            operand: 0,

            step_formatting,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self) {
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

    pub fn mode_state(&self) -> &CpuModeState {
        &self.mode_state
    }

    pub fn nmi_status(&self) -> NmiStatus {
        self.nmi_status
    }

    pub fn irq_status(&self) -> IrqStatus {
        self.irq_status
    }

    pub fn reset_status(&self) -> ResetStatus {
        self.reset_status
    }

    pub fn nmi_pending(&self) -> bool {
        self.nmi_status == NmiStatus::Pending
    }

    pub fn step_first_half(&mut self, mapper: &mut dyn Mapper, mem: &mut Memory, cycle_parity: CycleParity) -> Option<Step> {
        if mem.cpu_pinout.reset.current_value() == SignalLevel::Low {
            // The CPU doesn't do anything while the RESET button is held down.
            return None;
        }

        self.mode_state.clear_new_instruction();
        if self.mode_state.is_jammed() {
            return None;
        }

        if self.nmi_status == NmiStatus::Pending {
            self.nmi_status = NmiStatus::Ready;
        } else if self.irq_status == IrqStatus::Pending && !self.status.interrupts_disabled {
            self.irq_status = IrqStatus::Ready;
        }

        let mut step = self.mode_state.current_step();

        mem.dmc_dma.step(step.is_read(), cycle_parity);
        match mem.dmc_dma.latest_action() {
            DmcDmaAction::DoNothing => {}
            DmcDmaAction::Halt | DmcDmaAction::Dummy | DmcDmaAction::Align => step = step.with_actions_removed(),
            DmcDmaAction::Read => step = DMC_READ_STEP,
        }

        let block_oam_dma_memory_access = mem.dmc_dma.latest_action() == DmcDmaAction::Read;
        mem.oam_dma.step(step.is_read(), cycle_parity, block_oam_dma_memory_access);
        step = match mem.oam_dma.latest_action() {
            OamDmaAction::DoNothing => step,
            OamDmaAction::Halt | OamDmaAction::Align => step.with_actions_removed(),
            OamDmaAction::Read => OAM_READ_STEP,
            OamDmaAction::Write => OAM_WRITE_STEP,
        };

        let value;
        match step {
            Step::Read(from, _) => {
                mem.set_cpu_address_bus(AddressBusType::Cpu, self.lookup_from_address(mem, from));
                value = mapper.cpu_read(mem, AddressBusType::Cpu);
            }
            Step::ReadField(field, from, _) => {
                mem.set_cpu_address_bus(AddressBusType::Cpu, self.lookup_from_address(mem, from));
                value = mapper.cpu_read(mem, AddressBusType::Cpu);
                self.set_field_value(field, value);
            }
            Step::Write(to, _) => {
                mem.set_cpu_address_bus(AddressBusType::Cpu, self.lookup_to_address(mem, to));
                value = mem.cpu_pinout.data_bus;
                mapper.cpu_write(mem, AddressBusType::Cpu);
            }
            Step::WriteField(field, to, _) => {
                mem.set_cpu_address_bus(AddressBusType::Cpu, self.lookup_to_address(mem, to));
                mem.cpu_pinout.data_bus = self.field_value(mem, field);
                value = mem.cpu_pinout.data_bus;
                mapper.cpu_write(mem, AddressBusType::Cpu);
            }
            Step::OamRead(from, _) => {
                mem.set_cpu_address_bus(AddressBusType::OamDma, self.lookup_from_address(mem, from));
                value = mapper.cpu_read(mem, AddressBusType::OamDma);
            }
            Step::OamWrite(to, _) => {
                mem.set_cpu_address_bus(AddressBusType::OamDma, self.lookup_to_address(mem, to));
                value = mem.cpu_pinout.data_bus;
                mapper.cpu_write(mem, AddressBusType::OamDma);
            }
            Step::DmcRead(from, _) => {
                mem.set_cpu_address_bus(AddressBusType::DmcDma, self.lookup_from_address(mem, from));
                value = mapper.cpu_read(mem, AddressBusType::DmcDma);
            }
        }

        let formatted_step = if log_enabled!(target: "cpustep", Info) {
            match self.step_formatting {
                CpuStepFormatting::NoData => format!("{step:?}"),
                CpuStepFormatting::Data => step.format_with_read_write_values(mem, value),
            }
        } else {
            String::new()
        };

        let original_program_counter = self.program_counter;
        for &action in step.actions() {
            self.execute_step_action(mapper, mem, action, value);
        }

        let halted = mem.dmc_dma.cpu_should_be_halted() || mem.oam_dma.cpu_should_be_halted();
        if log_enabled!(target: "cpustep", Info) {
            let step_name = if halted { "HALTED".to_string() } else { self.mode_state.step_name() };
            let cpu_cycle = mem.cpu_cycle();
            info!("\t {step_name} PC: {original_program_counter}, Cycle: {cpu_cycle}, {formatted_step}");
        }

        if step.has_start_new_instruction() {
            self.mode_state.set_current_instruction_with_address(
                Instruction::from_code_point(value),
                mem.cpu_pinout.address_bus,
            );
        }

        if !halted {
            self.mode_state.step();
        }

        Some(step)
    }

    pub fn step_second_half(&mut self, mapper: &mut dyn Mapper, mem: &mut Memory) {
        if mem.cpu_pinout.reset.current_value() == SignalLevel::Low {
            // The CPU doesn't do anything while the RESET button is held down.
            return;
        }

        let edge_detected = mem.cpu_pinout.nmi_signal_detector.detect();
        if edge_detected {
            self.nmi_status = NmiStatus::Pending;
        }

        // Keep irq_pending and irq_status in sync
        if mem.cpu_pinout.irq_asserted() {
            if self.irq_status == IrqStatus::Inactive && !self.status.interrupts_disabled {
                self.irq_status = IrqStatus::Pending;
            }
        } else {
            self.irq_status = IrqStatus::Inactive;
        }

        mapper.on_end_of_cpu_cycle(mem);
    }

    fn execute_step_action(&mut self, mapper: &mut dyn Mapper, mem: &mut Memory, action: StepAction, value: u8) {
        match action {
            StepAction::StartNextInstruction => {
                if self.mode_state.should_suppress_next_instruction_start() {
                    return;
                }

                if self.reset_status == ResetStatus::Ready {
                    self.reset_status = ResetStatus::Active;
                    mem.cpu_pinout.data_bus = 0x00;
                    self.mode_state.interrupt_sequence(InterruptType::Reset);
                } else if self.nmi_status == NmiStatus::Active {
                    mem.cpu_pinout.data_bus = 0x00;
                    self.mode_state.interrupt_sequence(InterruptType::Nmi);
                } else if self.irq_status == IrqStatus::Active && self.nmi_status == NmiStatus::Inactive {
                    mem.cpu_pinout.data_bus = 0x00;
                    self.mode_state.interrupt_sequence(InterruptType::Irq);
                } else {
                    self.mode_state.instruction(Instruction::from_code_point(value));
                }
            }

            StepAction::InterpretOpCode => {}
            StepAction::ExecuteOpCode => {
                let instruction = self.mode_state.current_instruction().unwrap();
                use OpCode::*;
                match instruction.op_code() {
                    // Implicit (and Accumulator) op codes.
                    INX => self.x = self.nz(self.x.wrapping_add(1)),
                    INY => self.y = self.nz(self.y.wrapping_add(1)),
                    DEX => self.x = self.nz(self.x.wrapping_sub(1)),
                    DEY => self.y = self.nz(self.y.wrapping_sub(1)),
                    TAX => self.x = self.nz(self.a),
                    TAY => self.y = self.nz(self.a),
                    TSX => self.x = self.nz(mem.stack_pointer()),
                    TXS => *mem.cpu_stack_pointer_mut() = self.x,
                    TXA => self.a = self.nz(self.x),
                    TYA => self.a = self.nz(self.y),
                    PLA => self.a = self.nz(self.operand),
                    PLP => self.status = Status::from_byte(self.operand),
                    CLC => self.status.carry = false,
                    SEC => self.status.carry = true,
                    CLD => self.status.decimal = false,
                    SED => self.status.decimal = true,
                    CLI => self.status.interrupts_disabled = false,
                    SEI => self.status.interrupts_disabled = true,
                    CLV => self.status.overflow = false,
                    NOP => { /* Do nothing. */ }

                    // Immediate op codes.
                    LDA => self.a = self.nz(self.operand),
                    LDX => self.x = self.nz(self.operand),
                    LDY => self.y = self.nz(self.operand),
                    CMP => self.cmp(self.operand),
                    CPX => self.cpx(self.operand),
                    CPY => self.cpy(self.operand),
                    ORA => self.a = self.nz(self.a | self.operand),
                    AND => self.a = self.nz(self.a & self.operand),
                    EOR => self.a = self.nz(self.a ^ self.operand),
                    ADC => self.a = self.adc(self.operand),
                    SBC => self.a = self.sbc(self.operand),
                    LAX => {
                        self.a = self.operand;
                        self.x = self.operand;
                        self.nz(self.operand);
                    }
                    ANC => {
                        self.a = self.nz(self.a & self.operand);
                        self.status.carry = self.status.negative;
                    }
                    ALR => {
                        self.a = self.nz(self.a & self.operand);
                        Cpu::lsr(&mut self.status, &mut self.a);
                    }
                    ARR => {
                        // TODO: What a mess.
                        let value = (self.a & self.operand) >> 1;
                        self.a = self.nz(value | if self.status.carry {0x80} else {0x00});
                        self.status.carry = self.a & 0x40 != 0;
                        self.status.overflow =
                            (u8::from(self.status.carry) ^ ((self.a >> 5) & 0x01)) != 0;
                    }
                    AXS => {
                        self.status.carry = self.a & self.x >= self.operand;
                        self.x = self.nz((self.a & self.x).wrapping_sub(self.operand));
                    }

                    BIT => {
                        self.status.negative = self.operand & 0b1000_0000 != 0;
                        self.status.overflow = self.operand & 0b0100_0000 != 0;
                        self.status.zero = self.operand & self.a == 0;
                    }

                    // Write op codes.
                    STA | STX | STY | SAX | SHX | SHY | TAS | AHX => panic!("ExecuteOpCode must not be implemented for {:?}", instruction.op_code()),

                    // Read-Modify-Write op codes.
                    ASL if instruction.access_mode() == AccessMode::Imp => Cpu::asl(&mut self.status, &mut self.a),
                    ROL if instruction.access_mode() == AccessMode::Imp => Cpu::rol(&mut self.status, &mut self.a),
                    LSR if instruction.access_mode() == AccessMode::Imp => Cpu::lsr(&mut self.status, &mut self.a),
                    ROR if instruction.access_mode() == AccessMode::Imp => Cpu::ror(&mut self.status, &mut self.a),
                    ASL => Cpu::asl(&mut self.status, &mut self.operand),
                    ROL => Cpu::rol(&mut self.status, &mut self.operand),
                    LSR => Cpu::lsr(&mut self.status, &mut self.operand),
                    ROR => Cpu::ror(&mut self.status, &mut self.operand),

                    INC => {
                        self.operand = self.operand.wrapping_add(1);
                        Cpu::nz_status(&mut self.status, self.operand);
                    }
                    DEC => {
                        self.operand = self.operand.wrapping_sub(1);
                        Cpu::nz_status(&mut self.status, self.operand);
                    }
                    SLO => {
                        Cpu::asl(&mut self.status, &mut self.operand);
                        self.a |= self.operand;
                        self.nz(self.a);
                    }
                    SRE => {
                        Cpu::lsr(&mut self.status, &mut self.operand);
                        self.a ^= self.operand;
                        self.nz(self.a);
                    }
                    RLA => {
                        Cpu::rol(&mut self.status, &mut self.operand);
                        self.a &= self.operand;
                        self.nz(self.a);
                    },
                    RRA => {
                        Cpu::ror(&mut self.status, &mut self.operand);
                        self.a = self.adc(self.operand);
                        self.nz(self.a);
                    }
                    ISC => {
                        self.operand = self.operand.wrapping_add(1);
                        self.a = self.sbc(self.operand);
                    }
                    DCP => {
                        self.operand = self.operand.wrapping_sub(1);
                        self.cmp(self.operand);
                    }

                    LAS => {
                        // FIXME: Remove this. It probably won't break any tests.
                        mapper.cpu_read(mem, AddressBusType::Cpu);
                        let value = self.operand & mem.stack_pointer();
                        self.a = value;
                        self.x = value;
                        *mem.cpu_stack_pointer_mut() = value;
                    }
                    XAA => {
                        self.a = self.nz(self.a & self.x & self.operand);
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
                    _ => todo!("{instruction:X?}"),
                }
            }

            StepAction::IncrementPC => {
                // FIXME : Rather than suppressing this here, this StepAction should have been
                // stripped out earlier.
                if !self.mode_state.should_suppress_next_instruction_start() && !self.mode_state.is_interrupt_sequence_active() {
                    self.program_counter.inc();
                }
            }
            // TODO: Make sure this isn't supposed to wrap within the same page.
            StepAction::IncrementAddress => self.computed_address = mem.cpu_pinout.address_bus.inc(),
            StepAction::IncrementAddressLow => self.computed_address = mem.cpu_pinout.address_bus.offset_low(1).0,
            StepAction::IncrementOamDmaAddress => mem.oam_dma.increment_address(),

            StepAction::IncrementStackPointer => mem.cpu_stack().increment_stack_pointer(),
            StepAction::DecrementStackPointer => mem.cpu_stack().decrement_stack_pointer(),

            StepAction::DisableInterrupts => self.status.interrupts_disabled = true,
            StepAction::SetInterruptVector => {
                self.current_interrupt_vector =
                    if self.reset_status != ResetStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to RESET.");
                        Some(InterruptType::Reset)
                    } else if self.nmi_status != NmiStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to NMI.");
                        Some(InterruptType::Nmi)
                    } else if self.irq_status != IrqStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to IRQ.");
                        Some(InterruptType::Irq)
                    } else if let Some(instruction) = self.mode_state.current_instruction() && instruction.op_code() == OpCode::BRK {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to BRK.");
                        Some(InterruptType::Irq)
                    } else {
                        None
                    };
                self.mode_state.interrupt_vector_set(self.current_interrupt_vector);

                // Clear interrupt statuses now that the vector is set.
                self.nmi_status = NmiStatus::Inactive;
                self.irq_status = IrqStatus::Inactive;
                self.reset_status = ResetStatus::Inactive;
            }
            StepAction::ClearInterruptVector => self.current_interrupt_vector = None,
            StepAction::PollInterrupts => {
                if self.nmi_status == NmiStatus::Ready {
                    self.nmi_status = NmiStatus::Active;
                } else if self.irq_status == IrqStatus::Ready && !self.status.interrupts_disabled {
                    self.irq_status = IrqStatus::Active;
                }
            }
            StepAction::MaybePollInterrupts => {
                if self.address_carry != 0 {
                    if self.nmi_status == NmiStatus::Ready {
                        self.nmi_status = NmiStatus::Ready;
                    } else if self.irq_status == IrqStatus::Ready && !self.status.interrupts_disabled {
                        self.irq_status = IrqStatus::Active;
                    }
                }
            }

            StepAction::SetDmcSampleBuffer => mem.set_dmc_sample_buffer(value),

            StepAction::XOffsetPendingAddressLow => {
                let carry;
                (self.pending_address_low, carry) =
                    self.pending_address_low.overflowing_add(self.x);
                if carry {
                    self.address_carry = 1;
                }
            }
            StepAction::YOffsetPendingAddressLow => {
                let carry;
                (self.pending_address_low, carry) =
                    self.pending_address_low.overflowing_add(self.y);
                if carry {
                    self.address_carry = 1;
                }
            }
            StepAction::XOffsetAddress => self.computed_address = mem.cpu_pinout.address_bus.offset_low(self.x).0,
            StepAction::YOffsetAddress => self.computed_address = mem.cpu_pinout.address_bus.offset_low(self.y).0,
            StepAction::MaybeInsertOopsStep => {
                if self.address_carry != 0 {
                    self.mode_state.oops();
                }
            }
            StepAction::MaybeInsertBranchOopsStep => {
                if self.address_carry != 0 {
                    self.mode_state.branch_oops();
                }
            }

            StepAction::CopyAddressToPC => {
                self.program_counter = mem.cpu_pinout.address_bus;
            }
            StepAction::AddCarryToAddress => {
                self.computed_address = mem.cpu_pinout.address_bus.offset_high(self.address_carry);
                self.address_carry = 0;
            }
            StepAction::AddCarryToPC => {
                if self.address_carry != 0 {
                    self.program_counter = self.program_counter.offset_high(self.address_carry);
                    self.address_carry = 0;
                }
            }
        }
    }

    fn lookup_from_address(&self, mem: &Memory, from: From) -> CpuAddress {
        use self::From::*;
        match from {
            OamDmaAddressTarget => mem.oam_dma.address(),
            DmcDmaAddressTarget => mem.dmc_dma_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget => CpuAddress::from_low_high(self.pending_address_low, self.pending_address_high),
            PendingZeroPageTarget => CpuAddress::from_low_high(self.pending_address_low, 0),
            ComputedTarget => self.computed_address,
            TopOfStack => mem.cpu_stack_pointer_address(),
            InterruptVectorLow => match self.current_interrupt_vector.unwrap() {
                InterruptType::Nmi => NMI_VECTOR_LOW,
                InterruptType::Reset => RESET_VECTOR_LOW,
                InterruptType::Irq => IRQ_VECTOR_LOW,
            }
            InterruptVectorHigh => match self.current_interrupt_vector.unwrap() {
                InterruptType::Nmi => NMI_VECTOR_HIGH,
                InterruptType::Reset => RESET_VECTOR_HIGH,
                InterruptType::Irq => IRQ_VECTOR_HIGH,
            }
        }
    }

    fn lookup_to_address(&self, mem: &Memory, to: To) -> CpuAddress {
        use self::To::*;
        match to {
            OamDmaAddressTarget => mem.oam_dma.address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget =>
                CpuAddress::from_low_high(self.pending_address_low, self.pending_address_high),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.pending_address_low, 0),
            ComputedTarget => self.computed_address,
            TopOfStack => mem.cpu_stack_pointer_address(),
            AddressTarget(address) => address,
        }
    }

    fn field_value(&mut self, mem: &mut Memory, field: Field) -> u8 {
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
            Operand => self.operand,
            PendingAddressLow => self.pending_address_low,
            PendingAddressHigh => self.pending_address_high,
            OpRegister => match self.mode_state.current_instruction().unwrap().op_code() {
                OpCode::STA => self.a,
                OpCode::STX => self.x,
                OpCode::STY => self.y,
                OpCode::SAX => self.a & self.x,
                // FIXME: Calculations should be done as part of an earlier StepAction.
                OpCode::SHX => {
                    let (low, high) = mem.cpu_pinout.address_bus.to_low_high();
                    mem.cpu_pinout.address_bus = CpuAddress::from_low_high(low, self.x & high);
                    self.x & high
                }
                // FIXME: Calculations should be done as part of an earlier StepAction.
                OpCode::SHY => {
                    let (low, high) = mem.cpu_pinout.address_bus.to_low_high();
                    mem.cpu_pinout.address_bus = CpuAddress::from_low_high(low, self.y & high);
                    self.y
                }
                // FIXME: Calculations should be done as part of an earlier StepAction.
                OpCode::AHX => {
                    let (low, high) = mem.cpu_pinout.address_bus.to_low_high();
                    // This is using later revision logic.
                    // For early revision logic, use self.a & self.x & self.a
                    mem.cpu_pinout.address_bus = CpuAddress::from_low_high(low, self.x & high);
                    self.a & self.x & high
                }
                OpCode::TAS => {
                    let sp = self.a & self.x;
                    *mem.cpu_stack_pointer_mut() = sp;
                    self.x & mem.cpu_pinout.address_bus.high_byte()
                }
                op_code => todo!("{:?}", op_code),
            }
        }
    }

    fn set_field_value(&mut self, field: Field, value: u8) {
        use Field::*;
        match field {
            ProgramCounterLowByte => unreachable!(),
            ProgramCounterHighByte => self.program_counter = CpuAddress::from_low_high(self.operand, value),

            Accumulator => self.a = value,
            Status => self.status = status::Status::from_byte(value),
            Operand => self.operand = value,
            PendingAddressLow => self.pending_address_low = value,
            PendingAddressHigh => self.pending_address_high = value,
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
        (self.program_counter, self.address_carry) = self.program_counter.offset_with_carry(self.operand as i8);
        self.mode_state.branch_taken();
    }
}

fn is_neg(value: u8) -> bool {
    (value >> 7) == 1
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum NmiStatus {
    #[default]
    Inactive,
    Pending,
    Ready,
    Active,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum IrqStatus {
    #[default]
    Inactive,
    Pending,
    Ready,
    Active,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum ResetStatus {
    #[default]
    Inactive,
    Ready,
    Active,
}
