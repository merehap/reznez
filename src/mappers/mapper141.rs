use crate::mapper::KIBIBYTE;
use crate::memory::layout::Layout;
use crate::mappers::common::sachen8259::{Sachen8259, Sachen8259Board};

use super::common::sachen8259;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(sachen8259::PRG_LAYOUT)
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_rom_outer_bank_size(32 * KIBIBYTE)
    .chr_layout(sachen8259::NORMAL_CHR_LAYOUT)
    .chr_layout(sachen8259::SIMPLE_CHR_LAYOUT)
    .name_table_mirrorings(sachen8259::NAME_TABLE_MIRRORINGS)
    .build();

// TODO: Support Q Boy once a suitable ROM is found.
pub const MAPPER141: Sachen8259 = Sachen8259::new(LAYOUT, Sachen8259Board::A);