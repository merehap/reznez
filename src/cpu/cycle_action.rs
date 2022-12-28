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

    PushProgramCounterHigh,
    PushProgramCounterLow,
    PushStatus,
    FetchProgramCounterLowFromIrqVector,
    FetchProgramCounterHighFromIrqVector,

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
