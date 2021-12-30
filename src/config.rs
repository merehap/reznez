use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use structopt::StructOpt;

use crate::cartridge::INes;
use crate::cpu::address::Address;
use crate::cpu::cpu::ProgramCounterSource;
use crate::gui::gui::Gui;
use crate::gui::sdl_gui::SdlGui;
use crate::gui::frame_dump_gui::FrameDumpGui;
use crate::ppu::palette::system_palette::SystemPalette;

pub struct Config {
    ines: INes,
    system_palette: SystemPalette,
    program_counter_source: ProgramCounterSource,
}

impl Config {
    pub fn default(opt: &Opt) -> Config {
        let rom_path = Path::new(&opt.rom_path);

        println!("Loading ROM '{}'.", rom_path.display());
        let mut rom = Vec::new();
        File::open(rom_path)
            .unwrap()
            .read_to_end(&mut rom)
            .unwrap();
        let ines = INes::load(&rom).unwrap();
        println!("ROM loaded.\n{}", ines);

        let system_palette = SystemPalette::parse(include_str!("../palettes/2C02.pal"))
            .unwrap();
        let program_counter_source = ProgramCounterSource::ResetVector;

        Config {ines, system_palette, program_counter_source}
    }

    pub fn with_override_program_counter(
        opt: &Opt,
        program_counter: Address,
    ) -> Config {
        let mut result = Config::default(&opt);
        result.program_counter_source = ProgramCounterSource::Override(program_counter);
        result
    }

    pub fn ines(&self) -> &INes {
        &self.ines
    }

    pub fn system_palette(&self) -> &SystemPalette {
        &self.system_palette
    }

    pub fn program_counter_source(&self) -> ProgramCounterSource {
        self.program_counter_source
    }

    pub fn gui(opt: &Opt) -> Box<dyn Gui> {
        match opt.gui {
            GuiType::Sdl => Box::new(SdlGui::initialize()) as Box<dyn Gui>,
            GuiType::FrameDump => Box::new(FrameDumpGui::initialize()) as Box<dyn Gui>,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "REZNEZ", about = "The ultra-accurate NES emulator.")]
pub struct Opt {
    #[structopt(name = "ROM", parse(from_os_str))]
    rom_path: PathBuf,

    #[structopt(short, long, default_value = "sdl")]
    gui: GuiType,
}

//#[derive(Debug, enum_utils::FromStr)]
#[derive(Debug)]
//#[enumeration(case_insensitive)]
enum GuiType {
    Sdl,
    FrameDump,
}

impl FromStr for GuiType {
    type Err = String;

    fn from_str(value: &str) -> Result<GuiType, String> {
        match value {
            "sdl" => Ok(GuiType::Sdl),
            "framedump" => Ok(GuiType::FrameDump),
            _ => Err(format!("Invalid gui type: {}", value)),
        }
    }
}

/*
impl fmt::Display for GuiType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
*/
