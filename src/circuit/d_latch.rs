use crate::memory::signal_level::SignalLevel;

pub struct DLatch {
    q: SignalLevel,
}

impl DLatch {
    pub fn new(initial_q: SignalLevel) -> Self {
        Self { q: initial_q }
    }

    pub fn q(&self) -> SignalLevel {
        self.q
    }

    pub fn nq(&self) -> SignalLevel {
        !self.q
    }

    pub fn update(&mut self, inputs: DLInputs) {
        if inputs.e.is_high() {
            self.q = inputs.d;
        }
    }
}

#[derive(Clone, Copy)]
pub struct DLInputs {
    pub e: SignalLevel,
    pub d: SignalLevel,
}