use crate::memory::ppu_address::PpuAddress;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

const MEMORY_SIZE: usize = 0x4000;

const NAME_TABLE_START: u16 = 0x2000;
const NAME_TABLE_SIZE: u16 = 0x400;
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
const NAME_TABLE_INDEXES: [PpuAddress; 4] =
    [
        PpuAddress::from_u16(NAME_TABLE_START + 0 * NAME_TABLE_SIZE),
        PpuAddress::from_u16(NAME_TABLE_START + 1 * NAME_TABLE_SIZE),
        PpuAddress::from_u16(NAME_TABLE_START + 2 * NAME_TABLE_SIZE),
        PpuAddress::from_u16(NAME_TABLE_START + 3 * NAME_TABLE_SIZE),
    ];

pub const PALETTE_TABLE_START: PpuAddress = PpuAddress::from_u16(0x3F00);

// VRAM/CIRAM
pub struct Vram {
    ram: [u8; MEMORY_SIZE],
    name_table_mirroring: NameTableMirroring,
}

impl Vram {
    pub fn new(name_table_mirroring: NameTableMirroring) -> Vram {
        Vram {
            ram: [0; MEMORY_SIZE],
            name_table_mirroring,
        }
    }

    pub fn read(&self, address: PpuAddress) -> u8 {
        let address = self.map_if_name_table_address(address);
        self.ram[address.to_usize()]
    }

    pub fn write(&mut self, address: PpuAddress, value: u8) {
        let address = self.map_if_name_table_address(address);
        self.ram[address.to_usize()] = value;
    }

    #[inline]
    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    fn map_if_name_table_address(&self, address: PpuAddress) -> PpuAddress {
        // No modification if it's not a name table address.
        if address < NAME_TABLE_INDEXES[0] || address >= PALETTE_TABLE_START {
            return address;
        }

        use NameTableMirroring::*;
        match self.name_table_mirroring {
            Horizontal => PpuAddress::from_u16(address.to_u16() & 0b1111_1011_1111_1111),
            Vertical   => PpuAddress::from_u16(address.to_u16() & 0b1111_0111_1111_1111),
            FourScreen => todo!("FourScreen isn't supported yet."),
        }
    }

    pub fn slice(&self, start_address: PpuAddress, end_address: PpuAddress) -> &[u8] {
        let start_address = self.map_if_name_table_address(start_address);
        let end_address = self.map_if_name_table_address(end_address);
        &self.ram[start_address.to_usize()..=end_address.to_usize()]
    }
}

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
