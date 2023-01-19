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
            Step::new(From::DmaAddressTarget, To::DataBus, &[]),
            Step::new(From::DataBus         , To::OamData, &[IncrementDmaAddress]),
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
        (AbX, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |             BIT | LAX | NOP) => ABSOLUTE_X_READ_STEPS,
        (AbY, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |             BIT | LAX | NOP | LAS | TAS | AHX) => ABSOLUTE_Y_READ_STEPS,
        (ZP , LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP | CPX | CPY | BIT | LAX | NOP) => ZERO_PAGE_READ_STEPS,
        (ZPX, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |             BIT | LAX | NOP) => ZERO_PAGE_X_READ_STEPS,
        (ZPY, LDA | LDX | LDY | EOR | AND | ORA | ADC | SBC | CMP |             BIT | LAX | NOP) => ZERO_PAGE_Y_READ_STEPS,
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

pub const ZERO_PAGE_X_READ_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[XOffsetAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ABSOLUTE_Y_READ_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[MaybeInsertOopsStep, AddCarryToAddressBus]),
    Step::new(From::ProgramCounterTarget      , To::DataBus            , &[ExecuteOpCode, StartNextInstruction, IncrementProgramCounter]),
];

pub const ZERO_PAGE_Y_READ_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[YOffsetAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[]),
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

// TODO: The data bus needs to be set to the data written.
pub const ZERO_PAGE_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageAddress    , To::DataBus               , &[ExecuteOpCode]),
];

pub const ABSOLUTE_X_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithXOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::DataBus                   , To::DataBus               , &[ExecuteOpCode]),
];

pub const ZERO_PAGE_X_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageAddress    , To::DataBus               , &[XOffsetAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[ExecuteOpCode]),
];

pub const ABSOLUTE_Y_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::ProgramCounterTarget      , To::DataBus               , &[StorePendingAddressLowByteWithYOffset, IncrementProgramCounter]),
    Step::new(From::PendingAddressTarget      , To::DataBus               , &[AddCarryToAddressBus]),
    Step::new(From::DataBus                   , To::DataBus               , &[ExecuteOpCode]),
];

pub const ZERO_PAGE_Y_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageAddress    , To::DataBus               , &[YOffsetAddressBus]),
    Step::new(From::AddressBusTarget          , To::DataBus               , &[ExecuteOpCode]),
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

pub const ZERO_PAGE_X_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[XOffsetAddressBus]),
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

pub const ZERO_PAGE_Y_READ_MODIFY_WRITE_STEPS: &'static [Step] = &[
    Step::new(From::PendingZeroPageTarget     , To::DataBus               , &[YOffsetAddressBus]),
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
