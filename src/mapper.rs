pub use splitbits::{splitbits, splitbits_named, combinebits, splitbits_then_combine};

pub use crate::bus::Bus;
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
pub use crate::memory::layout::Layout;
pub use crate::memory::ppu::chr_memory::ChrMemory;
pub use crate::memory::ppu::ppu_address::PpuAddress;
pub use crate::memory::read_result::ReadResult;
pub use crate::memory::ppu::ciram::CiramSide;
pub use crate::memory::window::{PrgWindow, ChrWindow};
pub use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
pub use crate::ppu::name_table::name_table_mirroring::{NameTableMirroring, NameTableSource};
pub use crate::ppu::pattern_table_side::PatternTableSide;
pub use crate::util::unit::{KIBIBYTE, KIBIBYTE_U16};

use crate::memory::ppu::chr_memory::PpuPeek;

pub trait Mapper {
    // Should be const, but that's not yet allowed by Rust.
    // Every mapper must define a Layout.
    fn layout(&self) -> Layout;

    // Most mappers don't support peeking register values.
    fn peek_register(&self, _bus: &Bus, addr: CpuAddress) -> ReadResult {
        assert!(0x4020 <= *addr && *addr <= 0x5FFF);
        ReadResult::OPEN_BUS
    }

    // Every mapper must implement write_register.
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8);

    // Hack to allow Action 53 to use its own custom peeking logic.
    // TODO: Provide a proper solution for Action 53.
    fn peek_prg(&self, bus: &Bus, addr: CpuAddress) -> ReadResult {
        bus.prg_memory.peek(addr)
    }

    // Most mappers don't need to modify the MapperParams before ROM execution begins, but this
    // provides a relief valve for the rare settings that can't be expressed in a Layout.
    fn init_mapper_params(&self, _bus: &mut Bus) {}
    // Most mappers don't care about CPU cycles.
    fn on_end_of_cpu_cycle(&mut self, _bus: &mut Bus) {}
    fn on_cpu_read(&mut self, _bus: &mut Bus, _addr: CpuAddress, _value: u8) {}
    fn on_cpu_write(&mut self, _bus: &mut Bus, _addr: CpuAddress, _value: u8) {}
    // Most mappers don't care about PPU cycles.
    fn on_end_of_ppu_cycle(&mut self) {}
    // Most mappers don't trigger anything based upon ppu reads.
    fn on_ppu_read(&mut self, _bus: &mut Bus, _address: PpuAddress, _value: u8) {}
    // Most mappers don't care about changes to the current PPU address.
    fn on_ppu_address_change(&mut self, _bus: &mut Bus, _address: PpuAddress) {}
    // Most mappers don't have bus conflicts.
    fn has_bus_conflicts(&self) -> bool { false }
    // Used for debug screens.
    fn irq_counter_info(&self) -> Option<IrqCounterInfo> { None }

    // Hack? Only used by MMC5 for overriding. Should be a better way to do this.
    fn ppu_peek(&self, bus: &Bus, address: PpuAddress) -> PpuPeek {
        bus.ppu_peek(address)
    }

    fn supported(self) -> LookupResult where Self: Sized, Self: 'static {
        LookupResult::Supported(Box::new(self))
    }
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
