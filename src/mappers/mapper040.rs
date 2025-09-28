use crate::counter::incrementing_counter::{IncAutoTriggeredBy, WhenTargetReached};
use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    // TODO: Verify if this is the correct max size.
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(6)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(4)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(5)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(7)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .build();

const IRQ_COUNTER: IncrementingCounter = IncrementingCounterBuilder::new()
    .auto_triggered_by(IncAutoTriggeredBy::AlreadyOnTarget)
    .trigger_target(0xFFF)
    .when_target_reached(WhenTargetReached::Clear)
    .when_disabled_prevent(WhenDisabledPrevent::TickingAndTriggering)
    .build();

// NTDEC 2722 and NTDEC 2752 PCB and imitations.
// Used for conversions of the Japanese version of Super Mario Bros. 2
// TODO: Test this mapper. The IRQ was broken last time checked, but potential fix was added, just not tested.
pub struct Mapper040 {
    irq_counter: IncrementingCounter,
}

impl Mapper for Mapper040 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                mem.cpu_pinout.clear_mapper_irq_pending();
                self.irq_counter.disable();
            }
            0xA000..=0xBFFF => {
                self.irq_counter.enable();
            }
            0xC000..=0xDFFF => { /* TODO: NTDEC 2752 outer bank register. Test ROM needed. */ }
            0xE000..=0xFFFF => {
                mem.set_prg_register(P0, value)
            }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        let triggered = self.irq_counter.tick();
        if triggered {
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

impl Mapper040 {
    pub fn new() -> Self {
        Self { irq_counter: IRQ_COUNTER }
    }
}