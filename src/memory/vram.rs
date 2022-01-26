use std::ops::{Index, IndexMut};

const VRAM_SIZE: usize = 0x800;
const CHUNK_SIZE: usize = 0x400;

// CIRAM
pub struct Vram(Box<[u8; VRAM_SIZE]>);

impl Vram {
    pub fn new() -> Vram {
        Vram(Box::new([0; VRAM_SIZE]))
    }

    pub fn side(&self, side: VramSide) -> &[u8; CHUNK_SIZE] {
        let start_index = side as usize;
        (&self.0[start_index..start_index + CHUNK_SIZE]).try_into().unwrap()
    }

    pub fn side_mut(&mut self, side: VramSide) -> &mut [u8; CHUNK_SIZE] {
        let start_index = side as usize;
        (&mut self.0[start_index..start_index + CHUNK_SIZE]).try_into().unwrap()
    }
}

/*
#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::ppu_address::PpuAddress;
    use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

    #[test]
    fn horizontal_mirror_mapping_low() {
        let vram = Vram::new(NameTableMirroring::Horizontal);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x2C00));
        assert_eq!(result, PpuAddress::from_u16(0x2800));
    }

    #[test]
    fn horizontal_mirror_mapping_high() {
        let vram = Vram::new(NameTableMirroring::Horizontal);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x2FFF));
        assert_eq!(result, PpuAddress::from_u16(0x2BFF));
    }

    #[test]
    fn vertical_mirror_mapping_low() {
        let vram = Vram::new(NameTableMirroring::Vertical);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x2C00));
        assert_eq!(result, PpuAddress::from_u16(0x2400));
    }

    #[test]
    fn vertical_mirror_mapping_high() {
        let vram = Vram::new(NameTableMirroring::Vertical);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x2FFF));
        assert_eq!(result, PpuAddress::from_u16(0x27FF));
    }

    #[test]
    fn no_mapping_for_non_name_table_address_low() {
        let vram = Vram::new(NameTableMirroring::Horizontal);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x1FFF));
        assert_eq!(result, PpuAddress::from_u16(0x1FFF));

        let vram = Vram::new(NameTableMirroring::Vertical);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x1FFF));
        assert_eq!(result, PpuAddress::from_u16(0x1FFF));
    }

    #[test]
    fn no_mapping_for_palette_index() {
        let vram = Vram::new(NameTableMirroring::Horizontal);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x3F00));
        assert_eq!(result, PpuAddress::from_u16(0x3F00));

        let vram = Vram::new(NameTableMirroring::Vertical);
        let result = vram.map_if_name_table_address(PpuAddress::from_u16(0x3F00));
        assert_eq!(result, PpuAddress::from_u16(0x3F00));
    }
}
*/

impl Index<usize> for Vram {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for Vram {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

pub enum VramSide {
    Left = 0,
    Right = CHUNK_SIZE as isize,
}
