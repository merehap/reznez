use crate::memory::mapper::CpuAddress;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    IncrementProgramCounter,
    IncrementAddressBus,
    IncrementAddressBusLow,
    IncrementDmaAddress,
    StorePendingAddressLowByte,
    StorePendingAddressLowByteWithXOffset,
    StorePendingAddressLowByteWithYOffset,

    IncrementStackPointer,
    DecrementStackPointer,

    DisableInterrupts,

    CheckNegativeAndZero,

    XOffsetAddressBus,
    YOffsetAddressBus,
    MaybeInsertOopsStep,
    MaybeInsertBranchOopsStep,
    AddCarryToAddressBus,
    AddCarryToProgramCounter,

    StartNextInstruction,
    InterpretOpCode,
    ExecuteOpCode,
}

#[derive(Clone, Copy, Debug)]
pub enum From {
    AddressBusTarget,
    DmaAddressTarget,

    ProgramCounterTarget,
    PendingAddressTarget,
    PendingZeroPageTarget,
    PendingProgramCounterTarget,

    TopOfStack,

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
    AddressBusTarget,
    DmaAddressTarget,

    ProgramCounterTarget,
    PendingAddressTarget,
    PendingZeroPageTarget,
    PendingProgramCounterTarget,

    TopOfStack,

    OamData,

    AddressTarget(CpuAddress),
}

#[derive(Clone, Copy, Debug)]
pub enum Field {
    ProgramCounterLowByte,
    ProgramCounterHighByte,
    Accumulator,
    Status,
    StatusForInstruction,
    StatusForInterrupt,

    OpRegister,
}
