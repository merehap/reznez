use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(1024 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .build();

// Rumble Station (Color Dreams).
// NOTE: Untested.
#[derive(Default)]
pub struct Mapper046 {
    prg_high_bits: u8,
    chr_high_bits: u8,
}

impl Mapper for Mapper046 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => {
                // TODO: replacebits
                self.prg_high_bits = (value & 0b1111_0000) >> 3;
                self.chr_high_bits = (value & 0b0000_1111) << 3;
            }
            0x8000..=0xFFFF => {
                // TODO: replacebits
                let prg_bank_number = self.prg_high_bits | (value & 0b0000_0001);
                mem.set_prg_register(P0, prg_bank_number);
                let chr_bank_number = self.chr_high_bits | ((value << 1) >> 5);
                mem.set_chr_register(C0, chr_bank_number);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
