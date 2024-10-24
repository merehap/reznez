use crate::memory::mapper::*;

const PRG_LAYOUT_LAST_TWO_FIXED: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);
const PRG_LAYOUT_ENDS_FIXED: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C7)),
]);

const PRG_LAYOUTS: [PrgLayout; 2] =
[
    PRG_LAYOUT_LAST_TWO_FIXED,
    PRG_LAYOUT_ENDS_FIXED,
];

const MIRRORINGS: [NameTableMirroring; 2] =
[
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// Irem's G-101
pub struct Mapper032;

impl Mapper for Mapper032 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(32)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT_LAST_TWO_FIXED)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x8000..=0x8007 => params.set_bank_register(P0, value & 0b1_1111),
            0x9000..=0x9007 => {
                let fields = splitbits!(value, "......pm");
                params.set_prg_layout(PRG_LAYOUTS[fields.p as usize]);
                params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
            }
            0xA000..=0xA007 => params.set_bank_register(P1, value & 0b1_1111),
            0xB000 => params.set_bank_register(C0, value),
            0xB001 => params.set_bank_register(C1, value),
            0xB002 => params.set_bank_register(C2, value),
            0xB003 => params.set_bank_register(C3, value),
            0xB004 => params.set_bank_register(C4, value),
            0xB005 => params.set_bank_register(C5, value),
            0xB006 => params.set_bank_register(C6, value),
            0xB007 => params.set_bank_register(C7, value),
            _ => { /* Do nothing. */ }
        }
    }
}
