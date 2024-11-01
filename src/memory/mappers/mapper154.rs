use crate::memory::mapper::*;
use crate::memory::mappers::mapper088::{Mapper088, PRG_WINDOWS, CHR_WINDOWS};

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .prg_layout(PRG_WINDOWS)
    .chr_max_size(128 * KIBIBYTE)
    .chr_layout(CHR_WINDOWS)
    .name_table_mirrorings(&[
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::OneScreenRightBank,
    ])
    .build();

// NAMCOT-3453. Same as Mapper088, except adds a name table mirroring selection bit.
// FIXME: Devil Man scanline flickering.
pub struct Mapper154 {
    mapper088: Mapper088,
}

impl Mapper for Mapper154 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        if matches!(address.to_raw(), 0x8000..=0xFFFF) {
            let mirroring_index = splitbits_named!(min=u8, value, ".m......");
            params.set_name_table_mirroring(mirroring_index);
        }

        self.mapper088.write_to_cartridge_space(params, address, value);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper154 {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self { mapper088: Mapper088::new(cartridge) }
    }
}
