const INES_HEADER_CONSTANT: &'static [u8] = &[0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_CHUNK_LENGTH: usize = 0x4000;
const CHR_ROM_CHUNK_LENGTH: usize = 0x2000;

// See https://wiki.nesdev.org/w/index.php?title=INES
pub struct INes {
    mapper_number: u8,
    name_table_mirroring: NameTableMirroring,
    has_persistent_memory: bool,
    ines2: Option<INes2>,

    trainer: Option<[u8; 512]>,
    prg_rom: Vec<[u8; PRG_ROM_CHUNK_LENGTH]>,
    chr_rom: Vec<[u8; CHR_ROM_CHUNK_LENGTH]>,
    console_type: ConsoleType,
    title: Vec<u8>,
}

impl INes {
    pub fn load(rom: &[u8]) -> Result<INes, String> {
        if &rom[0..4] != INES_HEADER_CONSTANT {
            return Err(format!(
                "Cannot load non-iNES ROM. Found {:?} but need {:?}.",
                &rom[0..4],
                INES_HEADER_CONSTANT,
            ));
        }

        let prg_rom_chunk_count = rom[4];
        let chr_rom_chunk_count = rom[5];

        let lower_mapper_number   = rom[6] & 0b1111_0000;
        let four_screen           = rom[6] & 0b0000_1000 != 0;
        let trainer_enabled       = rom[6] & 0b0000_0100 != 0;
        let has_persistent_memory = rom[6] & 0b0000_0010 != 0;
        let vertical_mirroring    = rom[6] & 0b0000_0001 != 0;

        let upper_mapper_number   = rom[7] & 0b1111_0000;
        let ines2_present         = rom[7] & 0b0000_1100 == 0b0000_1100;
        let play_choice_enabled   = rom[7] & 0b0000_0010 != 0;
        let vs_unisystem_enabled  = rom[7] & 0b0000_0001 != 0;

        for i in 8..15 {
            if rom[i] != 0 {
                panic!("Unexpected high header field set! [{}]={:X}", i, rom[i]);
            }
        }

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

        let mut prg_rom = Vec::new();
        for _ in 0..prg_rom_chunk_count {
            prg_rom.push(rom[rom_index..rom_index + PRG_ROM_CHUNK_LENGTH].try_into().unwrap());
            rom_index += PRG_ROM_CHUNK_LENGTH;
        }

        let mut chr_rom = Vec::new();
        for _ in 0..chr_rom_chunk_count {
            chr_rom.push(rom[rom_index..rom_index + CHR_ROM_CHUNK_LENGTH].try_into().unwrap());
            rom_index += CHR_ROM_CHUNK_LENGTH;
        }

        let title = rom[rom_index..].to_vec();

        Ok(INes {
            mapper_number,
            name_table_mirroring,
            has_persistent_memory,
            ines2: None,

            trainer: None,
            prg_rom,
            chr_rom,
            console_type: ConsoleType::Nes,
            title,
        })
    }
}

enum NameTableMirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

struct INes2 {

}

enum ConsoleType {
    Nes,
    VsUnisystem,
    PlayChoice10(PlayChoice),
    Extended,
}

struct PlayChoice {
    inst_rom: [u8; 8192],
    prom_data: [u8; 16],
    prom_counter_out: [u8; 16],
}
