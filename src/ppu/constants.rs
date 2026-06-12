use crate::mapper::KIBIBYTE;

// The size of the name table proper plus attribute table.
pub const NAME_TABLE_WITH_ATTRIBUTES_SIZE: u32 = KIBIBYTE;
pub const ATTRIBUTE_TABLE_SIZE: u32 = 64;
// 0x3C0
pub const NAME_TABLE_SIZE: u32 = NAME_TABLE_WITH_ATTRIBUTES_SIZE - ATTRIBUTE_TABLE_SIZE;
