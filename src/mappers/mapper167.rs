use splitbits::replacebits;

use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    // UNROM-like
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(0x20)),
    ])
    // NROM-like, but reversed ordering of the 16KiB banks.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        // P, but with inverted low bits.
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("p₀₅p₀₄p₀₃p₀₂p₀₁1₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.rom_address_template("p₀₅p₀₄p₀₃p₀₂p₀₁0₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀")),
    ])
    // Reverse UNROM-like
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(0x1F)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    // Duplicate of the above PRG layout.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(0x1F)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM)
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();

// Subor
// TODO: Testing. Need to support non-NTSC.
#[derive(Default)]
pub struct Mapper167 {
    left_bits: u8,
    right_bits: u8,
}

impl Mapper for Mapper167 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                let (t, mirroring) = splitbits_named!(min=u8, value, "...t ...m");
                bus.set_name_table_mirroring(mirroring);
                self.left_bits = replacebits!(self.left_bits, "00t. ....");
            }
            0xA000..=0xBFFF => {
                let (t, prg_layout) = splitbits_named!(min=u8, value, "...t ll..");
                bus.set_prg_layout(prg_layout);
                self.right_bits = replacebits!(self.right_bits, "00t. ....");
            }
            0xC000..=0xDFFF => {
                let v = value;
                self.left_bits = replacebits!(self.left_bits, "...v vvvv");
            }
            0xE000..=0xFFFF => {
                let v = value;
                self.right_bits = replacebits!(self.right_bits, "...v vvvv");
            }
        }

        let prg_bank = self.left_bits ^ self.right_bits;
        bus.set_prg_register(P, prg_bank);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}