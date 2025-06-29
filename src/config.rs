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
use crate::ppu::render::frame_rate::{FrameRate, TargetFrameRate};

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
    pub cpu_step_formatting: CpuStepFormatting,
}

impl Config {
    pub fn new(opt: &Opt) -> Config {
        let cartridge = Config::load_rom(&opt.rom_path, !opt.prevent_saving);
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
            cpu_step_formatting: opt.cpu_step_formatting,
        }
    }

    pub fn gui(opt: &Opt) -> Box<dyn Gui> {
        match opt.gui {
            GuiType::NoGui => Box::new(NoGui::new()) as Box<dyn Gui>,
            GuiType::Egui => Box::new(EguiGui),
        }
    }

    pub fn with_new_rom(&self, path: &Path) -> Config {
        Config {
            cartridge: Self::load_rom(path, self.cartridge.allow_saving()),
            system_palette: self.system_palette.clone(),
            .. *self
        }
    }

    fn load_rom(path: &Path, allow_saving: bool) -> Cartridge {
        info!("Loading ROM '{}'.", path.display());
        let mut rom = Vec::new();
        File::open(path).unwrap().read_to_end(&mut rom).unwrap();
        let rom = RawMemory::from_vec(rom);
        let cartridge = Cartridge::load(path, &rom, &HeaderDb::load(), allow_saving).unwrap();
        info!("ROM loaded.\n{}", cartridge);
        cartridge
    }
}

#[derive(Clone, Debug, StructOpt)]
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

    #[structopt(name = "logcpumode", long)]
    pub log_cpu_mode: bool,

    #[structopt(name = "logdetailedcpumode", long)]
    pub log_detailed_cpu_mode: bool,

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

    #[structopt(name = "logmapperupdates", long)]
    pub log_mapper_updates: bool,

    #[structopt(name = "logtimings", long)]
    pub log_timings: bool,

    #[structopt(name = "cpustepformatting", long, default_value = "data")]
    pub cpu_step_formatting: CpuStepFormatting,

    #[structopt(name = "framedump", long)]
    pub frame_dump: bool,

    #[structopt(long)]
    pub analysis: bool,

    #[structopt(name = "disablecontrollers", long)]
    pub disable_controllers: bool,

    #[structopt(name = "preventsaving", long)]
    pub prevent_saving: bool,
}

impl Opt {
    pub fn new(rom_path: PathBuf) -> Self {
        Self {
            rom_path,
            gui: GuiType::Egui,
            stop_frame: None,
            target_frame_rate: TargetFrameRate::Value(FrameRate::NTSC),
            disable_audio: false,
            log_frames: false,
            log_cpu_all: false,
            log_ppu_all: false,
            log_apu_all: false,
            log_cpu_instructions: false,
            log_cpu_flow_control: false,
            log_cpu_mode: false,
            log_detailed_cpu_mode: false,
            log_cpu_steps: false,
            log_ppu_stages: false,
            log_ppu_flags: false,
            log_ppu_steps: false,
            log_oam_addr: false,
            log_apu_cycles: false,
            log_apu_events: false,
            log_mapper_updates: false,
            log_timings: false,
            cpu_step_formatting: CpuStepFormatting::Data,
            frame_dump: false,
            analysis: false,
            disable_controllers: false,
            prevent_saving: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum GuiType {
    NoGui,
    Egui,
}

impl FromStr for GuiType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "nogui" => Ok(GuiType::NoGui),
            "egui" => Ok(GuiType::Egui),
            _ => Err(format!("Invalid gui type: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CpuStepFormatting {
    NoData,
    Data,
}

impl FromStr for CpuStepFormatting {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "nodata" => Ok(CpuStepFormatting::NoData),
            "data" => Ok(CpuStepFormatting::Data),
            _ => Err(format!("Invalid cpu step formatting: {value}")),
        }
    }
}
