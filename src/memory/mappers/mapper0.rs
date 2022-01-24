use crate::cartridge::Cartridge;
use crate::cpu::address::Address as CpuAddress;
use crate::cpu::memory::Memory as CpuMemory;
use crate::memory::mapper::Mapper;
use crate::memory::ppu_address::PpuAddress;
use crate::memory::ppu_ram::PpuRam;
use crate::memory::vram::Vram;

const PRG_ROM_START: CpuAddress = CpuAddress::new(0x8000);
const CHR_ROM_END: PpuAddress = PpuAddress::from_u16(0x2000);

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
        let palette_ram = &ppu_ram.palette_ram;
        let vram = &ppu_ram.vram;

        let index = address.to_usize();
        match address.to_u16() {
            0x0000..=0x1FFF => self.chr_rom[index],
            0x3F00..=0x3FFF => palette_ram[index % 0x20],
            // Address is out of normal range so mirror down and try again.
            0x4000..=0xFFFF => self.ppu_read(ppu_ram, address.reduce()),
            _ => vram.read(address),
        }
    }

    #[inline]
    fn ppu_write(&mut self, ppu_ram: &mut PpuRam, address: PpuAddress, value: u8) {
        let palette_ram = &mut ppu_ram.palette_ram;
        let vram = &mut ppu_ram.vram;

        let index = address.to_usize();
        match address.to_u16() {
            0x0000..=0x1FFF => self.chr_rom[index] = value,
            0x3F00..=0x3FFF => palette_ram[index % 0x20] = value,
            // Address is out of normal range so mirror down and try again.
            0x4000..=0xFFFF =>
                self.ppu_write(ppu_ram, address.reduce(), value),
            _ => vram.write(address, value),
        }
    }

    /*
    fn cpu_slice<'a>(
        &'a self,
        memory: &'a CpuMemory,
        start_address: CpuAddress,
        end_address: CpuAddress,
    ) -> &'a [u8]

        if start_address >= PRG_ROM_START && end_address > PRG_ROM_START {
            &self.prg_rom[start_address.to_usize()..end_address.to_usize() + 1]
        } else {
            memory.slice(start_address, end_address)
        }
    }
    */

    fn ppu_slice<'a>(
        &'a self,
        vram: &'a Vram,
        start_address: PpuAddress,
        end_address: PpuAddress,
    ) -> &'a [u8] {

        if start_address < CHR_ROM_END && end_address < CHR_ROM_END {
            &self.chr_rom[start_address.to_usize()..end_address.to_usize() + 1]
        } else {
            vram.slice(start_address, end_address)
        }
    }
}
