use crate::memory::mapper::*;

pub const PRG_LAYOUT_8000_SWITCHABLE: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

pub const PRG_LAYOUT_C000_SWITCHABLE: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

pub const CHR_BIG_WINDOWS_FIRST: ChrLayout = ChrLayout::new(&[
    // Big windows.
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
    // Small windows.
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C5)),
]);

pub const CHR_SMALL_WINDOWS_FIRST: ChrLayout = ChrLayout::new(&[
    // Small windows.
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C2)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C3)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C4)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C5)),
    // Big windows.
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
]);

pub const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(64)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows(PRG_LAYOUT_8000_SWITCHABLE)
    .chr_max_bank_count(256)
    .chr_bank_size(1 * KIBIBYTE)
    .chr_windows(CHR_BIG_WINDOWS_FIRST)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

pub const BANK_INDEX_REGISTER_IDS: [BankIndexRegisterId; 8] = [C0, C1, C2, C3, C4, C5, P0, P1];

pub fn bank_select(
    params: &mut MapperParams,
    selected_register_id: &mut BankIndexRegisterId,
    value: u8,
) {
    let chr_big_windows_first =                     (value & 0b1000_0000) == 0;
    let prg_switchable_8000 =                       (value & 0b0100_0000) == 0;
    *selected_register_id = BANK_INDEX_REGISTER_IDS[(value & 0b0000_0111) as usize];

    if chr_big_windows_first {
        params.set_chr_layout(CHR_BIG_WINDOWS_FIRST);
    } else {
        params.set_chr_layout(CHR_SMALL_WINDOWS_FIRST);
    }

    if prg_switchable_8000 {
        params.set_prg_layout(PRG_LAYOUT_8000_SWITCHABLE);
    } else {
        params.set_prg_layout(PRG_LAYOUT_C000_SWITCHABLE);
    }
}

pub fn set_bank_index(
    params: &mut MapperParams,
    selected_register_id: &mut BankIndexRegisterId,
    value: u8,
) {
    let mut bank_index = value;
    if matches!(*selected_register_id, C0 | C1) {
        // Double-width windows can only use even banks.
        bank_index &= 0b1111_1110;
    }

    if matches!(*selected_register_id, P0 | P1) {
        assert_eq!(value & 0b1100_0000, 0, "ROM hack.");
    }

    params.set_bank_index_register(*selected_register_id, bank_index);
}

pub fn set_mirroring(params: &mut MapperParams, value: u8) {
    use NameTableMirroring::*;
    match (params.name_table_mirroring(), value & 0b0000_0001) {
        (Vertical, 1) => params.set_name_table_mirroring(Horizontal),
        (Horizontal, 0) => params.set_name_table_mirroring(Vertical),
        _ => { /* Other mirrorings cannot be changed. */ }
    }
}
