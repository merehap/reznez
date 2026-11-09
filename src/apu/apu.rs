use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use log::{info, warn, log_enabled};
use log::Level;
use rodio::{OutputStream, Sink};
use rodio::source::Source;

use crate::apu::apu_clock::CycleParity;
use crate::apu::apu_registers::ApuRegisters;
use crate::apu::mixer::{HighPassFilter, LowPassFilter, Mixer};
use crate::bus::Bus;

const MAX_QUEUE_LENGTH: usize = 2 * Mixer::SAMPLE_RATE as usize;

pub struct Apu {
    mixer: Mixer,
    #[allow(unused)]
    high90_filter: HighPassFilter,
    #[allow(unused)]
    high440_filter: HighPassFilter,
    #[allow(unused)]
    low14000_filter: LowPassFilter,

    pulse_queue: Arc<Mutex<VecDeque<f32>>>,
}

impl Apu {
    pub fn new(disable_audio: bool) -> Apu {
        // TODO: Select a proper capacity value.
        let pulse_queue = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_QUEUE_LENGTH)));

        if !disable_audio {
            let cloned_queue = pulse_queue.clone();
            thread::spawn(move || {
                let source = AudioSource::new(cloned_queue);
                let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                let sink = Sink::try_new(&stream_handle).unwrap();
                sink.append(source);

                loop {
                    thread::sleep(Duration::from_millis(1000));
                }
            });
        }

        Apu {
            mixer: Mixer::new(),
            high90_filter: HighPassFilter::new(0.98735),
            high440_filter: HighPassFilter::new(0.94099),
            low14000_filter: LowPassFilter::new(0.666),

            pulse_queue,
        }
    }

    pub fn mute(&mut self) {
        self.mixer.pulse_1_force_muted = true;
        self.mixer.pulse_2_force_muted = true;
        self.mixer.triangle_force_muted = true;
        self.mixer.noise_force_muted = true;
        self.mixer.dmc_force_muted = true;
    }

    pub fn mute_pulse_1(&mut self) {
        self.mixer.pulse_1_force_muted = true;
    }

    pub fn mute_pulse_2(&mut self) {
        self.mixer.pulse_2_force_muted = true;
    }

    pub fn mute_triangle(&mut self) {
        self.mixer.triangle_force_muted = true;
    }

    pub fn mute_noise(&mut self) {
        self.mixer.noise_force_muted = true;
    }

    pub fn mute_dmc(&mut self) {
        self.mixer.dmc_force_muted = true;
    }

    pub fn step(bus: &mut Bus) {
        let cycle = bus.master_clock.apu_clock.cpu_cycle();
        let parity = bus.master_clock.apu_clock.cycle_parity();
        let sample_index = bus.master_clock.cpu_cycle() as usize % bus.apu_regs.mixed_samples.len();
        info!(target: "apucycles", "APU cycle: {cycle} ({parity})");

        let mixed_sample = bus.apu.mixer.mix(&bus.apu_regs);
        bus.apu_regs.mixed_samples[sample_index] = mixed_sample;

        ApuRegisters::tick(bus);
        if parity == CycleParity::Put {
            bus.joypad1.tick();
            bus.joypad2.tick();

            bus.apu_regs.pulse1_volumes.push(u8::from(bus.apu_regs.pulse_1.sample_volume()).into());
            bus.apu_regs.pulse2_volumes.push(u8::from(bus.apu_regs.pulse_2.sample_volume()).into());
            bus.apu_regs.triangle_volumes.push(u8::from(bus.apu_regs.triangle.sample_volume()).into());
            bus.apu_regs.noise_volumes.push(u8::from(bus.apu_regs.noise.sample_volume()).into());
            bus.apu_regs.dmc_volumes.push(u8::from(bus.apu_regs.dmc.sample_volume()).into());
            let audio_output = bus.apu_regs.mixed_samples.iter().sum::<f32>() / bus.apu_regs.mixed_samples.len() as f32;
            bus.apu_regs.mixed_values.push(audio_output.into());

            if bus.apu_clock().raw_apu_cycle().is_multiple_of(20) {
                let mut queue = bus.apu.pulse_queue.lock().unwrap();
                if queue.len() < MAX_QUEUE_LENGTH {
                    queue.push_back(audio_output);
                } else {
                    warn!("Samples dropped: maximum APU queue length exceeded. Length: {}", queue.len());
                }

                if log_enabled!(target: "apusamples", Level::Info) {
                    fn disp(volume: u8) -> String {
                        if volume == 0 { String::new() } else { volume.to_string() }
                    }

                    let regs = &bus.apu_regs;
                    info!("{:05} ({:08}), PPU Frame: {:05}, P1: {:>2}, P2: {:>2}, T: {:>2}, N: {:>2}, D: {:>2}, Mix: {:>4}",
                        bus.master_clock.apu_clock.cpu_cycle(),
                        bus.apu_clock().raw_apu_cycle(),
                        bus.ppu_clock().frame(),
                        disp(regs.pulse_1.sample_volume().into()),
                        disp(regs.pulse_2.sample_volume().into()),
                        disp(regs.triangle.sample_volume()),
                        disp(regs.noise.sample_volume().into()),
                        disp(regs.dmc.sample_volume()),
                        mixed_sample.to_string(),
                    );
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AudioSource {
    queue: Arc<Mutex<VecDeque<f32>>>,
    previous_value: f32,
}

impl AudioSource {
    #[inline]
    pub fn new(queue: Arc<Mutex<VecDeque<f32>>>) -> Self {
        AudioSource {
            queue,
            previous_value: 0.0,
        }
    }
}

impl Iterator for AudioSource {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if let Some(value) = self.queue.lock().unwrap().pop_front() {
            self.previous_value = value;
            Some(value)
        } else {
            // If enqueuing has fallen behind, just repeat the previous mixed value and hope no one notices.
            Some(self.previous_value)
        }
    }
}

impl Source for AudioSource {
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
        Mixer::SAMPLE_RATE
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
