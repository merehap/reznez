use std::marker::ConstParamTy;

use crate::apu::apu_registers::CycleParity;

pub struct DmcDma {
    state: DmcDmaState,
    latest_action: DmcDmaAction,
}

impl DmcDma {
    pub const IDLE: Self = Self {
        state: DmcDmaState::Idle,
        latest_action: DmcDmaAction::DoNothing,
    };

    pub fn active(&self) -> bool {
        self.state != DmcDmaState::Idle
    }

    pub fn state(&self) -> DmcDmaState {
        self.state
    }

    pub fn latest_action(&self) -> DmcDmaAction {
        self.latest_action
    }

    pub fn cpu_should_be_halted(&self) -> bool {
        self.latest_action.cpu_should_be_halted()
    }

    /*
     * Load DMAs occur when the program tells the DMA unit to start a new sample.
     * * Triggered by a write to $4015
     * * There must be no sample bytes remaining (or nothing will happen)
     * * The sample buffer must be empty (or nothing will happen)
     *
     * Idle -> WaitingForGet -> FirstSkip -> SecondSkip -> TryHalt -> Dummy -> TryRead -> Idle
     *             |   ^                                    |   ^               |   ^
     *             |   |                                    |   |               |   |
     *             +---+                                    +---+               +---+
     */
    pub fn start_load(&mut self, parity: CycleParity) {
        assert_eq!(self.state, DmcDmaState::Idle);
        *self = DmcDma {
            // If we're already on a GET, then skip WaitingForGet.
            state: if parity == CycleParity::Get { DmcDmaState::FirstSkip } else { DmcDmaState::WaitingForGet },
            latest_action: DmcDmaAction::DoNothing,
        };
    }

    /*
     * Reload DMAs occur when the current sample runs out and a new one must start.
     * Reloads are triggered by:
     * * Being on a PUT cycle AND
     * * All of the sample bits are exhausted AND
     * * There must be no more cycles remaining on the current sample bit.
     *
     * Idle -> TryHalt -> Dummy -> TryRead -> Idle
     *          |   ^               |   ^
     *          |   |               |   |
     *          +---+               +---+
     */
    pub fn start_reload(&mut self) {
        assert_eq!(self.state, DmcDmaState::Idle);
        *self = DmcDma {
            state: DmcDmaState::TryHalt,
            latest_action: DmcDmaAction::DoNothing,
        };
    }

    pub fn step(&mut self, is_cpu_read_step: bool, parity: CycleParity) {
        self.latest_action = self.state.step(is_cpu_read_step, parity);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DmcDmaState {
    Idle,
    WaitingForGet,
    FirstSkip,
    SecondSkip,
    TryHalt,
    Dummy,
    TryRead,
}

impl DmcDmaState {
    fn step(&mut self, is_cpu_read_step: bool, parity: CycleParity) -> DmcDmaAction {
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

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum DmcDmaAction {
    #[default]
    DoNothing,
    Halt,
    Dummy,
    Align,
    Read,
}

impl DmcDmaAction {
    pub fn cpu_should_be_halted(self) -> bool {
        self != DmcDmaAction::DoNothing
    }
}