use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use log::info;
use rodio::{OutputStream, Sink};
use rodio::source::Source;

use crate::apu::apu_registers::{ApuRegisters, CycleParity};
use crate::memory::memory::Memory;

const SAMPLE_RATE: u32 = 44100;
const MAX_QUEUE_LENGTH: usize = 2 * SAMPLE_RATE as usize;

pub struct Apu {
    pulse_queue: Arc<Mutex<VecDeque<f32>>>,
    muted: Arc<Mutex<bool>>,
}

impl Apu {
    pub fn new(disable_audio: bool) -> Apu {
        // TODO: Select a proper capacity value.
        let pulse_queue = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_LENGTH)));
        let muted = Arc::new(Mutex::new(false));

        if !disable_audio {
            let cloned_queue = pulse_queue.clone();
            let cloned_muted = muted.clone();
            thread::spawn(move || {
                let source = PulseWave::new(cloned_queue, cloned_muted);
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&stream_handle).unwrap();
                sink.append(source);

                loop {
                    thread::sleep(Duration::from_millis(1000));
                }
            });
        }

        Apu {
            pulse_queue,
            muted,
        }
    }

    pub fn mute(&mut self) {
        *self.muted.lock().unwrap() = true;
    }

    pub fn step(&mut self, mem: &mut Memory) {
        mem.apu_regs.maybe_update_step_mode();

        match mem.apu_regs.clock().cycle_parity() {
            CycleParity::Get => Apu::execute_get_cycle(mem),
            CycleParity::Put => self.execute_put_cycle(mem),
        }
    }

    fn execute_put_cycle(&mut self, mem: &mut Memory) {
        let cycle = mem.apu_regs.clock().cycle();
        info!(target: "apucycles", "APU cycle: {cycle} (PUT)");

        mem.apu_regs.maybe_set_frame_irq_pending(&mut mem.cpu_pinout);
        mem.apu_regs.maybe_decrement_counters();
        mem.apu_regs.apply_length_counter_pending_values();
        mem.apu_regs.execute_put_cycle(&mut mem.dmc_dma);

        if mem.apu_regs.clock().raw_cycle().is_multiple_of(20) {
            let mut queue = self.pulse_queue
                .lock()
                .unwrap();
            if queue.len() < MAX_QUEUE_LENGTH {
                queue.push_back(Apu::mix_samples(&mem.apu_regs));
            }
        }
    }

    fn execute_get_cycle(mem: &mut Memory) {
        let cycle = mem.apu_regs.clock().cycle();
        info!(target: "apucycles", "APU cycle: {cycle} (GET)");
        mem.apu_regs.maybe_set_frame_irq_pending(&mut mem.cpu_pinout);
        mem.apu_regs.apply_length_counter_pending_values();
        mem.apu_regs.execute_get_cycle();
    }

    fn mix_samples(regs: &ApuRegisters) -> f32 {
        let pulse_1 = regs.pulse_1.sample_volume();
        let pulse_2 = regs.pulse_2.sample_volume();
        let triangle = regs.triangle.sample_volume();
        let noise = regs.noise.sample_volume();
        let dmc = regs.dmc.sample_volume();

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
