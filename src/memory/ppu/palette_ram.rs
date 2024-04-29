const PALETTE_RAM_SIZE: usize = 0x20;
const INITIAL_PALETTE_DATA: [u8; PALETTE_RAM_SIZE] = [
    0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00, 0x04, 0x2C,
    0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02, 0x00, 0x20, 0x2C, 0x08,
];

pub struct PaletteRam([u8; PALETTE_RAM_SIZE]);

impl PaletteRam {
    pub fn new() -> PaletteRam {
        PaletteRam(INITIAL_PALETTE_DATA)
    }

    pub fn read(&self, index: usize) -> u8 {
        self.0[index]
    }

    pub fn write(&mut self, index: usize, value: u8) {
        // First two bits are always 0 for palette RAM bytes.
        // See https://wiki.nesdev.org/w/index.php?title=PPU_palettes#Memory_Map
        self.0[index] = value & 0b0011_1111;
    }

    pub fn to_slice(&self) -> &[u8; PALETTE_RAM_SIZE] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_first_bits() {
        let mut palette_ram = PaletteRam::new();
        assert_eq!(palette_ram.read(12), 0b0000_0000);
        palette_ram.write(12, 0b1110_1010);
        assert_eq!(palette_ram.read(12), 0b0010_1010);
    }
}
