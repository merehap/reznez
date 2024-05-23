use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc4::Vrc4;

// VRC4a
pub fn mapper021_1() -> Vrc4 {
    let mappings = &[
        (0xB000, 0xB002, C0),
        (0xB004, 0xB006, C1),
        (0xC000, 0xC002, C2),
        (0xC004, 0xC006, C3),
        (0xD000, 0xD002, C4),
        (0xD004, 0xD006, C5),
        (0xE000, 0xE002, C6),
        (0xE004, 0xE006, C7),
    ];

    Vrc4::new(mappings)
}
