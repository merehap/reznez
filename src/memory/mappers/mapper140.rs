use crate::memory::mapper::*;

lazy_static! {
    static ref PRG_LAYOUT: PrgLayout = PrgLayout::builder()
        .max_bank_count(4)
        .bank_size(32 * KIBIBYTE)
        .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
        .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST))
        .build();

    static ref CHR_LAYOUT: ChrLayout = ChrLayout::builder()
        .max_bank_count(16)
        .bank_size(8 * KIBIBYTE)
        .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST))
        .build();
}

// Same as GNROM, except the writable port is moved to 0x6000 and more CHR banks are allowed.
pub struct Mapper140 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper140 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper140, String> {
        Ok(Mapper140 {
            prg_memory: PrgMemory::new(PRG_LAYOUT.clone(), cartridge.prg_rom()),
            chr_memory: ChrMemory::new(CHR_LAYOUT.clone(), cartridge.chr_rom()),
            name_table_mirroring: cartridge.name_table_mirroring(),
        })
    }
}

impl Mapper for Mapper140 {
    fn write_to_cartridge_space(&mut self, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => {
                assert_eq!(value & 0b1100_0000, 0);
                self.prg_memory.window_at(0x8000)
                    .switch_bank_to((value & 0b0011_0000) >> 4);
                self.chr_memory.window_at(0x0000)
                    .switch_bank_to(value & 0b0000_1111);
            }
            0x8000..=0xFFFF => { /* Do nothing. */ }
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
