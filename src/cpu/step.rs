use lazy_static::lazy_static;

use crate::cpu::cycle_action::{From, To};
use crate::cpu::cycle_action::CycleAction;
use crate::cpu::cycle_action::CycleAction::*;
use crate::cpu::instruction::*;

lazy_static! {
    pub static ref INSTRUCTIONS: [CpuInstruction; 256] =
        INSTRUCTION_TEMPLATES.map(template_to_instruction);

    pub static ref OAM_DMA_TRANSFER_STEPS: [Step; 513] = {
        let mut steps = Vec::with_capacity(513);

        steps.push(Step::new(From::DataBus, To::DataBus, &[SetAddressBusToOamDmaStart]));
        for _ in 0..256 {
            steps.push(Step::new(From::AddressBusTarget, To::DataBus, &[]));
            steps.push(Step::new(From::DataBus         , To::OamData, &[IncrementAddressBus]));
        }

        steps.try_into().unwrap()
    };
}

fn template_to_instruction(template: InstructionTemplate) -> CpuInstruction {
    use AccessMode::*;
    use OpCode::*;
    let steps = match (template.access_mode, template.op_code, template.cycle_count as u8) {
        (Imp, BRK, _) => BRK_STEPS,
        (Imp, RTI, _) => RTI_STEPS,
        (Imp, RTS, _) => RTS_STEPS,
        (Imp, PHA, _) => PHA_STEPS,
        (Imp, PHP, _) => PHP_STEPS,
        (Imp, PLA, _) => PLA_STEPS,
        (Imp, PLP, _) => PLP_STEPS,
        (Abs, JSR, _) => JSR_STEPS,
        (Abs, JMP, _) => JMP_ABS_STEPS,
        (Ind, JMP, _) => JMP_IND_STEPS,

        (Imp,   _, _) => IMPLICIT_ADDRESSING_STEPS,
        (Imm,   _, _) => IMMEDIATE_ADDRESSING_STEPS,

        (_  ,   _, 2) => OTHER_2_STEPS,
        (_  ,   _, 3) => OTHER_3_STEPS,
        (_  ,   _, 4) => OTHER_4_STEPS,
        (_  ,   _, 5) => OTHER_5_STEPS,
        (_  ,   _, 6) => OTHER_6_STEPS,
        (_  ,   _, 7) => OTHER_7_STEPS,
        (_  ,   _, 8) => OTHER_8_STEPS,
        (_  ,   _, _) => unreachable!(),
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

pub const READ_AND_INTERPRET_OP_CODE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::NextOpCode            , &[IncrementProgramCounter]    ),
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[InterpretOpCode, IncrementProgramCounter]),
];

pub const IMPLICIT_ADDRESSING_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[ExecuteOpCode]),
];

pub const IMMEDIATE_ADDRESSING_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[ExecuteOpCode, IncrementProgramCounter]),
];

pub const NOP_STEP: Step =
    Step::new(From::DataBus                   , To::DataBus               , &[]                           );
pub const FULL_INSTRUCTION_STEP: Step =
    Step::new(From::DataBus                   , To::DataBus               , &[Instruction]                );

pub const NMI_STEPS: &'static [Step] = &[
    // FIXME: Fix first two steps in accordance to:
    // https://www.nesdev.org/wiki/CPU_interrupts#IRQ_and_NMI_tick-by-tick_execution
    Step::new(From::DataBus                   , To::DataBus               , &[]                           ),
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[]                           ),
    Step::new(From::ProgramCounterHighByte    , To::TopOfStack            , &[DecrementStackPointer]      ),
    Step::new(From::ProgramCounterLowByte     , To::TopOfStack            , &[DecrementStackPointer]      ),
    Step::new(From::StatusForInterrupt        , To::TopOfStack            , &[DecrementStackPointer]      ),
    // Copy the new ProgramCounterLowByte to the data bus.
    Step::new(From::NMI_VECTOR_LOW            , To::DataBus               , &[DisableInterrupts]          ),
    Step::new(From::NMI_VECTOR_HIGH           , To::ProgramCounterHighByte, &[]                           ),
];

pub const BRK_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterHighByte    , To::TopOfStack            , &[DecrementStackPointer]      ),
    Step::new(From::ProgramCounterLowByte     , To::TopOfStack            , &[DecrementStackPointer]      ),
    Step::new(From::StatusForInstruction      , To::TopOfStack            , &[DecrementStackPointer]      ),
    // Copy the new ProgramCounterLowByte to the data bus.
    Step::new(From::IRQ_VECTOR_LOW            , To::DataBus               , &[DisableInterrupts]          ),
    Step::new(From::IRQ_VECTOR_HIGH           , To::ProgramCounterHighByte, &[]                           ),
];

pub const RTI_STEPS: &'static [Step] = &[
    Step::new(From::TopOfStack                , To::DataBus               , &[IncrementStackPointer]      ),
    Step::new(From::TopOfStack                , To::Status                , &[IncrementStackPointer]      ),
    Step::new(From::TopOfStack                , To::DataBus               , &[IncrementStackPointer]      ),
    Step::new(From::TopOfStack                , To::ProgramCounterHighByte, &[]                           ),
];

pub const RTS_STEPS: &'static [Step] = &[
    // Dummy read.
    Step::new(From::TopOfStack                , To::DataBus                , &[IncrementStackPointer]     ),
    // Read low byte of next program counter.
    Step::new(From::TopOfStack                , To::DataBus                , &[IncrementStackPointer]     ),
    Step::new(From::TopOfStack                , To::ProgramCounterHighByte , &[]                          ),
    // TODO: Make sure this dummy read is correct.
    Step::new(From::ProgramCounterTarget      , To::DataBus                , &[IncrementProgramCounter]   ),
];

pub const PHA_STEPS: &'static [Step] = &[
    Step::new(From::Accumulator               , To::TopOfStack             , &[DecrementStackPointer]     ),
];

pub const PHP_STEPS: &'static [Step] = &[
    Step::new(From::StatusForInstruction      , To::TopOfStack             , &[DecrementStackPointer]     ),
];

pub const PLA_STEPS: &'static [Step] = &[
    Step::new(From::TopOfStack                , To::DataBus                , &[IncrementStackPointer]     ),
    Step::new(From::TopOfStack                , To::Accumulator            , &[CheckNegativeAndZero]      ),
];

pub const PLP_STEPS: &'static [Step] = &[
    Step::new(From::TopOfStack                , To::DataBus                , &[IncrementStackPointer]     ),
    Step::new(From::TopOfStack                , To::Status                 , &[]                          ),
];

pub const JSR_STEPS: &'static [Step] = &[
    Step::new(From::DataBus                    , To::DataBus               , &[StorePendingAddressLowByte]),
    Step::new(From::ProgramCounterHighByte     , To::TopOfStack            , &[DecrementStackPointer]     ),
    Step::new(From::ProgramCounterLowByte      , To::TopOfStack            , &[DecrementStackPointer]     ),
    // Put the pending address high byte on the data bus.
    Step::new(From::ProgramCounterTarget       , To::DataBus               , &[]                          ),
    Step::new(From::PendingProgramCounterTarget, To::NextOpCode            , &[IncrementProgramCounter]   ),
    Step::new(From::ProgramCounterTarget       , To::DataBus               , &[InterpretOpCode, IncrementProgramCounter]),
];

pub const JMP_ABS_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget       , To::ProgramCounterHighByte, &[]                          ),
];

pub const JMP_IND_STEPS: &'static [Step] = &[
    // High byte of the index address.
    Step::new(From::ProgramCounterTarget       , To::DataBus               , &[StorePendingAddressLowByte]),
    // Low byte of the looked-up address.
    Step::new(From::PendingAddressTarget       , To::DataBus               , &[IncrementAddressBusLow]    ),
    // High byte of the looked-up address.
    Step::new(From::AddressBusTarget           , To::DataBus               , &[StorePendingAddressLowByte]),
    // Jump to next instruction.
    Step::new(From::PendingProgramCounterTarget, To::NextOpCode            , &[IncrementProgramCounter]   ),
    Step::new(From::ProgramCounterTarget       , To::DataBus               , &[InterpretOpCode, IncrementProgramCounter]),
];

pub const OTHER_2_STEPS: &'static [Step] = &[
];

pub const OTHER_3_STEPS: &'static [Step] = &[
    FULL_INSTRUCTION_STEP,
];

pub const OTHER_4_STEPS: &'static [Step] = &[
    NOP_STEP,
    FULL_INSTRUCTION_STEP,
];

pub const OTHER_5_STEPS: &'static [Step] = &[
    NOP_STEP,
    NOP_STEP,
    FULL_INSTRUCTION_STEP,
];

pub const OTHER_6_STEPS: &'static [Step] = &[
    NOP_STEP,
    NOP_STEP,
    NOP_STEP,
    FULL_INSTRUCTION_STEP,
];

pub const OTHER_7_STEPS: &'static [Step] = &[
    NOP_STEP,
    NOP_STEP,
    NOP_STEP,
    NOP_STEP,
    FULL_INSTRUCTION_STEP,
];

pub const OTHER_8_STEPS: &'static [Step] = &[
    NOP_STEP,
    NOP_STEP,
    NOP_STEP,
    NOP_STEP,
    NOP_STEP,
    FULL_INSTRUCTION_STEP,
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
