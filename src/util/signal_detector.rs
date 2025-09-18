use std::marker::ConstParamTy;

pub struct EdgeDetector<const DETECTION_LEVEL: SignalLevel> {
    previous_level: SignalLevel,
    level: SignalLevel,
}

impl <const DETECTION_LEVEL: SignalLevel> EdgeDetector<DETECTION_LEVEL> {
    pub fn new() -> Self {
        Self {
            previous_level: SignalLevel::High,
            level: SignalLevel::High,
        }
    }

    pub fn level(&self) -> SignalLevel {
        self.level
    }

    pub fn pull_low(&mut self) {
        self.level = SignalLevel::Low;
    }

    pub fn reset_to_high(&mut self) {
        self.level = SignalLevel::High;
    }

    pub fn detect_edge(&mut self) -> bool {
        let edge_detected = self.level == DETECTION_LEVEL && self.previous_level != self.level;
        self.previous_level = self.level;
        edge_detected
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, ConstParamTy)]
pub enum SignalLevel {
    High,
    Low,
}