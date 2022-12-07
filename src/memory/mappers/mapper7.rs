use crate::cartridge::Cartridge;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrType, AddDefaultRamIfRomMissing};
use crate::memory::cpu::prg_memory::{PrgMemory, PrgType};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

const PRG_ROM_BANK_SIZE: usize = 32 * KIBIBYTE;

// AxROM
pub struct Mapper7 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper7 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper7, String> {
        if cartridge.chr_rom_chunks().len() >= 2 {
            return Err(format!(
                "CHR ROM size must be 0K or 8K for mapper 7, but was {}K",
                8 * cartridge.chr_rom_chunks().len(),
            ))
        }

        let prg_rom = cartridge.prg_rom();
        let prg_rom_len = prg_rom.len();
        assert_eq!(prg_rom_len % PRG_ROM_BANK_SIZE, 0);

        let bank_count: u8 = (prg_rom.len() / PRG_ROM_BANK_SIZE).try_into()
            .map_err(|err| format!("Way too many banks. {}", err))?;

        let prg_memory = PrgMemory::builder()
            .raw_memory(prg_rom)
            .bank_count(bank_count)
            .bank_size(PRG_ROM_BANK_SIZE)
            .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
            .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
            .build();

        let chr_memory = ChrMemory::builder()
            .raw_memory(cartridge.chr_rom())
            .bank_count(1)
            .bank_size(8 * KIBIBYTE)
            .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::Ram { bank_index: 0 })
            .build(AddDefaultRamIfRomMissing::Yes);

        Ok(Mapper7 {
            prg_memory,
            chr_memory,
            name_table_mirroring: NameTableMirroring::OneScreenLeftBank,
        })
    }
}

impl Mapper for Mapper7 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    fn chr_memory_mut(&mut self) -> &mut ChrMemory {
        &mut self.chr_memory
    }

    fn write_to_prg_memory(&mut self, address: CpuAddress, value: u8) {
        if address.to_raw() >= 0x8000 {
            let bank = value & 0b0000_0111;
            self.prg_memory.switch_bank_at(0x8000, bank);

            self.name_table_mirroring = if value & 0b0001_0000 == 0 {
                NameTableMirroring::OneScreenLeftBank
            } else {
                NameTableMirroring::OneScreenRightBank
            };
        }
    }
}
