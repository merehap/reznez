use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::info;
use structopt::StructOpt;

use crate::cartridge::cartridge::Cartridge;
use crate::cartridge::header_db::HeaderDb;
use crate::gui::egui_gui::EguiGui;
use crate::gui::gui::Gui;
use crate::gui::no_gui::NoGui;
use crate::memory::raw_memory::RawMemory;
use crate::ppu::clock::Clock;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::render::frame_rate::TargetFrameRate;

pub struct Config {
    pub cartridge: Cartridge,
    pub starting_cpu_cycle: i64,
    pub ppu_clock: Clock,
    pub system_palette: SystemPalette,
    pub target_frame_rate: TargetFrameRate,
    pub disable_audio: bool,
    pub stop_frame: Option<i64>,
    pub frame_dump: bool,
    pub joypad_enabled: bool,
}

impl Config {
    pub fn new(opt: &Opt) -> Config {
        let rom_path = Path::new(&opt.rom_path);

        info!("Loading ROM '{}'.", rom_path.display());
        let mut rom = Vec::new();
        File::open(rom_path).unwrap().read_to_end(&mut rom).unwrap();
        let rom = RawMemory::from_vec(rom);
        let file_name = rom_path.file_name().unwrap().to_str().unwrap().to_string();
        let cartridge = Cartridge::load(file_name, &rom, &HeaderDb::load()).unwrap();
        info!("ROM loaded.\n{}", cartridge);

        let system_palette =
            SystemPalette::parse(include_str!("../palettes/2C02.pal")).unwrap();

        Config {
            cartridge,
            starting_cpu_cycle: 0,
            ppu_clock: Clock::mesen_compatible(),
            system_palette,
            target_frame_rate: opt.target_frame_rate,
            disable_audio: opt.disable_audio,
            stop_frame: opt.stop_frame,
            frame_dump: opt.frame_dump,
            joypad_enabled: !opt.disable_controllers,
        }
    }

    pub fn gui(opt: &Opt) -> Box<dyn Gui> {
        match opt.gui {
            GuiType::NoGui => Box::new(NoGui::new()) as Box<dyn Gui>,
            GuiType::Egui => Box::new(EguiGui::new()),
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
    pub stop_frame: Option<i64>,

    #[structopt(name = "disableaudio", long)]
    pub disable_audio: bool,

    #[structopt(name = "logcpuall", long)]
    pub log_cpu_all: bool,

    #[structopt(name = "logcpuinstructions", long)]
    pub log_cpu_instructions: bool,

    #[structopt(name = "logcpuflowcontrol", long)]
    pub log_cpu_flow_control: bool,

    #[structopt(name = "logframes", long)]
    pub log_frames: bool,

    #[structopt(name = "logcpusteps", long)]
    pub log_cpu_steps: bool,

    #[structopt(name = "logppuall", long)]
    pub log_ppu_all: bool,

    #[structopt(name = "logppustages", long)]
    pub log_ppu_stages: bool,

    #[structopt(name = "logppuflags", long)]
    pub log_ppu_flags: bool,

    #[structopt(name = "logppusteps", long)]
    pub log_ppu_steps: bool,

    #[structopt(name = "logapuall", long)]
    pub log_apu_all: bool,

    #[structopt(name = "logapucycles", long)]
    pub log_apu_cycles: bool,

    #[structopt(name = "logapuevents", long)]
    pub log_apu_events: bool,

    #[structopt(name = "logoamaddr", long)]
    pub log_oam_addr: bool,

    #[structopt(name = "logtimings", long)]
    pub log_timings: bool,

    #[structopt(name = "framedump", long)]
    pub frame_dump: bool,

    #[structopt(long)]
    pub analysis: bool,

    #[structopt(name = "disablecontrollers", long)]
    pub disable_controllers: bool,
}

#[derive(Debug)]
pub enum GuiType {
    NoGui,
    Egui,
}

impl FromStr for GuiType {
    type Err = String;

    fn from_str(value: &str) -> Result<GuiType, String> {
        match value.to_lowercase().as_str() {
            "nogui" => Ok(GuiType::NoGui),
            "egui" => Ok(GuiType::Egui),
            _ => Err(format!("Invalid gui type: {value}")),
        }
    }
}
