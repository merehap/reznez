use std::fs;
use std::path::{Path, PathBuf};

use splitbits::combinebits;

use crate::assembler::raw_mem::RawMem;

pub fn assemble(source_code_path: &Path) {
    let cartridge_name = source_code_path
        .file_stem().expect("assembly file to have a file name")
        .to_str().expect("assembly file to have a UTF8-compatible file name");
    let mut assembled_cartridge_path: PathBuf = ["assembled", cartridge_name].iter().collect();
    assembled_cartridge_path.set_extension("nes");
    println!("Path: {}", assembled_cartridge_path.as_os_str().to_str().unwrap());

    let mut cartridge = Cartridge::nrom();
    cartridge.prg_rom.set_next_raw_n(0xEA, 32 * KIBIBYTE);
    let cartridge_contents = cartridge.to_bytes();

    fs::write(assembled_cartridge_path, cartridge_contents).unwrap();
}

struct Cartridge {
    mapper: u8,
    vertical_name_table_mirroring: bool,
    prg_rom: PrgRom,
    chr_rom: ChrRom,
}

impl Cartridge {
    fn nrom() -> Self {
        Self {
            mapper: 0,
            vertical_name_table_mirroring: false,
            prg_rom: PrgRom::new(32 * KIBIBYTE),
            chr_rom: ChrRom::new(8 * KIBIBYTE),
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.append(&mut vec![
            b'N',
            b'E',
            b'S',
            0x1A,
            (self.prg_rom.rom.size() / PRG_ROM_CHUNK_SIZE).try_into().unwrap(),
            (self.chr_rom.rom.size() / CHR_ROM_CHUNK_SIZE).try_into().unwrap(),
            combinebits!(self.mapper & 0b1111, self.vertical_name_table_mirroring, "mmmm000n"),
            combinebits!(self.mapper >> 4, "mmmm0000"),
            0, // 8
            0, // 9
            0, // 10
            0, // 11
            0, // 12
            0, // 13
            0, // 14
            0, // 15
        ]);
        bytes.append(&mut self.prg_rom.rom.resolve());
        bytes.append(&mut self.chr_rom.rom.resolve());

        bytes
    }
}

const KIBIBYTE: u32 = 1024;
const PRG_ROM_CHUNK_SIZE: u32 = 16 * KIBIBYTE;
const CHR_ROM_CHUNK_SIZE: u32 = 8 * KIBIBYTE;

pub struct PrgRom {
    rom: RawMem,
    index: u32,
}

impl PrgRom {
    fn new(size: u32) -> Self {
        assert!(size.is_multiple_of(PRG_ROM_CHUNK_SIZE));
        Self {
            rom: RawMem::new(size),
            index: 0,
        }
    }

    fn set_next_raw(&mut self, value: u8) {
        self.rom.set_at(self.index, value);
        self.index = self.index.checked_add(1).unwrap();
    }

    fn set_next_raw_n(&mut self, value: u8, n: u32) {
        for _ in 0..n {
            self.set_next_raw(value);
        }
    }
}

pub struct ChrRom {
    rom: RawMem,
}

impl ChrRom {
    fn new(size: u32) -> Self {
        assert!(size.is_multiple_of(CHR_ROM_CHUNK_SIZE));
        Self { rom: RawMem::new(size) }
    }
}