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

const IRQ_COUNTER: IncrementingCounter = IncrementingCounterBuilder::new()
    .auto_triggered_by(IncAutoTriggeredBy::AlreadyOnTarget)
    .trigger_target(0x0FFF)
    // TODO: Verify this is correct. Disch's notes say it is, but it's contradicted here:
    // https://www.nesdev.org/wiki/INES_Mapper_050
    // Currently we disable IRQ manually when the target is hit. Will this continue ticking if IRQ
    // enable is called again with no IRQ acknowledge/clear in the mean time?
    .when_target_reached(WhenTargetReached::ContinueThenClearAfter(0xFFFF))
    .when_disabled_prevent(WhenDisabledPrevent::TickingAndTriggering)
    .build();

// N-32 conversion of Super Mario Bros. 2 (J). PCB code 761214.
pub struct Mapper050 {
    irq_counter: IncrementingCounter,
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
                    mem.cpu_pinout.clear_mapper_irq_pending();
                    self.irq_counter.disable();
                    // TODO: Verify if this happens immediately or if it's delayed until the next tick.
                    self.irq_counter.clear();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        let triggered = self.irq_counter.tick();
        if triggered {
            mem.cpu_pinout.set_mapper_irq_pending();
            // TODO: Verify if this is needed, or if the count should just stop instead.
            self.irq_counter.disable();
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