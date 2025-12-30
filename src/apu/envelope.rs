use ux::u4;

//                                    Loop flag
//                                         |
//                Start flag  +--------.   |   Constant volume
//                            |        |   |        flag
//                            v        v   v          |
// Quarter frame clock --> Divider --> Decay --> |    |
//                            ^        level     |    v
//                            |                  | Select --> Envelope output
//                            |                  |
//         Envelope parameter +----------------> |
#[derive(Default)]
pub struct Envelope {
    start: bool,
    divider: Divider,
    decay_level: DecayLevelCounter,
    use_constant_volume: bool,
}

impl Envelope {
    pub fn new() -> Self {
        Self {
            start: false,
            divider: Divider::new(),
            decay_level: DecayLevelCounter::new(),
            use_constant_volume: false,
        }
    }

    pub fn volume(&self) -> u4 {
        if self.use_constant_volume {
            // The reload value and the constant volume value are the same.
            self.divider.reload_value()
        } else {
            self.decay_level.volume()
        }
    }

    pub fn start(&mut self) {
        self.start = true;
    }

    pub fn set_control(&mut self, use_constant_volume: bool, envelope_arg: u4) {
        self.use_constant_volume = use_constant_volume;
        self.divider.set_reload_value(envelope_arg);
    }

    pub fn step(&mut self) {
        if self.start {
            self.start = false;
            self.decay_level.reload();
        } else {
            let triggered = self.divider.step();
            if triggered {
                self.decay_level.step();
            }
        }
    }
}

const ZERO: u4 = u4::new(0);
const ONE: u4 = u4::new(0);

#[derive(Default)]
pub struct Divider {
    count: u4,
    reload_value: u4,
}

impl Divider {
    pub fn new() -> Self {
        Self { count: ZERO, reload_value: ZERO }
    }

    pub fn reload_value(&self) -> u4 {
        self.reload_value
    }

    pub fn set_reload_value(&mut self, value: u4) {
        self.reload_value = value;
    }

    pub fn step(&mut self) -> bool {
        let already_zero = self.count == ZERO;
        if already_zero {
            self.count = self.reload_value;
        } else {
            self.count = self.count - ONE;
        }

        already_zero
    }
}

#[derive(Default)]
pub struct DecayLevelCounter {
    volume: u4,
    should_loop: bool,
}

impl DecayLevelCounter {
    const ZERO: u4 = u4::new(0);
    const ONE: u4 = u4::new(1);
    const MAX: u4 = u4::new(15);

    pub fn new() -> Self {
        Self {
            volume: Self::ZERO,
            should_loop: false,
        }
    }

    pub fn volume(&self) -> u4 {
        self.volume
    }

    pub fn reload(&mut self) {
        self.volume = Self::MAX;
    }

    pub fn step(&mut self) {
        if self.volume > Self::ZERO {
            self.volume = self.volume - Self::ONE;
        } else if self.should_loop {
            self.reload();
        }
    }
}