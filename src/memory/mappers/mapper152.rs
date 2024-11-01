use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .override_initial_name_table_mirroring(NameTableMirroring::OneScreenLeftBank)
    .name_table_mirrorings(&[
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::OneScreenRightBank,
    ])
    .build();

// Similar to Mapper070, but with one screen mirroring control.
pub struct Mapper152;

impl Mapper for Mapper152 {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, value, "mpppcccc");
                params.set_name_table_mirroring(fields.m);
                params.set_bank_register(P0, fields.p);
                params.set_bank_register(C0, fields.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
