use crate::mapper::*;
use crate::mappers::vrc::vrc4::Vrc4;

// VRC4e
pub fn mapper023_2() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB004, C),
        (0xB008, 0xB00C, D),
        (0xC000, 0xC004, E),
        (0xC008, 0xC00C, F),
        (0xD000, 0xD004, G),
        (0xD008, 0xD00C, H),
        (0xE000, 0xE004, I),
        (0xE008, 0xE00C, J),
    ];

    Vrc4::new(mappings)
}
