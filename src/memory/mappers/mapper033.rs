use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .chr_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::switchable_rom(C0)),
        Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// Taito's TC0190
pub struct Mapper033;

impl Mapper for Mapper033 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x8000 => {
                let fields = splitbits!(value, ".mpppppp");
                params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                params.set_bank_register(P0, fields.p);
            }
            0x8001 => params.set_bank_register(P1, value & 0b0011_1111),
            // Large CHR windows: this allows accessing 512KiB CHR by doubling the bank indexes.
            0x8002 => params.set_bank_register(C0, 2 * u16::from(value)),
            0x8003 => params.set_bank_register(C1, 2 * u16::from(value)),
            // Small CHR windows.
            0xA000 => params.set_bank_register(C2, value),
            0xA001 => params.set_bank_register(C3, value),
            0xA002 => params.set_bank_register(C4, value),
            0xA003 => params.set_bank_register(C5, value),
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
