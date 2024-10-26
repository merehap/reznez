use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // Oversize PRG. On real cartridges, 256KiB is the max.
    .prg_max_size(512 * KIBIBYTE)
    .chr_max_size(8 * KIBIBYTE)
    .name_table_mirroring_source(NameTableMirroring::OneScreenLeftBank.to_source())
    .prg_layouts(&[
        PrgLayout::new(&[
            PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
            PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
        ])
    ])
    .chr_layouts(&[
        ChrLayout::new(&[
            ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
        ])
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenRightBank,
];

// AxROM
pub struct Axrom {
    has_bus_conflicts: HasBusConflicts,
}

impl Mapper for Axrom {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        self.has_bus_conflicts
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(value, "...mpppp");
                params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                params.set_bank_register(P0, fields.p);
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
