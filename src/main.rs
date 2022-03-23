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

use bevy::prelude::*;
use bevy_pixels::prelude::*;
use structopt::StructOpt;

use crate::config::{Config, Opt};
use crate::nes::Nes;
use crate::util::logger;
use crate::util::logger::Logger;

use crate::ppu::pixel_index::PixelIndex;

use crate::gui::no_gui::NoGui;

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
    let nes = Nes::new(config);
    let pixels = [0; 3 * PixelIndex::PIXEL_COUNT];

    App::new()
        .insert_resource(PixelsOptions {
            width: 256,
            height: 240,
        })
        .insert_non_send_resource(nes)
        .insert_resource(pixels)
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
    mut pixels: ResMut<PixelsResource>,
) {
    nes.step_frame(&mut NoGui::new());

    let mask = nes.memory_mut().as_ppu_memory().regs().mask;

    let frame = pixels.pixels.get_frame();

    let mut pixels = [0; 3 * PixelIndex::PIXEL_COUNT];
    nes.ppu().frame().update_pixel_data(mask, &mut pixels);
    let mut i = 0;
    for pixel in pixels.iter() {
        if i >= frame.len() {
            return;
        }

        frame[i] = *pixel;
        i += 1;
        if i % 4 == 3 {
            frame[i] = 0xFF;
            i += 1;
        }
    }
}
