use crate::cartridge::Cartridge;
use crate::cpu::address::Address as CpuAddress;
use crate::cpu::memory::Memory as CpuMemory;
use crate::memory::mapper::*;
use crate::memory::ppu_address::PpuAddress;
use crate::memory::ppu_ram::PpuRam;

const PRG_ROM_START: CpuAddress = CpuAddress::new(0x8000);

pub struct Mapper0 {
    prg_rom: Box<[u8; 0x8000]>,
    chr_rom: Box<[u8; 0x2000]>,
}

impl Mapper0 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper0, String> {
        let mut prg_rom = Box::new([0; 0x8000]);
        match cartridge.prg_rom_chunk_count() {
            /* Nrom128 - Mirrored mappings. */
            1 => {
                for (i, byte) in cartridge.prg_rom().iter().enumerate().take(0x4000) {
                    prg_rom[i] = *byte;
                    prg_rom[i + 0x4000] = *byte;
                }
            },
            /* Nrom256 - A single long mapping. */
            2 => {
                for (i, byte) in cartridge.prg_rom().iter().enumerate().take(0x8000) {
                    prg_rom[i] = *byte;
                }
            },
            c => return Err(format!(
                     "PRG ROM size must be 16K or 32K for mapper 0, but was {}K",
                     16 * u16::from(c),
                 )),
        }

        if cartridge.chr_rom_chunk_count() > 1 {
            return Err(format!(
                "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                8 * cartridge.chr_rom_chunk_count()
            ));
        }

        let chr_rom =
            if cartridge.chr_rom().is_empty() {
                // Provide empty CHR ROM if the cartridge doesn't provide any.
                Box::new([0; 0x2000])
            } else {
                Box::new(cartridge.chr_rom()[0x0..0x2000].try_into().unwrap())
            };

        Ok(Mapper0 {prg_rom, chr_rom})
    }
}

impl Mapper for Mapper0 {
    #[inline]
    fn cpu_read(&self, memory: &mut CpuMemory, address: CpuAddress) -> u8 {
        if address < PRG_ROM_START {
            memory.read(address)
        } else {
            self.prg_rom[address.to_usize() - PRG_ROM_START.to_usize()]
        }
    }

    #[inline]
    fn cpu_write(&self, memory: &mut CpuMemory, address: CpuAddress, value: u8) {
        if address < PRG_ROM_START {
            memory.write(address, value);
        } else {
            println!("ROM CPU write ignored ({}).", address);
        }
    }

    #[inline]
    fn ppu_read(&self, ppu_ram: &PpuRam, address: PpuAddress) -> u8 {
        let index = address.to_usize();
        match address.to_u16() {
            0x0000..=0x1FFF => self.chr_rom[index],
            0x2000..=0x3EFF => self.name_table_byte(&ppu_ram, address),
            0x3F00..=0x3FFF => self.palette_table_byte(&ppu_ram.palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn ppu_write(&mut self, ppu_ram: &mut PpuRam, address: PpuAddress, value: u8) {
        let index = address.to_usize();
        match address.to_u16() {
            0x0000..=0x1FFF => self.chr_rom[index] = value,
            0x2000..=0x3EFF => *self.name_table_byte_mut(ppu_ram, address) = value,
            0x3F00..=0x3FFF => *self.palette_table_byte_mut(&mut ppu_ram.palette_ram, address) = value,
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    fn raw_pattern_table(&self) -> &[u8; PATTERN_TABLE_SIZE] {
        self.chr_rom.as_ref()
    }

    fn raw_pattern_table_mut(&mut self) -> &mut [u8; PATTERN_TABLE_SIZE] {
        self.chr_rom.as_mut()
    }
}
