use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_rom_outer_bank_size(1024 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .chr_rom_max_size(1024 * KIBIBYTE)
    .chr_rom_outer_bank_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// ET-4310 and K-1010
// TODO: See if there is any reason to implement the 0x5800 RAM bits. No other emulators seem to implement them for some reason.
pub struct Mapper225;

impl Mapper for Mapper225 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* No regs here. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, *addr, ".oml pppp ppcc cccc");
                bus.set_prg_rom_outer_bank_number(fields.o);
                bus.set_chr_rom_outer_bank_number(fields.o);
                bus.set_prg_layout(fields.l);
                bus.set_name_table_mirroring(fields.m);
                bus.set_prg_register(P, fields.p);
                bus.set_chr_register(C, fields.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}