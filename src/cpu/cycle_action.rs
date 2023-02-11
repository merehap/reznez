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
    SetInterruptVector,
    ClearInterruptVector,
    ClearNmi,
    ClearIrq,

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

    InterruptVectorLow,
    InterruptVectorHigh,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum To {
    AddressBusTarget,
    DmaAddressTarget,

    ProgramCounterTarget,
    PendingAddressTarget,
    PendingZeroPageTarget,
    PendingProgramCounterTarget,

    TopOfStack,

    AddressTarget(CpuAddress),
}

impl To {
    pub const OAM_DATA: To = To::AddressTarget(CpuAddress::new(0x2004));
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Field {
    ProgramCounterLowByte,
    ProgramCounterHighByte,
    Accumulator,
    Status,
    StatusForInstruction,
    StatusForInterrupt,

    OpRegister,
}
