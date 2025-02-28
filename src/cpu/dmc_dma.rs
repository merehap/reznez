use crate::apu::apu_registers::CycleParity;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DmcDma {
    Idle,
    WaitingForGet,
    FirstSkip,
    SecondSkip,
    TryHalt,
    Dummy,
    TryRead,
}

impl DmcDma {
    pub const IDLE: Self = Self::Idle;

    pub fn start_load(&mut self) {
        assert_eq!(*self, Self::Idle);
        *self = Self::WaitingForGet
    }

    pub fn start_reload(&mut self) {
        assert_eq!(*self, Self::Idle);
        *self = Self::TryHalt
    }

    pub fn step(&mut self, is_cpu_read_step: bool, parity: CycleParity) -> DmcDmaAction {
        let still_waiting_for_get = *self == Self::WaitingForGet && parity == CycleParity::Put;
        let still_trying_to_halt = *self == Self::TryHalt && !is_cpu_read_step;
        if still_waiting_for_get || still_trying_to_halt {
            return DmcDmaAction::DoNothing;
        }

        if *self == Self::TryRead && parity == CycleParity::Get {
            return DmcDmaAction::Align;
        }

        let (action, next_stage) = match *self {
            Self::Idle => (DmcDmaAction::DoNothing, Self::Idle),
            Self::WaitingForGet => (DmcDmaAction::DoNothing, Self::FirstSkip),
            Self::FirstSkip => (DmcDmaAction::DoNothing, Self::SecondSkip),
            Self::SecondSkip => (DmcDmaAction::DoNothing, Self::TryHalt),
            Self::TryHalt => (DmcDmaAction::Halt, Self::Dummy),
            Self::Dummy => (DmcDmaAction::Dummy, Self::TryRead),
            Self::TryRead => (DmcDmaAction::Read, Self::Idle),
        };

        *self = next_stage;
        action
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DmcDmaAction {
    DoNothing,
    Halt,
    Dummy,
    Align,
    Read,
}
