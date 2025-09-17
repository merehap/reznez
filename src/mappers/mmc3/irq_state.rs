use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;

pub trait IrqState {
    fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress);
    fn decrement_suppression_cycle_count(&mut self);
    fn set_counter_reload_value(&mut self, value: u8);
    fn reload_counter(&mut self);
    fn enable(&mut self);
    fn disable(&mut self, mem: &mut Memory);
}
