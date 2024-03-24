use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc4::Vrc4;

// VRC4d
pub fn mapper025_2() -> Box<dyn Mapper> {
    let mappings = &[
        (0xB000, 0xB008, C0),
        (0xB004, 0xB00C, C1),
        (0xC000, 0xC008, C2),
        (0xC004, 0xC00C, C3),
        (0xD000, 0xD008, C4),
        (0xD004, 0xD00C, C5),
        (0xE000, 0xE008, C6),
        (0xE004, 0xE00C, C7),
    ];

    Box::new(Vrc4::new(mappings))
}
