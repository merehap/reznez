use std::collections::BTreeMap;
use std::ops::Add;
use std::time::{Duration, SystemTime};

use log::{info, warn};

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::nes::Nes;
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::render::frame::Frame;
use crate::ppu::render::frame_rate::TargetFrameRate;

pub trait Gui {
    fn run(&mut self, nes: Nes, config: Config);
}

pub fn execute_frame<F>(nes: &mut Nes, config: &Config, events: Events, display_frame: F)
    where F: FnOnce(&Frame, Mask, u64) {

    let frame_index = nes.ppu().clock().frame();
    let start_time = SystemTime::now();
    let target_frame_rate = config.target_frame_rate;
    let intended_frame_end_time = start_time.add(frame_duration(target_frame_rate));

    nes.process_gui_events(&events);
    nes.step_frame();
    let mask = nes.memory_mut().as_ppu_memory().regs().mask;
    display_frame(&nes.ppu().frame(), mask, frame_index);

    end_frame(frame_index, start_time, intended_frame_end_time);
    if events.should_quit || Some(frame_index) == config.stop_frame {
        std::process::exit(0);
    }
}

#[inline]
fn end_frame(frame_index: u64, start_time: SystemTime, intended_frame_end_time: SystemTime) {
    let end_time = SystemTime::now();
    if let Ok(duration) = intended_frame_end_time.duration_since(end_time) {
        std::thread::sleep(duration);
    }

    let end_time = SystemTime::now();
    if let Ok(duration) = end_time.duration_since(start_time) {
        info!(
            "Frame {} rendered. Framerate: {}",
            frame_index,
            1_000_000_000.0 / duration.as_nanos() as f64,
        );
    } else {
        warn!("Unknown framerate. System clock went backwards.");
    }
}

fn frame_duration(target_frame_rate: TargetFrameRate) -> Duration {
    match target_frame_rate {
        TargetFrameRate::Value(frame_rate) => frame_rate.to_frame_duration(),
        TargetFrameRate::Unbounded => Duration::ZERO,
    }
}

pub struct Events {
    pub should_quit: bool,
    pub joypad1_button_statuses: BTreeMap<Button, ButtonStatus>,
    pub joypad2_button_statuses: BTreeMap<Button, ButtonStatus>,
}

impl Events {
    pub fn none() -> Events {
        Events {
            should_quit: false,
            joypad1_button_statuses: BTreeMap::new(),
            joypad2_button_statuses: BTreeMap::new(),
        }
    }
}
