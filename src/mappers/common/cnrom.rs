use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
    ])
    .chr_rom_max_size(2048 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// CNROM
pub struct Cnrom {
    has_bus_conflicts: bool,
}

impl Mapper for Cnrom {
    fn has_bus_conflicts(&self) -> bool {
        self.has_bus_conflicts
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => bus.set_chr_register(C0, value),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Cnrom {
    pub const fn with_bus_conflicts(has_bus_conflicts: bool) -> Cnrom {
        Cnrom { has_bus_conflicts }
    }
}
