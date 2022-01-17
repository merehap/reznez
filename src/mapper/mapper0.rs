use crate::cartridge::Cartridge;
use crate::cpu::address::Address as CpuAddress;
use crate::cpu::memory::Memory as CpuMem;
use crate::ppu::memory::Memory as PpuMem;

pub struct Mapper0;

impl Mapper0 {
    pub fn new() -> Mapper0 {
        Mapper0 {}
    }

    pub fn map(
        &self,
        cartridge: &Cartridge,
        cpu_mem: &mut CpuMem,
        ppu_mem: &mut PpuMem,
    ) -> Result<(), String> {

        if cartridge.chr_rom_chunk_count() > 1 {
            return Err(format!(
                    "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                    8 * cartridge.chr_rom_chunk_count()
                   ));
        }

        let high_source_index = match cartridge.prg_rom_chunk_count() {
            1 => /* Nrom128 */ 0,
            2 => /* Nrom256 */ 0x4000,
            c => return Err(format!(
                     "PRG ROM size must be 16K or 32K for mapper 0, but was {}K",
                     16 * u16::from(c),
                 )),
        };

        let prg_rom = cartridge.prg_rom();

        let mut low_address = CpuAddress::new(0x8000);
        let mut high_address = CpuAddress::new(0xC000);
        for i in 0..0x4000 {
            cpu_mem.write(low_address, prg_rom[i]);
            // Copy high ROM (for NROM256) or mirror low ROM (for NROM128).
            cpu_mem.write(high_address, prg_rom[high_source_index + i]);
            low_address.inc();
            high_address.inc();
        }

        if !cartridge.chr_rom().is_empty() {
            ppu_mem.load_chr(cartridge.chr_rom()[0x0..0x2000].try_into().unwrap());
        }

        Ok(())
    }
}
