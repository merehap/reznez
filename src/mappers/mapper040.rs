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
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// The wiki and Mesen disagree here. I'm thinking the wiki is right because it has more detailed behavior.
const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(1)
    .wraps(true)
    .full_range(0, 0x1FFF)
    .initial_range(0, 0)
    .auto_trigger_when(AutoTriggerWhen::EndingOn(0x1000))
    // TODO: Verify correct timing.
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();

// NTDEC 2722 and NTDEC 2752 PCB and imitations.
// Used for conversions of the Japanese version of Super Mario Bros. 2
// TODO: Test this mapper. The IRQ was broken last time checked, but potential fix was added, just not tested.
pub struct Mapper040 {
    irq_counter: ReloadDrivenCounter,
}

impl Mapper for Mapper040 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                mem.cpu_pinout.acknowledge_mapper_irq();
                self.irq_counter.disable();
            }
            0xA000..=0xBFFF => {
                self.irq_counter.enable();
                self.irq_counter.force_reload();
            }
            0xC000..=0xDFFF => { /* TODO: NTDEC 2752 outer bank register. Test ROM needed. */ }
            0xE000..=0xFFFF => {
                mem.set_prg_register(P0, value);
            }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        let tick_result = self.irq_counter.tick();
        if tick_result.triggered {
            mem.cpu_pinout.assert_mapper_irq();
        }

        // TODO: Verify if this is correct.
        if tick_result.wrapped {
            mem.cpu_pinout.acknowledge_mapper_irq();
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