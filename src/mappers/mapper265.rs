use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_rom_outer_bank_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₃o₀₂o₀₁o₀₀p₀₂p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₃o₀₂o₀₁o₀₀1₀₂1₀₁1₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₃o₀₂o₀₁o₀₀p₀₂p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₃o₀₂o₀₁o₀₀p₀₂p₀₁p₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₃o₀₂o₀₁o₀₀p₀₂p₀₁0₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₃o₀₂o₀₁o₀₀1₀₂1₀₁1₀₀a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₃o₀₂o₀₁o₀₀p₀₂p₀₁a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    .chr_rom_max_size(0 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Chr::RAM)
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// T-262 multicarts
// TODO: Test me. Test ROM needed.
pub struct Mapper265 {
    address_latch_locked: bool,
}

impl Mapper for Mapper265 {
    fn reset(&mut self, _: &mut Bus) {
        self.address_latch_locked = false;
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* No regs here. */ }
            0x6000..=0xFFFF => {
                let fields = splitbits!(value, "..a. ..oo loo. ..ml");
                if !self.address_latch_locked {
                    self.address_latch_locked = fields.a;
                    bus.set_prg_rom_outer_bank_number(fields.o);
                    bus.set_prg_layout(fields.l);
                    bus.set_name_table_mirroring(fields.m as u8);
                }

                bus.set_prg_register(P, value & 0b111);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper265 {
    pub fn new() -> Self {
        Self { address_latch_locked: false }
    }
}
