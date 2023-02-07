#[derive(Default)]
pub struct FrameCounter {
    pub(super) step_mode: StepMode,
    pub(super) frame_interrupt: bool,
}

impl FrameCounter {
    pub fn write(&mut self, value: u8) {
        use StepMode::*;
        self.step_mode = if value & 0b1000_0000 == 0 { FourStep } else { FiveStep };
        self.frame_interrupt = value & 0b0100_0000 == 0;
    }
}

#[derive(PartialEq, Clone, Copy, Default)]
pub enum StepMode {
    #[default]
    FourStep,
    FiveStep,
}

impl StepMode {
    pub const FOUR_STEP_FRAME_LENGTH: u16 = 14915;
    pub const FIVE_STEP_FRAME_LENGTH: u16 = 18641;

    pub const fn frame_length(self) -> u16 {
        match self {
            StepMode::FourStep => StepMode::FOUR_STEP_FRAME_LENGTH,
            StepMode::FiveStep => StepMode::FIVE_STEP_FRAME_LENGTH,
        }
    }
}
