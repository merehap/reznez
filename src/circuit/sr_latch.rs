use crate::memory::signal_level::SignalLevel;

// NOR-based
pub struct SRLatch {
    q: SignalLevel,
    nq: SignalLevel,
}

impl SRLatch {
    // Starting in the "invalid" state is currently unsupported.
    pub fn new(initial_q: SignalLevel) -> Self {
        Self {
            q: initial_q,
            nq: !initial_q,
        }
    }

    pub fn q(&self) -> SignalLevel {
        self.q
    }

    #[allow(dead_code)]
    pub fn nq(&self) -> SignalLevel {
        self.nq
    }

    pub fn update(&mut self, SRInputs { s, r }: SRInputs) {
        if s.is_high() || r.is_high() {
            self.q = s & !r;
            self.nq = !s & r;
        }
    }
}

#[derive(Clone, Copy)]
pub struct SRInputs {
    pub s: SignalLevel,
    pub r: SignalLevel,
}