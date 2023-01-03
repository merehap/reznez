use crate::memory::mapper::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    IncrementProgramCounter,
    IncrementAddressBus,
    SetAddressBus(CpuAddress),
    StorePendingAddressLowByte,

    IncrementStackPointer,
    DecrementStackPointer,

    DisableInterrupts,

    CheckNegativeAndZero,

    Nop,
    Instruction,
    InstructionReturn,
}

#[derive(Clone, Copy, Debug)]
pub enum From {
    DataBus,
    AddressBusTarget,

    ProgramCounterTarget,
    PendingProgramCounterTarget,

    TopOfStack,

    ProgramCounterLowByte,
    ProgramCounterHighByte,
    Accumulator,
    StatusForInstruction,
    StatusForInterrupt,

    AddressTarget(CpuAddress),
}

impl From {
    pub const NMI_VECTOR_LOW : From = From::AddressTarget(CpuAddress::new(0xFFFA));
    pub const NMI_VECTOR_HIGH: From = From::AddressTarget(CpuAddress::new(0xFFFB));
    pub const IRQ_VECTOR_LOW : From = From::AddressTarget(CpuAddress::new(0xFFFE));
    pub const IRQ_VECTOR_HIGH: From = From::AddressTarget(CpuAddress::new(0xFFFF));
}

#[derive(Clone, Copy, Debug)]
pub enum To {
    DataBus,

    TopOfStack,

    ProgramCounterHighByte,
    PendingAddressHighByte,

    OamData,

    Accumulator,
    Status,

    Instruction,
}
