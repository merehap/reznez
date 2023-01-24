#[derive(Default)]
pub struct FrameCounter {
    step_mode: StepMode,
    suppress_irq: bool,
}

impl FrameCounter {
    pub fn write(&mut self, value: u8) {
        use StepMode::*;
        self.step_mode = if value & 0b1000_0000 == 0 { FourStep } else { FiveStep };
        self.suppress_irq = value & 0b0100_0000 != 0;
    }
}

#[derive(Default)]
pub enum StepMode {
    #[default]
    FourStep,
    FiveStep,
}
