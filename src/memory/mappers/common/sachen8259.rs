use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    // TODO: This is only the limit for board B. A and C have smaller sizes.
    .chr_rom_max_size(512 * KIBIBYTE)
    // Normal layout.
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C1)),
        Window::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C3)),
    ])
    // "Simple" layout. C4, C5, and C6 are the same as C0, except for their low bits.
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::ROM.switchable(C5)),
        Window::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C6)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        // L R
        // R R
        NameTableMirroring::new(
            NameTableSource::Ciram(CiramSide::Left),
            NameTableSource::Ciram(CiramSide::Right),
            NameTableSource::Ciram(CiramSide::Right),
            NameTableSource::Ciram(CiramSide::Right),
        ),
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
    ])
    .build();

const VERTICAL: u8 = 0;

// UNL-Sachen-8259A, UNL-Sachen-8259B, UNL-Sachen-8259C
pub struct Sachen8259 {
    chr_bank_shift: u8,
    chr_outer_bank: u8,
    chr_inner_banks: [u8; 4],
    chr_bank_low_bits: [u8; 4],
    register_value: RegisterValue,
}

impl Mapper for Sachen8259 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address & 0xC101 {
            0x4100 => {
                let value = value & 0b111;
                self.register_value = match value {
                    0 => RegisterValue::ChrSelect(C0),
                    1 => RegisterValue::ChrSelect(C1),
                    2 => RegisterValue::ChrSelect(C2),
                    3 => RegisterValue::ChrSelect(C3),
                    4 => RegisterValue::ChrOuterBank,
                    5 => RegisterValue::PrgBank,
                    6 => RegisterValue::Nop,
                    7 => RegisterValue::ModeSelect,
                    _ => unreachable!(),
                };
            }
            0x4101 => {
                match self.register_value {
                    RegisterValue::ChrSelect(reg_id) => {
                        self.chr_inner_banks[reg_id.to_raw_chr_id().unwrap() as usize] = value & 0b111;
                        self.update_chr_banks(params);
                    }
                    RegisterValue::ChrOuterBank => {
                        self.chr_outer_bank = value & 0b111;
                        self.update_chr_banks(params);
                    }
                    RegisterValue::PrgBank => {
                        params.set_bank_register(P0, value & 0b111);
                    }
                    RegisterValue::Nop => {}
                    RegisterValue::ModeSelect => {
                        let (mirroring, simple_layout) = splitbits_named!(value, ".... .mms");
                        params.set_chr_layout(simple_layout as u8);
                        if simple_layout {
                            params.set_name_table_mirroring(VERTICAL);
                        } else {
                            params.set_name_table_mirroring(mirroring);
                        }
                    }
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}


impl Sachen8259 {
    pub const fn new(board: Sachen8259Board) -> Self {
        let (chr_bank_shift, chr_bank_low_bits) = match board {
            Sachen8259Board::A => (1, [0b00, 0b01, 0b00, 0b01]),
            Sachen8259Board::B => (0, [0b00, 0b00, 0b00, 0b00]),
            Sachen8259Board::C => (2, [0b00, 0b01, 0b10, 0b11]),
        };
        Self {
            chr_bank_shift,
            chr_outer_bank: 0,
            chr_inner_banks: [0; 4],
            chr_bank_low_bits,
            register_value: RegisterValue::ChrSelect(C0),
        }
    }

    fn update_chr_banks(&self, params: &mut MapperParams) {
        let meta_data = [
            (C0, 0, 0),
            (C1, 1, 1),
            (C2, 2, 2),
            (C3, 3, 3),

            (C4, 0, 1),
            (C5, 0, 2),
            (C6, 0, 3),
        ];

        for (reg_id, inner_bank_index, low_bits_index) in meta_data {
            self.update_chr_bank(params, reg_id, inner_bank_index, low_bits_index);
        }
    }

    fn update_chr_bank(&self, params: &mut MapperParams, cx: BankRegisterId, inner_bank_index: u8, low_bits_index: u8) {
        let bank_base = (self.chr_outer_bank << 3) | self.chr_inner_banks[inner_bank_index as usize];
        let bank = (bank_base << self.chr_bank_shift) | self.chr_bank_low_bits[low_bits_index as usize];
        params.set_bank_register(cx, bank);
    }
}

enum RegisterValue {
    ChrSelect(BankRegisterId),
    ChrOuterBank,
    PrgBank,
    Nop,
    ModeSelect,
}

pub enum Sachen8259Board {
    A,
    B,
    C,
}
