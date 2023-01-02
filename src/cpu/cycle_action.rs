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

    Address(CpuAddress),
}

impl From {
    pub const NMI_VECTOR_LOW : From = From::Address(CpuAddress::new(0xFFFA));
    pub const NMI_VECTOR_HIGH: From = From::Address(CpuAddress::new(0xFFFB));
    pub const IRQ_VECTOR_LOW : From = From::Address(CpuAddress::new(0xFFFE));
    pub const IRQ_VECTOR_HIGH: From = From::Address(CpuAddress::new(0xFFFF));
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
