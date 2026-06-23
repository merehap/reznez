use crate::mapper::mappers::common::mapper114::Mapper114;

// MMC3 Clone with scrambled registers, Boogerman scrambling pattern
// FIXME: Boogerman freezes randomly.
pub fn mapper114_1() -> Mapper114 {
    let scrambled_addrs = [
        (0x8000, 0xA001),
        (0x8001, 0x8001),
        (0xA000, 0x8000),
        (0xA001, 0xC001),
        (0xC000, 0xA000),
        (0xC001, 0xC000),
        (0xE000, 0xE000),
        (0xE001, 0xE001),
    ].into();

    let scrambled_bank_regs: [u8; 8] = [0, 2, 5, 3, 6, 1, 7, 4];
    Mapper114::new(scrambled_addrs, scrambled_bank_regs)
}