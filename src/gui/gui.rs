use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::ops::Add;
use std::time::{Duration, SystemTime};

use log::{info, warn};

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::nes::Nes;
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::render::frame::Frame;
use crate::ppu::render::frame_rate::TargetFrameRate;

const FRAME_DUMP_DIRECTORY: &str = "framedump";

pub trait Gui {
    fn run(&mut self, nes: Nes, config: Config);
}

pub fn execute_frame<F>(nes: &mut Nes, config: &Config, events: &Events, display_frame: F)
where
    F: FnOnce(&Frame, Mask, u64),
{
    let frame_index = nes.ppu().clock().frame();
    let start_time = SystemTime::now();
    let target_frame_rate = config.target_frame_rate;
    let intended_frame_end_time = start_time.add(frame_duration(target_frame_rate));

    nes.process_gui_events(events);
    nes.step_frame();
    let mask = nes.memory_mut().as_ppu_memory().regs().mask;
    display_frame(nes.frame(), mask, frame_index);

    if config.frame_dump {
        dump_frame(nes.frame(), mask, frame_index);
    }

    end_frame(frame_index, start_time, intended_frame_end_time);

    if events.should_quit || Some(frame_index) == config.stop_frame {
        std::process::exit(0);
    }
}

fn dump_frame(frame: &Frame, mask: Mask, frame_index: u64) {
    if let Err(err) = fs::create_dir(FRAME_DUMP_DIRECTORY) {
        assert!(err.kind() == ErrorKind::AlreadyExists, "{:?}", err.kind());
    }
    let file_name = format!("{}/frame{:03}.ppm", FRAME_DUMP_DIRECTORY, frame_index,);
    let mut file = File::create(file_name).unwrap();
    file.write_all(&frame.to_ppm(mask).to_bytes()).unwrap();
}

#[inline]
fn end_frame(
    frame_index: u64,
    start_time: SystemTime,
    intended_frame_end_time: SystemTime,
) {
    let mut current_time;
    loop {
        current_time = SystemTime::now();
        if current_time.duration_since(intended_frame_end_time).is_ok() {
            break;
        }

        // TODO: We can get more accurate by skipping the yield_now when we get close.
        std::thread::yield_now();
    }

    if let Ok(duration) = current_time.duration_since(start_time) {
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
