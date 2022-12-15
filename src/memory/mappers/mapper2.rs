use crate::memory::mapper::*;

// UxROM (common usages)
pub struct Mapper2 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper2 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper2, String> {
        let prg_memory = PrgMemory::builder()
            .raw_memory(cartridge.prg_rom())
            .max_bank_count(256)
            .bank_size(16 * KIBIBYTE)
            .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
            .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Rom(BankIndex::FIRST))
            .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Rom(BankIndex::LAST))
            .build();

        // Only one bank, so not bank-switched.
        let chr_memory = ChrMemory::builder()
            .raw_memory(cartridge.chr_rom())
            .max_bank_count(1)
            .bank_size(8 * KIBIBYTE)
            .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::Rom(BankIndex::FIRST))
            .add_default_ram_if_chr_data_missing();

        Ok(Mapper2 {
            prg_memory,
            chr_memory,
            name_table_mirroring: cartridge.name_table_mirroring(),
        })
    }
}

impl Mapper for Mapper2 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF =>
                self.prg_memory.window_at(0x8000).switch_bank_to(BankIndex::from_u8(value)),
        }
    }

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
}
