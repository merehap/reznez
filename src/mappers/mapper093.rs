use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.fixed_index(0).read_write_status(R0, W0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// Sunsoft-2 IC on the Sunsoft-3R board
pub struct Mapper093;

impl Mapper for Mapper093 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(value, ".ppp...e");
                bus.set_prg_register(P0, fields.p);
                bus.set_reads_enabled(R0, fields.e);
                bus.set_writes_enabled(W0, fields.e);
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
