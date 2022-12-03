use crate::cartridge::Cartridge;
use crate::memory::cpu::prg_memory::{PrgMemory, WindowType};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::mapped_array::{MappedArray, Chunk};
use crate::util::unit::KIBIBYTE;

const PRG_ROM_BANK_SIZE: usize = 32 * KIBIBYTE;

// AxROM
pub struct Mapper7 {
    prg_memory: PrgMemory,
    raw_pattern_tables: RawPatternTablePair,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper7 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper7, String> {
        let prg_rom = cartridge.prg_rom();
        let prg_rom_len = prg_rom.len();
        assert_eq!(prg_rom_len % PRG_ROM_BANK_SIZE, 0);

        let bank_count: u8 = (prg_rom.len() / PRG_ROM_BANK_SIZE).try_into()
            .map_err(|err| format!("Way too many banks. {}", err))?;

        let prg_memory = PrgMemory::builder()
            .raw_memory(prg_rom)
            .bank_count(bank_count)
            .bank_size(PRG_ROM_BANK_SIZE)
            .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, WindowType::Empty)
            .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, WindowType::Rom { bank_index: 0 })
            .build();

        assert_eq!(cartridge.chr_rom_chunks().len(), 0);
        let raw_pattern_tables = [MappedArray::<4>::empty(), MappedArray::<4>::empty()];

        Ok(Mapper7 {
            prg_memory,
            raw_pattern_tables,
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

    fn is_chr_writable(&self) -> bool {
        true
    }

    fn chr_rom_bank_string(&self) -> String {
        "Blah".to_string()
    }

    fn raw_pattern_table(&self, side: PatternTableSide) -> &RawPatternTable {
        &self.raw_pattern_tables[side as usize]
    }

    fn chr_bank_chunks(&self) -> Vec<Vec<Chunk>> {
        Vec::new()
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
