use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0xF)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0x8)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0x9)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0xB)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.fixed_index(0)),
    ])
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .initial_count_and_reload_value(0)
    .step(1)
    .auto_triggered_by(AutoTriggeredBy::StepSizedTransitionTo, 0x1000)
    // Verify that this is correct. The wiki, Disch, and Mesen all disagree in different ways.
    .when_target_reached(WhenTargetReached::ContinueThenReloadAfter(0x1FFF))
    // TODO: Verify correct timing.
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::TickingAndTriggering)
    .build_reload_driven_counter();

// N-32 conversion of Super Mario Bros. 2 (J). PCB code 761214.
pub struct Mapper050 {
    irq_counter: ReloadDrivenCounter,
}

impl Mapper for Mapper050 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0x4120 {
            0x4020 => {
                //println!("Setting PRG bank. Value: {value:b} . Address: 0x{cpu_address:04X}");
                let prg_bank = splitbits_then_combine!(value, "....hllm",
                                                              "0000hmll");

                let prg_bank2 = (value & 0x08) | ((value & 0x01) << 2) | ((value & 0x06) >> 1);
                assert_eq!(prg_bank, prg_bank2);

                mem.set_prg_register(P0, prg_bank);
            }
            0x4120 => {
                if value & 1 == 1 {
                    self.irq_counter.enable();
                } else {
                    mem.cpu_pinout.acknowledge_mapper_irq();
                    self.irq_counter.disable();
                    // TODO: Verify if this happens immediately or if it's delayed until the next tick.
                    self.irq_counter.force_reload();
                }
            }
            _ => { /* Do nothing. */ }
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

impl Mapper050 {
    pub fn new() -> Self {
        Self { irq_counter: IRQ_COUNTER }
    }
}