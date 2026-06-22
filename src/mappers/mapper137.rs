use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Prg::ROM).switchable(P),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Chr::ROM).rom_address_template("0₀₀0₀₀c₀₂c₀₁c₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Chr::ROM).rom_address_template("i₀₀0₀₀d₀₂d₀₁d₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Chr::ROM).rom_address_template("j₀₀0₀₀e₀₂e₀₁e₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Chr::ROM).rom_address_template("k₀₀m₀₀f₀₂f₀₁f₀₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Chr::ROM).rom_address_template("1₀₀1₀₀1₀₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
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
    // Some ROMs incorrectly set 4-screen mirroring, so force horizontal. Or maybe nes20db.xml is wrong, too?
    .cartridge_selection_name_table_mirrorings([
        Some(NameTableMirroring::HORIZONTAL),
        Some(NameTableMirroring::HORIZONTAL),
        Some(NameTableMirroring::HORIZONTAL),
        Some(NameTableMirroring::HORIZONTAL),
    ])
    .build();

// Sachen 8259D
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
                        bus.set_chr_register(reg_id, value & 0b111);
                    }
                    Register::ChrHigh => {
                        let high_bits = splitbits!(min=u8, value, ".... .kji");
                        bus.set_chr_register(I, high_bits.i);
                        bus.set_chr_register(J, high_bits.j);
                        bus.set_chr_register(K, high_bits.k);
                    }
                    Register::Prg => {
                        bus.set_prg_register(P, value & 0b111);
                    }
                    Register::ChrFMid => {
                        bus.set_chr_register(M, value & 1);
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
