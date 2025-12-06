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

    let cartridge = demo_cartridge();
    let cartridge_contents = cartridge.to_bytes();

    fs::write(assembled_cartridge_path, cartridge_contents).unwrap();
}

fn demo_cartridge() -> Cartridge {
    let mut cartridge = Cartridge::new(Metadata::NROM_NO_WRAM);
    cartridge.prg_rom.set_next_raw_n(0xEA, 32 * KIBIBYTE);
    cartridge
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
enum Mirroring {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
struct Metadata {
    mapper: u8,
    mirroring: Mirroring,
    prg_rom_size: u32,
    chr_rom_size: u32,
}

impl Metadata {
    pub const NROM_NO_WRAM: Metadata = MetadataBuilder::new()
        .mapper(0)
        .mirroring(Mirroring::Horizontal)
        .prg_rom_size(32 * KIBIBYTE)
        .chr_rom_size(8 * KIBIBYTE)
        .build();
}

#[derive(Clone, Copy)]
struct MetadataBuilder {
    mapper: Option<u8>,
    mirroring: Option<Mirroring>,
    prg_rom_size: Option<u32>,
    chr_rom_size: Option<u32>,
}

impl MetadataBuilder {
    const fn new() -> Self {
        Self {
            mapper: None,
            mirroring: None,
            prg_rom_size: None,
            chr_rom_size: None,
        }
    }

    pub const fn mapper(mut self, value: u8) -> Self {
        self.mapper = Some(value);
        self
    }

    pub const fn mirroring(mut self, value: Mirroring) -> Self {
        self.mirroring = Some(value);
        self
    }

    pub const fn prg_rom_size(mut self, value: u32) -> Self {
        self.prg_rom_size = Some(value);
        self
    }

    pub const fn chr_rom_size(mut self, value: u32) -> Self {
        self.chr_rom_size = Some(value);
        self
    }

    pub const fn build(self) -> Metadata {
        Metadata {
            mapper: self.mapper.expect("mapper must be set"),
            mirroring: self.mirroring.expect("mirroring must be set"),
            prg_rom_size: self.prg_rom_size.expect("prg_rom_size must be set"),
            chr_rom_size: self.chr_rom_size.expect("chr_rom_size must be set"),
        }
    }
}

struct Cartridge {
    metadata: Metadata,
    prg_rom: PrgRom,
    chr_rom: ChrRom,
}

impl Cartridge {
    pub fn new(metadata: Metadata) -> Self {
        Self {
            metadata,
            prg_rom: PrgRom::new(metadata.prg_rom_size),
            chr_rom: ChrRom::new(metadata.chr_rom_size),
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
            combinebits!(self.metadata.mapper & 0b1111, self.metadata.mirroring == Mirroring::Vertical, "mmmm000n"),
            combinebits!(self.metadata.mapper >> 4, "mmmm0000"),
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