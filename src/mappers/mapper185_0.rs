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

// CNROM with CHR disable
#[derive(Default)]
pub struct Mapper185_0 {
    ppu_data_read_count: u8,
}

impl Mapper for Mapper185_0 {
    fn on_cpu_read(&mut self, bus: &mut Bus, addr: CpuAddress, _value: u8) {
        if *addr == 0x2007 {
            if self.ppu_data_read_count < 2 {
                bus.set_reads_enabled(R0, false);
                self.ppu_data_read_count += 1;
            } else {
                bus.set_reads_enabled(R0, true);
            }
        }
    }

    fn write_register(&mut self, _bus: &mut Bus, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
