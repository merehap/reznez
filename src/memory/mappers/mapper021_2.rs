use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc4::Vrc4;

// VRC4c
pub fn mapper021_2() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB040, C0),
        (0xB080, 0xB0C0, C1),
        (0xC000, 0xC040, C2),
        (0xC080, 0xC0C0, C3),
        (0xD000, 0xD040, C4),
        (0xD080, 0xD0C0, C5),
        (0xE000, 0xE040, C6),
        (0xE080, 0xE0C0, C7),
    ];

    Vrc4::new(mappings)
}
