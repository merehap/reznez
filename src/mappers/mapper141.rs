use crate::mapper::KIBIBYTE;
use crate::memory::layout::Layout;
use crate::mappers::common::sachen8259::{Sachen8259, Sachen8259Board};

use super::common::sachen8259;

const LAYOUT: Layout = sachen8259::LAYOUT.into_builder()
    .chr_rom_max_size(256 * KIBIBYTE)
    .build();

// TODO: Support Q Boy once a suitable ROM is found.
pub const MAPPER141: Sachen8259 = Sachen8259::new(LAYOUT, Sachen8259Board::A);