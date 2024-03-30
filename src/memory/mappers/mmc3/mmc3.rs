use crate::memory::mapper::*;

pub const PRG_LAYOUT_C000_FIXED: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgBank::WorkRam),
    PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgBank::WorkRam),
    PrgWindow::new(0x7400, 0x7FFF, 3 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);
// Same as PRG_LAYOUT_C000_FIXED, except the 0x8000 and 0xC000 windows are swapped.
pub const PRG_LAYOUT_8000_FIXED: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgBank::WorkRam),
    PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgBank::WorkRam),
    PrgWindow::new(0x7400, 0x7FFF, 3 * KIBIBYTE, PrgBank::WorkRam),
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
    .prg_max_bank_count(32)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows(PRG_LAYOUT_C000_FIXED)
    .chr_max_bank_count(256)
    .chr_bank_size(1 * KIBIBYTE)
    .chr_windows(CHR_BIG_WINDOWS_FIRST)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const BANK_INDEX_REGISTER_IDS: [BankIndexRegisterId; 8] = [C0, C1, C2, C3, C4, C5, P0, P1];

pub fn bank_select(
    params: &mut MapperParams,
    selected_register_id: &mut BankIndexRegisterId,
    value: u8,
) {
    let chr_big_windows_first =                         (value & 0b1000_0000) == 0;
    let prg_fixed_c000 =                                (value & 0b0100_0000) == 0;
    //self.prg_ram_enabled =                            (value & 0b0010_0000) != 0;
    *selected_register_id = BANK_INDEX_REGISTER_IDS[    (value & 0b0000_0111) as usize];

    if chr_big_windows_first {
        params.set_chr_layout(CHR_BIG_WINDOWS_FIRST);
    } else {
        params.set_chr_layout(CHR_SMALL_WINDOWS_FIRST);
    }

    if prg_fixed_c000 {
        params.set_prg_layout(PRG_LAYOUT_C000_FIXED);
    } else {
        params.set_prg_layout(PRG_LAYOUT_8000_FIXED);
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

pub fn prg_ram_protect(_params: &mut MapperParams, _value: u8) {
    // TODO: Once NES 2.0 is supported, then MMC3 and MMC6 can properly be supported.
    /*
    if !self.prg_ram_enabled {
        return;
    }

    // MMC6 logic only here since MMC3 logic conflicts:
    // https://www.nesdev.org/wiki/MMC3#iNES_Mapper_004_and_MMC6
    // TODO: Attempt to support Low G Man.
    let mut status_7000 = Mapper004::work_ram_status_from_bits(value & 0b1100_0000 >> 6);
    let mut status_7200 = Mapper004::work_ram_status_from_bits(value & 0b0011_0000 >> 4);

    // "If only one bank is enabled for reading, the other reads back as zero."
    use WorkRamStatus::*;
    match (status_7000, status_7200) {
        (ReadOnly | ReadWrite, Disabled            ) => status_7200 = ReadOnlyZeros,
        (Disabled            , ReadOnly | ReadWrite) => status_7000 = ReadOnlyZeros,
    }

    self.prg_memory.set_work_ram_status_at(0x7000, status_7000);
    self.prg_memory.set_work_ram_status_at(0x7200, status_7200);
    */
}

/*
fn work_ram_status_from_bits(value: u8) -> WorkRamStatus {
    assert_eq!(value & 0b1111_1100, 0);

    match value {
        0b00 => WorkRamStatus::Disabled,
        0b01 => WorkRamStatus::Disabled,
        0b10 => WorkRamStatus::ReadOnly,
        0b11 => WorkRamStatus::ReadWrite,
        _ => unreachable!(),
    }
}
*/
