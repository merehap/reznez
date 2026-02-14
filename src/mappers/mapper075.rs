use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(R)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(D)),
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
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => bus.set_prg_register(P, value & 0b0000_1111),
            0x9000..=0x9FFF => {
                let fields = splitbits!(min=u8, value, ".....rlm");

                self.chr_right_high_bit = fields.r << 4;
                self.chr_left_high_bit = fields.l << 4;
                if matches!(bus.name_table_mirroring(), NameTableMirroring::VERTICAL | NameTableMirroring::HORIZONTAL) {
                    bus.set_name_table_mirroring(fields.m);
                } else {
                    todo!("Handle four screen mirroring");
                }
            }
            0xA000..=0xAFFF => bus.set_prg_register(Q, value & 0b0000_1111),
            0xB000..=0xBFFF => { /* Do nothing. */ }
            0xC000..=0xCFFF => bus.set_prg_register(R, value & 0b0000_1111),
            0xD000..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xEFFF => {
                let bank_number = self.chr_left_high_bit | (value & 0b0000_1111);
                bus.set_chr_register(C, bank_number);
            }
            0xF000..=0xFFFF => {
                let bank_number = self.chr_right_high_bit | (value & 0b0000_1111);
                bus.set_chr_register(D, bank_number);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
