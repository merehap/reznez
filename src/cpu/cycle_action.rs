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
}

#[derive(Clone, Copy, Debug)]
pub enum From {
    DataBus,
    AddressBus,

    ProgramCounter,

    ProgramCounterLowByte,
    ProgramCounterHighByte,

    StatusForInstruction,
    StatusForInterrupt,
    TopOfStack,

    NmiVectorLow,
    NmiVectorHigh,
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
