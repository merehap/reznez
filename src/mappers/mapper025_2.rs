use crate::mapper::*;
use crate::mappers::vrc::vrc4::Vrc4;

// VRC4d
pub fn mapper025_2() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB008, C),
        (0xB004, 0xB00C, D),
        (0xC000, 0xC008, E),
        (0xC004, 0xC00C, F),
        (0xD000, 0xD008, G),
        (0xD004, 0xD00C, H),
        (0xE000, 0xE008, I),
        (0xE004, 0xE00C, J),
    ];

    Vrc4::new(mappings)
}
