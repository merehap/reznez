pub fn pack_bools(bools: [bool; 8]) -> u8 {
    let mut result = 0;
    for i in 0..8 {
        if bools[7 - i] {
            result += 1 << i;
        }
    }

    result
}

pub fn unpack_bools(value: u8) -> [bool; 8] {
    let mut bools = [false; 8];

    for (i, b) in bools.iter_mut().enumerate() {
        *b = (value & (0b1000_0000 >> i)) != 0;
    }

    bools
}

#[inline]
pub fn get_bit(byte: u8, index: usize) -> bool {
    assert!(index < 8);
    let mask = 0b1000_0000 >> index as u8;
    byte & mask != 0
}

#[allow(dead_code)]
pub fn clear_bit(byte: u8, index: usize) -> u8 {
    assert!(index < 8);
    byte & !(1 << (7 - index))
}

#[allow(dead_code)]
pub fn set_bit(byte: u8, index: usize) -> u8 {
    assert!(index < 8);
    byte & (1 << (7 - index))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_all_true() {
        assert_eq!(pack_bools([true; 8]), 0xFF);
    }

    #[test]
    fn pack_all_false() {
        assert_eq!(pack_bools([false; 8]), 0x00);
    }

    #[test]
    fn pack_mixture() {
        assert_eq!(pack_bools([true, true, false, true, true, false, true, false]), 0xDA);
    }

    #[test]
    fn unpack_all_true() {
        assert_eq!(unpack_bools(0xFF), [true; 8]);
    }

    #[test]
    fn unpack_all_false() {
        assert_eq!(unpack_bools(0x00), [false; 8]);
    }

    #[test]
    fn unpack_mixture() {
        assert_eq!(unpack_bools(0xDA), [true, true, false, true, true, false, true, false]);
    }

    #[test]
    fn get_bit_0_true() {
        assert_eq!(get_bit(0b10100110, 0), true);
    }

    #[test]
    fn get_bit_7_false() {
        assert_eq!(get_bit(0b10100110, 7), false);
    }

    #[test]
    fn clear_bit_clears() {
        assert_eq!(clear_bit(0b10100110, 5), 0b10100010);
    }

    #[test]
    fn clear_already_cleared_bit() {
        assert_eq!(clear_bit(0b10100110, 4), 0b10100110);
    }
}
