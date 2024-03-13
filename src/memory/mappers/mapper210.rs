use crate::memory::mapper::*;

const PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    // FIXME: This is supposed to be 2KiBs mirrored. Family Circuit doesn't work without it.
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P2)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C7)),
]);

// Namco 175 (submapper 1) and Namco 340 (submapper 2)
// TODO: Untested! Need relevant ROMs to test against (everything is mapper 19 instead).
pub struct Mapper210 {
    variant: Variant,
}

impl Mapper for Mapper210 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(64)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => {
                self.variant = Variant::Namco175;
                params.write_prg(address, value);
            }
            0x8000..=0x87FF => params.set_bank_index_register(C0, value),
            0x8800..=0x8FFF => params.set_bank_index_register(C1, value),
            0x9000..=0x97FF => params.set_bank_index_register(C2, value),
            0x9800..=0x9FFF => params.set_bank_index_register(C3, value),
            0xA000..=0xA7FF => params.set_bank_index_register(C4, value),
            0xA800..=0xAFFF => params.set_bank_index_register(C5, value),
            0xB000..=0xB7FF => params.set_bank_index_register(C6, value),
            0xB800..=0xBFFF => params.set_bank_index_register(C7, value),
            0xC000..=0xC7FF => {
                self.variant = Variant::Namco175;
                /* TODO: External PRG RAM enable. */
            }
            0xC800..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xE7FF => {
                // In practice, only Namco 340 ROMs set either of the first two bits.
                if self.variant == Variant::Unknown && value & 0b1100_0000 != 0 {
                    self.variant = Variant::Namco340;
                }

                // TODO: Confirm that we'll actually know the variant in time for setting the mirroring.
                if self.variant == Variant::Namco340 {
                    let mirroring = match value >> 6 {
                        0b00 => NameTableMirroring::OneScreenLeftBank,
                        0b01 => NameTableMirroring::Vertical,
                        0b10 => NameTableMirroring::OneScreenRightBank,
                        0b11 => NameTableMirroring::Horizontal,
                        _ => unreachable!(),
                    };
                    params.set_name_table_mirroring(mirroring);
                }

                params.set_bank_index_register(P0, value & 0b0011_1111);
            }
            0xE800..=0xEFFF => params.set_bank_index_register(P1, value & 0b0011_1111),
            0xF000..=0xF7FF => params.set_bank_index_register(P2, value & 0b0011_1111),
            0xF800..=0xFFFF => { /* Do nothing. */ }
        }
    }
}

impl Mapper210 {
    pub fn new() -> Self {
        Self { variant: Variant::Unknown }
    }
}

#[derive(PartialEq)]
enum Variant {
    Unknown,
    // Submapper 1
    Namco175,
    // Submapper 2 
    Namco340,
}
