use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .override_chr_meta_register(M0, C1)
    .override_chr_meta_register(M1, C3)
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM.meta_switchable(M0)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM.meta_switchable(M1)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// MMC4 (FxROM) - Similar to MMC2, but with Work RAM, bigger PRG ROM windows, and different bank-switching.
pub struct Mapper010;

impl Mapper for Mapper010 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        let bank_number = value & 0b0001_1111;
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xAFFF => mem.set_prg_register(P0, bank_number & 0b0000_1111),
            0xB000..=0xBFFF => mem.set_chr_register(C0, bank_number),
            0xC000..=0xCFFF => mem.set_chr_register(C1, bank_number),
            0xD000..=0xDFFF => mem.set_chr_register(C2, bank_number),
            0xE000..=0xEFFF => mem.set_chr_register(C3, bank_number),
            0xF000..=0xFFFF => mem.set_name_table_mirroring(value & 1),
        }
    }

    fn on_ppu_read(&mut self, mem: &mut Memory, address: PpuAddress, _value: u8) {
        let (meta_id, bank_register_id) = match address.to_u16() {
            0x0FD8..=0x0FDF => (M0, C0),
            0x0FE8..=0x0FEF => (M0, C1),
            0x1FD8..=0x1FDF => (M1, C2),
            0x1FE8..=0x1FEF => (M1, C3),
            // Skip to standard CHR memory operation.
            _ => return,
        };

        mem.set_chr_meta_register(meta_id, bank_register_id);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
