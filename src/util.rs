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

    for i in 0..8 {
        bools[i] = (value & 0b1000_0000) != 0;
    }

    bools
}
