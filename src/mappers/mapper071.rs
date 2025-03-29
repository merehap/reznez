use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Similar to UxROM.
pub struct Mapper071;

impl Mapper for Mapper071 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        let fields = splitbits!(min=u8, value, "...mpppp");
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x8FFF => { /* Do nothing. */ }
            // https://www.nesdev.org/wiki/INES_Mapper_071#Mirroring_($8000-$9FFF)
            0x9000..=0x9FFF => params.set_name_table_mirroring(fields.m),
            0xA000..=0xBFFF => { /* Do nothing. */ }
            0xC000..=0xFFFF => params.set_bank_register(P0, fields.p),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
