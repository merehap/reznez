use num_traits::FromPrimitive;

use crate::memory::cpu_address::CpuAddress;
use crate::memory::cpu_internal_ram::CpuInternalRam;
use crate::memory::ppu_internal_ram::PpuInternalRam;
use crate::memory::ppu_address::PpuAddress;
use crate::memory::palette_ram::PaletteRam;
use crate::memory::ports::Ports;
use crate::memory::vram::VramSide;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::name_table::name_table_number::NameTableNumber;

pub const PATTERN_TABLE_SIZE: usize = 0x2000;
pub const NAME_TABLE_SIZE: usize = 0x400;

pub trait Mapper {
    fn prg_rom(&self) -> &[u8; 0x8000];

    fn raw_pattern_table(&self) -> &[u8; PATTERN_TABLE_SIZE];
    fn raw_pattern_table_mut(&mut self) -> &mut [u8; PATTERN_TABLE_SIZE];

    #[inline]
    fn cpu_read(
        &self,
        cpu_internal_ram: &CpuInternalRam,
        ports: &mut Ports,
        address: CpuAddress,
    ) -> u8 {
        match address.to_raw() {
            0x0000..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF],
            0x2000..=0x2007 => ports.get(address),
            0x2008..=0x3FFF => ports.get(CpuAddress::new(0x2000 + address.to_raw() % 8)),
            0x4000..=0x4013 | 0x4015 => {/* APU */ 0},
            0x4014 | 0x4016 | 0x4017 => ports.get(address),
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0x7FFF => {println!("Read from non-ROM cartridge space."); 0},
            0x8000..=0xFFFF => self.prg_rom()[address.to_usize() - 0x8000],
        }
    }

    #[inline]
    fn cpu_write(
        &self,
        cpu_internal_ram: &mut CpuInternalRam,
        ports: &mut Ports,
        address: CpuAddress,
        value: u8,
    ) {
        match address.to_raw() {
            0x0000..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF] = value,
            0x2000..=0x2007 => ports.set(address, value),
            0x2008..=0x3FFF => ports.set(CpuAddress::new(0x2000 + address.to_raw() % 8), value),
            0x4000..=0x4013 | 0x4015 => {/* APU */},
            0x4014 | 0x4016..=0x4017 => ports.set(address, value),
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0x7FFF => println!("Ignored writes to non-ROM cartridge space."),
            0x8000..=0xFFFF => println!("ROM CPU write ignored ({}).", address),
        }
    }

    #[inline]
    fn ppu_read(&self, ppu_internal_ram: &PpuInternalRam, address: PpuAddress) -> u8 {
        match address.to_u16() {
            0x0000..=0x1FFF => self.pattern_table_byte(address),
            0x2000..=0x3EFF => self.name_table_byte(ppu_internal_ram, address),
            0x3F00..=0x3FFF => self.palette_table_byte(&ppu_internal_ram.palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn ppu_write(&mut self, ppu_internal_ram: &mut PpuInternalRam, address: PpuAddress, value: u8) {
        match address.to_u16() {
            0x0000..=0x1FFF => *self.pattern_table_byte_mut(address) = value,
            0x2000..=0x3EFF => *self.name_table_byte_mut(ppu_internal_ram, address) = value,
            0x3F00..=0x3FFF => *self.palette_table_byte_mut(&mut ppu_internal_ram.palette_ram, address) = value,
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn raw_name_table<'a>(
        &'a self,
        ppu_internal_ram: &'a PpuInternalRam,
        number: NameTableNumber,
    ) -> &'a [u8; NAME_TABLE_SIZE] {
        (&ppu_internal_ram.vram).side(vram_side(number, ppu_internal_ram.name_table_mirroring))
    }

    #[inline]
    fn raw_name_table_mut<'a>(
        &'a mut self,
        ppu_internal_ram: &'a mut PpuInternalRam,
        number: NameTableNumber,
    ) -> &'a mut [u8; NAME_TABLE_SIZE] {
        (&mut ppu_internal_ram.vram).side_mut(vram_side(number, ppu_internal_ram.name_table_mirroring))
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
    fn name_table_byte(&self, ppu_internal_ram: &PpuInternalRam, address: PpuAddress) -> u8 {
        let (name_table_number, index) = address_to_name_table_index(address);
        self.raw_name_table(ppu_internal_ram, name_table_number)[index]
    }

    #[inline]
    fn name_table_byte_mut<'a>(&'a mut self, ppu_internal_ram: &'a mut PpuInternalRam, address: PpuAddress) -> &'a mut u8 {
        let (name_table_number, index) = address_to_name_table_index(address);
        &mut self.raw_name_table_mut(ppu_internal_ram, name_table_number)[index]
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