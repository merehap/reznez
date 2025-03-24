use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0800, 0x1FFF, 6 * KIBIBYTE, Bank::SaveRam(0x0800)),
    ])
    // Cartridges for some reason don't specify a CHR Save RAM size.
    .chr_save_ram_size(8 * KIBIBYTE)
    .override_initial_name_table_mirroring(NameTableMirroring::new(
        NameTableSource::SaveRam(0x0000),
        NameTableSource::SaveRam(0x0400),
        NameTableSource::Ciram(CiramSide::Left),
        NameTableSource::Ciram(CiramSide::Right),
    ))
    .build();

// Irem (Napoleon Senki)
pub struct Mapper077;

impl Mapper for Mapper077 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "ccccpppp");
                params.set_bank_register(C0, banks.c);
                params.set_bank_register(P0, banks.p);
            }
        }
    }

    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
