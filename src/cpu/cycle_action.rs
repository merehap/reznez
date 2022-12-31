use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    Copy { from: Location, to: Location },

    // BRK
    IncrementProgramCounter,
    DecrementStackPointer,

    // RTI
    IncrementStackPointer,

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
    PendingAddressHighByte,

    Status,
    InstructionStatus,

    TopOfStack,

    IrqVectorLow,
    IrqVectorHigh,

    Instruction,
}
