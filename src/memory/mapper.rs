use num_traits::FromPrimitive;

use crate::cpu::address::Address as CpuAddress;
use crate::cpu::memory::Memory as CpuMemory;
use crate::memory::ppu_ram::PpuRam;
use crate::memory::ppu_address::PpuAddress;
use crate::memory::palette_ram::PaletteRam;
use crate::memory::vram::VramSide;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::name_table::name_table_number::NameTableNumber;

pub const PATTERN_TABLE_SIZE: usize = 0x2000;
pub const NAME_TABLE_SIZE: usize = 0x400;

pub trait Mapper {
    fn cpu_read(&self, memory: &mut CpuMemory, address: CpuAddress) -> u8;
    fn cpu_write(&self, memory: &mut CpuMemory, address: CpuAddress, value: u8);
    fn ppu_read(&self, ram: &PpuRam, address: PpuAddress) -> u8;
    fn ppu_write(&mut self, ram: &mut PpuRam, address: PpuAddress, value: u8);

    fn raw_pattern_table(&self) -> &[u8; PATTERN_TABLE_SIZE];
    fn raw_pattern_table_mut(&mut self) -> &mut [u8; PATTERN_TABLE_SIZE];

    #[inline]
    fn raw_name_table<'a>(
        &'a self,
        ppu_ram: &'a PpuRam,
        number: NameTableNumber,
    ) -> &'a [u8; NAME_TABLE_SIZE] {
        (&ppu_ram.vram).side(vram_side(number, ppu_ram.name_table_mirroring))
    }

    #[inline]
    fn raw_name_table_mut<'a>(
        &'a mut self,
        ppu_ram: &'a mut PpuRam,
        number: NameTableNumber,
    ) -> &'a mut [u8; NAME_TABLE_SIZE] {
        (&mut ppu_ram.vram).side_mut(vram_side(number, ppu_ram.name_table_mirroring))
    }

    #[inline]
    fn pattern_table_byte(&self, address: PpuAddress) -> u8 {
        self.raw_pattern_table()[address.to_usize()]
    }

    #[inline]
    fn pattern_table_byte_mut(&mut self, address: PpuAddress) -> &mut u8 {
        &mut self.raw_pattern_table_mut()[address.to_usize()]
    }

    #[inline]
    fn name_table_byte(&self, ppu_ram: &PpuRam, address: PpuAddress) -> u8 {
        let (name_table_number, index) = address_to_name_table_index(address);
        self.raw_name_table(ppu_ram, name_table_number)[index]
    }

    #[inline]
    fn name_table_byte_mut<'a>(&'a mut self, ppu_ram: &'a mut PpuRam, address: PpuAddress) -> &'a mut u8 {
        let (name_table_number, index) = address_to_name_table_index(address);
        &mut self.raw_name_table_mut(ppu_ram, name_table_number)[index]
    }

    #[inline]
    fn palette_table_byte(&self, palette_ram: &PaletteRam, address: PpuAddress) -> u8 {
        palette_ram[address_to_palette_ram_index(address)]
    }

    #[inline]
    fn palette_table_byte_mut<'a>(&self, palette_ram: &'a mut PaletteRam, address: PpuAddress) -> &'a mut u8 {
        &mut palette_ram[address_to_palette_ram_index(address)]
    }
}

#[inline]
fn address_to_name_table_index(address: PpuAddress) -> (NameTableNumber, usize) {
    const NAME_TABLE_START:    usize = 0x2000;
    const MIRROR_START:        usize = 0x3000;
    const PALETTE_TABLE_START: usize = 0x3F00;

    let address = address.to_usize();
    assert!(address >= NAME_TABLE_START);
    assert!(address < PALETTE_TABLE_START);

    let mut index = address;
    if index >= MIRROR_START {
        index -= 0x1000;
    }

    let index = index - NAME_TABLE_START;

    let name_table_number =
        NameTableNumber::from_usize(index / NAME_TABLE_SIZE).unwrap();
    let index = index % NAME_TABLE_SIZE;
    (name_table_number, index)
}

#[inline]
fn vram_side(
    name_table_number: NameTableNumber,
    mirroring: NameTableMirroring,
) -> VramSide {

    use NameTableNumber::*;
    use NameTableMirroring::*;
    match (name_table_number, mirroring) {
        (Zero , _         ) => VramSide::Left,
        (One  , Horizontal) => VramSide::Left,
        (Two  , Horizontal) => VramSide::Right,
        (One  , Vertical  ) => VramSide::Right,
        (Two  , Vertical  ) => VramSide::Left,
        (Three, _         ) => VramSide::Right,
        (_    , FourScreen) => todo!("FourScreen isn't supported yet."),
    }
}

fn address_to_palette_ram_index(address: PpuAddress) -> usize {
    const PALETTE_TABLE_START: usize = 0x3F00;
    const HIGH_ADDRESS_START : usize = 0x4000;

    let mut address = address.to_usize();
    assert!(address >= PALETTE_TABLE_START);
    assert!(address < HIGH_ADDRESS_START);

    if matches!(address, 0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C) {
        address -= 0x10;
    }

    address % 0x20
}
