use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rodio::{OutputStream, Sink};
use rodio::source::Source;

use crate::apu::apu_registers::ApuRegisters;

const SAMPLE_RATE: u32 = 44100;
const MAX_QUEUE_LENGTH: usize = 2 * SAMPLE_RATE as usize;

pub struct Apu {
    pulse_queue: Arc<Mutex<VecDeque<f32>>>,
    cycle: u64,
}

impl Apu {
    pub fn new() -> Apu {
        // TODO: Select a proper capacity value.
        let pulse_queue = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_LENGTH)));

        let cloned = pulse_queue.clone();
        thread::spawn(move || {
            let source = PulseWave::new(cloned);
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            sink.append(source);

            loop {}
        });

        Apu {
            pulse_queue,
            cycle: 0,
        }
    }

    pub fn step_triangle_channel_only(&self, _regs: &mut ApuRegisters) {
        //regs.triangle.step();
    }

    pub fn step(&mut self, regs: &mut ApuRegisters) {
        regs.pulse_1.step();
        regs.pulse_2.step();
        if self.cycle % 20 == 0 {
            let mut queue = self.pulse_queue
                .lock()
                .unwrap();
            if queue.len() < MAX_QUEUE_LENGTH {
                queue.push_back(Apu::mix_samples(regs));
            }
        }
        //regs.triangle.step();

        self.cycle += 1;
    }

    fn mix_samples(regs: &ApuRegisters) -> f32 {
        let pulse_1 = regs.pulse_1.sample_volume();
        let pulse_2 = regs.pulse_2.sample_volume();
        let triangle = 0.0;
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
    previous_value: f32,
}

impl PulseWave {
    #[inline]
    pub fn new(queue: Arc<Mutex<VecDeque<f32>>>) -> Self {
        PulseWave {
            queue,
            previous_value: 0.0,
        }
    }
}

impl Iterator for PulseWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if let Some(value) = self.queue.lock().unwrap().pop_front() {
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
