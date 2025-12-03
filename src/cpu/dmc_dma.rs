use std::marker::ConstParamTy;

use log::info;
use splitbits::combinebits;

use crate::apu::apu_registers::CycleParity;

pub struct DmcDma {
    state: DmcDmaState,
    latest_action: DmcDmaAction,

    // TODO: Switch to u12.
    sample_length: u16,
    sample_bytes_remaining: u16,
}

impl DmcDma {
    pub const IDLE: Self = Self {
        state: DmcDmaState::Idle,
        latest_action: DmcDmaAction::DoNothing,

        sample_length: 1,
        sample_bytes_remaining: 0,
    };

    pub fn sample_bytes_remain(&self) -> bool {
        self.sample_bytes_remaining > 0
    }

    pub fn reload_sample_bytes_remaining(&mut self) {
        self.sample_bytes_remaining = self.sample_length;
    }

    pub fn clear_sample_bytes_remaining(&mut self) {
        self.sample_bytes_remaining = 0;
    }

    pub fn decrement_sample_bytes_remaining(&mut self) {
        self.sample_bytes_remaining = self.sample_bytes_remaining.saturating_sub(1);
    }

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

    // Write 0x4013
    pub fn write_sample_length(&mut self, length: u8) {
        self.sample_length = combinebits!(length, "0000 llll llll 0001");
        //println!("Setting sample length to {}", self.sample_length);
    }

    /*
     * Load DMAs occur when the program tells the DMA unit to start a new sample.
     * * Triggered by a write to $4015
     * * There must be no sample bytes remaining (or nothing will happen)
     * * The sample buffer must be empty (or nothing will happen)
     *
     * Idle -> WaitingForGet -> FirstSkip -> SecondSkip -> TryHalt -> Dummy -> TryRead -> Idle
     *                                                      |   ^               |   ^
     *                                                      |   |               |   |
     *                                                      +---+               +---+
     */
    pub fn start_load(&mut self, parity: CycleParity) {
        info!(target: "apuevents", "DMC DMA Load starting. CPU will halt soon.");
        assert_eq!(self.state, DmcDmaState::Idle);
        self.latest_action = DmcDmaAction::DoNothing;
        self.state = match parity {
            // If we're already on a GET, then skip WaitingForGet.
            CycleParity::Get => DmcDmaState::FirstSkip,
            CycleParity::Put => DmcDmaState::WaitingForGet,
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
        info!(target: "apuevents", "DMC DMA Reload starting. CPU will halt soon.");
        assert_eq!(self.state, DmcDmaState::Idle);
        self.state = DmcDmaState::TryEnable;
        self.latest_action = DmcDmaAction::DoNothing;
    }

    pub fn step(&mut self, is_cpu_read_step: bool, parity: CycleParity) {
        use DmcDmaAction as Action;
        use DmcDmaState as State;
        (self.latest_action, self.state) = match self.state {
            State::Idle                                  => (Action::DoNothing, State::Idle),
            State::WaitingForGet                         => (Action::DoNothing, State::FirstSkip),
            State::FirstSkip                             => (Action::DoNothing, State::SecondSkip),
            State::SecondSkip                            => (Action::DoNothing, State::TryEnable),
            State::TryEnable if !is_cpu_read_step        => (Action::DoNothing, State::TryEnable),
            State::TryEnable                             => (Action::Halt     , State::Dummy),
            State::Dummy                                 => (Action::Dummy    , State::TryRead),
            State::TryRead if parity == CycleParity::Get => (Action::Align    , State::TryRead),
            State::TryRead                               => (Action::Read     , State::Idle),
        };
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DmcDmaState {
    Idle,
    WaitingForGet,
    FirstSkip,
    SecondSkip,
    TryEnable,
    Dummy,
    TryRead,
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