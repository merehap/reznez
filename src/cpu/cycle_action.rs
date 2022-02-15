use crate::cpu::instruction::Instruction;
use crate::memory::cpu::cpu_address::CpuAddress;

#[derive(Debug)]
pub enum CycleAction {
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
    Read,
    Write(CpuAddress),
}
