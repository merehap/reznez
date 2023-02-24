use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rodio::{OutputStream, Sink};
use rodio::source::Source;

use crate::apu::apu_registers::{ApuRegisters, FrameResetStatus, StepMode};

const SAMPLE_RATE: u32 = 44100;
const MAX_QUEUE_LENGTH: usize = 2 * SAMPLE_RATE as usize;

pub struct Apu {
    pulse_queue: Arc<Mutex<VecDeque<f32>>>,
    muted: Arc<Mutex<bool>>,
    cycle: u64,
}

impl Apu {
    pub fn new() -> Apu {
        // TODO: Select a proper capacity value.
        let pulse_queue = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_LENGTH)));
        let muted = Arc::new(Mutex::new(false));

        let cloned_queue = pulse_queue.clone();
        let cloned_muted = muted.clone();
        thread::spawn(move || {
            let source = PulseWave::new(cloned_queue, cloned_muted);
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(source);

            loop {}
        });

        Apu {
            pulse_queue,
            muted,
            cycle: 0,
        }
    }

    pub fn mute(&mut self) {
        *self.muted.lock().unwrap() = true;
    }

    pub fn half_step(&self, regs: &mut ApuRegisters) {
        regs.frame_reset_status.even_cycle_reached();

        const FIRST_STEP : u16 = 03728;
        const SECOND_STEP: u16 = 07456;
        const THIRD_STEP : u16 = 11185;

        let cycle_within_frame = self.cycle_within_frame(regs);

        use StepMode::*;
        match (regs.step_mode(), cycle_within_frame) {
            (_, FIRST_STEP) => {}
            (_, SECOND_STEP) => {
                regs.decrement_length_counters();
            }
            (_, THIRD_STEP) => {}
            (FourStep, _) if cycle_within_frame == StepMode::FOUR_STEP_FRAME_LENGTH - 1 => {
                regs.decrement_length_counters();
            }
            (FiveStep, _) if cycle_within_frame == StepMode::FIVE_STEP_FRAME_LENGTH - 1 => {
                regs.decrement_length_counters();
            }
            (FourStep, _) if cycle_within_frame >= StepMode::FOUR_STEP_FRAME_LENGTH => unreachable!(),
            (FiveStep, _) if cycle_within_frame >= StepMode::FIVE_STEP_FRAME_LENGTH => unreachable!(),
            _ => { /* Do nothing. */ }
        }

        regs.triangle.step_quarter_frame();
    }

    pub fn step(&mut self, regs: &mut ApuRegisters) {
        if regs.frame_reset_status == FrameResetStatus::NextCycle {
            self.cycle = 0;
            regs.frame_reset_status.finished();
        }

        regs.pulse_1.step();
        regs.pulse_2.step();
        regs.triangle.step_half_frame();
        regs.noise.step();

        if self.cycle % 20 == 0 {
            let mut queue = self.pulse_queue
                .lock()
                .unwrap();
            if queue.len() < MAX_QUEUE_LENGTH {
                queue.push_back(Apu::mix_samples(regs));
            }
        }

        if self.cycle_within_frame(regs) == StepMode::FOUR_STEP_FRAME_LENGTH - 1 {
            regs.maybe_set_frame_irq_pending();
        }

        self.cycle += 1;
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn cycle_within_frame(&self, regs: &ApuRegisters) -> u16 {
        u16::try_from(self.cycle % u64::from(regs.step_mode().frame_length())).unwrap()
    }

    fn mix_samples(regs: &ApuRegisters) -> f32 {
        let pulse_1 = regs.pulse_1.sample_volume();
        let pulse_2 = regs.pulse_2.sample_volume();
        let triangle = regs.triangle.sample_volume();
        let noise = 0.0;
        let dmc = 0.0;

        let pulse_out = 95.88 / (8128.0 / (pulse_1 + pulse_2) + 100.0);
        let tnd_out = 159.79 / ((1.0 / (triangle / 8227.0 + noise / 12241.0 + dmc / 22368.0)) + 100.0);
        let mix = pulse_out + tnd_out;

        assert!(mix >= 0.0);
        assert!(mix <= 1.0);
        mix
    }
}

#[derive(Clone, Debug)]
pub struct PulseWave {
    queue: Arc<Mutex<VecDeque<f32>>>,
    muted: Arc<Mutex<bool>>,
    previous_value: f32,
}

impl PulseWave {
    #[inline]
    pub fn new(queue: Arc<Mutex<VecDeque<f32>>>, muted: Arc<Mutex<bool>>) -> Self {
        PulseWave {
            queue,
            muted,
            previous_value: 0.0,
        }
    }

    pub fn mute(&mut self) {
        *self.muted.lock().unwrap() = true;
    }
}

impl Iterator for PulseWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if *self.muted.lock().unwrap() {
            None
        } else if let Some(value) = self.queue.lock().unwrap().pop_front() {
            self.previous_value = value;
            Some(value)
        } else {
            Some(self.previous_value)
        }
    }
}

impl Source for PulseWave {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
