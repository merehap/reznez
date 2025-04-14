use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // Oversize PRG. On real cartridges, 256KiB is the max.
    .initial_name_table_mirroring(NameTableMirroring::ONE_SCREEN_LEFT_BANK)
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.fixed_index(0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// AxROM
pub struct Axrom {
    has_bus_conflicts: HasBusConflicts,
}

impl Mapper for Axrom {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        self.has_bus_conflicts
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, value, "...mpppp");
                params.set_name_table_mirroring(fields.m);
                params.set_prg_register(P0, fields.p);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Axrom {
    pub const fn new(has_bus_conflicts: HasBusConflicts) -> Axrom {
        Axrom { has_bus_conflicts }
    }
}
