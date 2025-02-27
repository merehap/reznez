use crate::cpu::step_action::Field;
use crate::cpu::step_action::Field::*;
use crate::cpu::step_action::{From, To};
use crate::cpu::step_action::StepAction;
use crate::cpu::step_action::StepAction::*;
use crate::cpu::step::Step::{Read, ReadField, Write, WriteField};
use crate::memory::mapper::CpuAddress;

pub static OAM_READ_STEP: Step = Read(From::OamDmaAddressTarget, &[]);
pub static OAM_WRITE_STEP: Step = Write(To::OAM_DATA, &[IncrementOamDmaAddress]);

pub static DMC_READ_STEP: Step = Read(From::DmcDmaAddressTarget, &[SetDmcSampleBuffer]);

pub const READ_OP_CODE_STEP: Step =
    Read(From::ProgramCounterTarget, &[StartNextInstruction, IncrementPC]);

pub const OOPS_STEP: Step =
    ReadField(Field::Argument, From::ComputedTarget, &[]);

pub const BRANCH_TAKEN_STEP: Step =
    // TODO: Double check that MaybePollInterrupts shouldn't be a cycle earlier.
    Read(From::ProgramCounterTarget,
        &[MaybeInsertBranchOopsStep, MaybePollInterrupts, AddCarryToPC, StartNextInstruction, IncrementPC]);

pub const IMPLICIT_ADDRESSING_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode, PollInterrupts]),
    // FIXME: Is this comment out of date, or is the configuration wrong?
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];
pub const IMMEDIATE_ADDRESSING_STEPS: &[Step] = &[
    ReadField(Argument, From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC, PollInterrupts]),
    // FIXME: Is this comment out of date, or is the configuration wrong?
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];
pub const RELATIVE_ADDRESSING_STEPS: &[Step] = &[
    ReadField(Argument, From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC, PollInterrupts]),
    Read(               From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ABSOLUTE_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[IncrementPC]),
    ReadField(Argument          , From::PendingAddressTarget, &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ZERO_PAGE_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(Argument         , From::PendingZeroPageTarget, &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ABSOLUTE_X_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementPC]),
    // TODO: Is this PollInterrupts too early if an Oops step occurs?
    ReadField(Argument          , From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ZERO_PAGE_X_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(Argument         , From::ComputedTarget       , &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ABSOLUTE_Y_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementPC]),
    ReadField(Argument, From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(               From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ZERO_PAGE_Y_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress]),
    ReadField(Argument         , From::ComputedTarget       , &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const INDEXED_INDIRECT_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[]),
    ReadField(Argument          , From::PendingAddressTarget , &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const INDIRECT_INDEXED_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    ReadField(Argument          , From::PendingAddressTarget , &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ABSOLUTE_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[IncrementPC]),
    WriteField(OpRegister, To::PendingAddressTarget, &[PollInterrupts]),
];

pub const ZERO_PAGE_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    WriteField(OpRegister      , To::PendingZeroPageTarget , &[PollInterrupts]),
];

pub const ABSOLUTE_X_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    WriteField(OpRegister       , To::ComputedTarget        , &[PollInterrupts]),
];

pub const ZERO_PAGE_X_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress]),
    WriteField(OpRegister      , To::ComputedTarget         , &[PollInterrupts]),
];

pub const ABSOLUTE_Y_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    WriteField(OpRegister,        To::ComputedTarget        , &[PollInterrupts]),
];

pub const ZERO_PAGE_Y_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress]),
    WriteField(OpRegister,       To::ComputedTarget         , &[PollInterrupts]),
];

pub const INDEXED_INDIRECT_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[]),
    WriteField(OpRegister       , To::PendingAddressTarget   , &[PollInterrupts]),
];

pub const INDIRECT_INDEXED_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    Read(                         From::PendingAddressTarget , &[AddCarryToAddress]),
    WriteField(OpRegister       , To::ComputedTarget         , &[PollInterrupts]),
];

pub const ABSOLUTE_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[IncrementPC]),
    Read(                         From::PendingAddressTarget, &[]),
    Write(To::PendingAddressTarget, &[ExecuteOpCode]),
    Write(To::PendingAddressTarget, &[PollInterrupts]),
];

pub const ZERO_PAGE_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[]),
    Write(                       To::PendingZeroPageTarget  , &[ExecuteOpCode]),
    Write(                       To::PendingZeroPageTarget  , &[PollInterrupts]),
];

pub const ABSOLUTE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    Read(                         From::ComputedTarget      , &[]),
    Write(                        To::ComputedTarget        , &[ExecuteOpCode]),
    Write(                        To::ComputedTarget        , &[PollInterrupts]),
];

pub const ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress]),
    Read(                        From::ComputedTarget       , &[]),
    Write(                       To::ComputedTarget         , &[ExecuteOpCode]),
    Write(                       To::ComputedTarget         , &[PollInterrupts]),
];

pub const ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    Read(                         From::ComputedTarget      , &[]),
    Write(                        To::ComputedTarget        , &[ExecuteOpCode]),
    Write(                        To::ComputedTarget        , &[PollInterrupts]),
];

pub const ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress]),
    Read(                        From::ComputedTarget       , &[]),
    Write(                       To::ComputedTarget         , &[ExecuteOpCode]),
    Write(                       To::ComputedTarget         , &[PollInterrupts]),
];

pub const INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow,  From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[]),
    Read(                         From::PendingAddressTarget , &[]),
    Write(                        To::PendingAddressTarget   , &[ExecuteOpCode]),
    Write(                        To::PendingAddressTarget   , &[PollInterrupts]),
];

pub const INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    Read(                         From::PendingAddressTarget , &[AddCarryToAddress]),
    Read(                         From::ComputedTarget       , &[]),
    Write(                        To::ComputedTarget         , &[ExecuteOpCode]),
    Write(                        To::ComputedTarget         , &[PollInterrupts]),
];

pub const RESET_STEPS: &[Step] = &[
    Read(                              From::ProgramCounterTarget, &[IncrementPC]),
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
    ReadField(Argument,                From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
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
    ReadField(Argument,               From::TopOfStack, &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack, &[PollInterrupts]),
];

pub const RTS_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    // Dummy read.
    Read(                             From::TopOfStack          , &[IncrementStackPointer]),
    // Read low byte of next program counter.
    ReadField(Argument,               From::TopOfStack          , &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack          , &[]),
    // TODO: Make sure this dummy read is correct.
    Read(                             From::ProgramCounterTarget, &[IncrementPC, PollInterrupts]),
];

pub const PHA_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode, PollInterrupts]),
    WriteField(Accumulator         , To::TopOfStack, &[DecrementStackPointer, PollInterrupts]),
];
pub const PHP_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    WriteField(Status              , To::TopOfStack, &[DecrementStackPointer, PollInterrupts]),
];

pub const PLA_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    Read(From::TopOfStack          , &[IncrementStackPointer]),
    ReadField(Argument, From::TopOfStack          , &[PollInterrupts]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const PLP_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    Read(From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Argument, From::TopOfStack, &[PollInterrupts]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const JSR_STEPS: &[Step] = &[
    ReadField(PendingAddressLow      , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    // TODO: Verify this dummy read is correct. No Read nor Write is specified for this step in the
    // manual, but this matches the address bus location in the manual.
    Read(                              From::TopOfStack, &[]),
    WriteField(ProgramCounterHighByte, To::TopOfStack,   &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack,   &[DecrementStackPointer]),
    ReadField(PendingAddressHigh     , From::ProgramCounterTarget, &[PollInterrupts]),
    Read(                              From::PendingAddressTarget, &[CopyAddressToPC, StartNextInstruction, IncrementPC]),
];

pub const JMP_ABS_STEPS: &[Step] = &[
    ReadField(Argument, From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(ProgramCounterHighByte, From::ProgramCounterTarget, &[PollInterrupts]),
];

pub const JMP_IND_STEPS: &[Step] = &[
    // Low byte of the index address.
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    // High byte of the index address.
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[]),
    // Low byte of the looked-up address.
    ReadField(PendingAddressLow , From::PendingAddressTarget, &[IncrementAddressLow]),
    // High byte of the looked-up address.
    ReadField(PendingAddressHigh, From::ComputedTarget      , &[PollInterrupts]),
    // Jump to next instruction.
    Read(                         From::PendingAddressTarget, &[CopyAddressToPC, StartNextInstruction, IncrementPC]),
];

// FIXME: These certainly aren't the real AHX steps. Somehow AHX must take 5 cycles but is
// classified as absolute y (and is a write operation which generally don't take 5 cycles).
pub const ABSOLUTE_Y_AHX_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    // Hackily add an extra cycle.
    Read(                         From::ComputedTarget      , &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

// FIXME: These certainly aren't the real AHX steps. Somehow AHX must take 6 cycles but is
// classified as INDIRECT_INDEXED (and is a write operation which generally don't take 6 cycles).
pub const INDIRECT_INDEXED_AHX_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    // Hackily add an extra cycle.
    Read(                         From::ComputedTarget       , &[]),
    Read(                         From::PendingAddressTarget , &[AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

// Same as Absolute Y Read Steps, except for some reason the 'oops' cycle is always taken.
pub const TAS_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    Read(                         From::ComputedTarget      , &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
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
        match *self {
            Step::Read(from, cycle_actions) =>
                format!("READ  [{address_bus}]=${data_bus:02X}  {:22} -> {:^18} {cycle_actions:?}",
                    format!("{from:?}"), "(bus)"),
            Step::ReadField(field, from, cycle_actions) =>
                format!("READ  [{address_bus}]=${data_bus:02X}  {:22} -> {:18} {cycle_actions:?}",
                    format!("{from:?}"), format!("{field:?}")),
            Step::Write(to, cycle_actions) =>
                format!("WRITE [{address_bus}]=${data_bus:02X}  {:^22} -> {:18} {cycle_actions:?}",
                    "(bus)", format!("{to:?}")),
            Step::WriteField(field, to, cycle_actions) =>
                format!("WRITE [{address_bus}]=${data_bus:02X}  {:22} -> {:18} {cycle_actions:?}",
                    format!("{field:?}"), format!("{to:?}")),
        }
    }
}
