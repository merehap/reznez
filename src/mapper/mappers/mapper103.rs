use crate::mapper::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .override_prg_rom_inner_bank_size(2 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::WORK_RAM_OR_ROM).fixed_number(0),
        PrgWindow::new(0x8000, 0xB7FF, 14 * KIBIBYTE, Prg::ROM).fixed_number(48),
        PrgWindow::new(0xB800, 0xD7FF,  8 * KIBIBYTE, Prg::WORK_RAM_OR_ROM).fixed_number(1),
        PrgWindow::new(0xD800, 0xFFFF, 10 * KIBIBYTE, Prg::ROM).fixed_number(59),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::ROM).switchable(P),
        PrgWindow::new(0x8000, 0xB7FF, 14 * KIBIBYTE, Prg::ROM).fixed_number(48),
        PrgWindow::new(0xB800, 0xD7FF,  8 * KIBIBYTE, Prg::ROM).fixed_number(55),
        PrgWindow::new(0xD800, 0xFFFF, 10 * KIBIBYTE, Prg::ROM).fixed_number(59),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Chr::ROM_OR_RAM),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Doki Doki Panic (pirate port of the FDS version)
// FIXME: Sub-8KiB bank size doesn't work yet, so this mapper is broken.
pub struct Mapper103;

impl Mapper for Mapper103 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* No regs here. */ }
            // This mapper writes to RAM even with the PRG RAM layout isn't selected.
            0x6000..=0x7FFF => bus.prg_memory.write_raw_work_ram(u32::from(*addr) - 0x6000, value),
            0x8000..=0x8FFF => bus.set_prg_register(P, value & 0b1111),
            0x9000..=0xB7FF => { /* No regs here. */ }
            // This mapper writes to RAM even with the PRG RAM layout isn't selected.
            0xB800..=0xD7FF => bus.prg_memory.write_raw_work_ram(u32::from(*addr) - 0x9800, value),
            0xD800..=0xDFFF => { /* No regs here. */ }
            0xE000..=0xEFFF => bus.set_name_table_mirroring((value >> 3) & 1),
            0xF000..=0xFFFF => bus.set_prg_layout((value >> 4) & 1),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
