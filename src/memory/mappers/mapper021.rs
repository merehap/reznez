use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc2_and_4::Vrc2And4;

// VRC4a and VRC4c (submapper 1 and 2, respectively)
pub fn mapper021() -> Box<dyn Mapper> {
    let mut mapper021_mappings = Vec::new();

    // VRC4a
    mapper021_mappings.extend_from_slice(&[
        (0xB000, 0xB002, C0),
        (0xB004, 0xB006, C1),
        (0xC000, 0xC002, C2),
        (0xC004, 0xC006, C3),
        (0xD000, 0xD002, C4),
        (0xD004, 0xD006, C5),
        (0xE000, 0xE002, C6),
        (0xE004, 0xE006, C7),
    ]);

    // VRC4c
    mapper021_mappings.extend_from_slice(&[
        (0xB000, 0xB040, C0),
        (0xB080, 0xB0C0, C1),
        (0xC000, 0xC040, C2),
        (0xC080, 0xC0C0, C3),
        (0xD000, 0xD040, C4),
        (0xD080, 0xD0C0, C5),
        (0xE000, 0xE040, C6),
        (0xE080, 0xE0C0, C7),
    ]);

    Box::new(Vrc2And4::new(&mapper021_mappings))
}
