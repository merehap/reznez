use crate::memory::mapper::*;

lazy_static! {
    static ref PRG_LAYOUT: PrgLayout = PrgLayout::builder()
        .max_bank_count(4)
        .bank_size(32 * KIBIBYTE)
        .window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
        .window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0)))
        .build();

    static ref CHR_LAYOUT: ChrLayout = ChrLayout::builder()
        .max_bank_count(16)
        .bank_size(8 * KIBIBYTE)
        .window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::Register(P0)))
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
                self.prg_memory_mut().set_bank_index_register(P0, value & 0b0000_0011);
                self.chr_memory_mut().set_bank_index_register(C0, value >> 4);
            }
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper11 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper11, String> {
        Ok(Mapper11 { params: MapperParams::new(
            cartridge,
            PRG_LAYOUT.clone(),
            CHR_LAYOUT.clone(),
            cartridge.name_table_mirroring(),
        )})
    }
}
