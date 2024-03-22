pub use lazy_static::lazy_static;

pub use crate::cartridge::cartridge::Cartridge;
pub use crate::memory::bank_index::{BankIndex, BankIndexRegisterId, MetaRegisterId, BankIndexRegisters};
pub use crate::memory::bank_index::BankIndexRegisterId::*;
pub use crate::memory::bank_index::MetaRegisterId::*;
pub use crate::memory::cpu::cpu_address::CpuAddress;
pub use crate::memory::cpu::prg_memory::{PrgMemory, PrgLayout, PrgWindow, PrgBank};
pub use crate::memory::initial_layout::{InitialLayout, NameTableMirroringSource};
pub use crate::memory::ppu::chr_memory::{ChrMemory, ChrLayout, ChrWindow, ChrBank};
pub use crate::memory::ppu::ppu_address::PpuAddress;
pub use crate::memory::writability::Writability::*;
pub use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
pub use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
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
use crate::ppu::sprite::oam::Oam;

pub trait Mapper {
    // Should be const, but that's not yet allowed by Rust.
    fn initial_layout(&self) -> InitialLayout;

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8);

    // Most mappers don't override the default cartridge peeking/reading behavior.
    fn peek_from_cartridge_space(&self, params: &MapperParams, address: CpuAddress) -> Option<u8> {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => None,
            0x6000..=0xFFFF => params.peek_prg(address),
        }
    }

    fn read_from_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress) -> Option<u8> {
        self.peek_from_cartridge_space(params, address)
    }

    // Most mappers don't care about CPU cycles.
    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {}
    fn on_cpu_read(&mut self, _address: CpuAddress) {}
    fn on_cpu_write(&mut self, _params: &mut MapperParams, _address: CpuAddress, _value: u8) {}
    // Most mappers don't care about PPU cycles.
    fn on_end_of_ppu_cycle(&mut self) {}
    // Most mappers don't trigger anything based upon ppu reads.
    fn on_ppu_read(&mut self, _params: &mut MapperParams, _address: PpuAddress, _value: u8) {}
    // Most mappers don't care about the current PPU address.
    fn process_current_ppu_address(&mut self, _address: PpuAddress) {}
    // Most mappers don't trigger custom IRQs.
    fn irq_pending(&self) -> bool { false }

    #[allow(clippy::too_many_arguments)]
    fn cpu_peek(
        &self,
        params: &MapperParams,
        cpu_internal_ram: &CpuInternalRam,
        ppu_internal_ram: &PpuInternalRam,
        oam: &Oam,
        ports: &Ports,
        ppu_registers: &PpuRegisters,
        apu_registers: &ApuRegisters,
        address: CpuAddress,
    ) -> Option<u8> {
        match address.to_raw() {
            0x0000..=0x07FF => Some(cpu_internal_ram[address.to_usize()]),
            0x0800..=0x1FFF => Some(cpu_internal_ram[address.to_usize() & 0x07FF]),
            0x2000..=0x3FFF => {
                let peeker = |ppu_address| self.ppu_peek(params, ppu_internal_ram, ppu_address);
                Some(match address.to_raw() & 0x2007 {
                    0x2000 => ppu_registers.peek(RegisterType::Ctrl, peeker),
                    0x2001 => ppu_registers.peek(RegisterType::Mask, peeker),
                    0x2002 => ppu_registers.peek(RegisterType::Status, peeker),
                    0x2003 => ppu_registers.peek(RegisterType::OamAddr, peeker),
                    0x2004 => oam.peek(ppu_registers.oam_addr),
                    0x2005 => ppu_registers.peek(RegisterType::Scroll, peeker),
                    0x2006 => ppu_registers.peek(RegisterType::PpuAddr, peeker),
                    0x2007 => ppu_registers.peek(RegisterType::PpuAddr, peeker),
                    _ => unreachable!(),
                })
            }
            0x4000..=0x4013 => { /* APU registers are write-only. */ None }
            0x4014          => { /* OAM DMA is write-only. TODO: Is open bus correct? */ None}
            0x4015          => Some(apu_registers.peek_status().to_u8()),
            // TODO: Open bus https://www.nesdev.org/wiki/Controller_reading
            0x4016          => Some(ports.joypad1.borrow().peek_status() as u8),
            0x4017          => Some(ports.joypad2.borrow().peek_status() as u8),
            0x4018..=0x401F => /* CPU Test Mode not yet supported. */ Some(0),
            0x4020..=0xFFFF => self.peek_from_cartridge_space(params, address),
        }
    }

    #[inline]
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    fn cpu_read(
        &mut self,
        params: &mut MapperParams,
        cpu_internal_ram: &CpuInternalRam,
        ppu_internal_ram: &PpuInternalRam,
        oam: &Oam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        apu_registers: &mut ApuRegisters,
        address: CpuAddress,
    ) -> Option<u8> {
        self.on_cpu_read(address);
        match address.to_raw() {
            0x0000..=0x07FF => Some(cpu_internal_ram[address.to_usize()]),
            0x0800..=0x1FFF => Some(cpu_internal_ram[address.to_usize() & 0x07FF]),
            0x2000..=0x3FFF => {
                let reader = |ppu_address| self.ppu_read(params, ppu_internal_ram, ppu_address, false);
                Some(match address.to_raw() & 0x2007 {
                    0x2000 => ppu_registers.read(RegisterType::Ctrl, reader),
                    0x2001 => ppu_registers.read(RegisterType::Mask, reader),
                    0x2002 => ppu_registers.read(RegisterType::Status, reader),
                    0x2003 => ppu_registers.read(RegisterType::OamAddr, reader),
                    0x2004 => {
                        let value = oam.peek(ppu_registers.oam_addr);
                        ppu_registers.ppu_io_bus.update_from_read(RegisterType::OamData, value);
                        value
                    }
                    0x2005 => ppu_registers.read(RegisterType::Scroll, reader),
                    0x2006 => ppu_registers.read(RegisterType::PpuAddr, reader),
                    0x2007 => ppu_registers.read(RegisterType::PpuData, reader),
                    _ => unreachable!(),
                })
            }
            0x4000..=0x4013 => { /* APU registers are write-only. */ None }
            0x4014          => { /* OAM DMA is write-only. TODO: Is open bus correct? */ None}
            0x4015          => Some(apu_registers.read_status().to_u8()),
            // TODO: Open bus https://www.nesdev.org/wiki/Controller_reading
            0x4016          => Some(ports.joypad1.borrow_mut().read_status() as u8),
            0x4017          => Some(ports.joypad2.borrow_mut().read_status() as u8),
            0x4018..=0x401F => /* CPU Test Mode not yet supported. */ Some(0),
            0x4020..=0xFFFF => self.read_from_cartridge_space(params, address),
        }
    }

    #[inline]
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    fn cpu_write(
        &mut self,
        params: &mut MapperParams,
        cpu_internal_ram: &mut CpuInternalRam,
        ppu_internal_ram: &mut PpuInternalRam,
        oam: &mut Oam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        apu_registers: &mut ApuRegisters,
        address: CpuAddress,
        value: u8,
    ) {
        self.on_cpu_write(params, address, value);
        match address.to_raw() {
            0x0000..=0x07FF => cpu_internal_ram[address.to_usize()] = value,
            0x0800..=0x1FFF => cpu_internal_ram[address.to_usize() & 0x07FF] = value,
            0x2000..=0x3FFF => match address.to_raw() & 0x2007 {
                0x2000 => ppu_registers.write(RegisterType::Ctrl, value),
                0x2001 => ppu_registers.write(RegisterType::Mask, value),
                0x2002 => ppu_registers.write(RegisterType::Status, value),
                0x2003 => ppu_registers.write(RegisterType::OamAddr, value),
                0x2004 => {
                    oam.write(ppu_registers.oam_addr, value);
                    ppu_registers.write(RegisterType::OamData, value);
                }
                0x2005 => ppu_registers.write(RegisterType::Scroll, value),
                0x2006 => ppu_registers.write(RegisterType::PpuAddr, value),
                0x2007 => {
                    self.ppu_write(params, ppu_internal_ram, ppu_registers.current_address(), value);
                    ppu_registers.write(RegisterType::PpuData, value);
                    self.process_current_ppu_address(ppu_registers.current_address());
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
            0x4011          => apu_registers.dmc.write_volume(value),
            0x4012          => apu_registers.dmc.write_sample_start_address(value),
            0x4013          => apu_registers.dmc.write_sample_length(value),
            0x4014          => ports.oam_dma.set_page(value),
            0x4015          => apu_registers.write_status_byte(value),
            0x4016          => ports.change_strobe(value),
            0x4017          => apu_registers.write_frame_counter(value),
            0x4018..=0x401F => /* CPU Test Mode not yet supported. */ {}
            0x4020..=0xFFFF => self.write_to_cartridge_space(params, address, value),
        }
    }

    fn ppu_peek(
        &self,
        params: &MapperParams,
        ppu_internal_ram: &PpuInternalRam,
        address: PpuAddress,
    ) -> u8 {
        let palette_ram = &ppu_internal_ram.palette_ram;
        match address.to_u16() {
            0x0000..=0x1FFF => params.peek_chr(address),
            0x2000..=0x3EFF => self.peek_name_table_byte(params.name_table_mirroring(), ppu_internal_ram, address),
            0x3F00..=0x3FFF => self.peek_palette_table_byte(palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn ppu_read(
        &mut self,
        params: &mut MapperParams,
        ppu_internal_ram: &PpuInternalRam,
        address: PpuAddress,
        rendering: bool,
    ) -> u8 {
        if rendering {
            self.process_current_ppu_address(address);
        }

        let value = self.ppu_peek(params, ppu_internal_ram, address);
        self.on_ppu_read(params, address, value);
        value
    }

    #[inline]
    fn ppu_write(
        &mut self,
        params: &mut MapperParams,
        internal_ram: &mut PpuInternalRam,
        address: PpuAddress,
        value: u8,
    ) {
        match address.to_u16() {
            0x0000..=0x1FFF => params.write_chr(address, value),
            0x2000..=0x3EFF => self.write_name_table_byte(params.name_table_mirroring(), internal_ram, address, value),
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
        name_table_mirroring: NameTableMirroring,
        ppu_internal_ram: &'a PpuInternalRam,
        quadrant: NameTableQuadrant,
    ) -> &'a [u8; KIBIBYTE] {
        let side = vram_side(quadrant, name_table_mirroring);
        ppu_internal_ram.vram.side(side)
    }

    #[inline]
    fn raw_name_table_mut<'a>(
        &'a mut self,
        name_table_mirroring: NameTableMirroring,
        ppu_internal_ram: &'a mut PpuInternalRam,
        position: NameTableQuadrant,
    ) -> &'a mut [u8; KIBIBYTE] {
        let side = vram_side(position, name_table_mirroring);
        ppu_internal_ram.vram.side_mut(side)
    }

    #[inline]
    fn peek_name_table_byte(
        &self,
        name_table_mirroring: NameTableMirroring,
        ppu_internal_ram: &PpuInternalRam,
        address: PpuAddress,
    ) -> u8 {
        let (name_table_quadrant, index) = address_to_name_table_index(address);
        self.raw_name_table(name_table_mirroring, ppu_internal_ram, name_table_quadrant)[index]
    }

    #[inline]
    fn write_name_table_byte(
        &mut self,
        name_table_mirroring: NameTableMirroring,
        ppu_internal_ram: &mut PpuInternalRam,
        address: PpuAddress,
        value: u8,
    ) {
        let (name_table_quadrant, index) = address_to_name_table_index(address);
        self.raw_name_table_mut(name_table_mirroring, ppu_internal_ram, name_table_quadrant)[index] = value;
    }

    #[inline]
    fn peek_palette_table_byte(
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

    fn prg_rom_bank_string(&self, params: &MapperParams) -> String {
        let indexes = params.resolve_prg_bank_indexes();
        let mut bank_text = indexes[0].to_string();
        for index in indexes.iter().skip(1) {
            bank_text.push_str(&format!(", {index}"));
        }

        bank_text.push_str(&format!(" ({} banks total)", params.prg_memory().bank_count()));

        bank_text
    }

    fn chr_rom_bank_string(&self, params: &MapperParams) -> String {
        let indexes = params.resolve_chr_bank_indexes();
        let mut bank_text = indexes[0].to_string();
        for index in indexes.iter().skip(1) {
            bank_text.push_str(&format!(", {index}"));
        }

        bank_text.push_str(&format!(" ({} banks total)", params.prg_memory().bank_count()));

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

pub struct MapperParams {
    pub prg_memory: PrgMemory,
    pub chr_memory: ChrMemory,
    pub bank_index_registers: BankIndexRegisters,
    pub name_table_mirroring: NameTableMirroring,
}

impl MapperParams {
    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    pub fn set_name_table_mirroring(&mut self, name_table_mirroring: NameTableMirroring) {
        self.name_table_mirroring = name_table_mirroring;
    }

    pub fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    pub fn set_prg_layout(&mut self, layout: PrgLayout) {
        self.prg_memory.set_layout(layout);
    }

    pub fn resolve_prg_bank_indexes(&self) -> Vec<u16> {
        self.prg_memory.resolve_selected_bank_indexes(&self.bank_index_registers)
    }

    pub fn peek_prg(&self, address: CpuAddress) -> Option<u8> {
        self.prg_memory.peek(&self.bank_index_registers, address)
    }

    pub fn write_prg(&mut self, address: CpuAddress, value: u8) {
        self.prg_memory.write(&self.bank_index_registers, address, value);
    }

    pub fn enable_work_ram(&mut self, address: u16) {
        self.prg_memory.enable_work_ram(address);
    }

    pub fn disable_work_ram(&mut self, address: u16) {
        self.prg_memory.disable_work_ram(address);
    }

    pub fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    pub fn resolve_chr_bank_indexes(&self) -> Vec<u16> {
        self.chr_memory.resolve_selected_bank_indexes(&self.bank_index_registers)
    }

    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        self.chr_memory.pattern_table(&self.bank_index_registers, side)
    }

    pub fn set_chr_layout(&mut self, layout: ChrLayout) {
        self.chr_memory.set_layout(layout);
    }

    pub fn peek_chr(&self, address: PpuAddress) -> u8 {
        self.chr_memory.peek(&self.bank_index_registers, address)
    }

    pub fn write_chr(&mut self, address: PpuAddress, value: u8) {
        self.chr_memory.write(&self.bank_index_registers, address, value);
    }

    pub fn set_bank_index_register<INDEX: Into<u16>>(
        &mut self,
        id: BankIndexRegisterId,
        value: INDEX,
    ) {
        self.bank_index_registers.set(id, BankIndex::from_u16(value.into()));
    }

    pub fn set_bank_index_register_bits(
        &mut self, id: BankIndexRegisterId, new_value: u16, mask: u16) {

        self.bank_index_registers.set_bits(id, new_value, mask);
    }

    pub fn set_meta_register(&mut self, id: MetaRegisterId, value: BankIndexRegisterId) {
        self.bank_index_registers.set_meta(id, value);
    }

    pub fn update_bank_index_register(
        &mut self,
        id: BankIndexRegisterId,
        updater: &dyn Fn(u16) -> u16,
    ) {
        self.bank_index_registers.update(id, updater);
    }
}
