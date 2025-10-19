use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // Oversize definition for BxROM. The actual BNROM cartridge only supports 128KiB.
    .prg_rom_max_size(8192 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// BNROM (BxROM): Irem I-IM and NES-BNROM boards
pub struct Mapper034_2;

impl Mapper for Mapper034_2 {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => mem.set_prg_register(P0, value),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
