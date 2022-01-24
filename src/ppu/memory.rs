use crate::ppu::address::Address;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

const MEMORY_SIZE: usize = 0x4000;

const PATTERN_TABLE_START: Address = Address::from_u16(0);
const PATTERN_TABLE_SIZE: u16 = 0x2000;

const NAME_TABLE_START: u16 = 0x2000;
const NAME_TABLE_SIZE: u16 = 0x400;
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
const NAME_TABLE_INDEXES: [Address; 4] =
    [
        Address::from_u16(NAME_TABLE_START + 0 * NAME_TABLE_SIZE),
        Address::from_u16(NAME_TABLE_START + 1 * NAME_TABLE_SIZE),
        Address::from_u16(NAME_TABLE_START + 2 * NAME_TABLE_SIZE),
        Address::from_u16(NAME_TABLE_START + 3 * NAME_TABLE_SIZE),
    ];

pub const PALETTE_TABLE_START: Address = Address::from_u16(0x3F00);
const PALETTE_TABLE_SIZE: u16 = 0x20;

pub struct Memory {
    memory: [u8; MEMORY_SIZE],
    name_table_mirroring: NameTableMirroring,
}

impl Memory {
    pub fn new(name_table_mirroring: NameTableMirroring) -> Memory {
        Memory {
            memory: [0; MEMORY_SIZE],
            name_table_mirroring,
        }
    }

    pub fn read(&self, address: Address) -> u8 {
        let address = self.map_if_name_table_address(address);
        self.memory[address.to_usize()]
    }

    pub fn write(&mut self, address: Address, value: u8) {
        let address = self.map_if_name_table_address(address);
        self.memory[address.to_usize()] = value;
    }

    #[inline]
    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    /*
    #[inline]
    pub fn pattern_table(&self) -> PatternTable {
        let raw = self.slice(
            PATTERN_TABLE_START,
            PATTERN_TABLE_START.advance(PATTERN_TABLE_SIZE - 1),
        );
        PatternTable::new(raw.try_into().unwrap())
    }

    #[inline]
    pub fn name_table(&self, number: NameTableNumber) -> NameTable {
        let index = NAME_TABLE_INDEXES[number as usize];
        let raw = self.slice(index, index.advance(NAME_TABLE_SIZE - 1));
        NameTable::new(raw.try_into().unwrap())
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        let raw = self.slice(
            PALETTE_TABLE_START,
            PALETTE_TABLE_START.advance(PALETTE_TABLE_SIZE - 1)
        );
        PaletteTable::new(raw.try_into().unwrap(), &self.system_palette)
    }
    */

    fn map_if_name_table_address(&self, address: Address) -> Address {
        // No modification if it's not a name table address.
        if address < NAME_TABLE_INDEXES[0] || address >= PALETTE_TABLE_START {
            return address;
        }

        use NameTableMirroring::*;
        match self.name_table_mirroring {
            Horizontal => Address::from_u16(address.to_u16() & 0b1111_1011_1111_1111),
            Vertical   => Address::from_u16(address.to_u16() & 0b1111_0111_1111_1111),
            FourScreen => todo!("FourScreen isn't supported yet."),
        }
    }

    pub fn slice<'a>(&'a self, start_address: Address, end_address: Address) -> &'a [u8] {
        let start_address = self.map_if_name_table_address(start_address);
        let end_address = self.map_if_name_table_address(end_address);
        &self.memory[start_address.to_usize()..=end_address.to_usize()]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ppu::address::Address;
    use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

    #[test]
    fn horizontal_mirror_mapping_low() {
        let memory = Memory::new(NameTableMirroring::Horizontal);
        let result = memory.map_if_name_table_address(Address::from_u16(0x2C00));
        assert_eq!(result, Address::from_u16(0x2800));
    }

    #[test]
    fn horizontal_mirror_mapping_high() {
        let memory = Memory::new(NameTableMirroring::Horizontal);
        let result = memory.map_if_name_table_address(Address::from_u16(0x2FFF));
        assert_eq!(result, Address::from_u16(0x2BFF));
    }

    #[test]
    fn vertical_mirror_mapping_low() {
        let memory = Memory::new(NameTableMirroring::Vertical);
        let result = memory.map_if_name_table_address(Address::from_u16(0x2C00));
        assert_eq!(result, Address::from_u16(0x2400));
    }

    #[test]
    fn vertical_mirror_mapping_high() {
        let memory = Memory::new(NameTableMirroring::Vertical);
        let result = memory.map_if_name_table_address(Address::from_u16(0x2FFF));
        assert_eq!(result, Address::from_u16(0x27FF));
    }

    #[test]
    fn no_mapping_for_non_name_table_address_low() {
        let memory = Memory::new(NameTableMirroring::Horizontal);
        let result = memory.map_if_name_table_address(Address::from_u16(0x1FFF));
        assert_eq!(result, Address::from_u16(0x1FFF));

        let memory = Memory::new(NameTableMirroring::Vertical);
        let result = memory.map_if_name_table_address(Address::from_u16(0x1FFF));
        assert_eq!(result, Address::from_u16(0x1FFF));
    }

    #[test]
    fn no_mapping_for_palette_index() {
        let memory = Memory::new(NameTableMirroring::Horizontal);
        let result = memory.map_if_name_table_address(Address::from_u16(0x3F00));
        assert_eq!(result, Address::from_u16(0x3F00));

        let memory = Memory::new(NameTableMirroring::Vertical);
        let result = memory.map_if_name_table_address(Address::from_u16(0x3F00));
        assert_eq!(result, Address::from_u16(0x3F00));
    }
}
