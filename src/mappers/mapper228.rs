use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // Actually 3 * 512 * KIBIBYTE, but non-powers-of-2 can't be represented here yet,
    // and the ROM is mirrored up to 2048KiB anyways.
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_rom_outer_bank_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_rom_outer_bank_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Active Enterprises
// TODO: Outer bank 2 (chip 2) doesn't exist and should read back as open bus. Currently it is just a mirroring of chip 3.
pub struct Mapper228;

impl Mapper for Mapper228 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* No regs here. The claimed RAM doesn't exist on either game. */ }
            0x8000..=0xFFFF => {
                let chr_inner_bank = splitbits_named!(value, ".... ..cc");
                bus.set_chr_register(C, chr_inner_bank);
                let fields = splitbits!(min=u8, *addr, "..mo oppp ppl. cccc");
                bus.set_name_table_mirroring(fields.m);
                bus.set_prg_rom_outer_bank_number(fields.o);
                bus.set_prg_register(P, fields.p);
                bus.set_prg_layout(fields.l);
                bus.set_chr_rom_outer_bank_number(fields.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}