use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

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
        match opt.gui.as_str() {
            "sdl" => Box::new(SdlGui::initialize()) as Box<dyn Gui>,
            "framedump" => Box::new(FrameDumpGui::initialize()) as Box<dyn Gui>,
            _ => panic!("Invalid GUI specified: '{}'", opt.gui),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "REZNEZ", about = "The ultra-accurate NES emulator.")]
pub struct Opt {
    #[structopt(short, long, default_value = "sdl")]
    gui: String,

    #[structopt(name = "ROM", parse(from_os_str))]
    rom_path: PathBuf,
}
