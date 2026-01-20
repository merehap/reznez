use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x67FF, 2 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x6800, 0x6FFF, 2 * KIBIBYTE, PrgBank::SAVE_RAM.fixed_number(0)),
        PrgWindow::new(0x7000, 0x77FF, 2 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x7800, 0x7FFF, 2 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .override_prg_bank_register(P3, -1)
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .fixed_name_table_mirroring()
    .build();

// G0151-1
// TODO: "When reading from CPU address $FFE4-$FFE7, $FFEC-$FFEF, $FFF4-$FFF7, or $FFFC-$FFFF,
//        PRG A17 is forced high, as if register $6003 were OR'd with $10."
pub struct Mapper246;

impl Mapper for Mapper246 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* No registers here. */ }
            0x6000 => bus.set_prg_register(P0, value & 0b0011_1111),
            0x6001 => bus.set_prg_register(P1, value & 0b0011_1111),
            0x6002 => bus.set_prg_register(P2, value & 0b0011_1111),
            0x6003 => bus.set_prg_register(P3, value & 0b0011_1111),
            0x6004 => bus.set_chr_register(C0, value),
            0x6005 => bus.set_chr_register(C1, value),
            0x6006 => bus.set_chr_register(C2, value),
            0x6007 => bus.set_chr_register(C3, value),
            0x6008..=0xFFFF => { /* No registers here. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}