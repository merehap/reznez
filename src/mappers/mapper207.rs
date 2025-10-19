use crate::mapper::*;

use crate::mappers::mapper080;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(mapper080::PRG_LAYOUT)
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(mapper080::CHR_LAYOUT)
    .complicated_name_table_mirroring()
    .read_write_statuses(mapper080::READ_WRITE_STATUSES)
    .build();

// Taito's X1-005 (alternate name table mirrorings)
pub struct Mapper207 {
    mapper080: mapper080::Mapper080,
}

impl Mapper for Mapper207 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x7EF0 => self.set_mirroring_and_bank(mem, value, C0, NameTableQuadrant::TopLeft, NameTableQuadrant::TopRight),
            0x7EF1 => self.set_mirroring_and_bank(mem, value, C1, NameTableQuadrant::BottomLeft, NameTableQuadrant::BottomRight),
            _ => self.mapper080.write_register(mem, addr, value),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper207 {
    pub fn new() -> Self {
        Self { mapper080: mapper080::Mapper080 }
    }

    fn set_mirroring_and_bank(
        &self,
        mem: &mut Memory,
        value: u8,
        chr_id: ChrBankRegisterId,
        left_quadrant: NameTableQuadrant,
        right_quadrant: NameTableQuadrant,
    ) {
        let (ciram_right, chr_bank) = splitbits_named!(value, "vccc cccc");
        let ciram_side = if ciram_right { CiramSide::Right } else { CiramSide::Left };
        mem.set_name_table_quadrant(left_quadrant, ciram_side);
        mem.set_name_table_quadrant(right_quadrant, ciram_side);
        mem.set_chr_register(chr_id, chr_bank);
    }
}