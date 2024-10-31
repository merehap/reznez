use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .chr_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Horizontal,
    NameTableMirroring::Vertical,
];

// Holy Diver 
// Identical to submapper 1 except the nametable mirrorings.
pub struct Mapper078_3;

impl Mapper for Mapper078_3 {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(value, "ccccmppp");
                params.set_bank_register(C0, fields.c);
                params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                params.set_bank_register(P0, fields.p);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
