use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(4096 * KIBIBYTE)
    .chr_max_size(8 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
    ])
    .build();

// UNROM, but the fixed bank and the switchable bank are swapped.
pub struct Mapper180;

impl Mapper for Mapper180 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => params.set_bank_register(P0, value & 0b0000_0111),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
