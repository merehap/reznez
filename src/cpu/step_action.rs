use crate::memory::mapper::CpuAddress;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum StepAction {
    IncrementPC,
    AddCarryToPC,
    CopyAddressToPC,

    IncrementAddress,
    IncrementAddressLow,
    XOffsetAddress,
    YOffsetAddress,
    AddCarryToAddress,

    XOffsetPendingAddressLow,
    YOffsetPendingAddressLow,

    IncrementStackPointer,
    DecrementStackPointer,

    IncrementOamDmaAddress,

    DisableInterrupts,
    SetInterruptVector,
    ClearInterruptVector,
    PollInterrupts,

    SetDmcSampleBuffer,

    CheckNegativeAndZero,

    MaybeInsertOopsStep,
    MaybeInsertBranchOopsStep,

    StartNextInstruction,
    InterpretOpCode,
    ExecuteOpCode,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum From {
    AddressBusTarget,
    OamDmaAddressTarget,
    DmcDmaAddressTarget,

    ProgramCounterTarget,
    PendingAddressTarget,
    PendingZeroPageTarget,
    ComputedTarget,

    TopOfStack,

    InterruptVectorLow,
    InterruptVectorHigh,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum To {
    AddressBusTarget,
    OamDmaAddressTarget,

    ProgramCounterTarget,
    PendingAddressTarget,
    PendingZeroPageTarget,
    ComputedTarget,

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

    // Called "Data" or "Operand" in the manual.
    Argument,
    PendingAddressLow,
    PendingAddressHigh,
    OpRegister,
}
