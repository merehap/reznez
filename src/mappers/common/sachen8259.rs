use crate::mapper::*;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    // This value is overridden by the individual boards.
    .chr_rom_max_size(512 * KIBIBYTE)
    // Normal layout.
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    // "Simple" layout. C4, C5, and C6 are the same as C0, except for their low bits.
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C6)),
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
    layout: Layout,

    chr_bank_shift: u8,
    chr_inner_banks: [u8; 4],
    chr_bank_low_bits: [u8; 4],
    register_value: RegisterValue,
}

impl Mapper for Sachen8259 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
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
                        self.chr_inner_banks[reg_id.to_raw_chr_id() as usize] = value & 0b111;
                        self.update_chr_banks(params);
                    }
                    RegisterValue::ChrOuterBank => {
                        params.set_chr_rom_outer_bank_index(value & 0b111);
                    }
                    RegisterValue::PrgBank => {
                        params.set_prg_register(P0, value & 0b111);
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
        self.layout.clone()
    }
}


impl Sachen8259 {
    pub const fn new(layout: Layout, board: Sachen8259Board) -> Self {
        // The CHR bank low bits are actually the respective PPU address line bits.
        let (chr_bank_shift, chr_bank_low_bits) = match board {
            Sachen8259Board::A => (1, [0b00, 0b01, 0b00, 0b01]),
            Sachen8259Board::B => (0, [0b00, 0b00, 0b00, 0b00]),
            Sachen8259Board::C => (2, [0b00, 0b01, 0b10, 0b11]),
        };
        Self {
            layout,

            chr_bank_shift,
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

    fn update_chr_bank(&self, params: &mut MapperParams, cx: ChrBankRegisterId, inner_bank_index: u8, low_bits_index: u8) {
        let bank = (self.chr_inner_banks[inner_bank_index as usize] << self.chr_bank_shift)
            | self.chr_bank_low_bits[low_bits_index as usize];
        params.set_chr_register(cx, bank);
    }
}

enum RegisterValue {
    ChrSelect(ChrBankRegisterId),
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
