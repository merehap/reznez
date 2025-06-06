use crate::mapper::*;
use crate::mappers::vrc::vrc4::Vrc4;

// VRC4e
pub fn mapper023_2() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB004, C0),
        (0xB008, 0xB00C, C1),
        (0xC000, 0xC004, C2),
        (0xC008, 0xC00C, C3),
        (0xD000, 0xD004, C4),
        (0xD008, 0xD00C, C5),
        (0xE000, 0xE004, C6),
        (0xE008, 0xE00C, C7),
    ];

    Vrc4::new(mappings)
}
