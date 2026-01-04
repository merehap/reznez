use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .fixed_name_table_mirroring()
    .cartridge_selection_name_table_mirrorings([
        Some(NameTableMirroring::VERTICAL),
        Some(NameTableMirroring::VERTICAL),
        Some(NameTableMirroring::VERTICAL),
        Some(NameTableMirroring::VERTICAL),
    ])
    .build();

const CHR_IDS: [ChrBankRegisterId; 4] = [C0, C1, C2, C3];

// Magic Kid Googoo by Zemina
pub struct Mapper190;

impl Mapper for Mapper190 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF | 0xE000..=0xFFFF => { /* No regs here. */ }
            0x8000..=0x9FFF => bus.set_prg_register(P0, value & 0b0111),
            0xC000..=0xDFFF => bus.set_prg_register(P0, (value & 0b0111) | 0b1000),
            0xA000..=0xBFFF => bus.set_chr_register(CHR_IDS[*addr as usize & 0b11], value),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}