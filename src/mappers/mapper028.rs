use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(8 * KIBIBYTE * KIBIBYTE)
    // Normal banking isn't used with Action 53, so this PrgLayout is ignored.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Action 53 
pub struct Mapper028 {
    selected_register: Register,
    action53_layout: Action53Layout,
    prg_outer_bank_size: OuterBankSize,
    outer_bank_bits: u8,
    inner_bank_bits: u8,
}

impl Mapper for Mapper028 {
    fn peek_cartridge_space(&self, params: &MapperParams, addr: u16) -> ReadResult {
        if addr < 0x4020 {
            unreachable!();
        }

        if addr < 0x8000 {
            return ReadResult::OPEN_BUS;
        }

        let bank_side = if addr < 0xC000 { BankSide::Low } else { BankSide::High };
        let bank_mask = Self::bank_mask(self.action53_layout, self.prg_outer_bank_size, bank_side);
        params.prg_memory.peek_raw_rom(self.create_memory_index(bank_mask, addr))
    }

    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => { /* Do nothing. */ }
            0x5000..=0x5FFF => {
                self.selected_register = match value & 0b1000_0001 {
                    0b0000_0000 => Register::ChrBank,
                    0b0000_0001 => Register::InnerPrgBank,
                    0b1000_0000 => Register::Mode,
                    0b1000_0001 => Register::OuterPrgBank,
                    _ => unreachable!(),
                };
            }
            0x6000..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                match self.selected_register {
                    Register::ChrBank => {
                        let (mirroring, chr_bank) = splitbits_named!(min=u8, value, "...m..cc");
                        params.set_chr_register(C0, chr_bank);
                        if params.name_table_mirroring().is_regular_one_screen() {
                            params.set_name_table_mirroring(mirroring);
                        }
                    }
                    Register::InnerPrgBank => {
                        let (mirroring, inner_bank_bits) = splitbits_named!(min=u8, value, "...mpppp");
                        self.inner_bank_bits = inner_bank_bits;
                        if params.name_table_mirroring().is_regular_one_screen() {
                            params.set_name_table_mirroring(mirroring);
                        }
                    }
                    Register::Mode => {
                        let (prg_outer_bank_size, action53_layout, mirroring) = splitbits_named!(value, "..oollmm");
                        self.prg_outer_bank_size = match prg_outer_bank_size {
                            0 => OuterBankSize::Kib32,
                            1 => OuterBankSize::Kib64,
                            2 => OuterBankSize::Kib128,
                            3 => OuterBankSize::Kib256,
                            _ => unreachable!(),
                        };

                        self.action53_layout = match action53_layout {
                            0 | 1 => Action53Layout::FullySwitchable,
                            2 => Action53Layout::FixedFirstHalf,
                            3 => Action53Layout::FixedSecondHalf,
                            _ => unreachable!(),
                        };

                        params.set_name_table_mirroring(mirroring);
                    }
                    Register::OuterPrgBank => {
                        self.outer_bank_bits = value;
                    }
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper028 {
    pub fn new() -> Self {
        Self {
            selected_register: Register::ChrBank,
            action53_layout: Action53Layout::FixedSecondHalf,
            prg_outer_bank_size: OuterBankSize::Kib32,
            outer_bank_bits: 0b1111_1111,
            inner_bank_bits: 0b0000,
        }
    }

    fn create_memory_index(&self, bank_mask: BankMask, addr: u16) -> u32 {
        let address_clear_count = bank_mask.total_count() - 7;
        let mut index: u32 = u32::from((addr << address_clear_count) >> address_clear_count);

        let inner_mask = 0b0000_1111 >> (4 - bank_mask.inner_bit_count);
        let inner_bank = self.inner_bank_bits & inner_mask;
        index |= u32::from(inner_bank) << (16 - address_clear_count);

        let outer_shift = 8 - bank_mask.outer_bit_count;
        let outer_mask = 0b1111_1111 << outer_shift;
        let outer_bank = (self.outer_bank_bits & outer_mask) >> outer_shift;
        index |= u32::from(outer_bank) << (16 - address_clear_count + bank_mask.inner_bit_count);

        index
    }

    const fn bank_mask(layout: Action53Layout, outer_bank_size: OuterBankSize, side: BankSide) -> BankMask {
        match (layout, outer_bank_size, side) {
            (Action53Layout::FullySwitchable, OuterBankSize::Kib32 , _             ) => BankMask { outer_bit_count: 8, inner_bit_count: 0 },
            (Action53Layout::FullySwitchable, OuterBankSize::Kib64 , _             ) => BankMask { outer_bit_count: 7, inner_bit_count: 1 },
            (Action53Layout::FullySwitchable, OuterBankSize::Kib128, _             ) => BankMask { outer_bit_count: 6, inner_bit_count: 2 },
            (Action53Layout::FullySwitchable, OuterBankSize::Kib256, _             ) => BankMask { outer_bit_count: 5, inner_bit_count: 3 },

            (Action53Layout::FixedFirstHalf , _                    , BankSide::Low ) => BankMask { outer_bit_count: 8, inner_bit_count: 0 },
            (Action53Layout::FixedFirstHalf , OuterBankSize::Kib32 , BankSide::High) => BankMask { outer_bit_count: 8, inner_bit_count: 1 },
            (Action53Layout::FixedFirstHalf , OuterBankSize::Kib64 , BankSide::High) => BankMask { outer_bit_count: 7, inner_bit_count: 2 },
            (Action53Layout::FixedFirstHalf , OuterBankSize::Kib128, BankSide::High) => BankMask { outer_bit_count: 6, inner_bit_count: 3 },
            (Action53Layout::FixedFirstHalf , OuterBankSize::Kib256, BankSide::High) => BankMask { outer_bit_count: 5, inner_bit_count: 4 },

            (Action53Layout::FixedSecondHalf, OuterBankSize::Kib32 , BankSide::Low ) => BankMask { outer_bit_count: 8, inner_bit_count: 1 },
            (Action53Layout::FixedSecondHalf, OuterBankSize::Kib64 , BankSide::Low ) => BankMask { outer_bit_count: 7, inner_bit_count: 2 },
            (Action53Layout::FixedSecondHalf, OuterBankSize::Kib128, BankSide::Low ) => BankMask { outer_bit_count: 6, inner_bit_count: 3 },
            (Action53Layout::FixedSecondHalf, OuterBankSize::Kib256, BankSide::Low ) => BankMask { outer_bit_count: 5, inner_bit_count: 4 },
            (Action53Layout::FixedSecondHalf, _                    , BankSide::High) => BankMask { outer_bit_count: 8, inner_bit_count: 0 },
        }
    }
}

#[derive(Debug)]
enum Register {
    ChrBank,
    InnerPrgBank,
    Mode,
    OuterPrgBank,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Action53Layout {
    // AOROM/BNROM
    FullySwitchable,
    // UNROM
    FixedFirstHalf,
    // UNROM alternate (Mapper 180, Crazy Climber)
    FixedSecondHalf,
}

#[derive(Clone, Copy, Debug)]
enum OuterBankSize {
    Kib32,
    Kib64,
    Kib128,
    Kib256,
}

#[derive(Debug)]
enum BankSide {
    Low,
    High,
}

#[derive(Clone, Copy)]
struct BankMask {
    outer_bit_count: u8,
    inner_bit_count: u8,
}

impl BankMask {
    fn total_count(self) -> u8 {
        self.outer_bit_count + self.inner_bit_count
    }
}