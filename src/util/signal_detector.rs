pub struct EdgeDetector {
    previous_level: SignalLevel,
    current_level: SignalLevel,
}

impl EdgeDetector {
    pub fn new() -> Self {
        Self {
            previous_level: SignalLevel::High,
            current_level: SignalLevel::High,
        }
    }

    pub fn pull_low(&mut self) {
        self.current_level = SignalLevel::Low;
    }

    pub fn reset_to_high(&mut self) {
        self.current_level = SignalLevel::High;
    }

    pub fn detect_edge(&mut self) -> bool {
        let edge_detected = self.previous_level == SignalLevel::High && self.current_level == SignalLevel::Low;
        self.previous_level = self.current_level;
        edge_detected
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SignalLevel {
    High,
    Low,
}