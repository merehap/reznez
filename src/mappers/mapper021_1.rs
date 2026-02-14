use crate::mapper::*;
use crate::mappers::vrc::vrc4::Vrc4;

// VRC4a
pub fn mapper021_1() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB002, C),
        (0xB004, 0xB006, D),
        (0xC000, 0xC002, E),
        (0xC004, 0xC006, F),
        (0xD000, 0xD002, G),
        (0xD004, 0xD006, H),
        (0xE000, 0xE002, I),
        (0xE004, 0xE006, J),
    ];

    Vrc4::new(mappings)
}
