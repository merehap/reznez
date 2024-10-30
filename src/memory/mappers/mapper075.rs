use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .chr_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::switchable_rom(C0)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_rom(C1)),
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// VRC1
pub struct Mapper075 {
    chr_left_high_bit: u8,
    chr_right_high_bit: u8,
}

impl Mapper for Mapper075 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => params.set_bank_register(P0, value & 0b0000_1111),
            0x9000..=0x9FFF => {
                let fields = splitbits!(value, ".....rlm");

                self.chr_right_high_bit = u8::from(fields.r) << 4;
                self.chr_left_high_bit = u8::from(fields.l) << 4;
                if params.name_table_mirroring() != NameTableMirroring::FourScreen {
                    params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                }
            }
            0xA000..=0xAFFF => params.set_bank_register(P1, value & 0b0000_1111),
            0xB000..=0xBFFF => { /* Do nothing. */ }
            0xC000..=0xCFFF => params.set_bank_register(P2, value & 0b0000_1111),
            0xD000..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xEFFF => {
                let bank_index = self.chr_left_high_bit | (value & 0b0000_1111);
                params.set_bank_register(C0, bank_index);
            }
            0xF000..=0xFFFF => {
                let bank_index = self.chr_right_high_bit | (value & 0b0000_1111);
                params.set_bank_register(C1, bank_index);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper075 {
    pub fn new() -> Self {
        Self {
            chr_left_high_bit: 0b0000_0000,
            chr_right_high_bit: 0b0000_0000,
        }
    }
}
