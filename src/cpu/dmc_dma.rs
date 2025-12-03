use std::marker::ConstParamTy;

use log::info;
use splitbits::combinebits;

use crate::apu::apu_registers::CycleParity;

pub struct DmcDma {
    disable_pending: bool,
    state: DmcDmaState,
    latest_action: DmcDmaAction,

    // TODO: Switch to u12.
    sample_length: u16,
    sample_bytes_remaining: u16,
}

impl DmcDma {
    pub const IDLE: Self = Self {
        disable_pending: false,
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

    pub fn disable(&mut self, parity: CycleParity) {
        self.disable_pending = true;
        let starting = self.enable_or_disable(parity);
        if starting {
            info!(target: "apuevents", "DMC DMA will be disabled soon.");
        } else {
            info!(target: "apuevents",
                "DMC DMA will be disabled soon, but a Load/Reload is already in progress. State: {:?}", self.state);
        }
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
        let starting = self.enable_or_disable(parity);
        if starting {
            info!(target: "apuevents", "DMC DMA Load starting. CPU will halt soon.");
        } else {
            info!(target: "apuevents", "DMC DMA Load ignored: a Load/Reload is already in progress. State: {:?}", self.state);
        }
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
        if self.state == DmcDmaState::Idle {
            info!(target: "apuevents", "DMC DMA Reload starting. CPU will halt soon.");
            self.state = DmcDmaState::TryEnable;
            self.latest_action = DmcDmaAction::DoNothing;
        } else {
            // TODO: Determine if it ignoring is correct.
            info!(target: "apuevents", "DMC DMA Reload ignored: a Load/Reload is already in progress. State: {:?}", self.state);
        }
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
            State::TryEnable if self.disable_pending     => (Action::DoNothing, State::Idle),
            State::TryEnable                             => (Action::Enable   , State::Dummy),
            State::Dummy                                 => (Action::Dummy    , State::TryRead),
            State::TryRead if parity == CycleParity::Get => (Action::Align    , State::TryRead),
            State::TryRead                               => (Action::Read     , State::Idle),
        };

        // TODO: This should probably be expanded to all states after Enable.
        if self.disable_pending && self.state == State::Idle {
            self.sample_bytes_remaining = 0;
            self.disable_pending = false;
        }
    }

    fn enable_or_disable(&mut self, parity: CycleParity) -> bool {
        let starting = self.state == DmcDmaState::Idle;
        if starting {
            self.latest_action = DmcDmaAction::DoNothing;
            self.state = match parity {
                // If we're already on a GET, then skip WaitingForGet.
                CycleParity::Get => DmcDmaState::FirstSkip,
                CycleParity::Put => DmcDmaState::WaitingForGet,
            };
        }

        starting
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
    Enable,
    Dummy,
    Align,
    Read,
}

impl DmcDmaAction {
    pub fn cpu_should_be_halted(self) -> bool {
        self != DmcDmaAction::DoNothing
    }
}