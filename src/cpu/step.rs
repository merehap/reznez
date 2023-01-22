use lazy_static::lazy_static;

use crate::cpu::cycle_action::Field;
use crate::cpu::cycle_action::Field::*;
use crate::cpu::cycle_action::{From, To};
use crate::cpu::cycle_action::CycleAction;
use crate::cpu::cycle_action::CycleAction::*;
use crate::cpu::instruction::*;
use crate::cpu::step::Step::{Read, ReadField, Write, WriteField};

lazy_static! {
    pub static ref INSTRUCTIONS: [CpuInstruction; 256] =
        INSTRUCTION_TEMPLATES.map(template_to_instruction);

    pub static ref OAM_DMA_TRANSFER_STEPS: [Step; 512] = {
        let read_write = &[
            Read(From::DmaAddressTarget, &[]),
            Write(To::OAM_DATA, &[IncrementDmaAddress]),
        ];

        read_write.repeat(256).try_into().unwrap()
    };
}

fn template_to_instruction(template: InstructionTemplate) -> CpuInstruction {
    use AccessMode::*;
    use OpCode::*;
    let steps = match (template.access_mode, template.op_code) {
        (Imp, BRK) => BRK_STEPS,
        (Imp, RTI) => RTI_STEPS,
        (Imp, RTS) => RTS_STEPS,
        (Imp, PHA) => PHA_STEPS,
        (Imp, PHP) => PHP_STEPS,
        (Imp, PLA) => PLA_STEPS,
        (Imp, PLP) => PLP_STEPS,
        (Abs, JSR) => JSR_STEPS,
        (Abs, JMP) => JMP_ABS_STEPS,
        (Ind, JMP) => JMP_IND_STEPS,

        (Imp,   _) => IMPLICIT_ADDRESSING_STEPS,
        (Imm,   _) => IMMEDIATE_ADDRESSING_STEPS,
        (Rel,   _) => RELATIVE_ADDRESSING_STEPS,

        // Read operations.
        (Abs, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | CPX | CPY | BIT | LAX | NOP) => ABSOLUTE_READ_STEPS,
        // TODO: Remove the unused combos.
        (AbX, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP) => ABSOLUTE_X_READ_STEPS,
        (AbY, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP | LAS | TAS | AHX) => ABSOLUTE_Y_READ_STEPS,
        (ZP , LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | CPX | CPY | BIT | LAX | NOP) => ZERO_PAGE_READ_STEPS,
        (ZPX, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP) => ZERO_PAGE_X_READ_STEPS,
        (ZPY, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |                   LAX | NOP) => ZERO_PAGE_Y_READ_STEPS,
        (IzX, LDA |             EOR | AND | ORA | ADC | SBC | CMP |                   LAX) => INDEXED_INDIRECT_READ_STEPS,
        (IzY, LDA |             EOR | AND | ORA | ADC | SBC | CMP |                   LAX | AHX) => INDIRECT_INDEXED_READ_STEPS,

        // Write operations.
        (Abs, STA | STX | STY | SAX) => ABSOLUTE_WRITE_STEPS,
        // TODO: Remove the unused combos.
        (AbX, STA | STX | STY |     SHY) => ABSOLUTE_X_WRITE_STEPS,
        (AbY, STA | STX | STY |     SHX) => ABSOLUTE_Y_WRITE_STEPS,
        (ZP , STA | STX | STY | SAX) => ZERO_PAGE_WRITE_STEPS,
        (ZPX, STA | STX | STY | SAX) => ZERO_PAGE_X_WRITE_STEPS,
        (ZPY, STA | STX | STY | SAX) => ZERO_PAGE_Y_WRITE_STEPS,
        (IzX, STA |             SAX) => INDEXED_INDIRECT_WRITE_STEPS,
        (IzY, STA) => INDIRECT_INDEXED_WRITE_STEPS,

        // Read-modify-write operations.
        (Abs, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ABSOLUTE_READ_MODIFY_WRITE_STEPS,
        // TODO: Remove the unused combos.
        (AbX, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ABSOLUTE_X_READ_MODIFY_WRITE_STEPS,
        (AbY, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS,
        (ZP , ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ZERO_PAGE_READ_MODIFY_WRITE_STEPS,
        (ZPX, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS,
        (ZPY, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP) => ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS,
        (IzX,                                     SLO | SRE | RLA | RRA | ISC | DCP) => INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS,
        (IzY,                                     SLO | SRE | RLA | RRA | ISC | DCP) => INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS,

        (_, _) => unreachable!("{:X?}", template),
    };

    CpuInstruction {
        steps,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CpuInstruction {
    steps: &'static [Step],
}

impl CpuInstruction {
    pub fn steps(&self) -> &'static [Step] {
        self.steps
    }
}

pub const READ_OP_CODE_STEP: Step =
    Read(From::ProgramCounterTarget, &[StartNextInstruction, IncrementProgramCounter]);

pub const INTERPRET_OP_CODE_STEP: Step =
    Read(From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]);

pub const ADDRESS_BUS_READ_STEP: Step =
    Read(From::AddressBusTarget, &[]);

pub const BRANCH_TAKEN_STEP: Step =
    Read(From::ProgramCounterTarget,
        &[MaybeInsertBranchOopsStep, AddCarryToProgramCounter, StartNextInstruction, IncrementProgramCounter]);

pub const IMPLICIT_ADDRESSING_STEPS: &'static [Step] = &[
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];
pub const IMMEDIATE_ADDRESSING_STEPS: &'static [Step] = &[
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];
pub const RELATIVE_ADDRESSING_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_READ_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByte, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_READ_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_X_READ_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_X_READ_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_Y_READ_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_Y_READ_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[YOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDEXED_INDIRECT_READ_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByte]),
    Read(From::PendingAddressTarget , &[]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDIRECT_INDEXED_READ_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByteWithYOffset]),
    Read(From::PendingAddressTarget , &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

// TODO: The data bus needs to be set to the data written.
pub const ABSOLUTE_WRITE_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByte, IncrementProgramCounter]),
    WriteField(OpRegister, To::PendingAddressTarget, &[]),
];

// TODO: The data bus needs to be set to the data written.
pub const ZERO_PAGE_WRITE_STEPS: &'static [Step] = &[
    WriteField(OpRegister, To::PendingZeroPageTarget, &[]),
];

pub const ABSOLUTE_X_WRITE_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ZERO_PAGE_X_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ABSOLUTE_Y_WRITE_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ZERO_PAGE_Y_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[YOffsetAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const INDEXED_INDIRECT_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByte]),
    WriteField(OpRegister, To::PendingAddressTarget, &[]),
];

pub const INDIRECT_INDEXED_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByteWithYOffset]),
    Read(From::PendingAddressTarget , &[AddCarryToAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ABSOLUTE_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget , &[StorePendingAddressLowByte, IncrementProgramCounter]),
    Read(From::PendingAddressTarget , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const ZERO_PAGE_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const ABSOLUTE_X_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    Read(From::AddressBusTarget    , &[]),
    Write(To::AddressBusTarget     , &[ExecuteOpCode]),
    Write(To::AddressBusTarget     , &[]),
];

pub const ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    Read(From::AddressBusTarget    , &[]),
    Write(To::AddressBusTarget     , &[ExecuteOpCode]),
    Write(To::AddressBusTarget     , &[]),
];

pub const ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[YOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByte]),
    Read(From::PendingAddressTarget , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Read(From::PendingZeroPageTarget, &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByteWithYOffset]),
    Read(From::PendingAddressTarget , &[AddCarryToAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const NMI_STEPS: &'static [Step] = &[
    WriteField(ProgramCounterHighByte, To::TopOfStack, &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack, &[DecrementStackPointer]),
    WriteField(StatusForInterrupt    , To::TopOfStack, &[DecrementStackPointer]),
    // Copy the new ProgramCounterLowByte to the data bus.
    Read(                              From::NMI_VECTOR_LOW , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte , From::NMI_VECTOR_HIGH, &[]),
];

pub const BRK_STEPS: &'static [Step] = &[
    WriteField(ProgramCounterHighByte, To::TopOfStack, &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack, &[DecrementStackPointer]),
    WriteField(StatusForInstruction  , To::TopOfStack, &[DecrementStackPointer]),
    // Copy the new ProgramCounterLowByte to the data bus.
    Read(                              From::IRQ_VECTOR_LOW , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte,  From::IRQ_VECTOR_HIGH, &[]),
];

pub const RTI_STEPS: &'static [Step] = &[
    // Dummy read.
    Read(                             From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Status,                 From::TopOfStack, &[IncrementStackPointer]),
    Read(                             From::TopOfStack, &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack, &[]),
];

pub const RTS_STEPS: &'static [Step] = &[
    // Dummy read.
    Read(                             From::TopOfStack          , &[IncrementStackPointer]),
    // Read low byte of next program counter.
    Read(                             From::TopOfStack          , &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack          , &[]),
    // TODO: Make sure this dummy read is correct.
    Read(                             From::ProgramCounterTarget, &[IncrementProgramCounter]),
];

pub const PHA_STEPS: &'static [Step] = &[
    WriteField(Accumulator         , To::TopOfStack, &[DecrementStackPointer]),
];
pub const PHP_STEPS: &'static [Step] = &[
    WriteField(StatusForInstruction, To::TopOfStack, &[DecrementStackPointer]),
];

pub const PLA_STEPS: &'static [Step] = &[
    Read(                  From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Accumulator, From::TopOfStack, &[CheckNegativeAndZero]),
];

pub const PLP_STEPS: &'static [Step] = &[
    Read(             From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Status, From::TopOfStack, &[]),
];

pub const JSR_STEPS: &'static [Step] = &[
    // TODO: Verify this dummy read is correct. No Read nor Write is specified for this step in the
    // manual, but this matches the address bus location in the manual.
    Read(                              From::TopOfStack, &[StorePendingAddressLowByte]),
    WriteField(ProgramCounterHighByte, To::TopOfStack  , &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack  , &[DecrementStackPointer]),
    // Put the pending address high byte on the data bus.
    Read(                              From::ProgramCounterTarget       , &[]),
    Read(                              From::PendingProgramCounterTarget, &[StartNextInstruction, IncrementProgramCounter]),
];

pub const JMP_ABS_STEPS: &'static [Step] = &[
    ReadField(ProgramCounterHighByte, From::ProgramCounterTarget, &[]),
];

pub const JMP_IND_STEPS: &'static [Step] = &[
    // High byte of the index address.
    Read(From::ProgramCounterTarget       , &[StorePendingAddressLowByte]),
    // Low byte of the looked-up address.
    Read(From::PendingAddressTarget       , &[IncrementAddressBusLow]    ),
    // High byte of the looked-up address.
    Read(From::AddressBusTarget           , &[StorePendingAddressLowByte]),
    // Jump to next instruction.
    Read(From::PendingProgramCounterTarget, &[StartNextInstruction, IncrementProgramCounter]),
];

#[derive(Clone, Copy, Debug)]
pub enum Step {
    Read(From, &'static [CycleAction]),
    Write(To, &'static [CycleAction]),
    ReadField(Field, From, &'static [CycleAction]),
    WriteField(Field, To, &'static [CycleAction]),
}

impl Step {
    pub fn actions(&self) -> &'static [CycleAction] {
        match *self {
            Step::Read(_, actions) => actions,
            Step::Write(_, actions) => actions,
            Step::ReadField(_, _, actions) => actions,
            Step::WriteField(_, _, actions) => actions,
        }
    }

    pub fn has_interpret_op_code(&self) -> bool {
        for action in self.actions() {
            if matches!(action, CycleAction::InterpretOpCode) {
                return true;
            }
        }

        false
    }
}
