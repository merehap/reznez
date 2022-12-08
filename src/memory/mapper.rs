use num_traits::FromPrimitive;

use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_internal_ram::CpuInternalRam;
use crate::memory::cpu::ports::Ports;
use crate::memory::cpu::prg_memory::PrgMemory;
use crate::memory::ppu::chr_memory::ChrMemory;
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ppu_internal_ram::PpuInternalRam;
use crate::memory::ppu::vram::VramSide;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::register::register_type::RegisterType;
use crate::util::unit::KIBIBYTE;

pub trait Mapper {
    fn name_table_mirroring(&self) -> NameTableMirroring;
    fn prg_memory(&self) -> &PrgMemory;
    fn chr_memory(&self) -> &ChrMemory;
    fn chr_memory_mut(&mut self) -> &mut ChrMemory;
    fn write_to_prg_memory(&mut self, address: CpuAddress, value: u8);

    #[inline]
    #[rustfmt::skip]
    fn cpu_read(
        &self,
        cpu_internal_ram: &CpuInternalRam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        address: CpuAddress,
    ) -> u8 {
        match address.to_raw() {
            0x0000..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF],
            0x2000..=0x3FFF => ppu_registers.read(address_to_ppu_register_type(address)),
            0x4000..=0x4013 => {/* TODO: APU */ 0},
            0x4014          => {/* OAM DMA is write-only. */ 0},
            0x4015          => {/* TODO: APU */ 0},
            0x4016          => ports.joypad1.borrow_mut().next_status() as u8,
            0x4017          => ports.joypad2.borrow_mut().next_status() as u8,
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0x5FFF => {/* TODO: Low registers. */ 0},
            0x6000..=0xFFFF => self.prg_memory().read(address),
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn cpu_write(
        &mut self,
        cpu_internal_ram: &mut CpuInternalRam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        address: CpuAddress,
        value: u8,
    ) {
        match address.to_raw() {
            0x0000..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF] = value,
            0x2000..=0x3FFF => ppu_registers.write(address_to_ppu_register_type(address), value),
            0x4000..=0x4013 => {/* APU */},
            0x4014          => ports.dma.set_page(value),
            0x4015          => {/* APU */},
            0x4016          => ports.change_strobe(value),
            0x4017          => {/* Do nothing? */},
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0xFFFF => self.write_to_prg_memory(address, value),
        }
    }

    #[inline]
    fn ppu_read(&self, ppu_internal_ram: &PpuInternalRam, address: PpuAddress) -> u8 {
        let palette_ram = &ppu_internal_ram.palette_ram;
        match address.to_u16() {
            0x0000..=0x1FFF => self.read_pattern_table_byte(address),
            0x2000..=0x3EFF => self.read_name_table_byte(ppu_internal_ram, address),
            0x3F00..=0x3FFF => self.read_palette_table_byte(palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn ppu_write(
        &mut self,
        internal_ram: &mut PpuInternalRam,
        address: PpuAddress,
        value: u8,
    ) {
        match address.to_u16() {
            0x0000..=0x1FFF => self.write_pattern_table_byte(address, value),
            0x2000..=0x3EFF => self.write_name_table_byte(internal_ram, address, value),
            0x3F00..=0x3FFF => self.write_palette_table_byte(
                &mut internal_ram.palette_ram,
                address,
                value,
            ),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn raw_name_table<'a>(
        &'a self,
        ppu_internal_ram: &'a PpuInternalRam,
        position: NameTableQuadrant,
    ) -> &'a [u8; KIBIBYTE] {
        let side = vram_side(position, self.name_table_mirroring());
        ppu_internal_ram.vram.side(side)
    }

    #[inline]
    fn raw_name_table_mut<'a>(
        &'a mut self,
        ppu_internal_ram: &'a mut PpuInternalRam,
        position: NameTableQuadrant,
    ) -> &'a mut [u8; KIBIBYTE] {
        let side = vram_side(position, self.name_table_mirroring());
        ppu_internal_ram.vram.side_mut(side)
    }

    #[inline]
    fn read_pattern_table_byte(&self, address: PpuAddress) -> u8 {
        self.chr_memory().read(address)
    }

    #[inline]
    fn write_pattern_table_byte(&mut self, address: PpuAddress, value: u8) {
        self.chr_memory_mut().write(address, value);
    }

    #[inline]
    fn read_name_table_byte(
        &self,
        ppu_internal_ram: &PpuInternalRam,
        address: PpuAddress,
    ) -> u8 {
        let (name_table_quadrant, index) = address_to_name_table_index(address);
        self.raw_name_table(ppu_internal_ram, name_table_quadrant)[index]
    }

    #[inline]
    fn write_name_table_byte(
        &mut self,
        ppu_internal_ram: &mut PpuInternalRam,
        address: PpuAddress,
        value: u8,
    ) {
        let (name_table_quadrant, index) = address_to_name_table_index(address);
        self.raw_name_table_mut(ppu_internal_ram, name_table_quadrant)[index] = value;
    }

    #[inline]
    fn read_palette_table_byte(
        &self,
        palette_ram: &PaletteRam,
        address: PpuAddress,
    ) -> u8 {
        palette_ram.read(address_to_palette_ram_index(address))
    }

    #[inline]
    fn write_palette_table_byte(
        &self,
        palette_ram: &mut PaletteRam,
        address: PpuAddress,
        value: u8,
    ) {
        palette_ram.write(address_to_palette_ram_index(address), value);
    }

    fn prg_rom_bank_string(&self) -> String {
        let indexes = self.prg_memory().selected_bank_indexes();
        let mut bank_text = indexes[0].to_string();
        for i in 1..indexes.len() {
            bank_text.push_str(&format!(", {}", indexes[i]));
        }

        bank_text.push_str(&format!(" ({} banks total)", self.prg_memory().bank_count()));

        bank_text
    }

    fn chr_rom_bank_string(&self) -> String {
        let indexes = self.chr_memory().selected_bank_indexes();
        let mut bank_text = indexes[0].to_string();
        for i in 1..indexes.len() {
            bank_text.push_str(&format!(", {}", indexes[i]));
        }

        bank_text.push_str(&format!(" ({} banks total)", self.prg_memory().bank_count()));

        bank_text
    }
}

pub fn validate_chr_data_length<P>(
    cartridge: &Cartridge,
    predicate: P,
) -> Result<(), String>
where P: Fn(usize) -> bool {
    let length = cartridge.chr_rom().len();
    if predicate(length) {
        Ok(())
    } else {
        Err(format!(
            "Mapper {} must not have a CHR ROM length of {}KiB.",
            cartridge.mapper_number(),
            8 * cartridge.chr_rom_chunks().len(),
        ))
    }
}

#[inline]
fn address_to_ppu_register_type(address: CpuAddress) -> RegisterType {
    FromPrimitive::from_usize(address.to_usize() % 8).unwrap()
}

#[inline]
#[rustfmt::skip]
fn address_to_name_table_index(address: PpuAddress) -> (NameTableQuadrant, usize) {
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

    let name_table_quadrant =
        NameTableQuadrant::from_usize(index / KIBIBYTE).unwrap();
    let index = index % KIBIBYTE;
    (name_table_quadrant, index)
}

fn address_to_palette_ram_index(address: PpuAddress) -> usize {
    const PALETTE_TABLE_START: usize = 0x3F00;
    const HIGH_ADDRESS_START: usize = 0x4000;

    let mut address = address.to_usize();
    assert!(address >= PALETTE_TABLE_START);
    assert!(address < HIGH_ADDRESS_START);

    // Mirror address down.
    address %= 0x20;
    if matches!(address, 0x10 | 0x14 | 0x18 | 0x1C) {
        address -= 0x10;
    }

    address
}

#[inline]
#[rustfmt::skip]
fn vram_side(
    name_table_quadrant: NameTableQuadrant,
    mirroring: NameTableMirroring,
) -> VramSide {

    use NameTableQuadrant::*;
    use NameTableMirroring::*;
    match (name_table_quadrant, mirroring) {
        (TopLeft    , _         ) => VramSide::Left,
        (TopRight   , Horizontal) => VramSide::Left,
        (BottomLeft , Horizontal) => VramSide::Right,
        (TopRight   , Vertical  ) => VramSide::Right,
        (BottomLeft , Vertical  ) => VramSide::Left,
        (BottomRight, _         ) => VramSide::Right,
        (_          , FourScreen) => todo!("FourScreen isn't supported yet."),
        (_          , OneScreenLeftBank) => VramSide::Left,
        (_          , OneScreenRightBank) => VramSide::Right,
    }
}
