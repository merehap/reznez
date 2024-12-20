use crate::memory::mapper::CpuAddress;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
    PollInterrupts,

    CheckNegativeAndZero,

    XOffsetAddressBus,
    YOffsetAddressBus,
    MaybeInsertOopsStep,
    MaybeInsertBranchOopsStep,
    CopyAddressToPC,
    AddCarryToAddressBus,
    AddCarryToProgramCounter,

    StartNextInstruction,
    InterpretOpCode,
    ExecuteOpCode,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum From {
    AddressBusTarget,
    DmaAddressTarget,

    ProgramCounterTarget,
    PendingAddressTarget,
    PendingZeroPageTarget,

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

    OpRegister,
}
