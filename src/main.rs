#![feature(if_let_guard)]
#![feature(type_ascription)]
#![feature(const_for)]
#![feature(const_option_ops)]
#![feature(const_trait_impl)]
#![feature(generic_const_parameter_types)]
#![feature(const_cmp)]
#![feature(adt_const_params)]
#![feature(unsized_const_params)]
#![feature(panic_update_hook)]
#![allow(incomplete_features)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]
#![allow(clippy::identity_op)]

mod apu;
mod analysis;
mod assembler;
mod bus;
mod cartridge;
mod config;
mod controller;
mod counter;
mod cpu;
mod gui;
mod logging;
mod mapper;
mod mapper_list;
mod mappers;
mod master_clock;
mod memory;
pub mod nes;
mod ppu;
mod util;

use std::panic;
use std::sync::{Arc, Mutex};

use structopt::StructOpt;

use crate::cartridge::cartridge_metadata::{ConsoleType, TimingMode};
use crate::cartridge::header_db::HeaderDb;
use crate::config::{Config, Opt};
use crate::logging::logger;
use crate::logging::logger::Logger;
use crate::nes::Nes;


fn main() {
    let opt = Opt::from_args();
    logger::init(logger(&opt)).unwrap();
    panic::update_hook(|prev, info| {
        log::logger().flush();
        prev(info);
    });

    if let Some(source_code_path) = opt.assemble {
        assert!(opt.rom_path.is_none(), "Can't specify both a ROM and code to assemble.");
        assembler::assembler::assemble(&source_code_path);
        return;
    }

    let config = Config::new(&opt);
    let mut gui = Config::gui(&opt);
    let nes = opt.rom_path.map(|path| {
        let cartridge = Nes::load_cartridge(&path)
            .map_err(|err| format!("Failed to start REZNEZ. {err}"))
            .unwrap();
        let nes = Nes::new(&HeaderDb::load(), &config, &cartridge)
            .map_err(|err| format!("Failed to start REZNEZ. {err}"))
            .unwrap();
        assert_eq!(nes.resolved_metadata().console_type, ConsoleType::NesFamiconDendy);
        assert_eq!(nes.resolved_metadata().miscellaneous_rom_count, 0, "Miscellaneous ROM sections not yet supported.");
        assert!(matches!(nes.resolved_metadata().region_timing_mode, TimingMode::Ntsc | TimingMode::MultiRegion));
        assert!(nes.resolved_metadata().vs.is_none());

        nes
    });

    gui.run(nes, config);
}

#[allow(clippy::similar_names)]
fn logger(opt: &Opt) -> Logger {
    let (log_cpu_instructions, log_cpu_steps, log_cpu_flow_control) = if opt.log_cpu_all {
        (true, true, true)
    } else {
        (opt.log_cpu_instructions, opt.log_cpu_steps, opt.log_cpu_flow_control)
    };

    let (log_ppu_stages, log_ppu_flags, log_ppu_steps) = if opt.log_ppu_all {
        (true, true, true)
    } else {
        (opt.log_ppu_stages, opt.log_ppu_flags, opt.log_ppu_steps)
    };

    let (log_apu_cycles, log_apu_events) = if opt.log_apu_all {
        (true, true)
    } else {
        (opt.log_apu_cycles, opt.log_apu_events)
    };

    Logger {
        disable_all: false,
        log_frames: opt.log_frames,
        log_cpu_instructions,
        log_cpu_steps,
        log_cpu_flow_control,
        log_cpu_mode: opt.log_cpu_mode,
        log_detailed_cpu_mode: opt.log_detailed_cpu_mode,
        log_ppu_stages,
        log_ppu_flags,
        log_ppu_steps,
        log_apu_cycles,
        log_apu_events,
        log_apu_samples: opt.log_apu_samples,
        log_oam_addr: opt.log_oam_addr,
        log_mapper_updates: opt.log_mapper_updates,
        log_mapper_irq_counter: opt.log_mapper_irq_counter,
        log_mapper_ram_writes: opt.log_mapper_ram_writes,
        log_timings: opt.log_timings,

        buffer: Arc::new(Mutex::new(String::new())),
    }
}
