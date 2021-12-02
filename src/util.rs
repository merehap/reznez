pub fn pack_bools(bools: [bool; 8]) -> u8 {
    let mut result = 0;
    for i in 0..8 {
        if bools[7 - i as usize] {
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

pub fn get_bit(byte: u8, index: usize) -> bool {
    (byte >> (7 - index as u8)) != 0
}

pub fn reset_bit(byte: &mut u8, index: usize) {
    *byte &= !(1 << (7 - index))
}

pub fn set_bit(byte: &mut u8, index: usize) {
    *byte |= 1 << (7 - index)
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
}
