use crate::cpu::step_action::Field;
use crate::cpu::step_action::Field::*;
use crate::cpu::step_action::{From, To};
use crate::cpu::step_action::StepAction;
use crate::cpu::step_action::StepAction::*;
use crate::cpu::step::Step::{Read, ReadField, WriteField, OamRead, OamWrite, DmcRead};
use crate::memory::memory::Bus;

pub static OAM_READ_STEP: Step = OamRead(From::OamDmaAddressTarget, &[]);
pub static OAM_WRITE_STEP: Step = OamWrite(To::OAM_DATA, &[IncrementOamDmaAddress]);

pub static DMC_READ_STEP: Step = DmcRead(From::DmcDmaAddressTarget, &[SetDmcSampleBuffer]);

pub const READ_OP_CODE_STEP: Step =
    Read(From::ProgramCounterTarget, &[StartNextInstruction, IncrementPC]);

pub const OOPS_STEP: Step =
    ReadField(Field::Operand, From::ComputedTarget, &[]);

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
    ReadField(Operand, From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC, PollInterrupts]),
    // FIXME: Is this comment out of date, or is the configuration wrong?
    // Read the NEXT op code, execute the CURRENT op code.
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];
pub const RELATIVE_ADDRESSING_STEPS: &[Step] = &[
    ReadField(Operand, From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC, PollInterrupts]),
    Read(              From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ABSOLUTE_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[IncrementPC]),
    ReadField(Operand           , From::PendingAddressTarget, &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ZERO_PAGE_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(Operand          , From::PendingZeroPageTarget, &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ABSOLUTE_X_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementPC]),
    // TODO: Is this PollInterrupts too early if an Oops step occurs?
    ReadField(Operand           , From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ZERO_PAGE_X_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(Operand          , From::ComputedTarget       , &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ABSOLUTE_Y_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementPC]),
    ReadField(Operand           , From::PendingAddressTarget, &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
    Read(                         From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const ZERO_PAGE_Y_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress]),
    ReadField(Operand          , From::ComputedTarget       , &[PollInterrupts]),
    Read(                        From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const INDEXED_INDIRECT_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[]),
    ReadField(Operand           , From::PendingAddressTarget , &[PollInterrupts]),
    Read(                         From::ProgramCounterTarget , &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const INDIRECT_INDEXED_READ_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    ReadField(Operand           , From::PendingAddressTarget , &[MaybeInsertOopsStep, AddCarryToAddress, PollInterrupts]),
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
    ReadField(Operand           , From::PendingAddressTarget, &[]),
    WriteField(Operand          , To::PendingAddressTarget, &[ExecuteOpCode]),
    WriteField(Operand          , To::PendingAddressTarget, &[PollInterrupts]),
];

pub const ZERO_PAGE_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(Operand          , From::PendingZeroPageTarget, &[]),
    WriteField(Operand         , To::PendingZeroPageTarget  , &[ExecuteOpCode]),
    WriteField(Operand         , To::PendingZeroPageTarget  , &[PollInterrupts]),
];

pub const ABSOLUTE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[XOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    ReadField(Operand           , From::ComputedTarget      , &[]),
    WriteField(Operand          , To::ComputedTarget        , &[ExecuteOpCode]),
    WriteField(Operand          , To::ComputedTarget        , &[PollInterrupts]),
];

pub const ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(Operand          , From::ComputedTarget       , &[]),
    WriteField(Operand         , To::ComputedTarget         , &[ExecuteOpCode]),
    WriteField(Operand         , To::ComputedTarget         , &[PollInterrupts]),
];

pub const ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressHigh, From::ProgramCounterTarget, &[YOffsetPendingAddressLow, IncrementPC]),
    Read(                         From::PendingAddressTarget, &[AddCarryToAddress]),
    ReadField(Operand           , From::ComputedTarget      , &[]),
    WriteField(Operand          , To::ComputedTarget        , &[ExecuteOpCode]),
    WriteField(Operand          , To::ComputedTarget        , &[PollInterrupts]),
];

pub const ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow, From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                        From::PendingZeroPageTarget, &[YOffsetAddress]),
    ReadField(Operand          , From::ComputedTarget       , &[]),
    WriteField(Operand         , To::ComputedTarget         , &[ExecuteOpCode]),
    WriteField(Operand         , To::ComputedTarget         , &[PollInterrupts]),
];

pub const INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow,  From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    Read(                         From::PendingZeroPageTarget, &[XOffsetAddress]),
    ReadField(PendingAddressLow , From::ComputedTarget       , &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[]),
    ReadField(Operand           , From::PendingAddressTarget , &[]),
    WriteField(Operand          , To::PendingAddressTarget   , &[ExecuteOpCode]),
    WriteField(Operand          , To::PendingAddressTarget   , &[PollInterrupts]),
];

pub const INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS: &[Step] = &[
    ReadField(PendingAddressLow , From::ProgramCounterTarget , &[InterpretOpCode, IncrementPC]),
    ReadField(PendingAddressLow , From::PendingZeroPageTarget, &[IncrementAddressLow]),
    ReadField(PendingAddressHigh, From::ComputedTarget       , &[YOffsetPendingAddressLow]),
    Read(                         From::PendingAddressTarget , &[AddCarryToAddress]),
    ReadField(Operand           , From::ComputedTarget       , &[]),
    WriteField(Operand          , To::ComputedTarget         , &[ExecuteOpCode]),
    WriteField(Operand          , To::ComputedTarget         , &[PollInterrupts]),
];

pub const RESET_STEPS: &[Step] = &[
    Read(                             From::ProgramCounterTarget, &[IncrementPC]),
    Read(                             From::ProgramCounterTarget, &[]),
    // NES Manual: "read/write line is disabled so that no writes to stack are accomplished".
    // Hopefully switching the writes to reads here is what is actually intended.
    Read(                             From::TopOfStack, &[DecrementStackPointer]),
    Read(                             From::TopOfStack, &[DecrementStackPointer]),
    Read(                             From::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    ReadField(Operand               , From::InterruptVectorLow , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte, From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const BRK_STEPS: &[Step] = &[
    ReadField(Operand                , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
    WriteField(ProgramCounterHighByte, To::TopOfStack, &[DecrementStackPointer]),
    WriteField(ProgramCounterLowByte , To::TopOfStack, &[DecrementStackPointer]),
    WriteField(Status                , To::TopOfStack, &[DecrementStackPointer, SetInterruptVector]),
    ReadField(Operand                , From::InterruptVectorLow , &[DisableInterrupts]),
    ReadField(ProgramCounterHighByte , From::InterruptVectorHigh, &[ClearInterruptVector]),
];

pub const RTI_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    // Dummy read.
    Read(                             From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Status                , From::TopOfStack, &[IncrementStackPointer]),
    ReadField(Operand               , From::TopOfStack, &[IncrementStackPointer]),
    ReadField(ProgramCounterHighByte, From::TopOfStack, &[PollInterrupts]),
];

pub const RTS_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    // Dummy read.
    Read(                             From::TopOfStack          , &[IncrementStackPointer]),
    // Read low byte of next program counter.
    ReadField(Operand               , From::TopOfStack          , &[IncrementStackPointer]),
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
    ReadField(Operand              , From::TopOfStack          , &[PollInterrupts]),
    Read(From::ProgramCounterTarget, &[ExecuteOpCode, StartNextInstruction, IncrementPC]),
];

pub const PLP_STEPS: &[Step] = &[
    Read(From::ProgramCounterTarget, &[InterpretOpCode]),
    Read(From::TopOfStack          , &[IncrementStackPointer]),
    ReadField(Operand              , From::TopOfStack, &[PollInterrupts]),
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
    ReadField(Operand               , From::ProgramCounterTarget, &[InterpretOpCode, IncrementPC]),
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Step {
    Read(From, &'static [StepAction]),
    Write(To, &'static [StepAction]),
    ReadField(Field, From, &'static [StepAction]),
    WriteField(Field, To, &'static [StepAction]),
    OamRead(From, &'static [StepAction]),
    OamWrite(To, &'static [StepAction]),
    DmcRead(From, &'static [StepAction]),
}

impl Step {
    pub fn actions(&self) -> &'static [StepAction] {
        match *self {
            Step::Read(_, actions) => actions,
            Step::Write(_, actions) => actions,
            Step::ReadField(_, _, actions) => actions,
            Step::WriteField(_, _, actions) => actions,
            Step::OamRead(_, actions) => actions,
            Step::OamWrite(_, actions) => actions,
            Step::DmcRead(_, actions) => actions,
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
            Step::OamRead(from, _) => Step::OamRead(from, &[]),
            Step::OamWrite(to, _) => Step::OamWrite(to, &[]),
            Step::DmcRead(from, _) => Step::DmcRead(from, &[]),
        }
    }

    pub fn format_with_read_write_values(&self, bus: &Bus, value: u8) -> String {
        match *self {
            Step::Read(from, cycle_actions) =>
                format!("READ  [{}]=${value:02X}  {:22} -> {:^18} {cycle_actions:?}",
                    bus.cpu_pinout.address_bus, format!("{from:?}"), "(CPU address bus)"),
            Step::ReadField(field, from, cycle_actions) =>
                format!("READ  [{}]=${value:02X}  {:22} -> {:18} {cycle_actions:?}",
                    bus.cpu_pinout.address_bus, format!("{from:?}"), format!("{field:?}")),
            Step::Write(to, cycle_actions) =>
                format!("WRITE [{}]=${value:02X}  {:^22} -> {:18} {cycle_actions:?}",
                    bus.cpu_pinout.address_bus, "(CPU address bus)", format!("{to:?}")),
            Step::WriteField(field, to, cycle_actions) =>
                format!("WRITE [{}]=${value:02X}  {:22} -> {:18} {cycle_actions:?}",
                    bus.cpu_pinout.address_bus, format!("{field:?}"), format!("{to:?}")),
            Step::OamRead(from, cycle_actions) =>
                format!("OAMRD [{}]=${value:02X}  {:22} -> {:^18} {cycle_actions:?}",
                    bus.oam_dma_address_bus, format!("{from:?}"), "(OAM address bus)"),
            Step::OamWrite(to, cycle_actions) =>
                format!("OAMWR [{}]=${value:02X}  {:^22} -> {:18} {cycle_actions:?}",
                    bus.oam_dma_address_bus, "(OAM address bus)", format!("{to:?}")),
            Step::DmcRead(from, cycle_actions) =>
                format!("DMCRD [{}]=${value:02X}  {:22} -> {:^18} {cycle_actions:?}",
                    bus.dmc_dma_address_bus, format!("{from:?}"), "(DMC address bus)"),
        }
    }
}
