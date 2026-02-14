use crate::mapper::*;
use crate::mappers::vrc::vrc4::Vrc4;

// VRC4c
pub fn mapper021_2() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB040, C),
        (0xB080, 0xB0C0, D),
        (0xC000, 0xC040, E),
        (0xC080, 0xC0C0, F),
        (0xD000, 0xD040, G),
        (0xD080, 0xD0C0, H),
        (0xE000, 0xE040, I),
        (0xE080, 0xE0C0, J),
    ];

    Vrc4::new(mappings)
}
