pub use splitbits::{splitbits, splitbits_named, combinebits, splitbits_then_combine};

pub(in crate::mapper) use crate::bus::Bus;
pub(in crate::mapper) use crate::counter::counter::{ReloadDrivenCounter, DirectlySetCounter, CounterBuilder, AutoTriggerWhen, ForcedReloadTiming, WhenDisabledPrevent};
pub(in crate::mapper) use crate::counter::irq_counter_info::IrqCounterInfo;
pub(in crate::mapper) use crate::memory::bank::bank_number::{BankNumber, PrgBankRegisterId, ChrBankRegisterId, ReadStatus, WriteStatus};
pub(in crate::mapper) use crate::memory::bank::bank_number::PrgBankRegisterId::*;
pub(in crate::mapper) use crate::memory::bank::bank_number::ChrBankRegisterId::*;
pub(in crate::mapper) use crate::memory::bank::bank_number::MetaRegisterId::*;
pub(in crate::mapper) use crate::memory::bank::bank::PrgSourceRegisterId::*;
pub(in crate::mapper) use crate::memory::bank::bank::ChrSourceRegisterId::*;
pub(in crate::mapper) use crate::memory::bank::bank::ReadStatusRegisterId::*;
pub(in crate::mapper) use crate::memory::bank::bank::WriteStatusRegisterId::*;
pub(in crate::mapper) use crate::memory::cpu::cpu_address::CpuAddress;
pub(in crate::mapper) use crate::memory::layout::Layout;
pub(in crate::mapper) use crate::memory::ppu::ppu_address::PpuAddress;
pub(in crate::mapper) use crate::memory::read_result::ReadResult;
pub(in crate::mapper) use crate::memory::regions::ciram::CiramSide;
pub(in crate::mapper) use crate::memory::window::{PrgWindow, ChrWindow};
pub(in crate::mapper) use crate::memory::window::ChrSourceProvider as Chr;
pub(in crate::mapper) use crate::memory::window::PrgSourceProvider as Prg;
pub(in crate::mapper) use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
pub(in crate::mapper) use crate::ppu::name_table::name_table_mirroring::{NameTableMirroring, NameTableSource};
pub(in crate::mapper) use crate::ppu::pattern_table_side::PatternTableSide;
pub(in crate::mapper) use crate::util::unit::KIBIBYTE;

use crate::memory::ppu::chr_memory::PpuPeek;

pub trait Mapper {
    // Should be const, but that's not yet allowed by Rust.
    // Every mapper must define a Layout.
    fn layout(&self) -> Layout;

    // Every mapper must implement write_register.
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8);

    // Many mappers clear bank registers or some mapper-specific state upon soft reset.
    // TODO: Require each mapper to implement this.
    fn reset(&mut self, _: &mut Bus) {}

    // Most mappers don't support peeking register values.
    fn peek_register(&self, _bus: &Bus, addr: CpuAddress) -> ReadResult {
        assert!(0x4020 <= *addr && *addr <= 0x5FFF);
        ReadResult::OPEN_BUS
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
