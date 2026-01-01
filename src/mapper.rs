pub use splitbits::{splitbits, splitbits_named, combinebits, splitbits_then_combine};

pub use crate::cartridge::cartridge::Cartridge;
pub use crate::counter::counter::{ReloadDrivenCounter, DirectlySetCounter, CounterBuilder, AutoTriggerWhen, ForcedReloadTiming, WhenDisabledPrevent};
pub use crate::counter::irq_counter_info::IrqCounterInfo;
pub use crate::memory::bank::bank_number::{BankNumber, PrgBankRegisterId, ChrBankRegisterId, MetaRegisterId, ReadStatus, WriteStatus};
pub use crate::memory::bank::bank_number::PrgBankRegisterId::*;
pub use crate::memory::bank::bank_number::ChrBankRegisterId::*;
pub use crate::memory::bank::bank_number::MetaRegisterId::*;
pub use crate::memory::bank::bank::{PrgBank, ChrBank, ChrSource};
pub use crate::memory::bank::bank::PrgSourceRegisterId::*;
pub use crate::memory::bank::bank::ChrSourceRegisterId::*;
pub use crate::memory::bank::bank::ReadStatusRegisterId::*;
pub use crate::memory::bank::bank::WriteStatusRegisterId::*;
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

use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::ppu::ciram::Ciram;
use crate::ppu::register::ppu_registers::WriteToggle;

use crate::memory::bank::bank_number::MemType;

pub trait Mapper {
    // Should be const, but that's not yet allowed by Rust.
    // Every mapper must define a Layout.
    fn layout(&self) -> Layout;

    // Most mappers don't support peeking register values.
    fn peek_register(&self, _mem: &Memory, addr: CpuAddress) -> ReadResult {
        assert!(0x4020 <= *addr && *addr <= 0x5FFF);
        ReadResult::OPEN_BUS
    }

    // Every mapper must implement write_register.
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8);

    // Hack to allow Action 53 to use its own custom peeking logic.
    // TODO: Provide a proper solution for Action 53.
    fn peek_prg(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        mem.prg_memory.peek(addr)
    }

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

    fn cpu_peek(&self, mem: &Memory, address_bus_type: AddressBusType, addr: CpuAddress) -> u8 {
        self.cpu_peek_unresolved(mem, address_bus_type, addr).resolve(mem.cpu_pinout.data_bus)
    }

    fn cpu_peek_unresolved(&self, mem: &Memory, _address_bus_type: AddressBusType, addr: CpuAddress) -> ReadResult {
        let normal_peek_value = match *addr {
            0x0000..=0x07FF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize]),
            0x0800..=0x1FFF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize & 0x07FF]),
            0x2000..=0x3FFF => {
                ReadResult::full(match *addr & 0x2007 {
                    0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2002 => mem.ppu_regs.peek_status(),
                    0x2004 => mem.ppu_regs.peek_oam_data(&mem.oam),
                    0x2007 => {
                        let old_value = self.ppu_peek(mem, mem.ppu_regs.current_address).value();
                        mem.ppu_regs.peek_ppu_data(old_value)
                    }
                    _ => unreachable!(),
                })
            }
            // APU registers can only be read if the current address bus AND the CPU address bus are in the correct range.
            0x4000..=0x401F => ReadResult::OPEN_BUS,
            0x4020..=0x5FFF => self.peek_register(mem, addr),
            0x6000..=0xFFFF => self.peek_prg(mem, addr),
        };

        let mut should_apu_read_dominate_normal_read = false;
        let apu_peek_value = if mem.apu_registers_active() {
            let addr = CpuAddress::new(0x4000 + *addr % 0x20);
            match *addr {
                0x4000..=0x4013 => { /* APU registers are write-only. */ ReadResult::OPEN_BUS }
                0x4014          => { /* OAM DMA is write-only. */ ReadResult::OPEN_BUS }
                0x4015 => {
                    should_apu_read_dominate_normal_read = true;
                    ReadResult::partial(mem.apu_regs.peek_status(&mem.cpu_pinout, &mem.dmc_dma).to_u8(), 0b1101_1111)
                }
                // TODO: Move ReadResult/mask specification into the controller.
                0x4016          => ReadResult::partial(mem.joypad1.peek_status() as u8, 0b0000_0111),
                0x4017          => ReadResult::partial(mem.joypad2.peek_status() as u8, 0b0000_0111),
                0x4018..=0x401F => /* CPU Test Mode not yet supported. */ ReadResult::OPEN_BUS,
                _ => unreachable!()
            }
        } else {
            ReadResult::OPEN_BUS
        };

        if should_apu_read_dominate_normal_read {
            apu_peek_value.dominate(normal_peek_value)
        } else {
            normal_peek_value.dominate(apu_peek_value)
        }
    }

    #[inline]
    #[rustfmt::skip]
    fn cpu_read(&mut self, mem: &mut Memory, address_bus_type: AddressBusType) -> u8 {
        let addr = mem.cpu_address_bus(address_bus_type);
        let normal_read_value = match *addr {
            0x0000..=0x07FF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize]),
            0x0800..=0x1FFF => ReadResult::full(mem.cpu_internal_ram()[*addr as usize & 0x07FF]),
            0x2000..=0x3FFF => {
                ReadResult::full(match *addr & 0x2007 {
                    0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 => mem.ppu_regs.peek_ppu_io_bus(),
                    0x2002 => mem.ppu_regs.read_status(),
                    0x2004 => mem.ppu_regs.read_oam_data(&mem.oam),
                    0x2007 => {
                        self.set_ppu_address_bus(mem, mem.ppu_regs.current_address);
                        // TODO: Instead of peeking the old data, it must be available as part of some register.
                        let old_data = self.ppu_peek(mem, mem.ppu_pinout.address()).value();
                        let new_data = mem.ppu_regs.read_ppu_data(old_data);

                        let pending_data_source = mem.ppu_regs.current_address.to_pending_data_source();
                        let buffered_data = self.ppu_peek(mem, pending_data_source).value();
                        self.on_ppu_read(mem, pending_data_source, buffered_data);
                        mem.ppu_regs.set_ppu_read_buffer_and_advance(buffered_data);
                        self.set_ppu_address_bus(mem, mem.ppu_regs.current_address);

                        new_data
                    }
                    _ => unreachable!(),
                })
            }
            // APU registers can only be read if the current address bus AND the CPU address bus are in the correct range.
            0x4000..=0x401F => ReadResult::OPEN_BUS,
            0x4020..=0x5FFF => self.peek_register(mem, addr),
            0x6000..=0xFFFF => self.peek_prg(mem, addr),
        };

        let mut should_apu_read_dominate_normal_read = false;
        let mut should_apu_read_update_data_bus = true;
        let apu_read_value = if mem.apu_registers_active() {
            let addr = CpuAddress::new(0x4000 + *addr % 0x20);
            match *addr {
                // Most APU registers are write-only.
                0x4000..=0x4013 => ReadResult::OPEN_BUS,
                // OAM DMA is write-only.
                0x4014 => ReadResult::OPEN_BUS,
                0x4015 => {
                    // APU status reads only use the data bus when using a DMA address bus.
                    should_apu_read_dominate_normal_read = true;
                    should_apu_read_update_data_bus = address_bus_type != AddressBusType::Cpu;
                    ReadResult::partial(mem.apu_regs.read_status(&mem.cpu_pinout, &mem.dmc_dma).to_u8(), 0b1101_1111)
                }
                // TODO: Move ReadResult/mask specification into the controller.
                0x4016 => ReadResult::partial(mem.joypad1.read_status() as u8, 0b0000_0111),
                0x4017 => ReadResult::partial(mem.joypad2.read_status() as u8, 0b0000_0111),
                // CPU Test Mode not yet supported.
                0x4018..=0x401F => ReadResult::OPEN_BUS,
                _ => unreachable!(),
            }
        } else {
            ReadResult::OPEN_BUS
        };

        let value = if should_apu_read_dominate_normal_read {
            apu_read_value.dominate(normal_read_value).resolve(mem.cpu_pinout.data_bus)
        } else {
            normal_read_value.dominate(apu_read_value).resolve(mem.cpu_pinout.data_bus)
        };

        mem.cpu_pinout.data_bus = if should_apu_read_update_data_bus {
            value
        } else {
            normal_read_value.resolve(mem.cpu_pinout.data_bus)
        };

        self.on_cpu_read(mem, addr, value);

        value
    }

    // TODO: APU register mirroring probably affects writes (at least for $2004/$4004), so implement it.
    #[inline]
    #[rustfmt::skip]
    fn cpu_write(&mut self, mem: &mut Memory, address_bus_type: AddressBusType) {
        let addr = mem.cpu_address_bus(address_bus_type);

        match *addr {
            0x0000..=0x07FF => mem.cpu_internal_ram[*addr as usize] = mem.cpu_pinout.data_bus,
            0x0800..=0x1FFF => mem.cpu_internal_ram[*addr as usize & 0x07FF] = mem.cpu_pinout.data_bus,
            0x2000..=0x3FFF => match *addr & 0x2007 {
                0x2000 => mem.ppu_regs.write_ctrl(mem.cpu_pinout.data_bus),
                0x2001 => mem.ppu_regs.write_mask(mem.cpu_pinout.data_bus),
                0x2002 => mem.ppu_regs.write_ppu_io_bus(mem.cpu_pinout.data_bus),
                0x2003 => mem.ppu_regs.write_oam_addr(mem.cpu_pinout.data_bus),
                0x2004 => mem.ppu_regs.write_oam_data(&mut mem.oam, mem.cpu_pinout.data_bus),
                0x2005 => mem.ppu_regs.write_scroll(mem.cpu_pinout.data_bus),
                0x2006 => {
                    mem.ppu_regs.write_ppu_addr(mem.cpu_pinout.data_bus);
                    if mem.ppu_regs.write_toggle() == WriteToggle::FirstByte {
                        self.set_ppu_address_bus(mem, mem.ppu_regs.current_address);
                    }
                }
                0x2007 => {
                    self.ppu_write(mem, mem.ppu_regs.current_address, mem.cpu_pinout.data_bus);
                    mem.ppu_regs.write_ppu_data(mem.cpu_pinout.data_bus);
                    self.set_ppu_address_bus(mem, mem.ppu_regs.current_address);
                }
                _ => unreachable!(),
            }
            0x4000          => mem.apu_regs.pulse_1.set_control(mem.cpu_pinout.data_bus),
            0x4001          => mem.apu_regs.pulse_1.set_sweep(mem.cpu_pinout.data_bus),
            0x4002          => mem.apu_regs.pulse_1.set_period_low(mem.cpu_pinout.data_bus),
            0x4003          => mem.apu_regs.pulse_1.set_length_and_period_high(mem.cpu_pinout.data_bus),
            0x4004          => mem.apu_regs.pulse_2.set_control(mem.cpu_pinout.data_bus),
            0x4005          => mem.apu_regs.pulse_2.set_sweep(mem.cpu_pinout.data_bus),
            0x4006          => mem.apu_regs.pulse_2.set_period_low(mem.cpu_pinout.data_bus),
            0x4007          => mem.apu_regs.pulse_2.set_length_and_period_high(mem.cpu_pinout.data_bus),
            0x4008          => mem.apu_regs.triangle.write_control_byte(mem.cpu_pinout.data_bus),
            0x4009          => { /* Unused. */ }
            0x400A          => mem.apu_regs.triangle.write_timer_low_byte(mem.cpu_pinout.data_bus),
            0x400B          => mem.apu_regs.triangle.write_length_and_timer_high_byte(mem.cpu_pinout.data_bus),
            0x400C          => mem.apu_regs.noise.set_control(mem.cpu_pinout.data_bus),
            0x400D          => { /* Unused. */ }
            0x400E          => mem.apu_regs.noise.set_loop_and_period(mem.cpu_pinout.data_bus),
            0x400F          => mem.apu_regs.noise.set_length(mem.cpu_pinout.data_bus),
            0x4010          => mem.apu_regs.dmc.write_control_byte(&mut mem.cpu_pinout),
            0x4011          => mem.apu_regs.dmc.write_volume(mem.cpu_pinout.data_bus),
            0x4012          => mem.apu_regs.dmc.write_sample_start_address(mem.cpu_pinout.data_bus),
            0x4013          => mem.dmc_dma.write_sample_length(mem.cpu_pinout.data_bus),
            0x4014          => mem.oam_dma.prepare_to_start(mem.cpu_pinout.data_bus),
            0x4015          => mem.apu_regs.write_status_byte(&mut mem.cpu_pinout, &mut mem.dmc_dma),
            0x4016          => {
                mem.joypad1.change_strobe(mem.cpu_pinout.data_bus);
                mem.joypad2.change_strobe(mem.cpu_pinout.data_bus);
            }
            0x4017          => mem.apu_regs.write_frame_counter(&mut mem.cpu_pinout),
            0x4018..=0x401F => { /* CPU Test Mode not yet supported. */ }
            0x4020..=0xFFFF => {
                if matches!(*addr, 0x6000..=0xFFFF) {
                    // TODO: Verify if bus conflicts only occur for address >= 0x6000.
                    if self.has_bus_conflicts() == HasBusConflicts::Yes {
                        let rom_value = self.cpu_peek_unresolved(mem, address_bus_type, addr);
                        mem.cpu_pinout.data_bus = rom_value.bus_conflict(mem.cpu_pinout.data_bus);
                    }

                    mem.prg_memory.write(addr, mem.cpu_pinout.data_bus);
                }

                self.write_register(mem, addr, mem.cpu_pinout.data_bus);
            }
        }

        self.on_cpu_write(mem, addr, mem.cpu_pinout.data_bus);
    }

    fn ppu_peek(&self, mem: &Memory, address: PpuAddress) -> PpuPeek {
        match address.to_u16() {
            0x0000..=0x1FFF => mem.peek_chr(address),
            0x2000..=0x3EFF => self.peek_name_table_byte(mem, &mem.ciram, address),
            0x3F00..=0x3FFF => mem.palette_ram.peek(address.to_palette_ram_index()),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    fn ppu_internal_read(&mut self, mem: &mut Memory) -> PpuPeek {
        let result = self.ppu_peek(mem, mem.ppu_pinout.address());
        self.on_ppu_read(mem, mem.ppu_pinout.address(), result.value());
        result
    }

    #[inline]
    fn ppu_write(&mut self, mem: &mut Memory, address: PpuAddress, value: u8) {
        match address.to_u16() {
            0x0000..=0x1FFF => mem.chr_memory.write(&mem.ppu_regs, &mut mem.ciram, &mut mem.mapper_custom_pages, address, value),
            0x2000..=0x3EFF => self.write_name_table_byte(mem, address, value),
            0x3F00..=0x3FFF => mem.palette_ram.write(address.to_palette_ram_index(), value),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    fn set_ppu_address_bus(&mut self, mem: &mut Memory, addr: PpuAddress) {
        let address_changed = mem.ppu_pinout.set_address_bus(addr);
        if address_changed {
            self.on_ppu_address_change(mem, addr);
        }
    }

    fn set_ppu_data_bus(&mut self, mem: &mut Memory, data: u8) {
        let address_changed = mem.ppu_pinout.set_data_bus(data);
        if address_changed {
            self.on_ppu_address_change(mem, mem.ppu_pinout.address());
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
            // FIXME: Hack
            NameTableSource::Rom { bank_number } => mem.chr_memory.rom_1kib_page(0x400 * u32::from(bank_number.to_raw())),
            // FIXME: Hack
            NameTableSource::Ram { bank_number } => mem.chr_memory.work_ram_1kib_page(0x400 * u32::from(bank_number.to_raw())),
            NameTableSource::MapperCustom { page_number, .. } => mem.mapper_custom_pages[page_number as usize].to_raw_ref(),
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
            NameTableSource::Rom {..} => { /* ROM is read-only. */}
            // FIXME: This currently ignores whether RAM writes are enabled. It shouldn't be possible to do that.
            NameTableSource::Ram { bank_number } =>
                mem.chr_memory.work_ram_1kib_page_mut(0x400 * u32::from(bank_number.to_raw()))[index as usize] = value,
            NameTableSource::MapperCustom { page_number, .. } => {
                if let Some(page) = mem.mapper_custom_pages[page_number as usize].to_raw_ref_mut() {
                    // This page must be writeable.
                    page[index as usize] = value;
                }
            }
        }
    }

    fn prg_rom_bank_string(&self, mem: &Memory) -> String {
        let prg_memory = &mem.prg_memory();

        let mut result = String::new();
        for prg_page_id_slot in prg_memory.current_memory_map().page_id_slots() {
            let bank_string = match prg_page_id_slot {
                PrgPageIdSlot::Normal(prg_source_and_page_number, _, _) => {
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
        for (page_id, _, _) in chr_memory.current_memory_map().pattern_table_page_ids() {
            let bank_string = match page_id {
                ChrPageId::Rom { page_number, .. } => page_number.to_string(),
                ChrPageId::Ram { page_number, .. } => format!("W{page_number}"),
                ChrPageId::Ciram(side) => format!("C{side:?}"),
                ChrPageId::SaveRam => "S".to_owned(),
                ChrPageId::MapperCustom { page_number } => format!("M{page_number}"),
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
    ReassignedMapper {correct_mapper: u16, correct_submapper: Option<u8> },
}
