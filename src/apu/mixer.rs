use crate::apu::apu_registers::ApuRegisters;

pub struct Mixer {
    pub pulse_1_force_muted: bool,
    pub pulse_2_force_muted: bool,
    pub triangle_force_muted: bool,
    pub noise_force_muted: bool,
    pub dmc_force_muted: bool,

    high90_filter: HighPassFilter,
    high440_filter: HighPassFilter,
    low14000_filter: LowPassFilter,
}

impl Mixer {
    pub const SAMPLE_RATE: u32 = 44100;

    pub fn new() -> Self {
        Self {
            pulse_1_force_muted: false,
            pulse_2_force_muted: false,
            triangle_force_muted: false,
            noise_force_muted: false,
            dmc_force_muted: false,

            high90_filter: HighPassFilter::new(0.996),
            high440_filter: HighPassFilter::new(0.983),
            low14000_filter: LowPassFilter::new(0.666),
        }
    }

    pub fn mix_filtered(&mut self, regs: &ApuRegisters) -> f32 {
        let mut sample = self.mix(regs);
        sample = self.high90_filter.transform(sample);
        sample = self.high440_filter.transform(sample);
        sample = self.low14000_filter.transform(sample);
        sample
    }

    pub fn mix(&self, regs: &ApuRegisters) -> f32 {
        let pulse_1 = if self.pulse_1_force_muted { 0.0 } else { f32::from(u8::from(regs.pulse_1.sample_volume())) };
        let pulse_2 = if self.pulse_2_force_muted { 0.0 } else { f32::from(u8::from(regs.pulse_2.sample_volume())) };
        let triangle = if self.triangle_force_muted { 0.0 } else { f32::from(regs.triangle.sample_volume()) };
        let noise = if self.noise_force_muted { 0.0 } else { f32::from(u8::from(regs.noise.sample_volume())) };
        let dmc = if self.dmc_force_muted { 0.0 } else { f32::from(regs.dmc.sample_volume()) };

        let pulse_out = 95.88 / (8128.0 / (pulse_1 + pulse_2) + 100.0);
        let tnd_out = 159.79 / ((1.0 / (triangle / 8227.0 + noise / 12241.0 + dmc / 22638.0)) + 100.0);
        let output = pulse_out + tnd_out;

        assert!(output >= 0.0);
        assert!(output <= 1.0);
        output
    }
}

pub struct HighPassFilter {
    k: f32,
    prev_input: f32,
    prev_output: f32,
}

impl HighPassFilter {
    pub fn new(k: f32) -> Self {
        Self { k, prev_input: 0.0, prev_output: 0.0 }
    }

    pub fn transform(&mut self, input: f32) -> f32 {
        let output = self.k * self.prev_output + input - self.prev_input;
        self.prev_input = input;
        self.prev_output = output;
        output
    }
}

pub struct LowPassFilter {
    k: f32,
    prev_output: f32,
}

impl LowPassFilter {
    pub fn new(k: f32) -> Self {
        Self { k, prev_output: 0.0 }
    }

    pub fn transform(&mut self, input: f32) -> f32 {
        let output = self.prev_output + self.k * (input - self.prev_output);
        self.prev_output = output;
        output
    }
}