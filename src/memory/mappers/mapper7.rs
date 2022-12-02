use crate::cartridge::Cartridge;
use crate::memory::cpu::cartridge_space::WindowStart::*;
use crate::memory::cpu::cartridge_space::WindowEnd::*;
use crate::memory::cpu::cartridge_space::{CartridgeSpace, PrgMemory, WindowType};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::mapped_array::{MappedArray, Chunk};
use crate::util::unit::KIBIBYTE;

const PRG_ROM_BANK_SIZE: usize = 32 * KIBIBYTE;

// AxROM
pub struct Mapper7 {
    cartridge_space: CartridgeSpace,
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
            .add_window(Ox6000, Ox7FFF,  8 * KIBIBYTE, WindowType::Empty)
            .add_window(Ox8000, OxFFFF, 32 * KIBIBYTE, WindowType::Rom { bank_index: 0 })
            .build();
        let cartridge_space = CartridgeSpace::new(prg_memory);

        assert_eq!(cartridge.chr_rom_chunks().len(), 0);
        let raw_pattern_tables = [MappedArray::<4>::empty(), MappedArray::<4>::empty()];

        Ok(Mapper7 {
            cartridge_space,
            raw_pattern_tables,
            name_table_mirroring: NameTableMirroring::OneScreenLeftBank,
        })
    }
}

impl Mapper for Mapper7 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    fn cartridge_space(&self) -> &CartridgeSpace {
        &self.cartridge_space
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

    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        if address.to_raw() >= 0x8000 {
            let bank = value & 0b0000_0111;
            self.cartridge_space.switch_prg_bank_at(Ox8000, bank);

            self.name_table_mirroring = if value & 0b0001_0000 == 0 {
                NameTableMirroring::OneScreenLeftBank
            } else {
                NameTableMirroring::OneScreenRightBank
            };
        }
    }
}
