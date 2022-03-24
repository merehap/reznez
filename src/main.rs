#![feature(array_chunks)]
#![feature(slice_as_chunks)]
#![feature(const_option)]
#![feature(if_let_guard)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

mod cartridge;
mod config;
mod controller;
mod cpu;
mod gui;
mod ppu;
mod memory;
pub mod nes;
mod util;

use std::ops::Add;
use std::time::{Duration, SystemTime};

use bevy::prelude::*;
use bevy_pixels::prelude::*;
use structopt::StructOpt;

use crate::config::{Config, Opt};
use crate::gui::gui::Events;
use crate::nes::Nes;
use crate::util::logger;
use crate::util::logger::Logger;
use crate::ppu::render::frame_rate::TargetFrameRate;

/*
fn main() {
    let opt = Opt::from_args();
    logger::init(Logger {log_cpu: opt.log_cpu}).unwrap();
    let config = Config::new(&opt);
    let mut gui = Config::gui(&opt);
    let mut nes = Nes::new(config);

    loop {
        nes.step_frame(&mut *gui);
    }
}
*/

fn main() {
    let opt = Opt::from_args();
    logger::init(Logger {log_cpu: opt.log_cpu}).unwrap();
    let config = Config::new(&opt);
    let nes = Nes::new(&config);
    App::new()
        .insert_resource(PixelsOptions {
            width: 256,
            height: 240,
        })
        .insert_non_send_resource(config)
        .insert_non_send_resource(nes)
        // Default plugins, minus logging.
        .add_plugin(bevy::core::CorePlugin)
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::input::InputPlugin)
        .add_plugin(bevy::window::WindowPlugin {add_primary_window: true, exit_on_close: true})
        .add_plugin(bevy::asset::AssetPlugin)
        .add_plugin(bevy::scene::ScenePlugin)
        .add_plugin(bevy::winit::WinitPlugin)

        // RezNEZ-specific.
        .add_plugin(PixelsPlugin)
        .add_system(main_system)
        .run();
}

fn main_system(
    mut nes: NonSendMut<Nes>,
    config: NonSend<Config>,
    mut pixels: ResMut<PixelsResource>,
) {
    step_frame(&mut nes, config);

    let mask = nes.memory_mut().as_ppu_memory().regs().mask;

    let frame = pixels.pixels.get_frame();

    nes.ppu().frame().copy_to_rgba_buffer(mask, frame.try_into().unwrap());
}

fn step_frame(nes: &mut NonSendMut<Nes>, config: NonSend<Config>) {
    let frame_index = nes.ppu().clock().frame();
    let start_time = SystemTime::now();
    let target_frame_rate: TargetFrameRate = config.target_frame_rate;
    let intended_frame_end_time = start_time.add(frame_duration(target_frame_rate));

    let events = Events::none();
    nes.process_gui_events(&events);
    nes.step_frame();

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
