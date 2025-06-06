use crate::mapper::KIBIBYTE;
use crate::memory::layout::Layout;
use crate::mappers::common::sachen8259::{Sachen8259, Sachen8259Board};

use super::common::sachen8259;

const LAYOUT: Layout = sachen8259::LAYOUT.into_builder()
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_rom_outer_bank_size(16 * KIBIBYTE)
    .build();

pub const MAPPER138: Sachen8259 = Sachen8259::new(LAYOUT, Sachen8259Board::B);