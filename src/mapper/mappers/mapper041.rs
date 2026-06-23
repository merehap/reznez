use crate::mapper::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Prg::ROM).switchable(P),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_rom_outer_bank_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Chr::ROM).switchable(C),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Caltron 6-in-1
// TODO: Properly model bus conflicts.
pub struct Mapper041;

impl Mapper for Mapper041 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* No regs here. */ }
            0x6000..=0x67FF => {
                let fields = splitbits!(*addr, "........ ..mccppp");
                bus.set_name_table_mirroring(fields.m as u8);
                bus.set_chr_rom_outer_bank_number(fields.c);
                bus.set_prg_register(P, fields.p);
            }
            0x6800..=0x7FFF => { /* No regs here. */ }
            0x8000..=0xFFFF => {
                if bus.prg_memory.bank_registers().get(P).to_raw() >= 4 {
                    bus.set_chr_register(C, value & 0b11);
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
