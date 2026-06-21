use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    // NROM128-mode
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Prg::ROM).switchable(P),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Prg::ROM).switchable(P),
    ])
    // NROM256-mode
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Prg::ROM).switchable(P),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_rom_outer_bank_size(64 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// GK
#[derive(Default)]
pub struct Mapper057 {
    inner_bank_left: u8,
    inner_bank_right: u8,
}

impl Mapper for Mapper057 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0x8800 {
            0x8000 => {
                let fields = splitbits!(min=u8, value, ".o.. .ccc");
                bus.set_chr_rom_outer_bank_number(fields.o);
                self.inner_bank_left = fields.c;
                bus.set_chr_register(C, self.inner_bank_left | self.inner_bank_right);
            }
            0x8800 => {
                let fields = splitbits!(min=u8, value, "pppl mccc");
                bus.set_prg_register(P, fields.p);
                bus.set_prg_layout(fields.l);
                bus.set_name_table_mirroring(fields.m);
                self.inner_bank_right = fields.c;
                bus.set_chr_register(C, self.inner_bank_left | self.inner_bank_right);
            }
            _ => { /* No regs here. */ }
        }

    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
