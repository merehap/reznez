use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    // BRK
    IncrementProgramCounter,
    Read,
    WriteProgramCounterHighToStack,
    DecrementStackPointer,
    WriteProgramCounterLowToStack,
    WriteStatusToStack,
    ReadProgramCounterHighFromIrqVector,
    ReadProgramCounterLowFromIrqVector,

    // RTI
    IncrementStackPointer,
    ReadStatusFromStack,
    ReadProgramCounterLowFromStack,
    ReadProgramCounterHighFromStack,

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
