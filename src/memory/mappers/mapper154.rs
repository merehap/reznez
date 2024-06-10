use crate::memory::mapper::*;
use crate::memory::mappers::mapper088::Mapper088;

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenRightBank,
];

// NAMCOT-3453. Same as Mapper088, except adds a name table mirroring selection bit.
// FIXME: Devil Man scanline flickering.
pub struct Mapper154 {
    mapper088: Mapper088,
}

impl Mapper for Mapper154 {
    fn initial_layout(&self) -> InitialLayout {
        self.mapper088.initial_layout()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        if matches!(address.to_raw(), 0x8000..=0xFFFF) {
            // TODO: splitbits single
            params.set_name_table_mirroring(MIRRORINGS[usize::from((value & 0b0100_0000) >> 6)]);
        }

        self.mapper088.write_to_cartridge_space(params, address, value);
    }
}

impl Mapper154 {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self { mapper088: Mapper088::new(cartridge) }
    }
}
