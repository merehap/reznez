use log::{info, error};

use crate::cpu::step::*;
use crate::cpu::cycle_action::{CycleAction, From, To};
use crate::cpu::cycle_action_queue::CycleActionQueue;
use crate::cpu::instruction;
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
    next_op_code: Option<(u8, CpuAddress)>,

    cycle_action_queue: CycleActionQueue,
    nmi_status: NmiStatus,

    dma_port: DmaPort,

    cycle: u64,

    jammed: bool,

    address_bus: CpuAddress,
    previous_address_bus_value: CpuAddress,
    data_bus: u8,
    previous_data_bus_value: u8,
    pending_address_low: u8,
    address_carry: i8,

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
            next_op_code: None,

            cycle_action_queue: CycleActionQueue::new(),
            nmi_status: NmiStatus::Inactive,
            dma_port: memory.ports().dma.clone(),

            // See startup sequence in NES-manual so this isn't hard-coded.
            cycle: 7,

            jammed: false,

            address_bus: CpuAddress::new(0x0000),
            previous_address_bus_value: CpuAddress::new(0x0000),
            data_bus: 0,
            previous_data_bus_value: 0,
            pending_address_low: 0,
            address_carry: 0,

            suppress_program_counter_increment: false,
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
        self.copy_data(memory, step.from(), step.to());
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
            SetAddressBusToOamDmaStart => self.address_bus = self.dma_port.start_address(),
            StorePendingAddressLowByte => self.pending_address_low = self.previous_data_bus_value,
            StorePendingAddressLowByteWithXOffset => {
                assert_eq!(self.address_carry, 0);
                let carry;
                (self.pending_address_low, carry) =
                    self.previous_data_bus_value.overflowing_add(self.x);
                if carry {
                    self.address_carry = 1;
                }
            }
            StorePendingAddressLowByteWithYOffset => {
                assert_eq!(self.address_carry, 0);
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
            MaybeInsertOopsStep => {
                if self.address_carry != 0 {
                    self.cycle_action_queue.skip_to_front(ADDRESS_BUS_READ_STEP);
                }
            }
            AddCarryToAddressBus => {
                self.address_bus.offset_high(self.address_carry);
                self.address_carry = 0;
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

                if instruction.template.access_mode == AccessMode::Rel {
                    self.execute_cycle_action(memory, Instruction);
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
                    STA => memory.write(self.address_bus, self.a),
                    STX => memory.write(self.address_bus, self.x),
                    STY => memory.write(self.address_bus, self.y),
                    SAX => memory.write(self.address_bus, self.a & self.x),


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

                    op_code => todo!("{:?}", op_code),
                }
            }
        }
    }

    // Note that most source/destination combos are invalid.
    // In particular, there is no way to directly copy from one memory location to another.
    fn copy_data(&mut self, memory: &mut CpuMemory, source: From, destination: To) {
        self.previous_address_bus_value = self.address_bus;
        self.previous_data_bus_value = self.data_bus;

        self.data_bus = match source {
            From::DataBus => self.data_bus,
            From::PendingAddress => {
                self.address_bus = CpuAddress::from_low_high(self.pending_address_low, self.data_bus);
                self.data_bus
            }
            From::PendingZeroPageAddress => {
                self.address_bus = CpuAddress::from_low_high(self.data_bus, 0);
                self.data_bus
            }
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
            From::PendingZeroPageTarget => {
                self.address_bus = CpuAddress::from_low_high(self.data_bus, 0);
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
            To::AddressBusTarget => memory.write(self.address_bus, self.data_bus),
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

            To::NextOpCode => {
                if self.dma_port.take_page().is_some() {
                    info!(target: "cpuoperation", "Starting DMA transfer at {}.", self.dma_port.start_address());
                    self.cycle_action_queue.enqueue_dma_transfer(self.cycle);
                    self.suppress_program_counter_increment = true;
                    // Seems like a hack. Normally this would be a declarative part of the step.
                    // It also may just be the wrong address bus value for this cycle.
                    self.address_bus = self.dma_port.start_address();
                    return;
                }

                match self.nmi_status {
                    NmiStatus::Inactive | NmiStatus::Pending => {
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
            info!(target: "cpuoperation", "'Oops' cycle added.");
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
                let mut value = memory.read(addr);
                Cpu::asl(&mut self.status, &mut value);
                memory.write(addr, value);
            }
            (ROL, Imp) => unreachable!(),
            (ROL, Addr(addr)) => {
                let mut value = memory.read(addr);
                Cpu::rol(&mut self.status, &mut value);
                memory.write(addr, value);
            }
            (LSR, Imp) => unreachable!(),
            (LSR, Addr(addr)) => {
                let mut value = memory.read(addr);
                Cpu::lsr(&mut self.status, &mut value);
                memory.write(addr, value);
            }
            (ROR, Imp) => unreachable!(),
            (ROR, Addr(addr)) => {
                let mut value = memory.read(addr);
                Cpu::ror(&mut self.status, &mut value);
                memory.write(addr, value);
            }

            // Undocumented op codes.
            (SLO, Addr(addr)) => {
                let mut value = memory.read(addr);
                Cpu::asl(&mut self.status, &mut value);
                memory.write(addr, value);
                self.a |= value;
                self.nz(self.a);
            }
            (RLA, Addr(addr)) => {
                let mut value = memory.read(addr);
                Cpu::rol(&mut self.status, &mut value);
                memory.write(addr, value);
                self.a &= value;
                self.nz(self.a);
            }
            (SRE, Addr(addr)) => {
                let mut value = memory.read(addr);
                Cpu::lsr(&mut self.status, &mut value);
                memory.write(addr, value);
                self.a ^= value;
                self.nz(self.a);
            }
            (RRA, Addr(addr)) => {
                let mut value = memory.read(addr);
                Cpu::ror(&mut self.status, &mut value);
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

    fn maybe_branch(
        &mut self,
        take_branch: bool,
        destination: CpuAddress,
    ) -> (bool, bool) {
        if !take_branch {
            return (false, false);
        }

        self.suppress_program_counter_increment = true;

        info!(target: "cpuoperation", "Branch taken, cycle added.");

        let oops = self.program_counter.offset(1).page() != destination.page();
        if oops {
            info!(target: "cpuoperation", "Branch crossed page boundary, 'Oops' cycle added.");
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

#[derive(PartialEq, Eq)]
enum NmiStatus {
    Inactive,
    Pending,
    Ready,
    Active,
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
