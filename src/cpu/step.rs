use crate::cpu::cycle_action::{From, To};
use crate::cpu::cycle_action::CycleAction;
use crate::cpu::cycle_action::CycleAction::*;

pub const READ_INSTRUCTION_STEP: Step =
    Step::new(From::ProgramCounterTarget, To::Instruction, &[IncrementProgramCounter]);
pub const NOP_STEP: Step =
    Step::new(From::DataBus, To::DataBus, &[]);
pub const OAM_DMA_START_TRANSFER_STEP: Step =
    Step::new(From::DataBus, To::DataBus, &[SetAddressBusToOamDmaStart]);
pub const OAM_DMA_READ_STEP: Step =
    Step::new(From::AddressBusTarget, To::DataBus, &[]);
pub const OAM_DMA_WRITE_STEP: Step =
    Step::new(From::DataBus, To::OamData, &[IncrementAddressBus]);
pub const PENDING_ADDRESS_LOW_BYTE_STEP: Step =
    Step::new(From::ProgramCounterTarget, To::DataBus, &[IncrementProgramCounter]);
pub const PENDING_ADDRESS_HIGH_BYTE_STEP: Step =
    Step::new(From::ProgramCounterTarget, To::PendingAddressHighByte, &[IncrementProgramCounter]);
pub const FULL_INSTRUCTION_STEP: Step =
    Step::new(From::DataBus, To::DataBus, &[CycleAction::Instruction]);
pub const INSTRUCTION_RETURN_STEP: Step =
    Step::new(From::DataBus, To::DataBus, &[InstructionReturn]);

pub const NMI_STEPS: &'static [Step] = &[
    // FIXME: Fix first two steps in accordance to:
    // https://www.nesdev.org/wiki/CPU_interrupts#IRQ_and_NMI_tick-by-tick_execution
    Step::new(From::DataBus               , To::DataBus               , &[]                     ),
    Step::new(From::ProgramCounterTarget  , To::DataBus               , &[]                     ),
    Step::new(From::ProgramCounterHighByte, To::TopOfStack            , &[DecrementStackPointer]),
    Step::new(From::ProgramCounterLowByte , To::TopOfStack            , &[DecrementStackPointer]),
    Step::new(From::StatusForInterrupt    , To::TopOfStack            , &[DecrementStackPointer]),
    // Copy the new ProgramCounterLowByte to the data bus.
    Step::new(From::NMI_VECTOR_LOW        , To::DataBus               , &[DisableInterrupts]    ),
    Step::new(From::NMI_VECTOR_HIGH       , To::ProgramCounterHighByte, &[]                     ),
];

pub const BRK_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget  , To::DataBus               , &[IncrementProgramCounter]),
    Step::new(From::ProgramCounterHighByte, To::TopOfStack            , &[DecrementStackPointer]  ),
    Step::new(From::ProgramCounterLowByte , To::TopOfStack            , &[DecrementStackPointer]  ),
    Step::new(From::StatusForInstruction  , To::TopOfStack            , &[DecrementStackPointer]  ),
    // Copy the new ProgramCounterLowByte to the data bus.
    Step::new(From::IRQ_VECTOR_LOW        , To::DataBus               , &[DisableInterrupts]      ),
    Step::new(From::IRQ_VECTOR_HIGH       , To::ProgramCounterHighByte, &[]                       ),
];

pub const RTI_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget, To::DataBus               , &[]                     ),
    Step::new(From::TopOfStack          , To::DataBus               , &[IncrementStackPointer]),
    Step::new(From::TopOfStack          , To::Status                , &[IncrementStackPointer]),
    Step::new(From::TopOfStack          , To::DataBus               , &[IncrementStackPointer]),
    Step::new(From::TopOfStack          , To::ProgramCounterHighByte, &[]                     ),
];

pub const RTS_STEPS: &'static [Step] = &[
    // Dummy read.
    Step::new(From::ProgramCounterTarget, To::DataBus               , &[]                       ),
    // Dummy read.
    Step::new(From::TopOfStack          , To::DataBus               , &[IncrementStackPointer]  ),
    // Read low byte of next program counter.
    Step::new(From::TopOfStack          , To::DataBus               , &[IncrementStackPointer]  ),
    Step::new(From::TopOfStack          , To::ProgramCounterHighByte, &[]                       ),
    // TODO: Make sure this dummy read is correct.
    Step::new(From::ProgramCounterTarget, To::DataBus               , &[IncrementProgramCounter]),
];

pub const PHA_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget, To::DataBus   , &[]                     ),
    Step::new(From::Accumulator         , To::TopOfStack, &[DecrementStackPointer]),
];

pub const PHP_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget, To::DataBus   , &[]                     ),
    Step::new(From::StatusForInstruction, To::TopOfStack, &[DecrementStackPointer]),
];

pub const PLA_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget, To::DataBus    , &[]),
    Step::new(From::TopOfStack          , To::DataBus    , &[IncrementStackPointer]),
    Step::new(From::TopOfStack          , To::Accumulator, &[CheckNegativeAndZero]),
];

pub const PLP_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget, To::DataBus, &[]),
    Step::new(From::TopOfStack          , To::DataBus, &[IncrementStackPointer]),
    Step::new(From::TopOfStack          , To::Status , &[]),
];

pub const JSR_STEPS: &'static [Step] = &[
    // Put the pending address low byte on the data bus.
    Step::new(From::ProgramCounterTarget       , To::DataBus    , &[IncrementProgramCounter]   ),
    Step::new(From::DataBus                    , To::DataBus    , &[StorePendingAddressLowByte]),
    Step::new(From::ProgramCounterHighByte     , To::TopOfStack , &[DecrementStackPointer]     ),
    Step::new(From::ProgramCounterLowByte      , To::TopOfStack , &[DecrementStackPointer]     ),
    // Put the pending address high byte on the data bus.
    Step::new(From::ProgramCounterTarget       , To::DataBus    , &[]                          ),
    Step::new(From::PendingProgramCounterTarget, To::Instruction, &[IncrementProgramCounter]   ),
];

pub const JMP_ABS_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget, To::DataBus               , &[IncrementProgramCounter]),
    Step::new(From::ProgramCounterTarget, To::ProgramCounterHighByte, &[]                       ),
];

pub const JMP_IND_STEPS: &'static [Step] = &[
    // Low byte of the index address.
    Step::new(From::ProgramCounterTarget       , To::DataBus    , &[IncrementProgramCounter]   ),
    // High byte of the index address.
    Step::new(From::ProgramCounterTarget       , To::DataBus    , &[StorePendingAddressLowByte]),
    // Low byte of the looked-up address.
    Step::new(From::PendingAddressTarget       , To::DataBus    , &[IncrementAddressBusLow]    ),
    // High byte of the looked-up address.
    Step::new(From::AddressBusTarget           , To::DataBus    , &[StorePendingAddressLowByte]),
    // Jump to next instruction.
    Step::new(From::PendingProgramCounterTarget, To::Instruction, &[IncrementProgramCounter]   ),
];

#[derive(Clone, Debug)]
pub struct Step {
    from: From,
    to: To,
    actions: &'static [CycleAction],
}

impl Step {
    pub const fn new(from: From, to: To, actions: &'static [CycleAction]) -> Step {
        Step { from, to, actions }
    }

    pub fn from(&self) -> From {
        self.from
    }

    pub fn to(&self) -> To {
        self.to
    }

    pub fn actions(&self) -> &'static [CycleAction] {
        &self.actions
    }
}
