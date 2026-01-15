use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_number(0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// BMC 31-IN-1
// Untested. Need test ROM.
pub struct Mapper229;

impl Mapper for Mapper229 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* No registers here. */ }
            0x8000..=0xFFFF => {
                let bank = addr.low_byte();
                bus.set_name_table_mirroring((bank >> 5) & 1);
                bus.set_prg_register(P0, bank);
                bus.set_chr_register(C0, bank);

                let use_switchable_prg_layout = bank > 0;
                bus.set_prg_layout(use_switchable_prg_layout as u8);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}