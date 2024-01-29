use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc2_and_4::Vrc2And4;

// VRC2c, VRC4b, and VRC4d (submappers 3, 1, and 2, respectively)
pub fn mapper025() -> Box<dyn Mapper> {
    let mut mapper025_mappings = Vec::new();

    // VRC2c and VRC4b
    mapper025_mappings.extend_from_slice(&[
        (0xB000, 0xB002, C0),
        (0xB001, 0xB003, C1),
        (0xC000, 0xC002, C2),
        (0xC001, 0xC003, C3),
        (0xD000, 0xD002, C4),
        (0xD001, 0xD003, C5),
        (0xE000, 0xE002, C6),
        (0xE001, 0xE003, C7),
    ]);

    // VRC4d
    mapper025_mappings.extend_from_slice(&[
        (0xB000, 0xB008, C0),
        (0xB004, 0xB00C, C1),
        (0xC000, 0xC008, C2),
        (0xC004, 0xC00C, C3),
        (0xD000, 0xD008, C4),
        (0xD004, 0xD00C, C5),
        (0xE000, 0xE008, C6),
        (0xE004, 0xE00C, C7),
    ]);

    Box::new(Vrc2And4::new(&mapper025_mappings))
}
