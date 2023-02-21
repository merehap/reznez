pub use lazy_static::lazy_static;

pub use crate::cartridge::Cartridge;
pub use crate::memory::bank_index::{BankIndex, BankIndexRegisterId};
pub use crate::memory::bank_index::BankIndexRegisterId::*;
pub use crate::memory::cpu::cpu_address::CpuAddress;
pub use crate::memory::cpu::prg_memory::{PrgMemory, PrgLayout, PrgType};
pub use crate::memory::ppu::chr_memory::{ChrMemory, ChrLayout, ChrType};
pub use crate::memory::ppu::ppu_address::PpuAddress;
pub use crate::memory::writability::Writability::*;
pub use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
pub use crate::ppu::pattern_table::PatternTableSide;
pub use crate::util::unit::KIBIBYTE;

use num_traits::FromPrimitive;

use crate::apu::apu_registers::ApuRegisters;
use crate::memory::cpu::cpu_internal_ram::CpuInternalRam;
use crate::memory::cpu::ports::Ports;
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ppu_internal_ram::PpuInternalRam;
use crate::memory::ppu::vram::VramSide;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::register::register_type::RegisterType;

pub trait Mapper {
    fn name_table_mirroring(&self) -> NameTableMirroring;
    fn prg_memory(&self) -> &PrgMemory;
    fn chr_memory(&self) -> &ChrMemory;
    fn chr_memory_mut(&mut self) -> &mut ChrMemory;

    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8);

    // Most mappers don't care about the current PPU address.
    fn process_current_ppu_address(&mut self, _address: PpuAddress) {}
    // Most mappers don't trigger IRQs.
    fn irq_pending(&self) -> bool { false }

    #[inline]
    #[rustfmt::skip]
    fn cpu_read(
        &self,
        cpu_internal_ram: &CpuInternalRam,
        ppu_internal_ram: &PpuInternalRam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        apu_registers: &mut ApuRegisters,
        address: CpuAddress,
    ) -> Option<u8> {
        match address.to_raw() {
            0x0000..=0x07FF => Some(cpu_internal_ram[address.to_usize()]),
            0x0800..=0x1FFF => Some(cpu_internal_ram[address.to_usize() & 0x07FF]),
            0x2000..=0x3FFF => Some(match address.to_raw() & 0x2007 {
                0x2000 => ppu_registers.read(RegisterType::Ctrl),
                0x2001 => ppu_registers.read(RegisterType::Mask),
                0x2002 => ppu_registers.read(RegisterType::Status),
                0x2003 => ppu_registers.read(RegisterType::OamAddr),
                0x2004 => ppu_registers.read(RegisterType::OamData),
                0x2005 => ppu_registers.read(RegisterType::Scroll),
                0x2006 => ppu_registers.read(RegisterType::PpuAddr),
                0x2007 => {
                    ppu_registers.update_ppu_data(|ppu_address| self.ppu_read(ppu_internal_ram, ppu_address));
                    ppu_registers.read(RegisterType::PpuData)
                }
                _ => unreachable!(),
            }),
            0x4000..=0x4013 => { /* APU registers are write-only. */ None }
            0x4014          => { /* OAM DMA is write-only. TODO: Is open bus correct? */ None}
            0x4015          => Some(apu_registers.read_status().to_u8()),
            // TODO: Open bus https://www.nesdev.org/wiki/Controller_reading
            0x4016          => Some(ports.joypad1.borrow_mut().next_status() as u8),
            0x4017          => Some(ports.joypad2.borrow_mut().next_status() as u8),
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0x5FFF => {/* TODO: Low registers. */ None},
            0x6000..=0xFFFF => self.prg_memory().read(address),
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn cpu_write(
        &mut self,
        cpu_internal_ram: &mut CpuInternalRam,
        ppu_internal_ram: &mut PpuInternalRam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        apu_registers: &mut ApuRegisters,
        address: CpuAddress,
        value: u8,
    ) {
        match address.to_raw() {
            0x0000..=0x07FF => cpu_internal_ram[address.to_usize()] = value,
            0x0800..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF] = value,
            0x2000..=0x3FFF => match address.to_raw() & 0x2007 {
                0x2000 => ppu_registers.write(RegisterType::Ctrl, value),
                0x2001 => ppu_registers.write(RegisterType::Mask, value),
                0x2002 => ppu_registers.write(RegisterType::Status, value),
                0x2003 => ppu_registers.write(RegisterType::OamAddr, value),
                0x2004 => ppu_registers.write(RegisterType::OamData, value),
                0x2005 => ppu_registers.write(RegisterType::Scroll, value),
                0x2006 => ppu_registers.write(RegisterType::PpuAddr, value),
                0x2007 => {
                    self.ppu_write(ppu_internal_ram, ppu_registers.current_address(), value);
                    ppu_registers.write(RegisterType::PpuData, value);
                }
                _ => unreachable!(),
            }
            0x4000          => apu_registers.pulse_1.write_control_byte(value),
            0x4001          => apu_registers.pulse_1.write_sweep_byte(value),
            0x4002          => apu_registers.pulse_1.write_timer_low_byte(value),
            0x4003          => apu_registers.pulse_1.write_length_and_timer_high_byte(value),
            0x4004          => apu_registers.pulse_2.write_control_byte(value),
            0x4005          => apu_registers.pulse_2.write_sweep_byte(value),
            0x4006          => apu_registers.pulse_2.write_timer_low_byte(value),
            0x4007          => apu_registers.pulse_2.write_length_and_timer_high_byte(value),
            0x4008          => apu_registers.triangle.write_control_byte(value),
            0x4009          => { /* Unused. */ }
            0x400A          => apu_registers.triangle.write_timer_low_byte(value),
            0x400B          => apu_registers.triangle.write_length_and_timer_high_byte(value),
            0x400C          => apu_registers.noise.write_control_byte(value),
            0x400D          => { /* Unused. */ }
            0x400E          => apu_registers.noise.write_loop_and_period_byte(value),
            0x400F          => apu_registers.noise.write_length_byte(value),
            0x4010          => apu_registers.dmc.write_control_byte(value),
            0x4011          => apu_registers.dmc.write_load_counter(value),
            0x4012          => apu_registers.dmc.write_sample_address(value),
            0x4013          => apu_registers.dmc.write_sample_length(value),
            0x4014          => ports.dma.set_page(value),
            0x4015          => apu_registers.write_status_byte(value),
            0x4016          => ports.change_strobe(value),
            0x4017          => apu_registers.write_frame_counter(value),
            0x4018..=0x401F => todo!("CPU Test Mode not yet supported."),
            0x4020..=0xFFFF => self.write_to_cartridge_space(address, value),
        }
    }

    #[inline]
    fn ppu_read(&self, ppu_internal_ram: &PpuInternalRam, address: PpuAddress) -> u8 {
        let palette_ram = &ppu_internal_ram.palette_ram;
        match address.to_u16() {
            0x0000..=0x1FFF => self.chr_memory().read(address),
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
            0x0000..=0x1FFF => self.chr_memory_mut().write(address, value),
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
        quadrant: NameTableQuadrant,
    ) -> &'a [u8; KIBIBYTE] {
        let side = vram_side(quadrant, self.name_table_mirroring());
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
        let indexes = self.prg_memory().resolve_selected_bank_indexes();
        let mut bank_text = indexes[0].to_string();
        for i in 1..indexes.len() {
            bank_text.push_str(&format!(", {}", indexes[i]));
        }

        bank_text.push_str(&format!(" ({} banks total)", self.prg_memory().bank_count()));

        bank_text
    }

    fn chr_rom_bank_string(&self) -> String {
        let indexes = self.chr_memory().resolve_selected_bank_indexes();
        let mut bank_text = indexes[0].to_string();
        for i in 1..indexes.len() {
            bank_text.push_str(&format!(", {}", indexes[i]));
        }

        bank_text.push_str(&format!(" ({} banks total)", self.prg_memory().bank_count()));

        bank_text
    }
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
        (_          , FourScreen) => todo!("FourScreen isn't supported yet."),
        (_          , OneScreenLeftBank) => VramSide::Left,
        (_          , OneScreenRightBank) => VramSide::Right,
        (TopLeft    , _         ) => VramSide::Left,
        (TopRight   , Horizontal) => VramSide::Left,
        (BottomLeft , Horizontal) => VramSide::Right,
        (TopRight   , Vertical  ) => VramSide::Right,
        (BottomLeft , Vertical  ) => VramSide::Left,
        (BottomRight, _         ) => VramSide::Right,
    }
}
