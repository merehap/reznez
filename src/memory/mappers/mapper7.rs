use crate::memory::mapper::*;

lazy_static! {
    static ref PRG_LAYOUT: PrgLayout = PrgLayout::builder()
        .max_bank_count(8)
        .bank_size(32 * KIBIBYTE)
        .window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
        .window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0)))
        .build();
    // Only one bank, so not bank-switched.
    static ref CHR_LAYOUT: ChrLayout = ChrLayout::builder()
        .max_bank_count(1)
        .bank_size(8 * KIBIBYTE)
        .window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Ram, BankIndex::FIRST))
        .build();
}

// AxROM
pub struct Mapper7 {
    params: MapperParams,
}

impl Mapper for Mapper7 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                self.prg_memory_mut().set_bank_index_register(P0, value & 0b0000_01111);
                self.set_name_table_mirroring(if value & 0b0001_0000 == 0 {
                    NameTableMirroring::OneScreenLeftBank
                } else {
                    NameTableMirroring::OneScreenRightBank
                });
            }
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper7 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper7, String> {
        Ok(Mapper7 { params: MapperParams::new(
            cartridge,
            PRG_LAYOUT.clone(),
            CHR_LAYOUT.clone(),
            NameTableMirroring::OneScreenLeftBank,
        )})
    }
}
