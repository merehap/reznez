use std::collections::BTreeSet;

pub use splitbits::{splitbits, splitbits_named, combinebits, splitbits_then_combine};

pub use crate::cartridge::cartridge::Cartridge;
use crate::cpu::dmc_dma::DmcDma;
use crate::cpu::oam_dma::OamDma;
pub use crate::memory::bank::bank_index::{BankIndex, BankRegisterId, MetaRegisterId, BankRegisters, ReadWriteStatus};
pub use crate::memory::bank::bank_index::BankRegisterId::*;
pub use crate::memory::bank::bank_index::MetaRegisterId::*;
pub use crate::memory::bank::bank::{Bank, ReadWriteStatusRegisterId};
pub use crate::memory::bank::bank::ReadWriteStatusRegisterId::*;
pub use crate::memory::cpu::cpu_address::CpuAddress;
pub use crate::memory::cpu::prg_memory::PrgMemory;
pub use crate::memory::layout::Layout;
pub use crate::memory::ppu::chr_memory::ChrMemory;
use crate::memory::ppu::chr_memory::ChrPageId;
pub use crate::memory::ppu::ppu_address::PpuAddress;
pub use crate::memory::read_result::ReadResult;
pub use crate::memory::ppu::ciram::CiramSide;
pub use crate::memory::window::Window;
pub use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
pub use crate::ppu::name_table::name_table_mirroring::{NameTableMirroring, NameTableSource};
pub use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
pub use crate::util::unit::KIBIBYTE;

use log::info;
use num_traits::FromPrimitive;

use crate::apu::apu_registers::ApuRegisters;
use crate::memory::cpu::cpu_internal_ram::CpuInternalRam;
use crate::memory::cpu::ports::Ports;
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ciram::Ciram;
use crate::ppu::register::ppu_registers::{PpuRegisters, WriteToggle};
use crate::ppu::sprite::oam::Oam;

use crate::memory::bank::bank::RomRamModeRegisterId;
use crate::memory::bank::bank_index::RomRamMode;

pub trait Mapper {
    // Should be const, but that's not yet allowed by Rust.
    fn layout(&self) -> Layout;

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8);

    // Most mappers don't override the default cartridge peeking/reading behavior.
    fn peek_cartridge_space(&self, params: &MapperParams, cpu_address: u16) -> ReadResult {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => params.peek_prg(cpu_address),
        }
    }

    fn read_from_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16) -> ReadResult {
        self.peek_cartridge_space(params, cpu_address)
    }

    // Most mappers don't need to modify the MapperParams before ROM execution begins, but this
    // provides a relief valve for the rare settings that can't be expressed in a Layout.
    fn init_mapper_params(&self, _params: &mut MapperParams) {}
    // Most mappers don't care about CPU cycles.
    fn on_end_of_cpu_cycle(&mut self, _params: &mut MapperParams, _cycle: i64) {}
    fn on_cpu_read(&mut self, _params: &mut MapperParams, _address: CpuAddress) {}
    fn on_cpu_write(&mut self, _params: &mut MapperParams, _address: CpuAddress, _value: u8) {}
    // Most mappers don't care about PPU cycles.
    fn on_end_of_ppu_cycle(&mut self) {}
    // Most mappers don't trigger anything based upon ppu reads.
    fn on_ppu_read(&mut self, _params: &mut MapperParams, _address: PpuAddress, _value: u8) {}
    // Most mappers don't care about changes to the current PPU address.
    fn on_ppu_address_change(&mut self, _params: &mut MapperParams, _address: PpuAddress) {}
    // Most mappers don't have bus conflicts.
    fn has_bus_conflicts(&self) -> HasBusConflicts { HasBusConflicts::No }
    // Most mappers don't use a fill-mode name table.
    fn fill_mode_name_table(&self) -> &[u8; KIBIBYTE as usize] { unimplemented!() }

    #[allow(clippy::too_many_arguments)]
    fn cpu_peek(
        &self,
        params: &MapperParams,
        cpu_internal_ram: &CpuInternalRam,
        ciram: &Ciram,
        palette_ram: &PaletteRam,
        oam: &Oam,
        ports: &Ports,
        ppu_registers: &PpuRegisters,
        apu_registers: &ApuRegisters,
        address: CpuAddress,
    ) -> ReadResult {
        match address.to_raw() {
            0x0000..=0x07FF => ReadResult::full(cpu_internal_ram[address.to_usize()]),
            0x0800..=0x1FFF => ReadResult::full(cpu_internal_ram[address.to_usize() & 0x07FF]),
            0x2000..=0x3FFF => {
                ReadResult::full(match address.to_raw() & 0x2007 {
                    0x2000 => ppu_registers.peek_ppu_io_bus(),
                    0x2001 => ppu_registers.peek_ppu_io_bus(),
                    0x2002 => ppu_registers.peek_status(),
                    0x2003 => ppu_registers.peek_ppu_io_bus(),
                    0x2004 => ppu_registers.peek_oam_data(oam),
                    0x2005 => ppu_registers.peek_ppu_io_bus(),
                    0x2006 => ppu_registers.peek_ppu_io_bus(),
                    0x2007 => {
                        let peeker = |ppu_address| self.ppu_peek(params, ciram, palette_ram, ppu_address);
                        ppu_registers.peek_ppu_data(peeker)
                    }
                    _ => unreachable!(),
                })
            }
            0x4000..=0x4013 => { /* APU registers are write-only. */ ReadResult::OPEN_BUS }
            0x4014          => { /* OAM DMA is write-only. */ ReadResult::OPEN_BUS }
            0x4015          => ReadResult::full(apu_registers.peek_status().to_u8()),
            // TODO: Move ReadResult/mask specification into the controller.
            0x4016          => ReadResult::partial_open_bus(ports.joypad1.peek_status() as u8, 0b0000_0001),
            0x4017          => ReadResult::partial_open_bus(ports.joypad2.peek_status() as u8, 0b0000_0001),
            0x4018..=0x401F => /* CPU Test Mode not yet supported. */ ReadResult::OPEN_BUS,
            0x4020..=0xFFFF => self.peek_cartridge_space(params, address.to_raw()),
        }
    }

    #[inline]
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    fn cpu_read(
        &mut self,
        params: &mut MapperParams,
        cpu_internal_ram: &CpuInternalRam,
        ciram: &Ciram,
        palette_ram: &PaletteRam,
        oam: &Oam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        apu_registers: &mut ApuRegisters,
        address: CpuAddress,
    ) -> ReadResult {
        self.on_cpu_read(params, address);
        match address.to_raw() {
            0x0000..=0x07FF => ReadResult::full(cpu_internal_ram[address.to_usize()]),
            0x0800..=0x1FFF => ReadResult::full(cpu_internal_ram[address.to_usize() & 0x07FF]),
            0x2000..=0x3FFF => {
                ReadResult::full(match address.to_raw() & 0x2007 {
                    0x2000 => ppu_registers.peek_ppu_io_bus(),
                    0x2001 => ppu_registers.peek_ppu_io_bus(),
                    0x2002 => ppu_registers.read_status(),
                    0x2003 => ppu_registers.peek_ppu_io_bus(),
                    0x2004 => ppu_registers.read_oam_data(oam),
                    0x2005 => ppu_registers.peek_ppu_io_bus(),
                    0x2006 => ppu_registers.peek_ppu_io_bus(),
                    0x2007 => {
                        let reader = |ppu_address| self.ppu_read(params, ciram, palette_ram, ppu_address, false);
                        let data = ppu_registers.read_ppu_data(reader);
                        self.on_ppu_address_change(params, ppu_registers.current_address());
                        data
                    }
                    _ => unreachable!(),
                })
            }
            0x4000..=0x4013 => { /* APU registers are write-only. */ ReadResult::OPEN_BUS }
            0x4014          => { /* OAM DMA is write-only. */ ReadResult::OPEN_BUS }
            0x4015          => ReadResult::full(apu_registers.read_status().to_u8()),
            // TODO: Move ReadResult/mask specification into the controller.
            0x4016          => ReadResult::partial_open_bus(ports.joypad1.read_status() as u8, 0b0000_0001),
            0x4017          => ReadResult::partial_open_bus(ports.joypad2.read_status() as u8, 0b0000_0001),
            0x4018..=0x401F => /* CPU Test Mode not yet supported. */ ReadResult::OPEN_BUS,
            0x4020..=0xFFFF => self.read_from_cartridge_space(params, address.to_raw()),
        }
    }

    #[inline]
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    fn cpu_write(
        &mut self,
        params: &mut MapperParams,
        cpu_internal_ram: &mut CpuInternalRam,
        ciram: &mut Ciram,
        palette_ram: &mut PaletteRam,
        dmc_dma: &mut DmcDma,
        oam: &mut Oam,
        oam_dma: &mut OamDma,
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
                0x2000 => ppu_registers.write_ctrl(value),
                0x2001 => ppu_registers.write_mask(value),
                0x2002 => ppu_registers.write_ppu_io_bus(value),
                0x2003 => ppu_registers.write_oam_addr(value),
                0x2004 => ppu_registers.write_oam_data(oam, value),
                0x2005 => ppu_registers.write_scroll(value),
                0x2006 => {
                    ppu_registers.write_ppu_addr(value);
                    if ppu_registers.write_toggle() == WriteToggle::FirstByte {
                        self.on_ppu_address_change(params, ppu_registers.current_address());
                    }
                }
                0x2007 => {
                    self.ppu_write(params, ciram, palette_ram, ppu_registers.current_address(), value);
                    ppu_registers.write_ppu_data(value);
                    self.on_ppu_address_change(params, ppu_registers.current_address());
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
            0x4014          => oam_dma.prepare_to_start(value),
            0x4015          => apu_registers.write_status_byte(dmc_dma, value),
            0x4016          => ports.change_strobe(value),
            0x4017          => apu_registers.write_frame_counter(value),
            0x4018..=0x401F => { /* CPU Test Mode not yet supported. */ }
            0x4020..=0xFFFF => {
                let value = if self.has_bus_conflicts() == HasBusConflicts::Yes {
                    let rom_value = self.cpu_peek(params, cpu_internal_ram, ciram, palette_ram, oam,
                        ports, ppu_registers, apu_registers, address);
                    rom_value.bus_conflict(value)
                } else {
                    value
                };

                params.prg_memory.write(&params.bank_registers, address, value);
                self.write_to_cartridge_space(params, address.to_raw(), value);
            }
        }
    }

    fn ppu_peek(
        &self,
        params: &MapperParams,
        ciram: &Ciram,
        palette_ram: &PaletteRam,
        address: PpuAddress,
    ) -> u8 {
        match address.to_u16() {
            0x0000..=0x1FFF => params.peek_chr(ciram, address),
            0x2000..=0x3EFF => self.peek_name_table_byte(params, ciram, address),
            0x3F00..=0x3FFF => self.peek_palette_table_byte(palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn ppu_read(
        &mut self,
        params: &mut MapperParams,
        ciram: &Ciram,
        palette_ram: &PaletteRam,
        address: PpuAddress,
        rendering: bool,
    ) -> u8 {
        if rendering {
            self.on_ppu_address_change(params, address);
        }

        let value = self.ppu_peek(params, ciram, palette_ram, address);
        self.on_ppu_read(params, address, value);
        value
    }

    #[inline]
    fn ppu_write(
        &mut self,
        params: &mut MapperParams,
        ciram: &mut Ciram,
        palette_ram: &mut PaletteRam,
        address: PpuAddress,
        value: u8,
    ) {
        match address.to_u16() {
            0x0000..=0x1FFF => params.write_chr(ciram, address, value),
            0x2000..=0x3EFF => self.write_name_table_byte(params, ciram, address, value),
            0x3F00..=0x3FFF => self.write_palette_table_byte(palette_ram, address, value),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn raw_name_table<'a>(
        &'a self,
        params: &'a MapperParams,
        ciram: &'a Ciram,
        quadrant: NameTableQuadrant,
    ) -> &'a [u8; KIBIBYTE as usize] {
        match params.name_table_mirroring.name_table_source_in_quadrant(quadrant) {
            NameTableSource::Ciram(side) => ciram.side(side),
            NameTableSource::SaveRam(start_index) => params.chr_memory.save_ram_1kib_page(start_index),
            NameTableSource::ExtendedRam => params.prg_memory.extended_ram().as_raw_slice().try_into().unwrap(),
            NameTableSource::FillModeTile => self.fill_mode_name_table(),
        }
    }

    #[inline]
    fn peek_name_table_byte(
        &self,
        params: &MapperParams,
        ciram: &Ciram,
        address: PpuAddress,
    ) -> u8 {
        let (name_table_quadrant, index) = address_to_name_table_index(address);
        self.raw_name_table(params, ciram, name_table_quadrant)[index as usize]
    }

    #[inline]
    fn write_name_table_byte(
        &mut self,
        params: &mut MapperParams,
        ciram: &mut Ciram,
        address: PpuAddress,
        value: u8,
    ) {
        let (quadrant, index) = address_to_name_table_index(address);
        match params.name_table_mirroring.name_table_source_in_quadrant(quadrant) {
            NameTableSource::Ciram(side) =>
                ciram.side_mut(side)[index as usize] = value,
            NameTableSource::SaveRam(start_index) =>
                params.chr_memory.save_ram_1kib_page_mut(start_index)[index as usize] = value,
            NameTableSource::ExtendedRam =>
                params.prg_memory.extended_ram_mut().as_raw_mut_slice()[index as usize] = value,
            NameTableSource::FillModeTile =>
                { /* The fill mode tile can't be overwritten through normal memory writes. */ }
        }
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
        let prg_memory = &params.prg_memory();

        let mut result = String::new();
        for window in prg_memory.current_layout().windows() {
            let bank_string = window.bank_string(
                &params.bank_registers,
                Some(prg_memory.bank_configuration()),
                prg_memory.work_ram_bank_configuration(),
                None,
            );
            let window_size = window.size().get() / KIBIBYTE as u16;

            let left_padding_len;
            let right_padding_len;
            if window_size < 8 {
                left_padding_len = 0;
                right_padding_len = 0;
            } else {
                let padding_size = window_size - 2 - u16::try_from(bank_string.len()).unwrap();
                left_padding_len = padding_size / 2;
                right_padding_len = padding_size - left_padding_len;
            }

            let left_padding = " ".repeat(left_padding_len as usize);
            let right_padding = " ".repeat(right_padding_len as usize);

            let segment = format!("|{left_padding}{bank_string}{right_padding}|");
            result.push_str(&segment);
        }

        result
    }

    fn chr_rom_bank_string(&self, params: &MapperParams) -> String {
        let chr_memory = &params.chr_memory();

        let mut result = String::new();
        for (page_id, _) in chr_memory.current_memory_map().pattern_table_page_ids() {
            let bank_string = match page_id {
                ChrPageId::Rom(page_number) => page_number.to_string(),
                ChrPageId::Ram(page_number) => format!("W{page_number}"),
                ChrPageId::Ciram(side) => format!("C{side:?}"),
                ChrPageId::SaveRam => "S".to_owned(),
                ChrPageId::ExtendedRam => "X".to_owned(),
                ChrPageId::FillModeTile => "F".to_owned(),
            };

            let window_size = 1;

            let padding_size = 5 * window_size - 2 - u16::try_from(bank_string.len()).unwrap();
            assert!(padding_size < 100);
            let left_padding_len = padding_size / 2;
            let right_padding_len = padding_size - left_padding_len;

            let left_padding = " ".repeat(left_padding_len as usize);
            let right_padding = " ".repeat(right_padding_len as usize);

            let segment = format!("|{left_padding}{bank_string}{right_padding}|");
            result.push_str(&segment);
        }

        result
    }

    fn supported(self) -> LookupResult where Self: Sized, Self: 'static {
        LookupResult::Supported(Box::new(self))
    }
}

#[inline]
#[rustfmt::skip]
fn address_to_name_table_index(address: PpuAddress) -> (NameTableQuadrant, u32) {
    const NAME_TABLE_START:    u32 = 0x2000;
    const MIRROR_START:        u32 = 0x3000;
    const PALETTE_TABLE_START: u32 = 0x3F00;

    let address = address.to_u32();
    assert!(address >= NAME_TABLE_START);
    assert!(address < PALETTE_TABLE_START);

    let mut index = address;
    if index >= MIRROR_START {
        index -= 0x1000;
    }

    let index = index - NAME_TABLE_START;

    let name_table_quadrant =
        NameTableQuadrant::from_u32(index / KIBIBYTE).unwrap();
    let index = index % KIBIBYTE;
    (name_table_quadrant, index)
}

fn address_to_palette_ram_index(address: PpuAddress) -> u32 {
    const PALETTE_TABLE_START: u32 = 0x3F00;
    const HIGH_ADDRESS_START: u32 = 0x4000;

    let mut address = address.to_u32();
    assert!(address >= PALETTE_TABLE_START);
    assert!(address < HIGH_ADDRESS_START);

    // Mirror address down.
    address %= 0x20;
    if matches!(address, 0x10 | 0x14 | 0x18 | 0x1C) {
        address -= 0x10;
    }

    address
}

pub struct MapperParams {
    pub prg_memory: PrgMemory,
    pub chr_memory: ChrMemory,
    pub bank_registers: BankRegisters,
    // TODO: Consolidate these into ChrMemory?
    pub name_table_mirroring: NameTableMirroring,
    pub name_table_mirrorings: &'static [NameTableMirroring],
    pub read_write_statuses: &'static [ReadWriteStatus],
    pub ram_not_present: BTreeSet<ReadWriteStatusRegisterId>,
    pub irq_pending: bool,
}

impl MapperParams {
    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    pub fn set_name_table_mirroring(&mut self, mirroring_index: u8) {
        self.name_table_mirroring = self.name_table_mirrorings[usize::from(mirroring_index)];
        self.chr_memory.set_name_table_mirroring(self.name_table_mirroring, &self.bank_registers);
    }

    pub fn set_name_table_quadrant(&mut self, quadrant: NameTableQuadrant, ciram_side: CiramSide) {
        self.name_table_mirroring.set_quadrant(quadrant, ciram_side);
        self.chr_memory.set_name_table_mirroring(self.name_table_mirroring, &self.bank_registers);
    }

    pub fn set_name_table_quadrant_to_source(&mut self, quadrant: NameTableQuadrant, source: NameTableSource) {
        self.name_table_mirroring.set_quadrant_to_source(quadrant, source);
        self.chr_memory.set_name_table_mirroring(self.name_table_mirroring, &self.bank_registers);
    }

    pub fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    pub fn set_prg_layout(&mut self, index: u8) {
        self.prg_memory.set_layout(index);
    }

    pub fn set_prg_rom_outer_bank_index(&mut self, index: u8) {
        self.prg_memory.set_prg_rom_outer_bank_index(index);
    }

    pub fn peek_prg(&self, cpu_address: u16) -> ReadResult {
        self.prg_memory.peek(&self.bank_registers, CpuAddress::new(cpu_address))
    }

    pub fn write_prg(&mut self, cpu_address: u16, value: u8) {
        self.prg_memory.write(&self.bank_registers, CpuAddress::new(cpu_address), value);
    }

    pub fn set_read_write_status(&mut self, id: ReadWriteStatusRegisterId, index: u8) {
        if self.ram_not_present.contains(&id) {
            info!(target: "mapperupdates",
                "Ignoring update to RamStatus register {id:?} because RAM isn't present.");
        } else {
            let read_write_status = self.read_write_statuses[index as usize];
            self.bank_registers.set_read_write_status(id, read_write_status);
            self.chr_memory.update_page_ids(&self.bank_registers);
        }
    }

    pub fn set_rom_ram_mode(&mut self, id: RomRamModeRegisterId, rom_ram_mode: RomRamMode) {
        self.bank_registers.set_rom_ram_mode(id, rom_ram_mode);
        self.chr_memory.update_page_ids(&self.bank_registers);
    }

    pub fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    pub fn pattern_table<'a>(&'a self, ciram: &'a Ciram, side: PatternTableSide) -> PatternTable<'a> {
        self.chr_memory.pattern_table(ciram, side)
    }

    pub fn set_chr_layout(&mut self, index: u8) {
        self.chr_memory.set_layout(index);
    }

    pub fn peek_chr(&self, ciram: &Ciram, address: PpuAddress) -> u8 {
        self.chr_memory.peek(ciram, address)
    }

    pub fn write_chr(&mut self, ciram: &mut Ciram, address: PpuAddress, value: u8) {
        self.chr_memory.write(ciram, address, value);
    }

    pub fn set_chr_rom_outer_bank_index(&mut self, index: u8) {
        self.chr_memory.set_chr_rom_outer_bank_index(index);
        self.chr_memory.update_page_ids(&self.bank_registers);
    }

    pub fn set_bank_register<INDEX: Into<u16>>(&mut self, id: BankRegisterId, value: INDEX) {
        self.bank_registers.set(id, BankIndex::from_u16(value.into()));
        self.chr_memory.update_page_ids(&self.bank_registers);
    }

    pub fn set_bank_register_bits(&mut self, id: BankRegisterId, new_value: u16, mask: u16) {
        self.bank_registers.set_bits(id, new_value, mask);
        self.chr_memory.update_page_ids(&self.bank_registers);
    }

    pub fn set_meta_register(&mut self, id: MetaRegisterId, value: BankRegisterId) {
        self.bank_registers.set_meta(id, value);
        self.chr_memory.update_page_ids(&self.bank_registers);
    }

    pub fn update_bank_register(
        &mut self,
        id: BankRegisterId,
        updater: &dyn Fn(u16) -> u16,
    ) {
        self.bank_registers.update(id, updater);
        self.chr_memory.update_page_ids(&self.bank_registers);
    }

    pub fn set_bank_register_to_ciram_side(
        &mut self,
        id: BankRegisterId,
        ciram_side: CiramSide,
    ) {
        self.bank_registers.set_to_ciram_side(id, ciram_side);
        self.chr_memory.update_page_ids(&self.bank_registers);
    }

    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    pub fn set_irq_pending(&mut self, pending: bool) {
        self.irq_pending = pending;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum HasBusConflicts {
    Yes,
    No,
}

// This should be in mapper_list.rs instead, but we can't write the supported() method there.
pub enum LookupResult {
    Supported(Box<dyn Mapper>),
    UnassignedMapper,
    UnassignedSubmapper,
    TodoMapper,
    TodoSubmapper,
    UnspecifiedSubmapper,
    ReassignedSubmapper {correct_mapper: u16, correct_submapper: u8 },
}
