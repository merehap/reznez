use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// Homebrew: Sealie Computing RET-CUFROM revD
// TODO: Reprogramming logic.
// FIXME: Untested!
pub struct Mapper029;

impl Mapper for Mapper029 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "...pppcc");
                mem.set_prg_register(P0, banks.p);
                mem.set_chr_register(C0, banks.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
