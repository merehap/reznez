use log::info;

use crate::cpu::instruction::{Instruction, OpCode, Argument};
use crate::cpu::status::Status;
use crate::cpu::dma_transfer::{DmaTransfer, DmaTransferState};
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

    nmi_scheduling_status: NmiSchedulingStatus,
    dma_port: DmaPort,
    dma_transfer: DmaTransfer,

    current_instruction_remaining_cycles: u8,
    cycle: u64,
}

impl Cpu {
    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn new(memory: &mut CpuMemory, program_counter_source: ProgramCounterSource) -> Cpu {
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

            nmi_scheduling_status: NmiSchedulingStatus::Unscheduled,
            dma_port: memory.ports().dma.clone(),
            dma_transfer: DmaTransfer::inactive(),

            current_instruction_remaining_cycles: 0,
            // Unclear why this is the case.
            cycle: 7,
        }
    }

    // From https://wiki.nesdev.org/w/index.php?title=CPU_power_up_state
    pub fn reset(&mut self, memory: &mut CpuMemory) {
        self.status.interrupts_disabled = true;
        self.program_counter = memory.reset_vector();
        self.cycle = 7;
        // TODO: APU resets?
    }

    pub fn state_string(&self, memory: &CpuMemory) -> String {
        let nesting = "";
        format!("{:010} PC:{}, A:{:02X}, X:{:02X}, Y:{:02X}, P:{:02X}, S:{:02X}, {} {}",
            self.cycle,
            self.program_counter,
            self.a,
            self.x,
            self.y,
            self.status.to_register_byte(),
            memory.stack_pointer(),
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

    pub fn program_counter(&self) -> CpuAddress {
        self.program_counter
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn nmi_scheduling_status(&self) -> NmiSchedulingStatus {
        self.nmi_scheduling_status
    }

    pub fn schedule_nmi(&mut self) {
        self.nmi_scheduling_status = NmiSchedulingStatus::AfterNextInstruction;
    }

    pub fn step(&mut self, memory: &mut CpuMemory) -> Option<Instruction> {
        if let Some(dma_page) = self.dma_port.take_page() {
            self.dma_transfer = DmaTransfer::new(dma_page, self.cycle);
        }

        self.cycle += 1;

        // Normal CPU operation is suspended while the DMA transfer completes.
        match self.dma_transfer.step() {
            DmaTransferState::Finished =>
                {/* No transfer in progress. Continue to normal CPU step.*/},
            DmaTransferState::Write(cpu_address) => {
                let value = memory.read(cpu_address);
                memory.write(CpuAddress::new(0x2004), value);
                return None;
            },
            _ => return None,
        }

        if self.current_instruction_remaining_cycles != 0 {
            self.current_instruction_remaining_cycles -= 1;
            return None;
        }

        use NmiSchedulingStatus::*;
        match self.nmi_scheduling_status {
            Unscheduled => {/* Do nothing. */},
            AfterCurrentInstruction => {
                self.nmi(memory);
                self.nmi_scheduling_status = Unscheduled;
            },
            AfterNextInstruction =>
                self.nmi_scheduling_status = AfterCurrentInstruction,
        }

        let instruction = Instruction::from_memory(
            self.program_counter,
            self.x,
            self.y,
            memory,
        );

        let cycle_count = self.execute_instruction(memory, instruction);
        self.current_instruction_remaining_cycles = cycle_count - 1;

        Some(instruction)
    }

    fn execute_instruction(&mut self, memory: &mut CpuMemory, instruction: Instruction) -> u8 {
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
            (TSX, Imp) => self.x = self.nz(memory.stack_pointer()),
            (TXS, Imp) => *memory.stack_pointer_mut() = self.x,
            (TXA, Imp) => self.a = self.nz(self.x),
            (TYA, Imp) => self.a = self.nz(self.y),
            (PHA, Imp) => memory.stack().push(self.a),
            (PHP, Imp) => memory.stack().push(self.status.to_instruction_byte()),
            (PLA, Imp) => {
                self.a = memory.stack().pop();
                self.nz(self.a);
            },
            (PLP, Imp) => self.status = Status::from_byte(memory.stack().pop()),
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
                memory.stack().push_address(self.program_counter);
                memory.stack().push(self.status.to_instruction_byte());
                self.status.interrupts_disabled = true;
                self.program_counter = memory.irq_vector();
            },
            (RTI, Imp) => {
                self.status = Status::from_byte(memory.stack().pop());
                self.program_counter = memory.stack().pop_address();
            },
            (RTS, Imp) => self.program_counter = memory.stack().pop_address().advance(1),

            (STA, Addr(addr)) => memory.write(addr, self.a),
            (STX, Addr(addr)) => memory.write(addr, self.x),
            (STY, Addr(addr)) => memory.write(addr, self.y),
            (DEC, Addr(addr)) => {
                let value = memory.read(addr).wrapping_sub(1);
                memory.write(addr, value);
                self.nz(value);
            },
            (INC, Addr(addr)) => {
                let value = memory.read(addr).wrapping_add(1);
                memory.write(addr, value);
                self.nz(value);
            },
            (BPL, Addr(addr)) =>
                if !self.status.negative {cycle_count += self.take_branch(addr);},
            (BMI, Addr(addr)) =>
                if self.status.negative {cycle_count += self.take_branch(addr);},
            (BVC, Addr(addr)) =>
                if !self.status.overflow {cycle_count += self.take_branch(addr);},
            (BVS, Addr(addr)) =>
                if self.status.overflow {cycle_count += self.take_branch(addr);},
            (BCC, Addr(addr)) =>
                if !self.status.carry {cycle_count += self.take_branch(addr);},
            (BCS, Addr(addr)) =>
                if self.status.carry {cycle_count += self.take_branch(addr);},
            (BNE, Addr(addr)) =>
                if !self.status.zero {cycle_count += self.take_branch(addr);},
            (BEQ, Addr(addr)) =>
                if self.status.zero {cycle_count += self.take_branch(addr);},
            (JSR, Addr(addr)) => {
                // Push the address one previous for some reason.
                memory.stack().push_address(self.program_counter.offset(-1));
                self.program_counter = addr;
            },
            (JMP, Addr(addr)) => self.program_counter = addr,

            (BIT, Addr(addr)) => {
                let val = memory.read(addr);
                self.status.negative = val & 0b1000_0000 != 0;
                self.status.overflow = val & 0b0100_0000 != 0;
                self.status.zero = val & self.a == 0;
            },

            (LDA, Imm(val)) => self.a = self.nz(val),
            (LDX, Imm(val)) => self.x = self.nz(val),
            (LDY, Imm(val)) => self.y = self.nz(val),
            (CMP, Imm(val)) => self.cmp(val),
            (CPX, Imm(val)) => self.cpx(val),
            (CPY, Imm(val)) => self.cpy(val),
            (ORA, Imm(val)) => self.a = self.nz(self.a | val),
            (AND, Imm(val)) => self.a = self.nz(self.a & val),
            (EOR, Imm(val)) => self.a = self.nz(self.a ^ val),
            (ADC, Imm(val)) => self.a = self.adc(val),
            (SBC, Imm(val)) => self.a = self.sbc(val),

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

            (LAX, Imm(val)) => {
                self.a = val;
                self.x = val;
                self.nz(val);
            },
            (LAX, Addr(addr)) => {
                let val = memory.read(addr);
                self.a = val;
                self.x = val;
                self.nz(val);
            },

            (ASL, Imp) => self.a = self.asl(self.a),
            (ASL, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.asl(value);
                memory.write(addr, value);
            },
            (ROL, Imp) => self.a = self.rol(self.a),
            (ROL, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.rol(value);
                memory.write(addr, value);
            },
            (LSR, Imp) => self.a = self.lsr(self.a),
            (LSR, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.lsr(value);
                memory.write(addr, value);
            },
            (ROR, Imp) => self.a = self.ror(self.a),
            (ROR, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.ror(value);
                memory.write(addr, value);
            },

            // Undocumented op codes.
            (SLO, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.asl(value);
                memory.write(addr, value);
                self.a |= value;
                self.nz(self.a);
            },
            (RLA, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.rol(value);
                memory.write(addr, value);
                self.a &= value;
                self.nz(self.a);
            },
            (SRE, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.lsr(value);
                memory.write(addr, value);
                self.a ^= value;
                self.nz(self.a);
            },
            (RRA, Addr(addr)) => {
                let value = memory.read(addr);
                let value = self.ror(value);
                memory.write(addr, value);
                self.a = self.adc(value);
                self.nz(self.a);
            },
            (SAX, Addr(addr)) => memory.write(addr, self.a & self.x),
            (DCP, Addr(addr)) => {
                let value = memory.read(addr).wrapping_sub(1);
                memory.write(addr, value);
                self.cmp(value);
            },
            (ISC, Addr(addr)) => {
                let value = memory.read(addr).wrapping_add(1);
                memory.write(addr, value);
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
            (SHY, Addr(addr)) => {
                let (low, high) = addr.to_low_high();
                let value = self.y & high.wrapping_add(1);
                let addr = CpuAddress::from_low_high(low, high & self.y);
                memory.write(addr, value);
            },
            (SHX, Addr(addr)) => {
                let (low, high) = addr.to_low_high();
                let value = self.x & high.wrapping_add(1);
                let addr = CpuAddress::from_low_high(low, high & self.x);
                memory.write(addr, value);
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

    fn take_branch(&mut self, destination: CpuAddress) -> u8 {
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
    fn nmi(&mut self, memory: &mut CpuMemory) {
        info!(target: "cpu", "Executing NMI.");
        memory.stack().push_address(self.program_counter);
        memory.stack().push(self.status.to_interrupt_byte());
        self.program_counter = memory.nmi_vector();
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NmiSchedulingStatus {
    Unscheduled,
    AfterCurrentInstruction,
    AfterNextInstruction,
}

#[cfg(test)]
mod tests {
    use crate::cartridge;
    use crate::memory::memory;
    use crate::memory::memory::Memory;

    use super::*;

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
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        cpu.schedule_nmi();
        assert_eq!(0xFD, mem.stack_pointer());
        assert_eq!(reset_vector.advance(1), cpu.program_counter());

        // Execute final cycle of the first instruction.
        cpu.step(&mut mem.as_cpu_memory());
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

        // Execute the first cycle of the NMI subroutine.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFA, mem.stack_pointer());
        assert_eq!(nmi_vector.advance(1), cpu.program_counter());
    }

    #[test]
    fn nmi_after_instruction() {
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

        // Execute the first cycle of the NMI subroutine.
        cpu.step(&mut mem.as_cpu_memory());
        assert_eq!(0xFA, mem.stack_pointer());
        assert_eq!(nmi_vector.advance(1), cpu.program_counter());
    }

    #[test]
    fn nmi_scheduled_before_branching() {
    }

    #[test]
    fn nmi_scheduled_before_oops() {
    }

    #[test]
    fn nmi_scheduled_before_branching_oops() {
    }

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

        memory::test_data::memory_with_cartridge(cartridge)
    }
}
