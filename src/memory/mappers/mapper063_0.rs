use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(4096 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(0).status_register(S0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .ram_statuses(&[
        RamStatus::ReadWrite,
        RamStatus::ReadOnly,
    ])
    .build();

// TH2291-3 and CH-011
// TODO: Untested. Test ROM needed.
pub struct Mapper063_0;

impl Mapper for Mapper063_0 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, cpu_address, ".... .rpp pppp pplm");
                params.set_ram_status(S0, fields.r);
                params.set_bank_register(P0, fields.p);
                params.set_prg_layout(fields.l);
                params.set_name_table_mirroring(fields.m);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
