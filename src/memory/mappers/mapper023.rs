use crate::memory::mapper::*;
use crate::memory::mappers::vrc::vrc2_and_4::Vrc2And4;

// VRC2b and VRC4a and VRC4c
pub fn mapper023_from_cartridge(_cartridge: &Cartridge) -> Box<dyn Mapper> {
    let mut mapper023_mappings = Vec::new();

    // VRC2b And VRC4f
    mapper023_mappings.extend_from_slice(&[
        (0xB000, 0xB001, C0),
        (0xB002, 0xB003, C1),
        (0xC000, 0xC001, C2),
        (0xC002, 0xC003, C3),
        (0xD000, 0xD001, C4),
        (0xD002, 0xD003, C5),
        (0xE000, 0xE001, C6),
        (0xE002, 0xE003, C7),
    ]);

    // VRC2e
    mapper023_mappings.extend_from_slice(&[
        (0xB000, 0xB004, C0),
        (0xB008, 0xB00C, C1),
        (0xC000, 0xC004, C2),
        (0xC008, 0xC00C, C3),
        (0xD000, 0xD004, C4),
        (0xD008, 0xD00C, C5),
        (0xE000, 0xE004, C6),
        (0xE008, 0xE00C, C7),
    ]);

    Box::new(Vrc2And4::new(&mapper023_mappings))
}
