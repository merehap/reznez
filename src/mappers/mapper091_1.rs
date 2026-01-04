use crate::mapper::*;
use crate::counter::counter::{PrescalerBehaviorOnForcedReload, PrescalerTriggeredBy, WhenDisabledPrevent};

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();

const HORIZONTAL: u8 = 0;
const VERTICAL: u8 = 1;

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(-5)
    .wraps(false)
    .full_range(0, 0xFFFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::EndingOn(0))
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .prescaler(4, PrescalerTriggeredBy::WrappingToZero, PrescalerBehaviorOnForcedReload::DoNothing)
    .build_reload_driven_counter();

// J.Y. Company JY830623C and YY840238C
pub struct Mapper091_1 {
    irq_counter: ReloadDrivenCounter,
}

impl Mapper for Mapper091_1 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0xF007 {
            0x6000 => bus.set_chr_register(C0, value),
            0x6001 => bus.set_chr_register(C1, value),
            0x6002 => bus.set_chr_register(C2, value),
            0x6003 => bus.set_chr_register(C3, value),
            0x6004 => bus.set_name_table_mirroring(HORIZONTAL),
            0x6005 => bus.set_name_table_mirroring(VERTICAL),
            0x7000 => bus.set_prg_register(P0, value & 0b00001111),
            0x7001 => bus.set_prg_register(P1, value & 0b00001111),

            0x6006 => {
                self.irq_counter.set_reload_value_low_byte(value);
                self.irq_counter.force_reload();
            }
            0x6007 => {
                self.irq_counter.set_reload_value_high_byte(value);
            }
            0x7006 => {
                self.irq_counter.disable();
                bus.cpu_pinout.acknowledge_mapper_irq();
            }
            0x7007 => {
                self.irq_counter.enable();
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        if self.irq_counter.tick().triggered {
            bus.cpu_pinout.assert_mapper_irq();
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