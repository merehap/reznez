use crate::memory::mapper::*;
use crate::memory::mappers::mapper088::Mapper088;

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
            let mirroring = if value & 0b0100_0000 == 0 {
                NameTableMirroring::OneScreenLeftBank
            } else {
                NameTableMirroring::OneScreenRightBank
            };
            params.set_name_table_mirroring(mirroring);
        }

        self.mapper088.write_to_cartridge_space(params, address, value);
    }
}

impl Mapper154 {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self { mapper088: Mapper088::new(cartridge) }
    }
}