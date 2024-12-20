use crate::memory::mapper::CpuAddress;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CycleAction {
    IncrementProgramCounter,
    IncrementAddressBus,
    IncrementAddressBusLow,
    IncrementOamDmaAddress,

    IncrementStackPointer,
    DecrementStackPointer,

    DisableInterrupts,
    SetInterruptVector,
    ClearInterruptVector,
    PollInterrupts,

    SetDmcSampleBuffer,

    CheckNegativeAndZero,

    XOffsetPendingAddressLow,
    YOffsetPendingAddressLow,
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
    OamDmaAddressTarget,
    DmcDmaAddressTarget,

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
    OamDmaAddressTarget,

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

    // Called "Data" or "Operand" in the manual.
    Argument,
    PendingAddressLow,
    PendingAddressHigh,
    OpRegister,
}
