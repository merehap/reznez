use crate::memory::mapper::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    Copy { from: Location, to: Location },

    IncrementProgramCounter,
    IncrementAddressBus,
    SetAddressBus(CpuAddress),
    IncrementStackPointer,
    DecrementStackPointer,

    DisableInterrupts,

    Nop,
    Instruction,
    InstructionReturn,
    Nmi,
}

#[derive(Clone, Copy, Debug)]
pub enum Location {
    DataBus,
    AddressBus,

    ProgramCounter,
    ProgramCounterLowByte,
    ProgramCounterHighByte,
    PendingAddressHighByte,
    OamData,

    Status,
    InstructionStatus,

    TopOfStack,

    IrqVectorLow,
    IrqVectorHigh,

    Instruction,
}
