use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Uchuusen - Cosmo Carrier
pub struct Mapper078_1;

impl Mapper for Mapper078_1 {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, value, "ccccmppp");
                params.set_bank_register(C0, fields.c);
                params.set_name_table_mirroring(fields.m);
                params.set_bank_register(P0, fields.p);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
