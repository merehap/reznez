use std::sync::LazyLock;

use crate::cpu::step_action::Field;
use crate::cpu::step_action::Field::*;
use crate::cpu::step_action::{From, To};
use crate::cpu::step_action::StepAction;
use crate::cpu::step_action::StepAction::*;
use crate::cpu::step::Step::{Read, ReadField, Write, WriteField};
use crate::memory::mapper::CpuAddress;

pub static OAM_DMA_TRANSFER_STEPS: LazyLock<[Step; 512]> = LazyLock::new(|| {
    let read_write = &[
        Read(From::OamDmaAddressTarget, &[]),
        Write(To::OAM_DATA, &[IncrementOamDmaAddress]),
    ];

    read_write.repeat(256).try_into().unwrap()
});

pub static ALIGNED_OAM_DMA_TRANSFER_STEPS: LazyLock<[Step; 513]> = LazyLock::new(|| {
    let mut steps = Vec::new();
    steps.push(Read(From::AddressBusTarget, &[]));
    for _ in 0..256 {
        steps.push(Read(From::OamDmaAddressTarget, &[]));
        steps.push(Write(To::OAM_DATA, &[IncrementOamDmaAddress]));
    }

    steps.try_into().unwrap()
});

pub static DMC_DMA_TRANSFER_STEPS: &[Step] = &[
    // Dummy cycle.
    Read(From::AddressBusTarget, &[]),
    Read(From::DmcDmaAddressTarget, &[SetDmcSampleBuffer]),
];

pub static ALIGNED_DMC_DMA_TRANSFER_STEPS: &[Step] = &[
    // Dummy cycle.
    Read(From::AddressBusTarget, &[]),
    // Alignment cycle.
    Read(From::AddressBusTarget, &[]),
    Read(From::DmcDmaAddressTarget, &[SetDmcSampleBuffer]),
];

pub const READ_OP_CODE_STEP: Step =
    Read(From::ProgramCounterTarget, &[StartNextInstruction, IncrementProgramCounter]);

pub const OOPS_STEP: Step =
    ReadField(Field::Argument, From::ComputedTarget, &[]);

pub const BRANCH_TAKEN_STEP: Step =
    Read(From::ProgramCounterTarget,
        &[MaybeInsertBranchOopsStep, AddCarryToProgramCounter, StartNextInstruction, IncrementProgramCounter]);

pub const IMPLICIT_ADDRESSING_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode, PollInterrupts]),
    // FIXME: Is this comment out of date, or is the configuration wrong?
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];
pub const IMMEDIATE_ADDRESSING_STEPS: &[Step] = &[
    ReadField(Argument, From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter, PollInterrupts]),
    // FIXME: Is this comment out of date, or is the configuration wrong?
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];
pub const RELATIVE_ADDRESSING_STEPS: &[Step] = &[
    ReadField(Argument, From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter, PollInterrupts]),
    Read(               From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[IncrementProgramCounter]),
    ReadField(Argument          , From::PendingAddressTarget, &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(Argument         , From::PendingZeroPageTarget, &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_X_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementProgramCounter]),
    // TODO: Is this PollInterrupts too early if an Oops step occurs?
    ReadField(Argument          , From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_X_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(Argument         , From::ComputedTarget       , &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_Y_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementProgramCounter]),
    ReadField(Argument, From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(               From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_Y_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress]),
    ReadField(Argument         , From::ComputedTarget       , &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDEXED_INDIRECT_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[]),
    ReadField(Argument          , From::PendingAddressTarget , &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDIRECT_INDEXED_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    ReadField(Argument          , From::PendingAddressTarget , &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[IncrementProgramCounter, PollInterrupts]),
    WriteField(OpRegister, To::PendingAddressTarget, &[]),
];

pub const ZERO_PAGE_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter, PollInterrupts]),
    WriteField(OpRegister      , To::PendingZeroPageTarget , &[]),
];

pub const ABSOLUTE_X_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementProgramCounter]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress, PollInterrupts]),
    WriteField(OpRegister       , To::ComputedTarget        , &[]),
];

pub const ZERO_PAGE_X_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress, PollInterrupts]),
    WriteField(OpRegister      , To::ComputedTarget         , &[]),
];

pub const ABSOLUTE_Y_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementProgramCounter]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress, PollInterrupts]),
    WriteField(OpRegister,        To::ComputedTarget        , &[]),
];

pub const ZERO_PAGE_Y_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress, PollInterrupts]),
    WriteField(OpRegister,       To::ComputedTarget         , &[]),
];

pub const INDEXED_INDIRECT_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[PollInterrupts]),
    WriteField(OpRegister       , To::PendingAddressTarget   , &[]),
];

pub const INDIRECT_INDEXED_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    Read(                         From::PendingAddressTarget , &[AddCarryToAddress, PollInterrupts]),
    WriteField(OpRegister       , To::ComputedTarget         , &[]),
];

pub const ABSOLUTE_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[IncrementProgramCounter]),
    Read(                         From::PendingAddressTarget, &[]),
    // TODO: Should PollInterrupts be on the previous step instead?
    Write(To::PendingAddressTarget, &[ExecuteOpCode, PollInterrupts]),
    Write(To::PendingAddressTarget, &[]),
];

pub const ZERO_PAGE_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                        From::PendingZeroPageTarget, &[]),
    Write(                       To::PendingZeroPageTarget  , &[ExecuteOpCode, PollInterrupts]),
    Write(                       To::PendingZeroPageTarget  , &[]),
];

pub const ABSOLUTE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementProgramCounter]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    Read(                         From::ComputedTarget      , &[]),
    Write(                        To::ComputedTarget        , &[ExecuteOpCode, PollInterrupts]),
    Write(                        To::ComputedTarget        , &[]),
];

pub const ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress]),
    Read(                        From::ComputedTarget       , &[]),
    Write(                       To::ComputedTarget         , &[ExecuteOpCode, PollInterrupts]),
    Write(                       To::ComputedTarget         , &[]),
];

pub const ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementProgramCounter]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    Read(                         From::ComputedTarget      , &[]),
    Write(                        To::ComputedTarget        , &[ExecuteOpCode, PollInterrupts]),
    Write(                        To::ComputedTarget        , &[]),
];

pub const ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress]),
    Read(                        From::ComputedTarget       , &[]),
    Write(                       To::ComputedTarget         , &[ExecuteOpCode, PollInterrupts]),
    Write(                       To::ComputedTarget         , &[]),
];

pub const INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow,  From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[]),
    Read(                         From::PendingAddressTarget , &[]),
    Write(                        To::PendingAddressTarget   , &[ExecuteOpCode, PollInterrupts]),
    Write(                        To::PendingAddressTarget   , &[]),
];

pub const INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    Read(                         From::PendingAddressTarget , &[AddCarryToAddress]),
    Read(                         From::ComputedTarget       , &[]),
    Write(                        To::ComputedTarget         , &[ExecuteOpCode, PollInterrupts]),
    Write(                        To::ComputedTarget         , &[]),
];

pub const RESET_STEPS: &[Step] = &[
    Read(                              From::ProgramCounterTarget, &[IncrementProgramCounter]),
    Read(                              From::ProgramCounterTarget, &[]),
    // NES Manual: "read/write line is disabled so that no writes to stack are accomplished".
    // Hopefully switching the writes to reads here is what is actually intended.
    Read(                              From::TopOfStack, &[DecrementStackPointer]),
    Read(                              From::TopOfStack, &[DecrementStackPointer]),
    Read(                              From::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    ReadField(Argument,                From::InterruptVectorLow , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte , From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const BRK_STEPS: &[Step] = &[
    ReadField(Argument,                From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    WriteField(ProgramCounterHighByte, To::TopOfStack, &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack, &[DecrementStackPointer]),
    WriteField(Status                , To::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    ReadField(Argument               , From::InterruptVectorLow , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte,  From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const RTI_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    // Dummy read.
    Read(                             From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Status,                 From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Argument,               From::TopOfStack, &[IncrementStackPointer, PollInterrupts]),
    ReadField(ProgramCounterHighByte, From::TopOfStack, &[]),
];

pub const RTS_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    // Dummy read.
    Read(                             From::TopOfStack          , &[IncrementStackPointer]),
    // Read low byte of next program counter.
    ReadField(Argument,               From::TopOfStack          , &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack          , &[PollInterrupts]),
    // TODO: Make sure this dummy read is correct.
    Read(                             From::ProgramCounterTarget, &[IncrementProgramCounter]),
];

pub const PHA_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode, PollInterrupts]),
    WriteField(Accumulator         , To::TopOfStack, &[DecrementStackPointer]),
];
pub const PHP_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode, PollInterrupts]),
    WriteField(Status              , To::TopOfStack, &[DecrementStackPointer]),
];

pub const PLA_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    Read(From::TopOfStack          , &[IncrementStackPointer]),
    ReadField(Argument, From::TopOfStack          , &[PollInterrupts]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const PLP_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    Read(From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Argument, From::TopOfStack, &[PollInterrupts]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const JSR_STEPS: &[Step] = &[
    ReadField(PendingAddressLow      , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    // TODO: Verify this dummy read is correct. No Read nor Write is specified for this step in the
    // manual, but this matches the address bus location in the manual.
    Read(                              From::TopOfStack, &[]),
    WriteField(ProgramCounterHighByte, To::TopOfStack,   &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack,   &[DecrementStackPointer]),
    ReadField(PendingAddressHigh     , From::ProgramCounterTarget, &[PollInterrupts]),
    Read(                              From::PendingAddressTarget, &[CopyAddressToPC, StartNextInstruction, IncrementProgramCounter]),
];

pub const JMP_ABS_STEPS: &[Step] = &[
    ReadField(Argument, From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    // For some reason, JMP seems to poll interrupts on the last step. scanline.nes fails otherwise.
    ReadField(ProgramCounterHighByte, From::ProgramCounterTarget, &[PollInterrupts]),
];

pub const JMP_IND_STEPS: &[Step] = &[
    // Low byte of the index address.
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    // High byte of the index address.
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[]),
    // Low byte of the looked-up address.
    ReadField(PendingAddressLow , From::PendingAddressTarget, &[IncrementAddressLow]),
    // High byte of the looked-up address.
    ReadField(PendingAddressHigh, From::ComputedTarget      , &[PollInterrupts]),
    // Jump to next instruction.
    Read(                         From::PendingAddressTarget, &[CopyAddressToPC, StartNextInstruction, IncrementProgramCounter]),
];

// FIXME: These certainly aren't the real AHX steps. Somehow AHX must take 5 cycles but is
// classified as absolute y (and is a write operation which generally don't take 5 cycles).
pub const ABSOLUTE_Y_AHX_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementProgramCounter]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress, PollInterrupts]),
    // Hackily add an extra cycle.
    Read(                         From::ComputedTarget      , &[]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

// FIXME: These certainly aren't the real AHX steps. Somehow AHX must take 6 cycles but is
// classified as INDIRECT_INDEXED (and is a write operation which generally don't take 6 cycles).
pub const INDIRECT_INDEXED_AHX_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    // Hackily add an extra cycle.
    Read(                         From::ComputedTarget       , &[]),
    Read(                         From::PendingAddressTarget , &[AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

// Same as Absolute Y Read Steps, except for some reason the 'oops' cycle is always taken.
pub const TAS_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementProgramCounter]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementProgramCounter]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress, PollInterrupts]),
    Read(                         From::ComputedTarget      , &[]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Step {
    Read(From, &'static [StepAction]),
    Write(To, &'static [StepAction]),
    ReadField(Field, From, &'static [StepAction]),
    WriteField(Field, To, &'static [StepAction]),
}

impl Step {
    pub fn actions(&self) -> &'static [StepAction] {
        match *self {
            Step::Read(_, actions) => actions,
            Step::Write(_, actions) => actions,
            Step::ReadField(_, _, actions) => actions,
            Step::WriteField(_, _, actions) => actions,
        }
    }

    pub fn has_start_new_instruction(&self) -> bool {
        for action in self.actions() {
            if matches!(action, StepAction::StartNextInstruction) {
                return true;
            }
        }

        false
    }

    pub fn has_interpret_op_code(&self) -> bool {
        for action in self.actions() {
            if matches!(action, StepAction::InterpretOpCode) {
                return true;
            }
        }

        false
    }

    pub fn is_read(&self) -> bool {
        matches!(&self, Step::Read(..) | Step::ReadField(..))
    }

    pub fn with_actions_removed(&self) -> Step {
        match *self {
            Step::Read(from, _) => Step::Read(from, &[]),
            Step::Write(to, _) => Step::Write(to, &[]),
            Step::ReadField(_field, from, _) => Step::Read(from, &[]),
            Step::WriteField(_field, to, _) => Step::Write(to, &[]),
        }
    }

    pub fn format_with_bus_values(&self, address_bus: CpuAddress, data_bus: u8) -> String {
        let address_bus = address_bus.to_mesen_string();
        match *self {
            Step::Read(from, cycle_actions) =>
                format!("READ  [{address_bus}]=${data_bus:02X}  {:21} -> {:^18} {cycle_actions:?}",
                    format!("{from:?}"), "(data bus)"),
            Step::ReadField(field, from, cycle_actions) =>
                format!("READ  [{address_bus}]=${data_bus:02X}  {:21} -> {:18} {cycle_actions:?}",
                    format!("{from:?}"), format!("{field:?}")),
            Step::Write(to, cycle_actions) =>
                format!("WRITE [{address_bus}]=${data_bus:02X}  {:^21} -> {:18} {cycle_actions:?}",
                    "(data bus)", format!("{to:?}")),
            Step::WriteField(field, to, cycle_actions) =>
                format!("WRITE [{address_bus}]=${data_bus:02X}  {:21} -> {:18} {cycle_actions:?}",
                    format!("{field:?}"), format!("{to:?}")),
        }
    }
}
