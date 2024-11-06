pub use splitbits::{splitbits, splitbits_named, combinebits, splitbits_then_combine};

pub use crate::cartridge::cartridge::Cartridge;
pub use crate::memory::bank::bank_index::{BankIndex, BankRegisterId, MetaRegisterId, BankRegisters, RamStatus};
pub use crate::memory::bank::bank_index::BankRegisterId::*;
pub use crate::memory::bank::bank_index::MetaRegisterId::*;
pub use crate::memory::bank::bank::{Bank, RamStatusRegisterId};
pub use crate::memory::bank::bank::RamStatusRegisterId::*;
pub use crate::memory::cpu::cpu_address::CpuAddress;
pub use crate::memory::cpu::prg_memory::PrgMemory;
pub use crate::memory::layout::Layout;
pub use crate::memory::ppu::chr_memory::ChrMemory;
pub use crate::memory::ppu::ppu_address::PpuAddress;
pub use crate::memory::read_result::ReadResult;
pub use crate::memory::window::Window;
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
use crate::ppu::register::ppu_registers::{PpuRegisters, WriteToggle};
use crate::ppu::sprite::oam::Oam;

pub trait Mapper {
    // Should be const, but that's not yet allowed by Rust.
    fn layout(&self) -> Layout;

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8);

    // Most mappers don't override the default cartridge peeking/reading behavior.
    fn peek_cartridge_space(&self, params: &MapperParams, cpu_address: u16) -> ReadResult {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => ReadResult::OPEN_BUS,
            0x6000..=0xFFFF => params.peek_prg(cpu_address),
        }
    }

    fn read_from_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16) -> ReadResult {
        self.peek_cartridge_space(params, cpu_address)
    }

    // Most mappers don't need to modify the MapperParams before ROM execution begins, but this
    // provides a relief valve for the rare settings that can't be expressed in a Layout.
    fn init_mapper_params(&self, _params: &mut MapperParams) {}
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
    // Most mappers don't have bus conflicts.
    fn has_bus_conflicts(&self) -> HasBusConflicts { HasBusConflicts::No }

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
                        let peeker = |ppu_address| self.ppu_peek(params, ppu_internal_ram, ppu_address);
                        ppu_registers.peek_ppu_data(peeker)
                    }
                    _ => unreachable!(),
                })
            }
            0x4000..=0x4013 => { /* APU registers are write-only. */ ReadResult::OPEN_BUS }
            0x4014          => { /* OAM DMA is write-only. */ ReadResult::OPEN_BUS }
            0x4015          => ReadResult::full(apu_registers.peek_status().to_u8()),
            // TODO: Move ReadResult/mask specification into the controller.
            0x4016          => ReadResult::partial_open_bus(ports.joypad1.borrow().peek_status() as u8, 0b0000_0001),
            0x4017          => ReadResult::partial_open_bus(ports.joypad2.borrow().peek_status() as u8, 0b0000_0001),
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
        ppu_internal_ram: &PpuInternalRam,
        oam: &Oam,
        ports: &mut Ports,
        ppu_registers: &mut PpuRegisters,
        apu_registers: &mut ApuRegisters,
        address: CpuAddress,
    ) -> ReadResult {
        self.on_cpu_read(address);
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
                        let reader = |ppu_address| self.ppu_read(params, ppu_internal_ram, ppu_address, false);
                        let data = ppu_registers.read_ppu_data(reader);
                        self.process_current_ppu_address(ppu_registers.current_address());
                        data
                    }
                    _ => unreachable!(),
                })
            }
            0x4000..=0x4013 => { /* APU registers are write-only. */ ReadResult::OPEN_BUS }
            0x4014          => { /* OAM DMA is write-only. */ ReadResult::OPEN_BUS }
            0x4015          => ReadResult::full(apu_registers.read_status().to_u8()),
            // TODO: Move ReadResult/mask specification into the controller.
            0x4016          => ReadResult::partial_open_bus(ports.joypad1.borrow_mut().read_status() as u8, 0b0000_0001),
            0x4017          => ReadResult::partial_open_bus(ports.joypad2.borrow_mut().read_status() as u8, 0b0000_0001),
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
                0x2000 => ppu_registers.write_ctrl(value),
                0x2001 => ppu_registers.write_mask(value),
                0x2002 => ppu_registers.write_ppu_io_bus(value),
                0x2003 => ppu_registers.write_oam_addr(value),
                0x2004 => ppu_registers.write_oam_data(oam, value),
                0x2005 => ppu_registers.write_scroll(value),
                0x2006 => {
                    ppu_registers.write_ppu_addr(value);
                    if ppu_registers.write_toggle() == WriteToggle::FirstByte {
                        self.process_current_ppu_address(ppu_registers.current_address());
                    }
                }
                0x2007 => {
                    self.ppu_write(params, ppu_internal_ram, ppu_registers.current_address(), value);
                    ppu_registers.write_ppu_data(value);
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
            0x4018..=0x401F => { /* CPU Test Mode not yet supported. */ }
            0x4020..=0xFFFF => {
                let value = if self.has_bus_conflicts() == HasBusConflicts::Yes {
                    let rom_value = self.cpu_peek(params, cpu_internal_ram, ppu_internal_ram, oam,
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
    ) -> &'a [u8; KIBIBYTE as usize] {
        let side = vram_side(quadrant, name_table_mirroring);
        ppu_internal_ram.vram.side(side)
    }

    #[inline]
    fn raw_name_table_mut<'a>(
        &'a mut self,
        name_table_mirroring: NameTableMirroring,
        ppu_internal_ram: &'a mut PpuInternalRam,
        position: NameTableQuadrant,
    ) -> &'a mut [u8; KIBIBYTE as usize] {
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
        self.raw_name_table(name_table_mirroring, ppu_internal_ram, name_table_quadrant)[index as usize]
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
        self.raw_name_table_mut(name_table_mirroring, ppu_internal_ram, name_table_quadrant)[index as usize] = value;
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
                prg_memory.bank_size(),
                prg_memory.bank_count(),
                true,
            );
            let window_size = window.size() / KIBIBYTE as u16;

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
        for window in chr_memory.current_layout().windows() {
            let bank_string = window.bank_string(
                &params.bank_registers,
                chr_memory.bank_size(),
                chr_memory.bank_count(),
                chr_memory.align_large_layouts(),
            );
            let window_size = window.size() / KIBIBYTE as u16;

            let left_padding_len;
            let right_padding_len;
            let padding_size = 5 * window_size - 2 - u16::try_from(bank_string.len()).unwrap();
            assert!(padding_size < 100);
            left_padding_len = padding_size / 2;
            right_padding_len = padding_size - left_padding_len;

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
    pub bank_registers: BankRegisters,
    // TODO: Consolidate these into ChrMemory?
    pub name_table_mirroring: NameTableMirroring,
    pub name_table_mirrorings: &'static [NameTableMirroring],
    pub ram_statuses: &'static [RamStatus],
}

impl MapperParams {
    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    pub fn set_name_table_mirroring(&mut self, mirroring_index: u8) {
        self.name_table_mirroring = self.name_table_mirrorings[usize::from(mirroring_index)];
    }

    pub fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    pub fn set_prg_layout(&mut self, index: u8) {
        self.prg_memory.set_layout(index);
    }

    pub fn peek_prg(&self, cpu_address: u16) -> ReadResult {
        self.prg_memory.peek(&self.bank_registers, CpuAddress::new(cpu_address))
    }

    pub fn write_prg(&mut self, cpu_address: u16, value: u8) {
        self.prg_memory.write(&self.bank_registers, CpuAddress::new(cpu_address), value);
    }

    pub fn set_ram_status(&mut self, id: RamStatusRegisterId, index: u8) {
        self.bank_registers.set_ram_status(id, self.ram_statuses[index as usize]);
    }

    pub fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        self.chr_memory.pattern_table(&self.bank_registers, side)
    }

    pub fn set_chr_layout(&mut self, index: u8) {
        self.chr_memory.set_layout(index);
    }

    pub fn peek_chr(&self, address: PpuAddress) -> u8 {
        self.chr_memory.peek(&self.bank_registers, address)
    }

    pub fn write_chr(&mut self, address: PpuAddress, value: u8) {
        self.chr_memory.write(&self.bank_registers, address, value);
    }

    pub fn set_bank_register<INDEX: Into<u16>>(
        &mut self,
        id: BankRegisterId,
        value: INDEX,
    ) {
        self.bank_registers.set(id, BankIndex::from_u16(value.into()));
    }

    pub fn set_bank_register_bits(
        &mut self, id: BankRegisterId, new_value: u16, mask: u16) {

        self.bank_registers.set_bits(id, new_value, mask);
    }

    pub fn set_meta_register(&mut self, id: MetaRegisterId, value: BankRegisterId) {
        self.bank_registers.set_meta(id, value);
    }

    pub fn update_bank_register(
        &mut self,
        id: BankRegisterId,
        updater: &dyn Fn(u16) -> u16,
    ) {
        self.bank_registers.update(id, updater);
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
