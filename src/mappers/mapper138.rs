use crate::mapper::KIBIBYTE;
use crate::memory::layout::Layout;
use crate::mappers::common::sachen8259::{Sachen8259, Sachen8259Board};

use super::common::sachen8259;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(sachen8259::PRG_LAYOUT)
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_rom_outer_bank_size(16 * KIBIBYTE)
    .chr_layout(sachen8259::NORMAL_CHR_LAYOUT)
    .chr_layout(sachen8259::SIMPLE_CHR_LAYOUT)
    .name_table_mirrorings(sachen8259::NAME_TABLE_MIRRORINGS)
    .build();

pub const MAPPER138: Sachen8259 = Sachen8259::new(LAYOUT, Sachen8259Board::B);