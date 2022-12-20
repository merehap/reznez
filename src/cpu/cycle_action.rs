use crate::cpu::instruction::Instruction;
use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    FetchInstruction,
    /*
    FetchLowAddressByte,
    FetchHighAddressByte,
    FetchData,
    */

    Nop,
    Instruction(Instruction),
    InstructionReturn(Instruction),
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
