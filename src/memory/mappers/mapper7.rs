use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_rom::PrgRom;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::mapped_array::{MappedArray, Chunk};

// 32 KiB.
const PRG_ROM_BANK_SIZE: usize = 32 * 0x400;

// AxROM
pub struct Mapper7 {
    prg_rom: PrgRom,
    raw_pattern_tables: RawPatternTablePair,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper7 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper7, String> {
        let prg_rom = cartridge.prg_rom();
        assert_eq!(prg_rom.len() % PRG_ROM_BANK_SIZE, 0);

        let bank_count = (prg_rom.len() / PRG_ROM_BANK_SIZE).try_into()
            .map_err(|err| format!("Way too many banks. {}", err))?;

        let selected_bank_indexes = vec![0];
        let prg_rom = PrgRom::multiple_banks(
            cartridge.prg_rom(),
            bank_count,
            selected_bank_indexes,
        );

        assert_eq!(cartridge.chr_rom_chunks().len(), 0);
        let raw_pattern_tables = [MappedArray::<4>::empty(), MappedArray::<4>::empty()];
        let name_table_mirroring = NameTableMirroring::OneScreenLeftBank;

        Ok(Mapper7 {
            prg_rom,
            raw_pattern_tables,
            name_table_mirroring,
        })
    }
}

impl Mapper for Mapper7 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    fn prg_rom(&self) -> &PrgRom {
        &self.prg_rom
    }

    fn is_chr_writable(&self) -> bool {
        true
    }

    fn prg_rom_bank_string(&self) -> String {
        "Blah".to_string()
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

    fn read_prg_ram(&self, _address: CpuAddress) -> u8 {
        // FIXME: Open bus.
        0
    }

    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        if address.to_raw() >= 0x8000 {
            let bank = value & 0b0000_0111;
            self.prg_rom.select_new_banks(vec![bank]);

            self.name_table_mirroring = if value & 0b0001_0000 == 0 {
                NameTableMirroring::OneScreenLeftBank
            } else {
                NameTableMirroring::OneScreenRightBank
            };
        }
    }
}