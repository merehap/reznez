use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc2::{Vrc2, ChrBankLowBitBehavior};

// VRC2b
pub fn mapper023_3() -> Box<dyn Mapper> {
    let mappings = &[
        (0xB000, 0xB001, C0),
        (0xB002, 0xB003, C1),
        (0xC000, 0xC001, C2),
        (0xC002, 0xC003, C3),
        (0xD000, 0xD001, C4),
        (0xD002, 0xD003, C5),
        (0xE000, 0xE001, C6),
        (0xE002, 0xE003, C7),
    ];

    Box::new(Vrc2::new(mappings, ChrBankLowBitBehavior::Keep))
}