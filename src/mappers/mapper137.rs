use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(D)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(E)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(F)),
        // TODO: Compress into just one.
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.fixed_index(-4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.fixed_index(-3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.fixed_index(-2)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.fixed_index(-1)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
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

pub struct Mapper137 {
    selected_reg: Register,
}

impl Mapper for Mapper137 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0xC101 {
            0x4100 => {
                self.selected_reg = match value & 0b111 {
                    0 => Register::ChrLow(C),
                    1 => Register::ChrLow(D),
                    2 => Register::ChrLow(E),
                    3 => Register::ChrLow(F),
                    4 => Register::ChrHigh,
                    5 => Register::Prg,
                    6 => Register::ChrFMid,
                    7 => Register::ModeAndMirroring,
                    _ => unreachable!(),
                };
            }
            0x4101 => {
                match self.selected_reg {
                    Register::ChrLow(reg_id) => {
                        bus.set_chr_bank_register_bits(reg_id, value.into(), 0b0000_0111);
                    }
                    Register::ChrHigh => {
                        let high_bits = splitbits!(min=u8, value, ".... .fed");
                        bus.set_chr_bank_register_bits(D, (high_bits.d << 4).into(), 0b0001_0000);
                        bus.set_chr_bank_register_bits(E, (high_bits.e << 4).into(), 0b0001_0000);
                        bus.set_chr_bank_register_bits(F, (high_bits.f << 4).into(), 0b0001_0000);
                    }
                    Register::Prg => {
                        bus.set_prg_register(P, value & 0b111);
                    }
                    Register::ChrFMid => {
                        bus.set_chr_bank_register_bits(F, ((value & 1) << 3).into(), 0b0000_1000);
                    }
                    Register::ModeAndMirroring => {
                        let (mirroring, simple_mode) = splitbits_named!(value, ".... .mms");
                        bus.set_name_table_mirroring(mirroring);
                        assert!(!simple_mode, "Simple mode not supported yet.");
                    }
                }
            }
            _ => { /* No regs here. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper137 {
    pub fn new() -> Self {
        Self {
            selected_reg: Register::ChrLow(C),
        }
    }
}

enum Register {
    ChrLow(ChrBankRegisterId),
    ChrHigh,
    Prg,
    ChrFMid,
    ModeAndMirroring,
}