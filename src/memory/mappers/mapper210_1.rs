use crate::memory::mapper::*;

const PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    // FIXME: This is supposed to be 2KiBs mirrored. Family Circuit doesn't work without it.
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P2)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C7)),
]);

// Namco 175
pub struct Mapper210_1;

impl Mapper for Mapper210_1 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(64)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => params.write_prg(address, value),
            0x8000..=0x87FF => params.set_bank_register(C0, value),
            0x8800..=0x8FFF => params.set_bank_register(C1, value),
            0x9000..=0x97FF => params.set_bank_register(C2, value),
            0x9800..=0x9FFF => params.set_bank_register(C3, value),
            0xA000..=0xA7FF => params.set_bank_register(C4, value),
            0xA800..=0xAFFF => params.set_bank_register(C5, value),
            0xB000..=0xB7FF => params.set_bank_register(C6, value),
            0xB800..=0xBFFF => params.set_bank_register(C7, value),
            0xC000..=0xC7FF => { /* TODO: External PRG RAM enable. */ }
            0xC800..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xE7FF => params.set_bank_register(P0, value & 0b0011_1111),
            0xE800..=0xEFFF => params.set_bank_register(P1, value & 0b0011_1111),
            0xF000..=0xF7FF => params.set_bank_register(P2, value & 0b0011_1111),
            0xF800..=0xFFFF => { /* Do nothing. */ }
        }
    }
}
