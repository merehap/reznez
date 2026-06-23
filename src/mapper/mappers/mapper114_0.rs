use crate::mapper::mappers::common::mapper114::Mapper114;

// MMC3 Clone with scrambled registers, normal scrambling pattern
pub fn mapper114_0() -> Mapper114 {
    let scrambled_addrs = [
        (0x8000, 0xA001),
        (0x8001, 0xA000),
        (0xA000, 0x8000),
        (0xA001, 0xC000),
        (0xC000, 0x8001),
        (0xC001, 0xC001),
        (0xE000, 0xE000),
        (0xE001, 0xE001),
    ].into();

    let scrambled_bank_regs: [u8; 8] = [0, 3, 1, 5, 6, 7, 2, 4];
    Mapper114::new(scrambled_addrs, scrambled_bank_regs)
}