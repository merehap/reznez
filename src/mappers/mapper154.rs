use crate::mapper::*;
use crate::mappers::mapper088::{Mapper088, PRG_WINDOWS, CHR_WINDOWS};

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(PRG_WINDOWS)
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(CHR_WINDOWS)
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// NAMCOT-3453. Same as Mapper088, except adds a name table mirroring selection bit.
// FIXME: Devil Man scanline flickering.
pub struct Mapper154 {
    mapper088: Mapper088,
}

impl Mapper for Mapper154 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        if matches!(cpu_address, 0x8000..=0xFFFF) {
            let mirroring_index = splitbits_named!(min=u8, value, ".m......");
            params.set_name_table_mirroring(mirroring_index);
        }

        self.mapper088.write_register(params, cpu_address, value);
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
