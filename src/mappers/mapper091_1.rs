use crate::mapper::*;
use crate::counter::decrementing_counter::{PrescalerBehaviorOnForcedReload, PrescalerTriggeredBy, WhenDisabledPrevent};

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();

const HORIZONTAL: u8 = 0;
const VERTICAL: u8 = 1;

const IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .auto_triggered_by(AutoTriggeredBy::EndingOnZero)
    .auto_reload(false)
    .forced_reload_behavior(ForcedReloadBehavior::SetReloadValueImmediately)
    .decrement_size(5)
    .when_disabled_prevent(WhenDisabledPrevent::TickingAndTriggering)
    .prescaler(4, PrescalerTriggeredBy::WrappingToZero, PrescalerBehaviorOnForcedReload::DoNothing)
    .build();

// J.Y. Company JY830623C and YY840238C
pub struct Mapper091_1 {
    irq_counter: DecrementingCounter,
}

impl Mapper for Mapper091_1 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xF007 {
            0x6000 => mem.set_chr_register(C0, value),
            0x6001 => mem.set_chr_register(C1, value),
            0x6002 => mem.set_chr_register(C2, value),
            0x6003 => mem.set_chr_register(C3, value),
            0x6004 => mem.set_name_table_mirroring(HORIZONTAL),
            0x6005 => mem.set_name_table_mirroring(VERTICAL),
            0x7000 => mem.set_prg_register(P0, value & 0b00001111),
            0x7001 => mem.set_prg_register(P1, value & 0b00001111),

            0x6006 => {
                self.irq_counter.set_reload_value_low_byte(value);
                self.irq_counter.force_reload();
            }
            0x6007 => {
                self.irq_counter.set_reload_value_high_byte(value);
            }
            0x7006 => {
                self.irq_counter.disable();
                mem.cpu_pinout.clear_mapper_irq_pending();
            }
            0x7007 => {
                self.irq_counter.enable();
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        let should_trigger_irq = self.irq_counter.tick();
        if should_trigger_irq {
            mem.cpu_pinout.set_mapper_irq_pending();
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper091_1 {
    pub fn new() -> Self {
        Self { irq_counter: IRQ_COUNTER }
    }
}