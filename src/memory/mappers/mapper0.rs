use crate::cartridge::Cartridge;
use crate::memory::cpu_address::CpuAddress;
use crate::memory::cpu_internal_ram::CpuInternalRam;
use crate::memory::mapper::*;
use crate::memory::ports::Ports;

const PRG_ROM_SIZE: usize = 0x8000;

pub struct Mapper0 {
    prg_rom: Box<[u8; 0x8000]>,
    chr_rom: Box<[u8; 0x2000]>,
}

impl Mapper0 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper0, String> {
        let mut prg_rom = Box::new([0; PRG_ROM_SIZE]);
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
    fn cpu_read(
        &self,
        cpu_internal_ram: &CpuInternalRam,
        ports: &mut Ports,
        address: CpuAddress,
    ) -> u8 {

        match address.to_raw() {
            0x0000..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF],
            0x2000..=0x2007 => ports.get(address),
            0x2008..=0x3FFF => ports.get(CpuAddress::new(0x2000 + address.to_raw() % 8)),
            0x4000..=0x4013 | 0x4015 => {/* APU */ 0},
            0x4014 | 0x4016 | 0x4017 => ports.get(address),
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0x7FFF => {println!("Read from non-ROM cartridge space."); 0},
            0x8000..=0xFFFF => self.prg_rom[address.to_usize() - 0x8000],
        }
    }

    #[inline]
    fn cpu_write(
        &self,
        cpu_internal_ram: &mut CpuInternalRam,
        ports: &mut Ports,
        address: CpuAddress,
        value: u8,
    ) {

        match address.to_raw() {
            0x0000..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF] = value,
            0x2000..=0x2007 => ports.set(address, value),
            0x2008..=0x3FFF => ports.set(CpuAddress::new(0x2000 + address.to_raw() % 8), value),
            0x4000..=0x4013 | 0x4015 => {/* APU */},
            0x4014 | 0x4016..=0x4017 => ports.set(address, value),
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0x7FFF => println!("Ignored writes to non-ROM cartridge space."),
            0x8000..=0xFFFF => println!("ROM CPU write ignored ({}).", address),
        }
    }

    fn raw_pattern_table(&self) -> &[u8; PATTERN_TABLE_SIZE] {
        self.chr_rom.as_ref()
    }

    fn raw_pattern_table_mut(&mut self) -> &mut [u8; PATTERN_TABLE_SIZE] {
        self.chr_rom.as_mut()
    }
}
