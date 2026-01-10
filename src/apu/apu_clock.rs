use std::fmt;

pub struct ApuClock {
    total_cpu_cycles: u64,
    cpu_cycle: u16,
    parity: CycleParity,
    step_mode: StepMode,
    is_in_frame_irq_window: bool,
}

impl ApuClock {
    pub fn new() -> Self {
        Self {
            total_cpu_cycles: 0,
            cpu_cycle: 0,
            parity: CycleParity::Get,
            step_mode: StepMode::FourStep,
            is_in_frame_irq_window: false,
        }
    }

    pub fn reset(&mut self) {
        self.cpu_cycle = 0;
        self.is_in_frame_irq_window = false;
    }

    // Called every CPU cycle (not APU cycle)
    pub fn tick(&mut self) {
        self.total_cpu_cycles += 1;
        self.cpu_cycle += 1;
        if self.step_mode == StepMode::FourStep && self.apu_cycle() == self.step_mode.frame_length() - 1 {
            self.is_in_frame_irq_window = true;
        }

        if self.cpu_cycle == 1 {
            self.is_in_frame_irq_window = false;
        }

        self.cpu_cycle %= 2 * self.step_mode.frame_length();
        self.parity.toggle();
    }

    pub fn step_mode(&self) -> StepMode {
        self.step_mode
    }

    pub fn set_step_mode(&mut self, step_mode: StepMode) {
        self.step_mode = step_mode;
        if self.step_mode == StepMode::FiveStep {
            self.is_in_frame_irq_window = false;
        }
    }

    pub fn cycle_parity(&self) -> CycleParity {
        self.parity
    }

    pub fn cpu_cycle(&self) -> u16 {
        self.cpu_cycle
    }

    pub fn apu_cycle(&self) -> u16 {
        self.cpu_cycle() / 2
    }

    pub fn raw_apu_cycle(&self) -> u64 {
        self.total_cpu_cycles / 2
    }

    pub fn is_in_frame_irq_window(&self) -> bool {
        self.is_in_frame_irq_window
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Default)]
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CycleParity {
    Get,
    Put,
}

impl CycleParity {
    pub fn toggle(&mut self) {
        match *self {
            Self::Get => *self = Self::Put,
            Self::Put => *self = Self::Get,
        }
    }
}

impl fmt::Display for CycleParity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CycleParity::Get => write!(f, "GET"),
            CycleParity::Put => write!(f, "PUT"),
        }
    }
}