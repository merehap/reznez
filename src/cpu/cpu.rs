use std::marker::ConstParamTy;

use log::{info, log_enabled};
use log::Level::Info;

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
use crate::bus::{AddressBusType, Bus, IRQ_VECTOR_HIGH, IRQ_VECTOR_LOW, NMI_VECTOR_HIGH, NMI_VECTOR_LOW, RESET_VECTOR_HIGH, RESET_VECTOR_LOW};
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::memory::signal_level::SignalLevel;

pub struct Cpu {
    // Accumulator
    a: u8,
    // X Index
    x: u8,
    // Y Index
    y: u8,
    stack_pointer: u8,
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
    pub fn new(step_formatting: CpuStepFormatting) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            // The RESET sequence will set this properly.
            program_counter: CpuAddress::ZERO,
            stack_pointer: 0x00,
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

    pub fn stack_pointer(&self) -> u8 {
        self.stack_pointer
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

    pub fn step_first_half(bus: &mut Bus, mapper: &mut dyn Mapper) -> Option<Step> {
        if bus.cpu_pinout.reset.current_value() == SignalLevel::Low {
            // The CPU doesn't do anything while the RESET button is held down.
            return None;
        }

        bus.cpu.mode_state.clear_new_instruction();
        if bus.cpu.mode_state.is_jammed() {
            return None;
        }

        if bus.cpu.nmi_status == NmiStatus::Pending {
            bus.cpu.nmi_status = NmiStatus::Ready;
        } else if bus.cpu.irq_status == IrqStatus::Pending && !bus.cpu.status.interrupts_disabled {
            bus.cpu.irq_status = IrqStatus::Ready;
        }

        let mut step = bus.cpu.mode_state.current_step();

        let cycle_parity = bus.apu_regs.clock().cycle_parity();
        bus.dmc_dma.step(step.is_read(), cycle_parity);
        match bus.dmc_dma.latest_action() {
            DmcDmaAction::DoNothing => {}
            DmcDmaAction::Halt | DmcDmaAction::Dummy | DmcDmaAction::Align => step = step.with_actions_removed(),
            DmcDmaAction::Read => step = DMC_READ_STEP,
        }

        let block_oam_dma_memory_access = bus.dmc_dma.latest_action() == DmcDmaAction::Read;
        bus.oam_dma.step(step.is_read(), cycle_parity, block_oam_dma_memory_access);
        step = match bus.oam_dma.latest_action() {
            OamDmaAction::DoNothing => step,
            OamDmaAction::Halt | OamDmaAction::Align => step.with_actions_removed(),
            OamDmaAction::Read => OAM_READ_STEP,
            OamDmaAction::Write => OAM_WRITE_STEP,
        };

        let value;
        match step {
            Step::Read(from, _) => {
                bus.set_cpu_address_bus(AddressBusType::Cpu, bus.cpu.lookup_from_address(bus, from));
                value = bus.cpu_read(mapper, AddressBusType::Cpu);
            }
            Step::ReadField(field, from, _) => {
                bus.set_cpu_address_bus(AddressBusType::Cpu, bus.cpu.lookup_from_address(bus, from));
                value = bus.cpu_read(mapper, AddressBusType::Cpu);
                bus.cpu.set_field_value(field, value);
            }
            Step::Write(to, _) => {
                bus.set_cpu_address_bus(AddressBusType::Cpu, bus.cpu.lookup_to_address(bus, to));
                value = bus.cpu_pinout.data_bus;
                bus.cpu_write(mapper, AddressBusType::Cpu);
            }
            Step::WriteField(field, to, _) => {
                bus.set_cpu_address_bus(AddressBusType::Cpu, bus.cpu.lookup_to_address(bus, to));
                bus.cpu_pinout.data_bus = bus.cpu.field_value(&mut bus.cpu_pinout, field);
                value = bus.cpu_pinout.data_bus;
                bus.cpu_write(mapper, AddressBusType::Cpu);
            }
            Step::OamRead(from, _) => {
                bus.set_cpu_address_bus(AddressBusType::OamDma, bus.cpu.lookup_from_address(bus, from));
                value = bus.cpu_read(mapper, AddressBusType::OamDma);
            }
            Step::OamWrite(to, _) => {
                bus.set_cpu_address_bus(AddressBusType::OamDma, bus.cpu.lookup_to_address(bus, to));
                value = bus.cpu_pinout.data_bus;
                bus.cpu_write(mapper, AddressBusType::OamDma);
            }
            Step::DmcRead(from, _) => {
                bus.set_cpu_address_bus(AddressBusType::DmcDma, bus.cpu.lookup_from_address(bus, from));
                value = bus.cpu_read(mapper, AddressBusType::DmcDma);
            }
        }

        let formatted_step = if log_enabled!(target: "cpustep", Info) {
            match bus.cpu.step_formatting {
                CpuStepFormatting::NoData => format!("{step:?}"),
                CpuStepFormatting::Data => step.format_with_read_write_values(bus, value),
            }
        } else {
            String::new()
        };

        let original_program_counter = bus.cpu.program_counter;
        for &action in step.actions() {
            Cpu::execute_step_action(bus, action, value);
        }

        let halted = bus.dmc_dma.cpu_should_be_halted() || bus.oam_dma.cpu_should_be_halted();
        if log_enabled!(target: "cpustep", Info) {
            let step_name = if halted { "HALTED".to_string() } else { bus.cpu.mode_state.step_name() };
            let cpu_cycle = bus.cpu_cycle();
            info!("\t {step_name} PC: {original_program_counter}, Cycle: {cpu_cycle}, {formatted_step}");
        }

        if !halted {
            bus.cpu.mode_state.step();

            if step.has_start_new_instruction() {
                bus.cpu.mode_state.set_current_instruction_with_address(
                    Instruction::from_code_point(value),
                    bus.cpu_pinout.address_bus,
                );
            }
        }

        Some(step)
    }

    pub fn step_second_half(bus: &mut Bus, mapper: &mut dyn Mapper) {
        if bus.cpu_pinout.reset.current_value() == SignalLevel::Low {
            // The CPU doesn't do anything while the RESET button is held down.
            return;
        }

        let edge_detected = bus.cpu_pinout.nmi_signal_detector.detect();
        if edge_detected {
            bus.cpu.nmi_status = NmiStatus::Pending;
        }

        // Keep irq_pending and irq_status in sync
        if bus.cpu_pinout.irq_asserted() {
            if bus.cpu.irq_status == IrqStatus::Inactive && !bus.cpu.status.interrupts_disabled {
                bus.cpu.irq_status = IrqStatus::Pending;
            }
        } else if bus.cpu.irq_status != IrqStatus::Active || !bus.cpu.mode_state.is_branch_delay_active() {
            bus.cpu.irq_status = IrqStatus::Inactive;
        }

        mapper.on_end_of_cpu_cycle(bus);
    }

    fn execute_step_action(Bus { cpu, cpu_pinout, apu_regs, dmc_dma, oam_dma, .. }: &mut Bus, action: StepAction, value: u8) {
        match action {
            StepAction::StartNextInstruction => {
                if cpu.mode_state.should_suppress_next_instruction_start() {
                    return;
                }

                if cpu.reset_status == ResetStatus::Ready {
                    cpu.reset_status = ResetStatus::Active;
                    cpu_pinout.data_bus = 0x00;
                    cpu.mode_state.interrupt_sequence(InterruptType::Reset);
                } else if cpu.nmi_status == NmiStatus::Active {
                    cpu_pinout.data_bus = 0x00;
                    cpu.mode_state.interrupt_sequence(InterruptType::Nmi);
                } else if cpu.irq_status == IrqStatus::Active && cpu.nmi_status == NmiStatus::Inactive {
                    cpu_pinout.data_bus = 0x00;
                    cpu.mode_state.interrupt_sequence(InterruptType::Irq);
                } else {
                    cpu.mode_state.instruction(Instruction::from_code_point(value));
                }
            }

            StepAction::InterpretOpCode => {}
            StepAction::ExecuteOpCode => {
                let instruction = cpu.mode_state.current_instruction().unwrap();
                use OpCode::*;
                match instruction.op_code() {
                    // Implicit (and Accumulator) op codes.
                    INX => cpu.x = cpu.nz(cpu.x.wrapping_add(1)),
                    INY => cpu.y = cpu.nz(cpu.y.wrapping_add(1)),
                    DEX => cpu.x = cpu.nz(cpu.x.wrapping_sub(1)),
                    DEY => cpu.y = cpu.nz(cpu.y.wrapping_sub(1)),
                    TAX => cpu.x = cpu.nz(cpu.a),
                    TAY => cpu.y = cpu.nz(cpu.a),
                    TSX => cpu.x = cpu.nz(cpu.stack_pointer),
                    TXS => cpu.stack_pointer = cpu.x,
                    TXA => cpu.a = cpu.nz(cpu.x),
                    TYA => cpu.a = cpu.nz(cpu.y),
                    PLA => cpu.a = cpu.nz(cpu.operand),
                    PLP => cpu.status = Status::from_byte(cpu.operand),
                    CLC => cpu.status.carry = false,
                    SEC => cpu.status.carry = true,
                    CLD => cpu.status.decimal = false,
                    SED => cpu.status.decimal = true,
                    CLI => cpu.status.interrupts_disabled = false,
                    SEI => cpu.status.interrupts_disabled = true,
                    CLV => cpu.status.overflow = false,
                    NOP => { /* Do nothing. */ }

                    // Immediate op codes.
                    LDA => cpu.a = cpu.nz(cpu.operand),
                    LDX => cpu.x = cpu.nz(cpu.operand),
                    LDY => cpu.y = cpu.nz(cpu.operand),
                    CMP => cpu.cmp(cpu.operand),
                    CPX => cpu.cpx(cpu.operand),
                    CPY => cpu.cpy(cpu.operand),
                    ORA => cpu.a = cpu.nz(cpu.a | cpu.operand),
                    AND => cpu.a = cpu.nz(cpu.a & cpu.operand),
                    EOR => cpu.a = cpu.nz(cpu.a ^ cpu.operand),
                    ADC => cpu.a = cpu.adc(cpu.operand),
                    SBC => cpu.a = cpu.sbc(cpu.operand),
                    LAX => {
                        cpu.a = cpu.operand;
                        cpu.x = cpu.operand;
                        cpu.nz(cpu.operand);
                    }
                    ANC => {
                        cpu.a = cpu.nz(cpu.a & cpu.operand);
                        cpu.status.carry = cpu.status.negative;
                    }
                    ALR => {
                        cpu.a = cpu.nz(cpu.a & cpu.operand);
                        Cpu::lsr(&mut cpu.status, &mut cpu.a);
                    }
                    ARR => {
                        // TODO: What a mess.
                        let value = (cpu.a & cpu.operand) >> 1;
                        cpu.a = cpu.nz(value | if cpu.status.carry {0x80} else {0x00});
                        cpu.status.carry = cpu.a & 0x40 != 0;
                        cpu.status.overflow =
                            (u8::from(cpu.status.carry) ^ ((cpu.a >> 5) & 0x01)) != 0;
                    }
                    AXS => {
                        cpu.status.carry = cpu.a & cpu.x >= cpu.operand;
                        cpu.x = cpu.nz((cpu.a & cpu.x).wrapping_sub(cpu.operand));
                    }

                    BIT => {
                        cpu.status.negative = cpu.operand & 0b1000_0000 != 0;
                        cpu.status.overflow = cpu.operand & 0b0100_0000 != 0;
                        cpu.status.zero = cpu.operand & cpu.a == 0;
                    }

                    // Write op codes.
                    STA | STX | STY | SAX | SHX | SHY | TAS | AHX => panic!("ExecuteOpCode must not be implemented for {:?}", instruction.op_code()),

                    // Read-Modify-Write op codes.
                    ASL if instruction.access_mode() == AccessMode::Imp => Cpu::asl(&mut cpu.status, &mut cpu.a),
                    ROL if instruction.access_mode() == AccessMode::Imp => Cpu::rol(&mut cpu.status, &mut cpu.a),
                    LSR if instruction.access_mode() == AccessMode::Imp => Cpu::lsr(&mut cpu.status, &mut cpu.a),
                    ROR if instruction.access_mode() == AccessMode::Imp => Cpu::ror(&mut cpu.status, &mut cpu.a),
                    ASL => Cpu::asl(&mut cpu.status, &mut cpu.operand),
                    ROL => Cpu::rol(&mut cpu.status, &mut cpu.operand),
                    LSR => Cpu::lsr(&mut cpu.status, &mut cpu.operand),
                    ROR => Cpu::ror(&mut cpu.status, &mut cpu.operand),

                    INC => {
                        cpu.operand = cpu.operand.wrapping_add(1);
                        Cpu::nz_status(&mut cpu.status, cpu.operand);
                    }
                    DEC => {
                        cpu.operand = cpu.operand.wrapping_sub(1);
                        Cpu::nz_status(&mut cpu.status, cpu.operand);
                    }
                    SLO => {
                        Cpu::asl(&mut cpu.status, &mut cpu.operand);
                        cpu.a |= cpu.operand;
                        cpu.nz(cpu.a);
                    }
                    SRE => {
                        Cpu::lsr(&mut cpu.status, &mut cpu.operand);
                        cpu.a ^= cpu.operand;
                        cpu.nz(cpu.a);
                    }
                    RLA => {
                        Cpu::rol(&mut cpu.status, &mut cpu.operand);
                        cpu.a &= cpu.operand;
                        cpu.nz(cpu.a);
                    },
                    RRA => {
                        Cpu::ror(&mut cpu.status, &mut cpu.operand);
                        cpu.a = cpu.adc(cpu.operand);
                        cpu.nz(cpu.a);
                    }
                    ISC => {
                        cpu.operand = cpu.operand.wrapping_add(1);
                        cpu.a = cpu.sbc(cpu.operand);
                    }
                    DCP => {
                        cpu.operand = cpu.operand.wrapping_sub(1);
                        cpu.cmp(cpu.operand);
                    }

                    LAS => {
                        let value = cpu.operand & cpu.stack_pointer;
                        cpu.a = value;
                        cpu.x = value;
                        cpu.stack_pointer = value;
                    }
                    XAA => {
                        cpu.a = cpu.nz(cpu.a & cpu.x & cpu.operand);
                    }

                    // Relative op codes.
                    BPL => if !cpu.status.negative { cpu.branch(); }
                    BMI => if cpu.status.negative { cpu.branch(); }
                    BVC => if !cpu.status.overflow { cpu.branch(); }
                    BVS => if cpu.status.overflow { cpu.branch(); }
                    BCC => if !cpu.status.carry { cpu.branch(); }
                    BCS => if cpu.status.carry { cpu.branch(); }
                    BNE => if !cpu.status.zero { cpu.branch(); }
                    BEQ => if cpu.status.zero { cpu.branch(); }

                    JAM => cpu.mode_state.jammed(),
                    _ => todo!("{instruction:X?}"),
                }
            }

            StepAction::IncrementPC => {
                // FIXME : Rather than suppressing this here, this StepAction should have been
                // stripped out earlier.
                if !cpu.mode_state.should_suppress_next_instruction_start() && !cpu.mode_state.is_interrupt_sequence_active() {
                    cpu.program_counter.inc();
                }
            }
            // TODO: Make sure this isn't supposed to wrap within the same page.
            StepAction::IncrementAddress => cpu.computed_address = cpu_pinout.address_bus.inc(),
            StepAction::IncrementAddressLow => cpu.computed_address = cpu_pinout.address_bus.offset_low(1).0,
            StepAction::IncrementOamDmaAddress => oam_dma.increment_address(),

            StepAction::IncrementStackPointer => cpu.stack_pointer = cpu.stack_pointer.wrapping_add(1),
            StepAction::DecrementStackPointer => cpu.stack_pointer = cpu.stack_pointer.wrapping_sub(1),

            StepAction::DisableInterrupts => cpu.status.interrupts_disabled = true,
            StepAction::SetInterruptVector => {
                cpu.current_interrupt_vector =
                    if cpu.reset_status != ResetStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to RESET.");
                        Some(InterruptType::Reset)
                    } else if cpu.nmi_status != NmiStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to NMI.");
                        Some(InterruptType::Nmi)
                    } else if cpu.irq_status != IrqStatus::Inactive {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to IRQ.");
                        Some(InterruptType::Irq)
                    } else if let Some(instruction) = cpu.mode_state.current_instruction() && instruction.op_code() == OpCode::BRK {
                        info!(target: "cpuflowcontrol", "Setting interrupt vector to IRQ due to BRK.");
                        Some(InterruptType::Irq)
                    } else {
                        None
                    };
                cpu.mode_state.interrupt_vector_set(cpu.current_interrupt_vector);

                // Clear interrupt statuses now that the vector is set.
                cpu.nmi_status = NmiStatus::Inactive;
                cpu.irq_status = IrqStatus::Inactive;
                cpu.reset_status = ResetStatus::Inactive;
            }
            StepAction::ClearInterruptVector => cpu.current_interrupt_vector = None,
            StepAction::PollInterrupts => {
                if cpu.nmi_status == NmiStatus::Ready {
                    cpu.nmi_status = NmiStatus::Active;
                } else if cpu.irq_status == IrqStatus::Ready && !cpu.status.interrupts_disabled {
                    cpu.irq_status = IrqStatus::Active;
                }
            }
            StepAction::MaybePollInterrupts => {
                if cpu.address_carry != 0 {
                    if cpu.nmi_status == NmiStatus::Ready {
                        cpu.nmi_status = NmiStatus::Ready;
                    } else if cpu.irq_status == IrqStatus::Ready && !cpu.status.interrupts_disabled {
                        cpu.irq_status = IrqStatus::Active;
                    }
                }
            }

            StepAction::SetDmcSampleBuffer => apu_regs.dmc.set_sample_buffer(cpu_pinout, dmc_dma, value),

            StepAction::XOffsetPendingAddressLow => {
                let carry;
                (cpu.pending_address_low, carry) =
                    cpu.pending_address_low.overflowing_add(cpu.x);
                if carry {
                    cpu.address_carry = 1;
                }
            }
            StepAction::YOffsetPendingAddressLow => {
                let carry;
                (cpu.pending_address_low, carry) =
                    cpu.pending_address_low.overflowing_add(cpu.y);
                if carry {
                    cpu.address_carry = 1;
                }
            }
            StepAction::XOffsetAddress => cpu.computed_address = cpu_pinout.address_bus.offset_low(cpu.x).0,
            StepAction::YOffsetAddress => cpu.computed_address = cpu_pinout.address_bus.offset_low(cpu.y).0,
            StepAction::MaybeInsertOopsStep => {
                if cpu.address_carry != 0 {
                    cpu.mode_state.oops();
                }
            }
            StepAction::MaybeInsertBranchOopsStep => {
                if cpu.address_carry != 0 {
                    cpu.mode_state.branch_oops();
                }
            }

            StepAction::CopyAddressToPC => {
                cpu.program_counter = cpu_pinout.address_bus;
            }
            StepAction::AddCarryToAddress => {
                cpu.computed_address = cpu_pinout.address_bus.offset_high(cpu.address_carry);
                cpu.address_carry = 0;
            }
            StepAction::AddCarryToPC => {
                if cpu.address_carry != 0 {
                    cpu.program_counter = cpu.program_counter.offset_high(cpu.address_carry);
                    cpu.address_carry = 0;
                }
            }
        }
    }

    fn lookup_from_address(&self, bus: &Bus, from: From) -> CpuAddress {
        use self::From::*;
        match from {
            OamDmaAddressTarget => bus.oam_dma.address(),
            DmcDmaAddressTarget => bus.dmc_dma_address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget => CpuAddress::from_low_high(self.pending_address_low, self.pending_address_high),
            PendingZeroPageTarget => CpuAddress::from_low_high(self.pending_address_low, 0),
            ComputedTarget => self.computed_address,
            TopOfStack => self.stack_pointer_address(),
            InterruptVectorLow => {
                if self.mode_state.is_irq_sequence_active() {
                    // FIXME: Hack
                    IRQ_VECTOR_LOW
                } else {
                    match self.current_interrupt_vector.unwrap() {
                        InterruptType::Nmi => NMI_VECTOR_LOW,
                        InterruptType::Reset => RESET_VECTOR_LOW,
                        InterruptType::Irq => IRQ_VECTOR_LOW,
                    }
                }
            }
            InterruptVectorHigh => {
                if self.mode_state.is_irq_sequence_active() {
                    // FIXME: Hack
                    IRQ_VECTOR_HIGH
                } else {
                    match self.current_interrupt_vector.unwrap() {
                        InterruptType::Nmi => NMI_VECTOR_HIGH,
                        InterruptType::Reset => RESET_VECTOR_HIGH,
                        InterruptType::Irq => IRQ_VECTOR_HIGH,
                    }
                }
            }
        }
    }

    fn lookup_to_address(&self, bus: &Bus, to: To) -> CpuAddress {
        use self::To::*;
        match to {
            OamDmaAddressTarget => bus.oam_dma.address(),
            ProgramCounterTarget => self.program_counter,
            PendingAddressTarget =>
                CpuAddress::from_low_high(self.pending_address_low, self.pending_address_high),
            PendingZeroPageTarget =>
                CpuAddress::from_low_high(self.pending_address_low, 0),
            ComputedTarget => self.computed_address,
            TopOfStack => self.stack_pointer_address(),
            AddressTarget(address) => address,
        }
    }

    fn field_value(&mut self, cpu_pinout: &mut CpuPinout, field: Field) -> u8 {
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
                    let (low, high) = cpu_pinout.address_bus.to_low_high();
                    cpu_pinout.address_bus = CpuAddress::from_low_high(low, self.x & high);
                    self.x & high
                }
                // FIXME: Calculations should be done as part of an earlier StepAction.
                OpCode::SHY => {
                    let (low, high) = cpu_pinout.address_bus.to_low_high();
                    cpu_pinout.address_bus = CpuAddress::from_low_high(low, self.y & high);
                    self.y
                }
                // FIXME: Calculations should be done as part of an earlier StepAction.
                OpCode::AHX => {
                    let (low, high) = cpu_pinout.address_bus.to_low_high();
                    // This is using later revision logic.
                    // For early revision logic, use self.a & self.x & self.a
                    cpu_pinout.address_bus = CpuAddress::from_low_high(low, self.x & high);
                    self.a & self.x & high
                }
                OpCode::TAS => {
                    let sp = self.a & self.x;
                    self.stack_pointer = sp;
                    self.x & cpu_pinout.address_bus.high_byte()
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

    fn stack_pointer_address(&self) -> CpuAddress {
        CpuAddress::from_low_high(self.stack_pointer(), 0x01)
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
