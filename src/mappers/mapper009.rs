use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .override_chr_meta_register(M0, C1)
    .override_chr_meta_register(M1, C3)
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        // TODO: PlayChoice uses this window.
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-3)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
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

// MMC2 (PNROM and PEEOROM boards)
pub struct Mapper009;

impl Mapper for Mapper009 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        let bank_index = value & 0b0001_1111;
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xAFFF => params.set_prg_register(P0, bank_index),
            0xB000..=0xBFFF => params.set_chr_register(C0, bank_index),
            0xC000..=0xCFFF => params.set_chr_register(C1, bank_index),
            0xD000..=0xDFFF => params.set_chr_register(C2, bank_index),
            0xE000..=0xEFFF => params.set_chr_register(C3, bank_index),
            0xF000..=0xFFFF => params.set_name_table_mirroring(value & 1),
        }
    }

    fn on_ppu_read(&mut self, params: &mut MapperParams, address: PpuAddress, _value: u8) {
        let (meta_id, bank_register_id) = match address.to_u16() {
            0x0FD8 => (M0, C0),
            0x0FE8 => (M0, C1),
            0x1FD8..=0x1FDF => (M1, C2),
            0x1FE8..=0x1FEF => (M1, C3),
            // Skip to standard CHR memory operation.
            _ => return,
        };

        params.set_chr_meta_register(meta_id, bank_register_id);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
