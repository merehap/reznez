use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    Copy { from: Location, to: Location },

    // BRK
    IncrementProgramCounter,
    DecrementStackPointer,

    // RTI
    IncrementStackPointer,

    FetchInstruction,
    FetchAddressLow,
    FetchAddressHigh,

    DisableInterrupts,

    Nop,
    Instruction,
    InstructionReturn,
    Nmi,
    DmaTransfer(DmaTransferState),
}

#[derive(Clone, Copy, Debug)]
pub enum DmaTransferState {
    WaitOnPreviousWrite,
    AlignToEven,
    Read(CpuAddress),
    Write,
}

#[derive(Clone, Copy, Debug)]
pub enum Location {
    DataBus,

    ProgramCounter,
    ProgramCounterLowByte,
    ProgramCounterHighByte,

    Status,
    InstructionStatus,

    TopOfStack,

    IrqVectorLow,
    IrqVectorHigh,
}
