const INES_HEADER_CONSTANT: &'static [u8] = &[0x4E, 0x45, 0x53, 0x1A];

pub struct INes {
    mapper: u8,
    name_table_mirroring: NameTableMirroring,
    persistent_memory: bool,
    ines2: Option<INes2>,

    trainer: Option<[u8; 512]>,
    prg_rom: Vec<[u8; 0x4000]>,
    chr_rom: Vec<[u8; 0x2000]>,
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

        let name_table_mirroring = match (rom[6] & 0b0000_1000 != 0, rom[6] & 0b0000_0001 != 0) {
            (true, _) => NameTableMirroring::FourScreen,
            (_, false) => NameTableMirroring::Horizontal,
            (_, true) => NameTableMirroring::Vertical,
        };

        Ok(INes {
            mapper: (rom[7] & 0b1111_0000) | ((rom[6] & 0b1111_0000) >> 4),
            name_table_mirroring,
            persistent_memory: rom[6] & 0b0000_0010 != 0,
            ines2: None,

            trainer: None,
            prg_rom: Vec::new(),
            chr_rom: Vec::new(),
            console_type: ConsoleType::Nes,
            title: Vec::new(),
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
