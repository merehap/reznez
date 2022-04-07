use std::fmt;

use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

const INES_HEADER_CONSTANT: &[u8] = &[0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_CHUNK_LENGTH: usize = 0x4000;
const CHR_ROM_CHUNK_LENGTH: usize = 0x2000;

// See https://wiki.nesdev.org/w/index.php?title=INES
#[derive(Clone, Debug)]
pub struct Cartridge {
    name: String,

    mapper_number: u8,
    name_table_mirroring: NameTableMirroring,
    has_persistent_memory: bool,
    ripper_name: String,
    ines2: Option<INes2>,

    trainer: Option<[u8; 512]>,
    prg_rom_chunks: Vec<Box<[u8; 0x4000]>>,
    chr_rom_chunks: Vec<Box<[u8; 0x2000]>>,
    console_type: ConsoleType,
    title: String,
}

impl Cartridge {
    pub fn load(name: String, rom: &[u8]) -> Result<Cartridge, String> {
        if &rom[0..4] != INES_HEADER_CONSTANT {
            return Err(format!(
                "Cannot load non-iNES ROM. Found {:?} but need {:?}.",
                &rom[0..4],
                INES_HEADER_CONSTANT,
            ));
        }

        let prg_rom_chunk_count = rom[4] as usize;
        let chr_rom_chunk_count = rom[5] as usize;

        let lower_mapper_number   = rom[6] & 0b1111_0000;
        let four_screen           = rom[6] & 0b0000_1000 != 0;
        let trainer_enabled       = rom[6] & 0b0000_0100 != 0;
        let has_persistent_memory = rom[6] & 0b0000_0010 != 0;
        let vertical_mirroring    = rom[6] & 0b0000_0001 != 0;

        let upper_mapper_number   = rom[7] & 0b1111_0000;
        let ines2_present         = rom[7] & 0b0000_1100 == 0b0000_1100;
        let play_choice_enabled   = rom[7] & 0b0000_0010 != 0;
        let vs_unisystem_enabled  = rom[7] & 0b0000_0001 != 0;

        let ripper_name: String = std::str::from_utf8(&rom[8..15])
            .map_err(|err| err.to_string())?
            .chars()
            .map(|c| if c.is_ascii_graphic() {c} else {'~'})
            .collect();

        if trainer_enabled {
            unimplemented!("Trainer isn't implemented yet.");
        }

        if ines2_present {
            unimplemented!("iNES2 isn't implemented yet.");
        }

        if play_choice_enabled {
            unimplemented!("PlayChoice isn't implemented yet.");
        }

        if vs_unisystem_enabled {
            unimplemented!("VS Unisystem isn't implemented yet.");
        }

        let mapper_number = upper_mapper_number | (lower_mapper_number >> 4);
        let name_table_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => NameTableMirroring::FourScreen,
            (_, false) => NameTableMirroring::Horizontal,
            (_, true) => NameTableMirroring::Vertical,
        };

        let mut rom_index = 0x10;

        let mut prg_rom_chunks = Vec::new();
        for _ in 0..prg_rom_chunk_count {
            prg_rom_chunks.push(Box::new(rom[rom_index..rom_index + PRG_ROM_CHUNK_LENGTH].try_into().unwrap()));
            rom_index += PRG_ROM_CHUNK_LENGTH;
        }

        let mut chr_rom_chunks = Vec::new();
        for _ in 0..chr_rom_chunk_count {
            //FIXME: Yoshi.nes panics with OutOfRange here.
            chr_rom_chunks.push(Box::new(rom[rom_index..rom_index + CHR_ROM_CHUNK_LENGTH].try_into().unwrap()));
            rom_index += CHR_ROM_CHUNK_LENGTH;
        }

        let title = rom[rom_index..].to_vec();
        let title_length_is_proper = title.is_empty() || title.len() == 127 || title.len() == 128;
        if !title_length_is_proper {
            return Err(format!("Title must be empty or 127 or 128 bytes, but was {} bytes.", title.len()));
        }

        let title = std::str::from_utf8(&title)
            .map_err(|err| err.to_string())?
            .chars()
            .take_while(|&c| c != '\u{0}')
            .collect();

        Ok(Cartridge {
            name,

            mapper_number,
            name_table_mirroring,
            has_persistent_memory,
            ripper_name,
            ines2: None,

            trainer: None,
            prg_rom_chunks,
            chr_rom_chunks,
            console_type: ConsoleType::Nes,
            title,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mapper_number(&self) -> u8 {
        self.mapper_number
    }

    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }
    
    pub fn prg_rom_chunks(&self) -> &[Box<[u8; 0x4000]>] {
        &self.prg_rom_chunks
    }

    pub fn prg_rom(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for chunk in &self.prg_rom_chunks {
            result.extend_from_slice(chunk.as_ref());
        }

        result
    }

    pub fn chr_rom_chunks(&self) -> &[Box<[u8; 0x2000]>] {
        &self.chr_rom_chunks
    }

    pub fn chr_rom_half_chunks(&self) -> Vec<[u8; 0x1000]> {
        let mut half_chunks = Vec::new();
        for chunk in &self.chr_rom_chunks {
            half_chunks.push(chunk[0x0000..0x1000].try_into().unwrap());
            half_chunks.push(chunk[0x1000..0x2000].try_into().unwrap());
        }

        half_chunks
    }

    pub fn chr_rom(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for chunk in &self.chr_rom_chunks {
            result.extend_from_slice(chunk.as_ref());
        }

        result
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mapper: {}", self.mapper_number)?;
        writeln!(f, "Nametable mirroring: {:?}", self.name_table_mirroring)?;
        writeln!(f, "Persistent memory: {}", self.has_persistent_memory)?;
        writeln!(f, "Ripper: {}", self.ripper_name)?;
        writeln!(f, "iNES2 present: {}", self.ines2.is_some())?;

        writeln!(f, "Trainer present: {}", self.trainer.is_some())?;
        writeln!(f, "PRG ROM chunk count: {}", self.prg_rom_chunks.len())?;
        writeln!(f, "CHR ROM chunk count: {}", self.chr_rom_chunks.len())?;
        writeln!(f, "Console type: {:?}", self.console_type)?;
        writeln!(f, "Title: {:?}", self.title)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct INes2 {

}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum ConsoleType {
    Nes,
    VsUnisystem,
    PlayChoice10(Box<PlayChoice>),
    Extended,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PlayChoice {
    inst_rom: [u8; 8192],
    prom_data: [u8; 16],
    prom_counter_out: [u8; 16],
}


#[cfg(test)]
pub mod test_data {
    use crate::memory::cpu::cpu_address::CpuAddress;

    use super::*;

    pub fn cartridge() -> Cartridge {
        let mut prg_rom_chunks = vec![Box::new([0xEA; PRG_ROM_CHUNK_LENGTH])];
        let len = prg_rom_chunks[0].len();
        // Overwrite the NMI/RESET/IRQ Vectors so they doesn't point to ROM.
        // This allows injection of custom instructions for testing.
        prg_rom_chunks[0][len - 6] = 0x00;
        prg_rom_chunks[0][len - 5] = 0x02;
        prg_rom_chunks[0][len - 4] = 0x00;
        prg_rom_chunks[0][len - 3] = 0x02;
        prg_rom_chunks[0][len - 2] = 0x00;
        prg_rom_chunks[0][len - 1] = 0x02;

        Cartridge {
            name: "Test".to_string(),
            mapper_number: 0,
            name_table_mirroring: NameTableMirroring::Horizontal,
            has_persistent_memory: false,
            ripper_name: "Test Ripper".to_string(),
            ines2: None,

            trainer: None,
            prg_rom_chunks,
            chr_rom_chunks: vec![Box::new([0x00; CHR_ROM_CHUNK_LENGTH])],
            console_type: ConsoleType::Nes,
            title: "Test ROM".to_string(),
        }
    }

    pub fn cartridge_with_prg_rom(
        prg_rom_chunks: [Vec<u8>; 2],
        nmi_vector: CpuAddress,
        reset_vector: CpuAddress,
        irq_vector: CpuAddress,
    ) -> Cartridge {

        // Filled with NOPs.
        let mut prg_chunks = [Box::new([0xEA; PRG_ROM_CHUNK_LENGTH]), Box::new([0xEA; PRG_ROM_CHUNK_LENGTH])];
        for chunk_index in 0..2 {
            for i in 0..prg_rom_chunks[chunk_index].len() {
                prg_chunks[chunk_index][i] = prg_rom_chunks[chunk_index][i];
            }
        }

        let len = prg_chunks[1].len();
        let (low, high) = nmi_vector.to_low_high();
        prg_chunks[1][len - 6] = low;
        prg_chunks[1][len - 5] = high;
        let (low, high) = reset_vector.to_low_high();
        prg_chunks[1][len - 4] = low;
        prg_chunks[1][len - 3] = high;
        let (low, high) = irq_vector.to_low_high();
        prg_chunks[1][len - 2] = low;
        prg_chunks[1][len - 1] = high;

        Cartridge {
            name: "Test".to_string(),

            mapper_number: 0,
            name_table_mirroring: NameTableMirroring::Horizontal,
            has_persistent_memory: false,
            ripper_name: "Test Ripper".to_string(),
            ines2: None,

            trainer: None,
            prg_rom_chunks: prg_chunks.to_vec(),
            chr_rom_chunks: vec![Box::new([0x00; CHR_ROM_CHUNK_LENGTH])],
            console_type: ConsoleType::Nes,
            title: "Test ROM".to_string(),
        }
    }
}
