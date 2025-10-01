pub use splitbits::{splitbits, splitbits_named, combinebits, splitbits_then_combine};

pub use crate::cartridge::cartridge::Cartridge;
pub use crate::counter::counter::{Counter, Direction, DirectlySetCounter, CounterBuilder, AutoTriggeredBy, ForcedReloadTiming, WhenDisabledPrevent};
pub use crate::counter::irq_counter_info::IrqCounterInfo;
pub use crate::counter::incrementing_counter::{IncrementingCounter, IncrementingCounterBuilder, IncAutoTriggeredBy, WhenTargetReached};
pub use crate::memory::bank::bank_index::{BankIndex, PrgBankRegisterId, ChrBankRegisterId, MetaRegisterId, ReadWriteStatus};
pub use crate::memory::bank::bank_index::PrgBankRegisterId::{P0, P1, P2, P3, P4};
pub use crate::memory::bank::bank_index::ChrBankRegisterId::*;
pub use crate::memory::bank::bank_index::MetaRegisterId::*;
pub use crate::memory::bank::bank::{PrgBank, ChrBank, ReadWriteStatusRegisterId};
pub use crate::memory::bank::bank::ReadWriteStatusRegisterId::*;
pub use crate::memory::bank::bank::RomRamModeRegisterId::*;
pub use crate::memory::cpu::cpu_address::CpuAddress;
pub use crate::memory::cpu::prg_memory::PrgMemory;
use crate::memory::cpu::prg_memory_map::PrgPageIdSlot;
pub use crate::memory::layout::Layout;
pub use crate::memory::memory::{AddressBusType, Memory};
pub use crate::memory::ppu::chr_memory::ChrMemory;
use crate::memory::ppu::chr_memory_map::ChrPageId;
pub use crate::memory::ppu::ppu_address::PpuAddress;
pub use crate::memory::read_result::ReadResult;
pub use crate::memory::ppu::ciram::CiramSide;
pub use crate::memory::window::{PrgWindow, ChrWindow};
pub use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
pub use crate::ppu::name_table::name_table_mirroring::{NameTableMirroring, NameTableSource};
pub use crate::ppu::pattern_table_side::PatternTableSide;
pub use crate::util::unit::{KIBIBYTE, KIBIBYTE_U16};

use num_traits::FromPrimitive;

use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::ppu::ciram::Ciram;
use crate::ppu::register::ppu_registers::WriteToggle;

use crate::memory::bank::bank_index::MemType;

pub trait Mapper {
    // Should be const, but that's not yet allowed by Rust.
    fn layout(&self) -> Layout;

    // Most mappers don't override the default cartridge peeking/reading behavior.
    // TODO: Rename this to peek_register once params.peek_prg() is handled separately.
    fn peek_cartridge_space(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => ReadResult::OPEN_BUS,
            0x6000..=0xFFFF => mem.peek_prg(addr),
        }
    }

    // TODO: Rename this to read_register once params.peek_prg() is handled separately.
    fn read_from_cartridge_space(&mut self, mem: &mut Memory, addr: CpuAddress) -> ReadResult {
        self.peek_cartridge_space(mem, addr)
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8);

    // Most mappers don't need to modify the MapperParams before ROM execution begins, but this
    // provides a relief valve for the rare settings that can't be expressed in a Layout.
    fn init_mapper_params(&self, _mem: &mut Memory) {}
    // Most mappers don't care about CPU cycles.
    fn on_end_of_cpu_cycle(&mut self, _mem: &mut Memory) {}
    fn on_cpu_read(&mut self, _mem: &mut Memory, _addr: CpuAddress, _value: u8) {}
    fn on_cpu_write(&mut self, _mem: &mut Memory, _addr: CpuAddress, _value: u8) {}
    // Most mappers don't care about PPU cycles.
    fn on_end_of_ppu_cycle(&mut self) {}
    // Most mappers don't trigger anything based upon ppu reads.
    fn on_ppu_read(&mut self, _mem: &mut Memory, _address: PpuAddress, _value: u8) {}
    // Most mappers don't care about changes to the current PPU address.
    fn on_ppu_address_change(&mut self, _mem: &mut Memory, _address: PpuAddress) {}
    // Most mappers don't have bus conflicts.
    fn has_bus_conflicts(&self) -> HasBusConflicts { HasBusConflicts::No }
    // Used for debug screens.
    fn irq_counter_info(&self) -> Option<IrqCounterInfo> { None }
    // Most mappers don't use a fill-mode name table.
    fn fill_mode_name_table(&self) -> &[u8; KIBIBYTE as usize] { unimplemented!() }

    fn cpu_peek(&self, mem: &Memory, address_bus_type: AddressBusType, addr: CpuAddress) -> u8 {
        self.cpu_peek_unresolved(mem, address_bus_type, addr).resolve(mem.cpu_pinout.data_bus).0
    }

    fn cpu_peek_unresolved(&self, mem: &Memory, address_bus_type: AddressBusType, mut addr: CpuAddress) -> ReadResult {
        // See "APU Register Activation" in the README and asm file here: https://github.com/100thCoin/AccuracyCoin
        let apu_registers_active = matches!(*mem.address_bus(AddressBusType::Cpu), 0x4000..=0x401F);
        // TODO: I assume that the mirrors occur over the whole address space, but need bus conflicts emulated to actually work.
        // Limit the range for now to just 0x4000 to 0x40FF to pass the relevant AccuracyCoin test.
        if apu_registers_active && address_bus_type != AddressBusType::Cpu && *addr >= 0x4000 && *addr < 0x4100 {
            // The APU registers are mirrored over the whole address space, but the mirrors are usually not accessible.
            // When the mirrors are accessible, convert them to the normal APU register range for processing below.
            addr = CpuAddress::new(0x4000 + *addr % 0x20);
        }

        match *addr {
            0x0000..=0x07FF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize]),
            0x0800..=0x1FFF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize & 0x07FF]),
            0x2000..=0x3FFF => {
                ReadResult::full(match *addr & 0x2007 {
                    0x2000 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2001 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2002 => mem.ppu_regs.peek_status(),
                    0x2003 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2004 => mem.ppu_regs.peek_oam_data(&mem.oam),
                    0x2005 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2006 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2007 => {
                        let old_value = self.ppu_peek(mem, mem.ppu_regs.current_address()).value();
                        mem.ppu_regs.peek_ppu_data(old_value)
                    }
                    _ => unreachable!(),
                })
            }
            // APU registers can only be read if the current address bus AND the CPU address bus are in the correct range.
            0x4000..=0x401F if !apu_registers_active => ReadResult::OPEN_BUS,
            0x4000..=0x4013 => { /* APU registers are write-only. */ ReadResult::OPEN_BUS }
            0x4014          => { /* OAM DMA is write-only. */ ReadResult::OPEN_BUS }
            0x4015 if address_bus_type == AddressBusType::Cpu => ReadResult::no_bus_update(mem.apu_regs.peek_status(&mem.cpu_pinout).to_u8()),
            // DMA values must always be copied to the bus, unlike with the normal CPU address bus.
            0x4015 => ReadResult::partial_open_bus(mem.apu_regs.peek_status(&mem.cpu_pinout).to_u8(), 0b1101_1111),
            // TODO: Move ReadResult/mask specification into the controller.
            0x4016          => ReadResult::partial_open_bus(mem.ports.joypad1.peek_status() as u8, 0b0000_0111),
            0x4017          => ReadResult::partial_open_bus(mem.ports.joypad2.peek_status() as u8, 0b0000_0111),
            0x4018..=0x401F => /* CPU Test Mode not yet supported. */ ReadResult::OPEN_BUS,
            0x4020..=0xFFFF => self.peek_cartridge_space(mem, addr),
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn cpu_read(&mut self, mem: &mut Memory, address_bus_type: AddressBusType) -> u8 {
        let mut addr = mem.address_bus(address_bus_type);

        // See "APU Register Activation" in the README and asm file here: https://github.com/100thCoin/AccuracyCoin
        let apu_registers_active = matches!(*mem.address_bus(AddressBusType::Cpu), 0x4000..=0x401F);
        // TODO: I assume that the mirrors occur over the whole address space, but need bus conflicts emulated to actually work.
        // Limit the range for now to just 0x4000 to 0x40FF to pass the relevant AccuracyCoin test.
        if apu_registers_active && address_bus_type != AddressBusType::Cpu && *addr >= 0x4000 && *addr < 0x4100 {
            // The APU registers are mirrored over the whole address space, but the mirrors are usually not accessible.
            // When the mirrors are accessible, convert them to the normal APU register range for processing below.
            addr = CpuAddress::new(0x4000 + *addr % 0x20);
        }

        let read_result = match *addr {
            0x0000..=0x07FF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize]),
            0x0800..=0x1FFF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize & 0x07FF]),
            0x2000..=0x3FFF => {
                ReadResult::full(match *addr & 0x2007 {
                    0x2000 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2001 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2002 => mem.ppu_regs.read_status(),
                    0x2003 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2004 => mem.ppu_regs.read_oam_data(&mem.oam),
                    0x2005 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2006 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2007 => {
                        let old_value = self.ppu_read(mem, mem.ppu_regs.current_address(), false).value();
                        let old_value = mem.ppu_regs.peek_ppu_data(old_value);
                        mem.ppu_regs.ppu_io_bus.update_from_read(old_value);
                        let data = self.ppu_read(mem, mem.ppu_regs.current_address().to_pending_data_source(), false).value();
                        let data = mem.ppu_regs.set_pending_ppu_data(data);
                        self.on_ppu_address_change(mem, mem.ppu_regs.current_address());
                        data
                    }
                    _ => unreachable!(),
                })
            }
            // APU registers can only be read if the current address bus AND the CPU address bus are in the correct range.
            0x4000..=0x401F if !apu_registers_active => ReadResult::OPEN_BUS,
            // Most APU registers are write-only.
            0x4000..=0x4013 => ReadResult::OPEN_BUS,
            // OAM DMA is write-only.
            0x4014 => ReadResult::OPEN_BUS,
            0x4015 if address_bus_type == AddressBusType::Cpu => ReadResult::no_bus_update(mem.apu_regs.read_status(&mem.cpu_pinout).to_u8()),
            // DMA values must always be copied to the bus, unlike with the normal CPU address bus.
            0x4015 => ReadResult::partial_open_bus(mem.apu_regs.read_status(&mem.cpu_pinout).to_u8(), 0b1101_1111),
            // TODO: Move ReadResult/mask specification into the controller.
            0x4016 => ReadResult::partial_open_bus(mem.ports.joypad1.read_status() as u8, 0b0000_0111),
            0x4017 => ReadResult::partial_open_bus(mem.ports.joypad2.read_status() as u8, 0b0000_0111),
            // CPU Test Mode not yet supported.
            0x4018..=0x401F => ReadResult::OPEN_BUS,
            0x4020..=0xFFFF => self.read_from_cartridge_space(mem, addr),
        };

        let (value, bus_update_needed) = read_result.resolve(mem.cpu_pinout.data_bus);
        if bus_update_needed {
            mem.cpu_pinout.data_bus = value;
        }

        self.on_cpu_read(mem, addr, value);

        value
    }

    #[inline]
    #[rustfmt::skip]
    #[allow(clippy::too_many_arguments)]
    fn cpu_write(&mut self, mem: &mut Memory, address_bus_type: AddressBusType) {
        let addr = mem.address_bus(address_bus_type);
        let value = mem.cpu_pinout.data_bus;
        // TODO: Move this into mapper, right after cpu_write() is called?
        self.on_cpu_write(mem, addr, value);
        match *addr {
            0x0000..=0x07FF => mem.cpu_internal_ram[*addr as usize] = value,
            0x0800..=0x1FFF => mem.cpu_internal_ram[*addr as usize & 0x07FF] = value,
            0x2000..=0x3FFF => match *addr & 0x2007 {
                0x2000 => mem.ppu_regs.write_ctrl(value),
                0x2001 => mem.ppu_regs.write_mask(value),
                0x2002 => mem.ppu_regs.write_ppu_io_bus(value),
                0x2003 => mem.ppu_regs.write_oam_addr(value),
                0x2004 => mem.ppu_regs.write_oam_data(&mut mem.oam, value),
                0x2005 => mem.ppu_regs.write_scroll(value),
                0x2006 => {
                    mem.ppu_regs.write_ppu_addr(value);
                    if mem.ppu_regs.write_toggle() == WriteToggle::FirstByte {
                        self.on_ppu_address_change(mem, mem.ppu_regs.current_address());
                    }
                }
                0x2007 => {
                    self.ppu_write(mem, mem.ppu_regs.current_address(), value);
                    mem.ppu_regs.write_ppu_data(value);
                    self.on_ppu_address_change(mem, mem.ppu_regs.current_address());
                }
                _ => unreachable!(),
            }
            0x4000          => mem.apu_regs.pulse_1.write_control_byte(value),
            0x4001          => mem.apu_regs.pulse_1.write_sweep_byte(value),
            0x4002          => mem.apu_regs.pulse_1.write_timer_low_byte(value),
            0x4003          => mem.apu_regs.pulse_1.write_length_and_timer_high_byte(value),
            0x4004          => mem.apu_regs.pulse_2.write_control_byte(value),
            0x4005          => mem.apu_regs.pulse_2.write_sweep_byte(value),
            0x4006          => mem.apu_regs.pulse_2.write_timer_low_byte(value),
            0x4007          => mem.apu_regs.pulse_2.write_length_and_timer_high_byte(value),
            0x4008          => mem.apu_regs.triangle.write_control_byte(value),
            0x4009          => { /* Unused. */ }
            0x400A          => mem.apu_regs.triangle.write_timer_low_byte(value),
            0x400B          => mem.apu_regs.triangle.write_length_and_timer_high_byte(value),
            0x400C          => mem.apu_regs.noise.write_control_byte(value),
            0x400D          => { /* Unused. */ }
            0x400E          => mem.apu_regs.noise.write_loop_and_period_byte(value),
            0x400F          => mem.apu_regs.noise.write_length_byte(value),
            0x4010          => mem.apu_regs.dmc.write_control_byte(&mut mem.cpu_pinout, value),
            0x4011          => mem.apu_regs.dmc.write_volume(value),
            0x4012          => mem.apu_regs.dmc.write_sample_start_address(value),
            0x4013          => mem.apu_regs.dmc.write_sample_length(value),
            0x4014          => mem.oam_dma.prepare_to_start(value),
            0x4015          => mem.apu_regs.write_status_byte(&mut mem.cpu_pinout, &mut mem.dmc_dma, value),
            0x4016          => mem.ports.change_strobe(value),
            0x4017          => mem.apu_regs.write_frame_counter(&mut mem.cpu_pinout, value),
            0x4018..=0x401F => { /* CPU Test Mode not yet supported. */ }
            0x4020..=0xFFFF => {
                // TODO: Verify if bus conflicts only occur for address >= 0x6000.
                let value = if self.has_bus_conflicts() == HasBusConflicts::Yes {
                    let rom_value = self.cpu_peek_unresolved(mem, address_bus_type, mem.address_bus(address_bus_type));
                    rom_value.bus_conflict(value)
                } else {
                    value
                };

                if matches!(*addr, 0x6000..=0xFFFF) {
                    mem.prg_memory.write(addr, value);
                }

                self.write_register(mem, addr, value);
            }
        }
    }

    fn ppu_peek(&self, mem: &Memory, address: PpuAddress) -> PpuPeek {
        match address.to_u16() {
            0x0000..=0x1FFF => mem.peek_chr(&mem.ciram, address),
            0x2000..=0x3EFF => self.peek_name_table_byte(&mem, &mem.ciram, address),
            0x3F00..=0x3FFF => self.peek_palette_table_byte(&mem.palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn ppu_read(&mut self, mem: &mut Memory, address: PpuAddress, rendering: bool) -> PpuPeek {
        if rendering {
            self.on_ppu_address_change(mem, address);
        }

        let result = self.ppu_peek(mem, address);
        self.on_ppu_read(mem, address, result.value());
        result
    }

    #[inline]
    fn ppu_write(&mut self, mem: &mut Memory, address: PpuAddress, value: u8) {
        match address.to_u16() {
            0x0000..=0x1FFF => mem.chr_memory.write(&mem.ppu_regs, &mut mem.ciram, address, value),
            0x2000..=0x3EFF => self.write_name_table_byte(mem, address, value),
            0x3F00..=0x3FFF => self.write_palette_table_byte(&mut mem.palette_ram, address, value),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn raw_name_table<'a>(
        &'a self,
        mem: &'a Memory,
        ciram: &'a Ciram,
        quadrant: NameTableQuadrant,
    ) -> &'a [u8; KIBIBYTE as usize] {
        match mem.name_table_mirroring().name_table_source_in_quadrant(quadrant) {
            NameTableSource::Ciram(side) => ciram.side(side),
            NameTableSource::SaveRam(start_index) => mem.chr_memory.save_ram_1kib_page(start_index),
            NameTableSource::ExtendedRam => mem.prg_memory.extended_ram().as_raw_slice().try_into().unwrap(),
            NameTableSource::FillModeTile => self.fill_mode_name_table(),
        }
    }

    #[inline]
    fn peek_name_table_byte(
        &self,
        mem: &Memory,
        ciram: &Ciram,
        address: PpuAddress,
    ) -> PpuPeek {
        let (name_table_quadrant, index) = address_to_name_table_index(address);
        let value = self.raw_name_table(mem, ciram, name_table_quadrant)[index as usize];
        PpuPeek::new(value, PeekSource::from_name_table_source(mem.name_table_mirroring().name_table_source_in_quadrant(name_table_quadrant)))
    }

    #[inline]
    fn write_name_table_byte(&mut self, mem: &mut Memory, address: PpuAddress, value: u8) {
        let (quadrant, index) = address_to_name_table_index(address);
        match mem.name_table_mirroring().name_table_source_in_quadrant(quadrant) {
            NameTableSource::Ciram(side) =>
                mem.ciram.write(&mem.ppu_regs, side, index, value),
            NameTableSource::SaveRam(start_index) =>
                mem.chr_memory.save_ram_1kib_page_mut(start_index)[index as usize] = value,
            NameTableSource::ExtendedRam =>
                mem.prg_memory.extended_ram_mut().as_raw_mut_slice()[index as usize] = value,
            NameTableSource::FillModeTile =>
                { /* The fill mode tile can't be overwritten through normal memory writes. */ }
        }
    }

    #[inline]
    fn peek_palette_table_byte(&self, palette_ram: &PaletteRam, address: PpuAddress) -> PpuPeek {
        let value = palette_ram.read(address_to_palette_ram_index(address));
        PpuPeek::new(value, PeekSource::PaletteTable)
    }

    #[inline]
    fn write_palette_table_byte(&self, palette_ram: &mut PaletteRam, address: PpuAddress, value: u8) {
        palette_ram.write(address_to_palette_ram_index(address), value);
    }

    fn prg_rom_bank_string(&self, mem: &Memory) -> String {
        let prg_memory = &mem.prg_memory();

        let mut result = String::new();
        for prg_page_id_slot in prg_memory.current_memory_map().page_id_slots() {
            let bank_string = match prg_page_id_slot {
                PrgPageIdSlot::Normal(prg_source_and_page_number, _) => {
                    match prg_source_and_page_number {
                        None => "E".to_string(),
                        // FIXME: This should be bank number, not page number.
                        Some((MemType::Rom, page_number)) => page_number.to_string(),
                        Some((MemType::WorkRam, page_number)) => format!("W{page_number}"),
                        Some((MemType::SaveRam, page_number)) => format!("S{page_number}"),
                    }
                }
                PrgPageIdSlot::Multi(_) => "M".to_string(),
            };

            let window_size = 8;

            let left_padding_len;
            let right_padding_len;
            if window_size < 8 {
                left_padding_len = 0;
                right_padding_len = 0;
            } else {
                let padding_size = window_size - 2u16.saturating_sub(u16::try_from(bank_string.len()).unwrap());
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

    fn chr_rom_bank_string(&self, mem: &Memory) -> String {
        let chr_memory = &mem.chr_memory();

        let mut result = String::new();
        for (page_id, _) in chr_memory.current_memory_map().pattern_table_page_ids() {
            let bank_string = match page_id {
                ChrPageId::Rom { page_number, .. } => page_number.to_string(),
                ChrPageId::Ram { page_number, .. } => format!("W{page_number}"),
                ChrPageId::Ciram(side) => format!("C{side:?}"),
                ChrPageId::SaveRam => "S".to_owned(),
                ChrPageId::ExtendedRam => "X".to_owned(),
                ChrPageId::FillModeTile => "F".to_owned(),
            };

            let window_size = 1;

            let padding_size = 5 * window_size - 2u16.saturating_sub(u16::try_from(bank_string.len()).unwrap());
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
fn address_to_name_table_index(address: PpuAddress) -> (NameTableQuadrant, u16) {
    const NAME_TABLE_START:    u16 = 0x2000;
    const MIRROR_START:        u16 = 0x3000;
    const PALETTE_TABLE_START: u16 = 0x3F00;

    let address = address.to_u16();
    assert!(address >= NAME_TABLE_START);
    assert!(address < PALETTE_TABLE_START);

    let mut index = address;
    if index >= MIRROR_START {
        index -= 0x1000;
    }

    let index = index - NAME_TABLE_START;

    let name_table_quadrant = NameTableQuadrant::from_u16(index / KIBIBYTE_U16).unwrap();
    let index = index % KIBIBYTE_U16;
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
