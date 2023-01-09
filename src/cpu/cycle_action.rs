use crate::memory::mapper::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    IncrementProgramCounter,
    IncrementAddressBus,
    IncrementAddressBusLow,
    SetAddressBusToOamDmaStart,
    StorePendingAddressLowByte,

    IncrementStackPointer,
    DecrementStackPointer,

    DisableInterrupts,

    CheckNegativeAndZero,

    Instruction,
    InterpretOpCode,
    ExecuteOpCode,
}

#[derive(Clone, Copy, Debug)]
pub enum From {
    DataBus,
    AddressBusTarget,

    ProgramCounterTarget,
    PendingAddressTarget,
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

    OamData,

    Accumulator,
    Status,

    NextOpCode,
}
