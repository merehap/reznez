use crate::mapper::*;
use crate::mappers::vrc::vrc2::{Vrc2, BankLowBitBehavior};

// VRC2c
pub fn mapper025_3() -> Vrc2 {
    let mappings = &[
        (0xB000, 0xB002, C),
        (0xB001, 0xB003, D),
        (0xC000, 0xC002, E),
        (0xC001, 0xC003, F),
        (0xD000, 0xD002, G),
        (0xD001, 0xD003, H),
        (0xE000, 0xE002, I),
        (0xE001, 0xE003, J),
    ];

    Vrc2::new(mappings, BankLowBitBehavior::Keep)
}
