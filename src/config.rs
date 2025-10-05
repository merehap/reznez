use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;

use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::egui_gui::EguiGui;
use crate::gui::gui::Gui;
use crate::gui::no_gui::NoGui;
use crate::ppu::clock::Clock;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::render::frame_rate::{FrameRate, TargetFrameRate};

pub struct Config {
    pub starting_cpu_cycle: i64,
    pub ppu_clock: Clock,
    pub system_palette: SystemPalette,
    pub target_frame_rate: TargetFrameRate,
    pub disable_audio: bool,
    pub stop_frame: Option<i64>,
    pub frame_dump: bool,
    pub cpu_step_formatting: CpuStepFormatting,
    pub allow_saving: bool,
    pub scheduled_button_events: BTreeMap<i64, (Button, ButtonStatus)>,
}

impl Config {
    pub fn new(opt: &Opt) -> Config {
        Config {
            starting_cpu_cycle: 0,
            ppu_clock: Clock::mesen_compatible(),
            system_palette: SystemPalette::parse(include_str!("../palettes/2C02.pal")).unwrap(),
            target_frame_rate: opt.target_frame_rate,
            disable_audio: opt.disable_audio,
            stop_frame: opt.stop_frame,
            frame_dump: opt.frame_dump,
            cpu_step_formatting: opt.cpu_step_formatting,
            allow_saving: !opt.prevent_saving,
            scheduled_button_events: Config::parse_scheduled_button_events(&opt.scheduled_button_presses),
        }
    }

    pub fn gui(opt: &Opt) -> Box<dyn Gui> {
        match opt.gui {
            GuiType::NoGui => Box::new(NoGui) as Box<dyn Gui>,
            GuiType::Egui => Box::new(EguiGui),
        }
    }

    fn parse_scheduled_button_events(raw_presses: &[String]) -> BTreeMap<i64, (Button, ButtonStatus)> {
        let mut events = BTreeMap::new();
        for raw_press in raw_presses {
            for button in Button::ALL {
                let button_text = format!("{button:?}").to_ascii_lowercase();
                if raw_press.to_ascii_lowercase().starts_with(&button_text) {
                    let raw_frame_number = &raw_press[button_text.len() ..];
                    let frame_number: i64 = raw_frame_number.parse().unwrap();
                    events.insert(frame_number, (button, ButtonStatus::Pressed));
                    events.insert(frame_number + 1, (button, ButtonStatus::Unpressed));
                }
            }
        }

        // One press and one release for every raw press.
        assert_eq!(events.len(), 2 * raw_presses.len());

        events
    }
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "REZNEZ", about = "The ultra-accurate NES emulator.")]
pub struct Opt {
    #[structopt(name = "ROM", parse(from_os_str))]
    pub rom_path: Option<PathBuf>,

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

    #[structopt(name = "logmapperirqcounter", long)]
    pub log_mapper_irq_counter: bool,

    #[structopt(name = "logtimings", long)]
    pub log_timings: bool,

    #[structopt(name = "cpustepformatting", long, default_value = "data")]
    pub cpu_step_formatting: CpuStepFormatting,

    #[structopt(name = "framedump", long)]
    pub frame_dump: bool,

    #[structopt(name = "preventsaving", long)]
    pub prevent_saving: bool,

    #[structopt(name = "press", long)]
    pub scheduled_button_presses: Vec<String>,
}

impl Opt {
    pub fn new(rom_path: Option<PathBuf>) -> Self {
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
            log_mapper_irq_counter: false,
            log_timings: false,
            cpu_step_formatting: CpuStepFormatting::Data,
            frame_dump: false,
            prevent_saving: false,
            scheduled_button_presses: Vec::new(),
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
