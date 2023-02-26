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

// Color Dreams. Same as GxROM except with different register locations.
pub struct Mapper11 {
    params: MapperParams,
}

impl Mapper for Mapper11 {
    fn write_to_cartridge_space(&mut self, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                self.params.prg_memory.window_at(0x8000)
                    .switch_bank_to(value & 0b0000_0011);
                self.params.chr_memory.window_at(0x0000)
                    .switch_bank_to(value >> 4);
            }
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper11 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper11, String> {
        let params = MapperParams::new(
            cartridge,
            PRG_LAYOUT.clone(),
            CHR_LAYOUT.clone(),
            cartridge.name_table_mirroring(),
        );
        Ok(Mapper11 { params })
    }
}
