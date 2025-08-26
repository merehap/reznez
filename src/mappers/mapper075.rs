use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// VRC1
// TODO: Support VS System (and its 4-screen mirroring).
#[derive(Default)]
pub struct Mapper075 {
    chr_left_high_bit: u8,
    chr_right_high_bit: u8,
}

impl Mapper for Mapper075 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => params.set_prg_register(P0, value & 0b0000_1111),
            0x9000..=0x9FFF => {
                let fields = splitbits!(min=u8, value, ".....rlm");

                self.chr_right_high_bit = fields.r << 4;
                self.chr_left_high_bit = fields.l << 4;
                if matches!(params.name_table_mirroring(), NameTableMirroring::VERTICAL | NameTableMirroring::HORIZONTAL) {
                    params.set_name_table_mirroring(fields.m);
                } else {
                    todo!("Handle four screen mirroring");
                }
            }
            0xA000..=0xAFFF => params.set_prg_register(P1, value & 0b0000_1111),
            0xB000..=0xBFFF => { /* Do nothing. */ }
            0xC000..=0xCFFF => params.set_prg_register(P2, value & 0b0000_1111),
            0xD000..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xEFFF => {
                let bank_index = self.chr_left_high_bit | (value & 0b0000_1111);
                params.set_chr_register(C0, bank_index);
            }
            0xF000..=0xFFFF => {
                let bank_index = self.chr_right_high_bit | (value & 0b0000_1111);
                params.set_chr_register(C1, bank_index);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
