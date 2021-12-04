use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::cartridge::INes;
use crate::cpu::address::Address;
use crate::cpu::cpu::ProgramCounterSource;
use crate::ppu::palette::system_palette::SystemPalette;

pub struct Config {
    ines: INes,
    system_palette: SystemPalette,
    program_counter_source: ProgramCounterSource,
}

impl Config {
    pub fn default(rom_path: &Path) -> Config {
        let mut rom = Vec::new();
        File::open(rom_path)
            .unwrap()
            .read_to_end(&mut rom)
            .unwrap();
        let ines = INes::load(&rom).unwrap();

        let system_palette = SystemPalette::parse(include_str!("../palettes/2C02.pal"))
            .unwrap();
        let program_counter_source = ProgramCounterSource::ResetVector;

        Config {ines, system_palette, program_counter_source}
    }

    pub fn with_override_program_counter(
        rom_path: &Path,
        program_counter: Address,
    ) -> Config {
        let mut result = Config::default(rom_path);
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
}
