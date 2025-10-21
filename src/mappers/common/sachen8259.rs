use crate::mapper::*;

pub const PRG_LAYOUT: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
];

pub const NORMAL_CHR_LAYOUT: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
];
// C4, C5, and C6 are the same as C0, except for their low bits.
pub const SIMPLE_CHR_LAYOUT: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C6)),
];

pub const NAME_TABLE_MIRRORINGS: &[NameTableMirroring] = &[
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
];

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
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xC101 {
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
                    RegisterValue::Nop => {}
                    RegisterValue::PrgBank => mem.set_prg_register(P0, value & 0b111),
                    RegisterValue::ChrOuterBank => mem.set_chr_rom_outer_bank_number(value & 0b111),
                    RegisterValue::ChrSelect(reg_id) => {
                        self.chr_inner_banks[reg_id.to_raw_chr_id() as usize] = value & 0b111;
                        self.update_chr_banks(mem);
                    }
                    RegisterValue::ModeSelect => {
                        let (mirroring, simple_layout) = splitbits_named!(value, ".... .mms");
                        mem.set_chr_layout(simple_layout as u8);
                        if simple_layout {
                            mem.set_name_table_mirroring(VERTICAL);
                        } else {
                            mem.set_name_table_mirroring(mirroring);
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

    // TODO: Put this code directly into the caller.
    fn update_chr_banks(&self, mem: &mut Memory) {
        let meta_data = [
            (C0, 0, 0),
            (C1, 1, 1),
            (C2, 2, 2),
            (C3, 3, 3),

            (C4, 0, 1),
            (C5, 0, 2),
            (C6, 0, 3),
        ];

        for (reg_id, inner_bank_number, low_bits_index) in meta_data {
            self.update_chr_bank(mem, reg_id, inner_bank_number, low_bits_index);
        }
    }

    fn update_chr_bank(&self, mem: &mut Memory, cx: ChrBankRegisterId, inner_bank_number: u8, low_bits_index: u8) {
        let bank = (self.chr_inner_banks[inner_bank_number as usize] << self.chr_bank_shift)
            | self.chr_bank_low_bits[low_bits_index as usize];
        mem.set_chr_register(cx, bank);
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
