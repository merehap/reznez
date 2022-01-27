use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::info;
use structopt::StructOpt;

use crate::cartridge::Cartridge;
use crate::cpu::cpu::ProgramCounterSource;
use crate::gui::gui::Gui;
use crate::gui::no_gui::NoGui;
use crate::gui::frame_dump_gui::FrameDumpGui;
use crate::gui::sdl_gui::SdlGui;
use crate::memory::cpu_address::CpuAddress;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::render::frame_rate::TargetFrameRate;

pub struct Config {
    pub cartridge: Cartridge,
    pub system_palette: SystemPalette,
    pub target_frame_rate: TargetFrameRate,
    pub stop_frame: Option<u64>,
    pub program_counter_source: ProgramCounterSource,
}

impl Config {
    pub fn new(opt: &Opt) -> Config {
        let rom_path = Path::new(&opt.rom_path);

        info!("Loading ROM '{}'.", rom_path.display());
        let mut rom = Vec::new();
        File::open(rom_path)
            .unwrap()
            .read_to_end(&mut rom)
            .unwrap();
        let cartridge = Cartridge::load(&rom).unwrap();
        info!("ROM loaded.\n{}", cartridge);

        let system_palette = SystemPalette::parse(include_str!("../palettes/2C02.pal"))
            .unwrap();

        let program_counter_source =
            if let Some(override_program_counter) = opt.override_program_counter {
                ProgramCounterSource::Override(override_program_counter)
            } else {
                ProgramCounterSource::ResetVector
            };

        Config {
            cartridge,
            system_palette,
            target_frame_rate: opt.target_frame_rate,
            stop_frame: opt.stop_frame,
            program_counter_source,
        }
    }

    pub fn gui(opt: &Opt) -> Box<dyn Gui> {
        match opt.gui {
            GuiType::NoGui => Box::new(NoGui::initialize()) as Box<dyn Gui>,
            GuiType::Sdl => Box::new(SdlGui::initialize()) as Box<dyn Gui>,
            GuiType::FrameDump => Box::new(FrameDumpGui::initialize()) as Box<dyn Gui>,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "REZNEZ", about = "The ultra-accurate NES emulator.")]
pub struct Opt {
    #[structopt(name = "ROM", parse(from_os_str))]
    pub rom_path: PathBuf,

    #[structopt(short, long, default_value = "sdl")]
    pub gui: GuiType,

    #[structopt(name = "targetframerate", long, default_value = "ntsc")]
    pub target_frame_rate: TargetFrameRate,

    #[structopt(name = "stopframe", long)]
    pub stop_frame: Option<u64>,

    #[structopt(name = "logcpu", long)]
    pub log_cpu: bool,

    pub override_program_counter: Option<CpuAddress>,
}

#[derive(Debug)]
pub enum GuiType {
    NoGui,
    Sdl,
    FrameDump,
}

impl FromStr for GuiType {
    type Err = String;

    fn from_str(value: &str) -> Result<GuiType, String> {
        match value.to_lowercase().as_str() {
            "nogui" => Ok(GuiType::NoGui),
            "sdl" => Ok(GuiType::Sdl),
            "framedump" => Ok(GuiType::FrameDump),
            _ => Err(format!("Invalid gui type: {}", value)),
        }
    }
}
