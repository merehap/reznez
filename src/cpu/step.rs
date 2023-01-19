use lazy_static::lazy_static;

use crate::cpu::cycle_action::{From, To};
use crate::cpu::cycle_action::CycleAction;
use crate::cpu::cycle_action::CycleAction::*;
use crate::cpu::instruction::*;

lazy_static! {
    pub static ref INSTRUCTIONS: [CpuInstruction; 256] =
        INSTRUCTION_TEMPLATES.map(template_to_instruction);

    pub static ref OAM_DMA_TRANSFER_STEPS: [Step; 512] = {
        let read_write = &[
            Step::new(From::AddressBusTarget, To::DataBus, &[]),
            Step::new(From::DataBus         , To::OamData, &[IncrementAddressBus]),
        ];

        read_write.repeat(256).try_into().unwrap()
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
        (Rel,   _, _) => RELATIVE_ADDRESSING_STEPS,

        // Read operations.
        (Abs, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | BIT | LAX | NOP, _) => ABSOLUTE_READ_STEPS,
        (ZP , LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | BIT | LAX | NOP, _) => ZERO_PAGE_READ_STEPS,
        // TODO: Remove the unused combos.
        (AbX, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | BIT | LAX | NOP | /*LAE |*/ TAS, _) => ABSOLUTE_X_READ_STEPS,
        (AbY, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | BIT | LAX | NOP | /*LAE |*/ TAS, _) => ABSOLUTE_Y_READ_STEPS,
        (IzX, LDA |             EOR | AND | ORA | ADC | SBC | CMP |       LAX, _) => INDEXED_INDIRECT_READ_STEPS,
        (IzY, LDA |             EOR | AND | ORA | ADC | SBC | CMP            , _) => INDIRECT_INDEXED_READ_STEPS,

        // Write operations.
        (Abs, STA | STX | STY | SAX, _) => ABSOLUTE_WRITE_STEPS,
        (ZP , STA | STX | STY | SAX, _) => ZERO_PAGE_WRITE_STEPS,
        // TODO: Remove the unused combos.
        (AbX, STA | STX | STY      , _) => ABSOLUTE_X_WRITE_STEPS,
        (AbY, STA | STX | STY      , _) => ABSOLUTE_Y_WRITE_STEPS,
        (IzX, STA |             SAX, _) => INDEXED_INDIRECT_WRITE_STEPS,
        (IzY, STA /*| SHA*/        , _) => INDIRECT_INDEXED_WRITE_STEPS,

        // Read-modify-write operations.
        (Abs, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP, _) => ABSOLUTE_READ_MODIFY_WRITE_STEPS,
        (ZP , ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP, _) => ZERO_PAGE_READ_MODIFY_WRITE_STEPS,
        // TODO: Remove the unused combos.
        (AbX, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP, _) => ABSOLUTE_X_READ_MODIFY_WRITE_STEPS,
        (AbY, ASL | LSR | ROL | ROR | INC | DEC | SLO | SRE | RLA | RRA | ISC | DCP, _) => ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS,
        (IzX,                                     SLO | SRE | RLA | RRA | ISC | DCP, _) => INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS,
        (IzY,                                     SLO | SRE | RLA | RRA | ISC | DCP, _) => INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS,

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

pub const READ_OP_CODE_STEP: Step =
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[StartNextInstruction, IncrementProgramCounter]    );

pub const INTERPRET_OP_CODE_STEP: Step =
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[InterpretOpCode, IncrementProgramCounter]);

pub const ADDRESS_BUS_READ_STEP: Step =
    Step::new(From::AddressBusTarget          , To::DataBus               , &[]);

pub const BRANCH_TAKEN_STEP: Step =
    Step::new(From::ProgramCounterTarget      , To::DataBus,
        &[MaybeInsertBranchOopsStep, AddCarryToProgramCounter, StartNextInstruction, IncrementProgramCounter]);

pub const IMPLICIT_ADDRESSING_STEPS: &'static [Step] = &[
    // Read the NEXT op code, execute the CURRENT op code.
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];
pub const IMMEDIATE_ADDRESSING_STEPS: &'static [Step] = &[
    // Read the NEXT op code, execute the CURRENT op code.
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];
pub const RELATIVE_ADDRESSING_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_READ_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByte, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_READ_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_X_READ_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_Y_READ_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDEXED_INDIRECT_READ_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[XOffsetAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[IncrementAddressBusLow]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[StorePendingAddressLowByte]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const INDIRECT_INDEXED_READ_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[IncrementAddressBusLow]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[StorePendingAddressLowByteWithYOffset]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

// TODO: The data bus needs to be set to the data written.
pub const ABSOLUTE_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByte, IncrementProgramCounter]),
    Step::new(From::PendingAddress            , To::DataBus               , &[ExecuteOpCode]),
];
//
// TODO: The data bus needs to be set to the data written.
pub const ZERO_PAGE_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageAddress    , To::DataBus               , &[ExecuteOpCode]),
];

pub const ABSOLUTE_X_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::DataBus                   , To::DataBus               , &[ExecuteOpCode]),
];

pub const ABSOLUTE_Y_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::DataBus                   , To::DataBus               , &[ExecuteOpCode]),
];

pub const INDEXED_INDIRECT_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[XOffsetAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[IncrementAddressBusLow]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[StorePendingAddressLowByte]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[ExecuteOpCode]),
];

pub const INDIRECT_INDEXED_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[IncrementAddressBusLow]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[StorePendingAddressLowByteWithYOffset]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[ExecuteOpCode]),
];

pub const ABSOLUTE_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByte, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[ExecuteOpCode]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[]),
];

pub const ZERO_PAGE_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[ExecuteOpCode]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[]),
];

pub const ABSOLUTE_X_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[ExecuteOpCode]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[]),
];

pub const ABSOLUTE_Y_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[ExecuteOpCode]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[]),
];

pub const INDEXED_INDIRECT_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[XOffsetAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[IncrementAddressBusLow]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[StorePendingAddressLowByte]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[ExecuteOpCode]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[]),
];

pub const INDIRECT_INDEXED_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[IncrementAddressBusLow]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[StorePendingAddressLowByteWithYOffset]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[ExecuteOpCode]),
    Step::new(From::DataBus                   , To::AddressBusTarget      , &[]),
];

pub const NOP_STEP: Step =
    Step::new(From::DataBus                   , To::DataBus               , &[]                           );
pub const FULL_INSTRUCTION_STEP: Step =
    Step::new(From::DataBus                   , To::DataBus               , &[Instruction]                );

pub const NMI_STEPS: &'static [Step] = &[
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
    Step::new(From::PendingProgramCounterTarget, To::DataBus            , &[StartNextInstruction, IncrementProgramCounter]   ),
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
    Step::new(From::PendingProgramCounterTarget, To::DataBus            , &[StartNextInstruction, IncrementProgramCounter]   ),
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

#[derive(Clone, Copy, Debug)]
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

    pub fn has_interpret_op_code(&self) -> bool {
        for action in self.actions {
            if matches!(action, CycleAction::InterpretOpCode) {
                return true;
            }
        }

        false
    }
}
