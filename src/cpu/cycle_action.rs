use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    FetchInstruction,
    FetchLowAddressByte,
    FetchHighAddressByte,
    FetchData,

    DummyRead,
    IncrementProgramCounter,
    DisableInterrupts,

    // BRK.
    PushProgramCounterHigh,
    PushProgramCounterLow,
    PushStatus,
    FetchProgramCounterLowFromIrqVector,
    FetchProgramCounterHighFromIrqVector,

    // RTI
    IncrementStackPointer,
    PeekStatus,
    PeekProgramCounterLow,
    PeekProgramCounterHigh,

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
