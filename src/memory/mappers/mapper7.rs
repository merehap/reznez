use crate::memory::mapper::*;

// AxROM
pub struct Mapper7 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper7 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper7, String> {
        let prg_memory = PrgMemory::builder()
            .raw_memory(cartridge.prg_rom())
            .max_bank_count(8)
            .bank_size(32 * KIBIBYTE)
            .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
            .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
            .build();

        // Not bank-switched.
        let chr_memory = ChrMemory::builder()
            .raw_memory(cartridge.chr_rom())
            .max_bank_count(1)
            .bank_size(8 * KIBIBYTE)
            .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::Ram { bank_index: 0 })
            .add_default_ram_if_chr_data_missing();

        Ok(Mapper7 {
            prg_memory,
            chr_memory,
            name_table_mirroring: NameTableMirroring::OneScreenLeftBank,
        })
    }
}

impl Mapper for Mapper7 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                let new_bank = (value & 0b0000_0111).into();
                self.prg_memory.switch_bank_at(0x8000, new_bank);

                self.name_table_mirroring = if value & 0b0001_0000 == 0 {
                    NameTableMirroring::OneScreenLeftBank
                } else {
                    NameTableMirroring::OneScreenRightBank
                };
            }
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
