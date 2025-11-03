use std::path::{Path, PathBuf};

use num_traits::FromPrimitive;
use splitbits::{combinebits, splitbits};
use ux::u2;

use crate::cartridge::cartridge_metadata::{CartridgeMetadata, CartridgeMetadataBuilder, ConsoleType};
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::util::unit::KIBIBYTE;

pub const PRG_ROM_CHUNK_LENGTH: u32 = 16 * KIBIBYTE;
pub const CHR_ROM_CHUNK_LENGTH: u32 = 8 * KIBIBYTE;
const INES_HEADER_CONSTANT: u32 = u32::from_be_bytes([b'N', b'E', b'S', 0x1A]);
const NES2_0_HEADER_CONSTANT: u8 = 0b10;

// TODO: Move path and allow_saving elsewhere.
// TODO: Rename? To CartridgeRom? Name depends on if the trainer can be called ROM or not.
// See https://wiki.nesdev.org/w/index.php?title=INES
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Cartridge {
    path: CartridgePath,

    header: CartridgeMetadata,
    title: String,
    prg_rom: RawMemory,
    chr_rom: RawMemory,
    trainer: Option<RawMemoryArray<512>>,
}

impl Cartridge {
    #[rustfmt::skip]
    pub fn load(path: &Path, raw_header_and_data: &RawMemory) -> Result<Cartridge, String> {
        let mut header = Self::parse(path, raw_header_and_data)?;

        let path = CartridgePath(path.to_path_buf());

        let prg_rom_start = 0x10;
        let prg_rom_end = prg_rom_start + header.prg_rom_size().unwrap();
        let Some(prg_rom) = raw_header_and_data.maybe_slice(prg_rom_start..prg_rom_end) else {
            return Err(format!("ROM {} was too short (claimed to have {}KiB PRG ROM).", path.rom_file_name(), header.prg_rom_size().unwrap() / KIBIBYTE));
        };
        let prg_rom = prg_rom.to_raw_memory();

        let chr_rom_start = prg_rom_end;
        let chr_rom_end = chr_rom_start + header.chr_rom_size().unwrap();
        let chr_rom = if let Some(rom) = raw_header_and_data.maybe_slice(chr_rom_start..chr_rom_end) {
            rom.to_raw_memory()
        } else {
            return Err(format!("ROM {} claimed to have {}KiB CHR ROM, but the CHR ROM section was too short ({}KiB)).",
                path.rom_file_name(),
                header.chr_rom_size().unwrap() / KIBIBYTE,
                (raw_header_and_data.size() - chr_rom_start) / KIBIBYTE,
            ));
        };

        let prg_rom_hash = crc32fast::hash(prg_rom.as_slice());
        header.set_prg_rom_hash(prg_rom_hash);
        let chr_rom_hash = crc32fast::hash(chr_rom.as_slice());
        header.set_chr_rom_hash(chr_rom_hash);

        let title_start = chr_rom_end;
        let title = raw_header_and_data.slice(title_start..raw_header_and_data.size()).to_raw().to_vec();
        let title_length_is_proper = title.is_empty() || title.len() == 127 || title.len() == 128;
        if !title_length_is_proper {
            return Err(format!("Title must be empty or 127 or 128 bytes, but was {} bytes.", title.len()));
        }

        let title = std::str::from_utf8(&title)
            .map_err(|err| err.to_string())?
            .chars()
            .take_while(|&c| c != '\u{0}')
            .collect();

        Ok(Cartridge { path, header, title, trainer: None, prg_rom, chr_rom })
    }

    pub fn name(&self) -> String {
        self.path.rom_name()
    }

    pub fn header(&self) -> &CartridgeMetadata {
        &self.header
    }

    pub fn path(&self) -> &CartridgePath {
        &self.path
    }

    pub fn prg_rom(&self) -> &RawMemory {
        &self.prg_rom
    }

    pub fn chr_rom(&self) -> &RawMemory {
        &self.chr_rom
    }

    pub fn prg_rom_size(&self) -> u32 {
        self.prg_rom.size()
    }

    pub fn chr_rom_size(&self) -> u32 {
        self.chr_rom.size()
    }

    fn parse(path: &Path, raw_header_and_data: &RawMemory) -> Result<CartridgeMetadata, String> {
        let Some(low_header) = raw_header_and_data.peek_u64(0..=7) else {
            return Err(format!("ROM file should have a 16 byte header. ROM: {}", path.display()));
        };
        let low_header = splitbits!(low_header, "iiiiiiii iiiiiiii iiiiiiii iiiiiiii pppppppp cccccccc llllntbn mmmmvvxx");
        if low_header.i != INES_HEADER_CONSTANT {
            return Err(format!("Cannot load non-iNES ROM. Found {:08X} but need {INES_HEADER_CONSTANT:08X}.", low_header.i));
        }

        if low_header.t {
            return Err(format!("Trainer isn't implemented yet. ROM: {}", path.display()));
        }

        let mut builder = CartridgeMetadataBuilder::new();
        builder
            .has_persistent_memory(low_header.b)
            .name_table_mirroring_index(u2::new(low_header.n))
            .full_hash(crc32fast::hash(raw_header_and_data.as_slice()));

        if low_header.v == NES2_0_HEADER_CONSTANT {
            // NES2.0 fields
            let Some(high_header) = raw_header_and_data.peek_u64(8..=15) else {
                return Err(format!("ROM file should have a 16 byte header. ROM: {}", path.display()));
            };
            let high_header = splitbits!(high_header, "ssssmmmm ccccpppp ffffgggg hhhhiiii ......tt vvvvxxxx ......rr ..dddddd");
            assert!(high_header.c != 0xF, "CHR exponent notation not yet supported.");
            assert!(high_header.p != 0xF, "PRG exponent notation not yet supported.");

            let mapper_number = combinebits!(high_header.m, low_header.m, low_header.l, "0000hhhh mmmmllll");
            let console_type = ConsoleType::extended(low_header.x, high_header.x)?;
            builder
                .mapper_and_submapper_number(mapper_number, Some(high_header.s))
                .prg_rom_size(combinebits!(high_header.p, low_header.p, "000000hh hhllllll ll000000 00000000"))
                .chr_rom_size(combinebits!(high_header.c, low_header.c, "0000000h hhhlllll lll00000 00000000"))
                .prg_save_ram_size(if high_header.f == 0 { 0 } else { 64 << high_header.f })
                .prg_work_ram_size(if high_header.g == 0 { 0 } else { 64 << high_header.g })
                .chr_save_ram_size(if high_header.h == 0 { 0 } else { 64 << high_header.h })
                .chr_work_ram_size(if high_header.i == 0 { 0 } else { 64 << high_header.i })
                .console_type(console_type)
                .timing_mode(FromPrimitive::from_u8(high_header.t).unwrap())
                .miscellaneous_rom_count(high_header.r)
                .default_expansion_device(FromPrimitive::from_u8(high_header.d).unwrap());

                if console_type == ConsoleType::Vs {
                    builder
                        .vs_hardware_type(FromPrimitive::from_u8(high_header.v).unwrap())
                        .vs_ppu_type(FromPrimitive::from_u8(high_header.x).unwrap());
                }
        } else {
            // iNES only (*no* NES2.0 fields)
            let mapper_number = combinebits!(low_header.m, low_header.l, "00000000 mmmmllll");
            builder
                .mapper_and_submapper_number(mapper_number, None)
                .prg_rom_size(u32::from(low_header.p) * PRG_ROM_CHUNK_LENGTH)
                .chr_rom_size(u32::from(low_header.c) * CHR_ROM_CHUNK_LENGTH)
                .console_type(ConsoleType::basic(low_header.x)?);
        }

        Ok(builder.build())
    }
}

#[derive(Clone, Debug)]
pub struct CartridgePath(PathBuf);

impl CartridgePath {
    pub fn rom_name(&self) -> String {
        self.0.file_stem().unwrap().to_str().unwrap().to_string()
    }

    pub fn rom_file_name(&self) -> String {
        self.0.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn to_prg_save_ram_file_path(&self) -> PathBuf {
        let mut save_path = PathBuf::new();
        save_path.push("saveram");
        save_path.push(self.0.file_stem().unwrap());
        save_path.set_extension("prg.saveram");
        save_path
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PlayChoice {
    inst_rom: [u8; 8192],
    prom_data: [u8; 16],
    prom_counter_out: [u8; 16],
}