use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.fixed_index(0).read_status(R0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// CNROM with copy protection
pub struct CnromWithChrDisable {
    correct_chip_select_value: u8,
}

impl Mapper for CnromWithChrDisable {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let copy_protection_passed = value & 0b11 == self.correct_chip_select_value;
                bus.set_reads_enabled(R0, copy_protection_passed);
            }
        }
    }

    fn has_bus_conflicts(&self) -> bool {
        true
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl CnromWithChrDisable {
    pub const fn new(correct_chip_select_value: u8) -> Self {
        Self { correct_chip_select_value }
    }
}
