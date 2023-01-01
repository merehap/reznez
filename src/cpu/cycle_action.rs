use crate::memory::mapper::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
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
pub enum From {
    DataBus,
    AddressBus,

    ProgramCounter,

    ProgramCounterLowByte,
    ProgramCounterHighByte,

    InstructionStatus,
    TopOfStack,

    IrqVectorLow,
    IrqVectorHigh,
}

#[derive(Clone, Copy, Debug)]
pub enum To {
    DataBus,

    ProgramCounterHighByte,
    PendingAddressHighByte,

    Status,
    TopOfStack,
    OamData,

    Instruction,
}
