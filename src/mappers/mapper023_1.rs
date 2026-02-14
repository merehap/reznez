use crate::mapper::*;
use crate::mappers::vrc::vrc4::Vrc4;

// VRC4f
pub fn mapper023_1() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB001, C),
        (0xB002, 0xB003, D),
        (0xC000, 0xC001, E),
        (0xC002, 0xC003, F),
        (0xD000, 0xD001, G),
        (0xD002, 0xD003, H),
        (0xE000, 0xE001, I),
        (0xE002, 0xE003, J),
    ];

    Vrc4::new(mappings)
}
