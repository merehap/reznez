use lazy_static::lazy_static;

use crate::cpu::cycle_action::Field;
use crate::cpu::cycle_action::Field::*;
use crate::cpu::cycle_action::{From, To};
use crate::cpu::cycle_action::CycleAction;
use crate::cpu::cycle_action::CycleAction::*;
use crate::cpu::step::Step::{Read, ReadField, Write, WriteField};

lazy_static! {
    pub static ref OAM_DMA_TRANSFER_STEPS: [Step; 512] = {
        let read_write = &[
            Read(From::DmaAddressTarget, &[]),
            Write(To::OAM_DATA, &[IncrementDmaAddress]),
        ];

        read_write.repeat(256).try_into().unwrap()
    };
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

pub const IMPLICIT_ADDRESSING_STEPS: &[Step] = &[
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[StartNextInstruction, ExecuteOpCode, IncrementProgramCounter]),
];
pub const IMMEDIATE_ADDRESSING_STEPS: &[Step] = &[
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[StartNextInstruction, ExecuteOpCode, IncrementProgramCounter]),
];
pub const RELATIVE_ADDRESSING_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_READ_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByte, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_READ_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_X_READ_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_X_READ_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_Y_READ_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_Y_READ_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[YOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDEXED_INDIRECT_READ_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByte]),
    Read(From::PendingAddressTarget , &[]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDIRECT_INDEXED_READ_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByteWithYOffset]),
    Read(From::PendingAddressTarget , &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Read(From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_WRITE_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByte, IncrementProgramCounter]),
    WriteField(OpRegister, To::PendingAddressTarget, &[]),
];

pub const ZERO_PAGE_WRITE_STEPS: &[Step] = &[
    WriteField(OpRegister, To::PendingZeroPageTarget, &[]),
];

pub const ABSOLUTE_X_WRITE_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ZERO_PAGE_X_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ABSOLUTE_Y_WRITE_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ZERO_PAGE_Y_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[YOffsetAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const INDEXED_INDIRECT_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByte]),
    WriteField(OpRegister, To::PendingAddressTarget, &[]),
];

pub const INDIRECT_INDEXED_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByteWithYOffset]),
    Read(From::PendingAddressTarget , &[AddCarryToAddressBus]),
    WriteField(OpRegister, To::AddressBusTarget, &[]),
];

pub const ABSOLUTE_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget , &[StorePendingAddressLowByte, IncrementProgramCounter]),
    Read(From::PendingAddressTarget , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const ZERO_PAGE_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const ABSOLUTE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    Read(From::AddressBusTarget    , &[]),
    Write(To::AddressBusTarget     , &[ExecuteOpCode]),
    Write(To::AddressBusTarget     , &[]),
];

pub const ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Read(From::PendingAddressTarget, &[AddCarryToAddressBus]),
    Read(From::AddressBusTarget    , &[]),
    Write(To::AddressBusTarget     , &[ExecuteOpCode]),
    Write(To::AddressBusTarget     , &[]),
];

pub const ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[YOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[XOffsetAddressBus]),
    Read(From::AddressBusTarget     , &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByte]),
    Read(From::PendingAddressTarget , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    Read(From::PendingZeroPageTarget, &[IncrementAddressBusLow]),
    Read(From::AddressBusTarget     , &[StorePendingAddressLowByteWithYOffset]),
    Read(From::PendingAddressTarget , &[AddCarryToAddressBus]),
    Read(From::AddressBusTarget     , &[]),
    Write(To::AddressBusTarget      , &[ExecuteOpCode]),
    Write(To::AddressBusTarget      , &[]),
];

pub const START_STEPS: &[Step] = &[
    Read(                              From::ProgramCounterTarget, &[IncrementProgramCounter]),
    Read(                              From::ProgramCounterTarget, &[]),
    // NES Manual: "read/write line is disabled so that no writes to stack are accomplished".
    // Hopefully switching the writes to reads here is what is actually intended.
    Read(                              From::TopOfStack, &[DecrementStackPointer]),
    Read(                              From::TopOfStack, &[DecrementStackPointer]),
    Read(                              From::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    // Copy the new ProgramCounterLowByte to the data bus.
    Read(                              From::InterruptVectorLow , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte , From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const NMI_STEPS: &[Step] = &[
    WriteField(ProgramCounterHighByte, To::TopOfStack, &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack, &[DecrementStackPointer]),
    WriteField(StatusForInterrupt    , To::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    // Copy the new ProgramCounterLowByte to the data bus.
    Read(                              From::InterruptVectorLow , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte , From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const IRQ_STEPS: &[Step] = &[
    WriteField(ProgramCounterHighByte, To::TopOfStack, &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack, &[DecrementStackPointer]),
    WriteField(StatusForInterrupt    , To::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    // Copy the new ProgramCounterLowByte to the data bus.
    Read(                              From::InterruptVectorLow , &[DisableInterrupts]),
    // TODO: Is ClearIrq supposed to be on the previous line? It was, then I moved it here for
    // consistency.
    ReadField(ProgramCounterHighByte , From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const BRK_STEPS: &[Step] = &[
    WriteField(ProgramCounterHighByte, To::TopOfStack, &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack, &[DecrementStackPointer]),
    WriteField(StatusForInstruction  , To::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    // Copy the new ProgramCounterLowByte to the data bus.
    Read(                              From::InterruptVectorLow , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte,  From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const RTI_STEPS: &[Step] = &[
    // Dummy read.
    Read(                             From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Status,                 From::TopOfStack, &[IncrementStackPointer]),
    Read(                             From::TopOfStack, &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack, &[]),
];

pub const RTS_STEPS: &[Step] = &[
    // Dummy read.
    Read(                             From::TopOfStack          , &[IncrementStackPointer]),
    // Read low byte of next program counter.
    Read(                             From::TopOfStack          , &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack          , &[]),
    // TODO: Make sure this dummy read is correct.
    Read(                             From::ProgramCounterTarget, &[IncrementProgramCounter]),
];

pub const PHA_STEPS: &[Step] = &[
    WriteField(Accumulator         , To::TopOfStack, &[DecrementStackPointer]),
];
pub const PHP_STEPS: &[Step] = &[
    WriteField(StatusForInstruction, To::TopOfStack, &[DecrementStackPointer]),
];

pub const PLA_STEPS: &[Step] = &[
    Read(From::TopOfStack          , &[IncrementStackPointer]),
    Read(From::TopOfStack          , &[]),
    // Note swapped order of StartNextInstruction and ExecuteOpCode.
    Read(From::ProgramCounterTarget, &[StartNextInstruction, ExecuteOpCode, IncrementProgramCounter]),
];

pub const PLP_STEPS: &[Step] = &[
    Read(             From::TopOfStack, &[IncrementStackPointer]),
    Read(From::TopOfStack, &[]),
    // Note swapped order of StartNextInstruction and ExecuteOpCode, necessary for IRQs.
    Read(From::ProgramCounterTarget, &[StartNextInstruction, ExecuteOpCode, IncrementProgramCounter]),
];

pub const JSR_STEPS: &[Step] = &[
    // TODO: Verify this dummy read is correct. No Read nor Write is specified for this step in the
    // manual, but this matches the address bus location in the manual.
    Read(                              From::TopOfStack, &[StorePendingAddressLowByte]),
    WriteField(ProgramCounterHighByte, To::TopOfStack  , &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack  , &[DecrementStackPointer]),
    // Put the pending address high byte on the data bus.
    Read(                              From::ProgramCounterTarget, &[]),
    Read(                              From::PendingAddressTarget, &[CopyAddressToPC, StartNextInstruction, IncrementProgramCounter]),
];

pub const JMP_ABS_STEPS: &[Step] = &[
    ReadField(ProgramCounterHighByte, From::ProgramCounterTarget, &[]),
];

pub const JMP_IND_STEPS: &[Step] = &[
    // High byte of the index address.
    Read(From::ProgramCounterTarget       , &[StorePendingAddressLowByte]),
    // Low byte of the looked-up address.
    Read(From::PendingAddressTarget       , &[IncrementAddressBusLow]    ),
    // High byte of the looked-up address.
    Read(From::AddressBusTarget           , &[StorePendingAddressLowByte]),
    // Jump to next instruction.
    Read(From::PendingAddressTarget       , &[CopyAddressToPC, StartNextInstruction, IncrementProgramCounter]),
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

    pub fn has_start_new_instruction(&self) -> bool {
        for action in self.actions() {
            if matches!(action, CycleAction::StartNextInstruction) {
                return true;
            }
        }

        false
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
