use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::info;
use structopt::StructOpt;

use crate::cartridge::Cartridge;
use crate::cpu::cpu::ProgramCounterSource;
#[cfg(feature = "bevy")]
use crate::gui::bevy_gui::BevyGui;
use crate::gui::egui_gui::EguiGui;
use crate::gui::gui::Gui;
use crate::gui::no_gui::NoGui;
#[cfg(feature = "sdl")]
use crate::gui::sdl_gui::SdlGui;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::render::frame_rate::TargetFrameRate;

pub struct Config {
    pub cartridge: Cartridge,
    pub system_palette: SystemPalette,
    pub target_frame_rate: TargetFrameRate,
    pub disable_audio: bool,
    pub stop_frame: Option<u64>,
    pub frame_dump: bool,
    pub program_counter_source: ProgramCounterSource,
}

impl Config {
    pub fn new(opt: &Opt) -> Config {
        let rom_path = Path::new(&opt.rom_path);

        info!("Loading ROM '{}'.", rom_path.display());
        let mut rom = Vec::new();
        File::open(rom_path).unwrap().read_to_end(&mut rom).unwrap();
        let file_name = rom_path.file_name().unwrap().to_str().unwrap().to_string();
        let cartridge = Cartridge::load(file_name, &rom).unwrap();
        info!("ROM loaded.\n{}", cartridge);

        let system_palette =
            SystemPalette::parse(include_str!("../palettes/2C02.pal")).unwrap();

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
            disable_audio: opt.disable_audio,
            stop_frame: opt.stop_frame,
            frame_dump: opt.frame_dump,
            program_counter_source,
        }
    }

    pub fn gui(opt: &Opt) -> Box<dyn Gui> {
        match opt.gui {
            GuiType::NoGui => Box::new(NoGui::new()) as Box<dyn Gui>,
            #[cfg(feature = "bevy")]
            GuiType::Bevy => Box::new(BevyGui::new()),
            GuiType::Egui => Box::new(EguiGui::new()),
            #[cfg(feature = "sdl")]
            GuiType::Sdl => Box::new(SdlGui::new()),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "REZNEZ", about = "The ultra-accurate NES emulator.")]
pub struct Opt {
    #[structopt(name = "ROM", parse(from_os_str))]
    pub rom_path: PathBuf,

    #[structopt(short, long, default_value = "egui")]
    pub gui: GuiType,

    #[structopt(name = "targetframerate", long, default_value = "ntsc")]
    pub target_frame_rate: TargetFrameRate,

    #[structopt(name = "stopframe", long)]
    pub stop_frame: Option<u64>,

    #[structopt(name = "disableaudio", long)]
    pub disable_audio: bool,

    #[structopt(name = "logcpuoperations", long)]
    pub log_cpu_operations: bool,

    #[structopt(name = "logcpusteps", long)]
    pub log_cpu_steps: bool,

    #[structopt(name = "logppuoperations", long)]
    pub log_ppu_operations: bool,

    #[structopt(name = "logppusteps", long)]
    pub log_ppu_steps: bool,

    #[structopt(name = "framedump", long)]
    pub frame_dump: bool,

    #[structopt(long)]
    pub analysis: bool,

    pub override_program_counter: Option<CpuAddress>,
}

#[derive(Debug)]
pub enum GuiType {
    NoGui,
    #[cfg(feature = "bevy")]
    Bevy,
    Egui,
    #[cfg(feature = "sdl")]
    Sdl,
}

impl FromStr for GuiType {
    type Err = String;

    fn from_str(value: &str) -> Result<GuiType, String> {
        match value.to_lowercase().as_str() {
            "nogui" => Ok(GuiType::NoGui),
            #[cfg(feature = "bevy")]
            "bevy" => Ok(GuiType::Bevy),
            "egui" => Ok(GuiType::Egui),
            #[cfg(feature = "sdl")]
            "sdl" => Ok(GuiType::Sdl),
            _ => Err(format!("Invalid gui type: {value}")),
        }
    }
}
