use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    // UNROM-like
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(0x20)),
    ])
    // NROM-like, but reversed ordering of the 16KiB banks.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P2)), // P0 with a low bit of 1
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)), // P0 with a low bit of 0
    ])
    // Reverse UNROM-like
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(0x1F)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    // Duplicate of the above PRG layout.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(0x1F)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM)
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();

// Subor
// TODO: Testing. Need to support non-NTSC.
#[derive(Default)]
pub struct Mapper167 {
    left_prg_top_bit: u8,
    right_prg_top_bit: u8,
    left_prg_bottom_bits: u8,
    right_prg_bottom_bits: u8,
}

impl Mapper for Mapper167 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                bus.set_name_table_mirroring(value & 1);
                self.left_prg_top_bit = (value << 1) & 0b0010_0000;
            }
            0xA000..=0xBFFF => {
                bus.set_prg_layout((value >> 2) & 0b11);
                self.right_prg_top_bit = (value << 1) & 0b0010_0000;
            }
            0xC000..=0xDFFF => self.left_prg_bottom_bits = value & 0b0001_1111,
            0xE000..=0xFFFF => self.right_prg_bottom_bits = value & 0b0001_1111,
        }

        let prg_left_input = self.left_prg_top_bit | self.left_prg_bottom_bits;
        let prg_right_input = self.right_prg_top_bit | self.right_prg_bottom_bits;
        let prg_bank = prg_left_input & prg_right_input;
        bus.set_prg_register(P0, prg_bank);
        bus.set_prg_register(P1, prg_bank & !1); // Clear bottom bit
        bus.set_prg_register(P2, prg_bank | 1); // Set bottom bit
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}