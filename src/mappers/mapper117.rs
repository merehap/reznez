use crate::mapper::*;
use crate::util::pattern_table_transition_detector::{PatternTableTransitionDetector, AllowedAddresses};

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(R)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(D)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(E)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(F)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(G)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(H)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(I)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(J)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .full_range(0, 255)
    .initial_range(0, 0)
    .step(-1)
    .wraps(false)
    .auto_trigger_when(AutoTriggerWhen::StepSizedTransitionTo(0))
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();

// Future Media
pub struct Mapper117 {
    irq_counter: ReloadDrivenCounter,
    transition_detector: PatternTableTransitionDetector,
}

impl Mapper for Mapper117 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x8000 => bus.set_prg_register(P, value),
            0x8001 => bus.set_prg_register(Q, value),
            0x8002 => bus.set_prg_register(R, value),

            0xA000 => bus.set_chr_register(C, value),
            0xA001 => bus.set_chr_register(D, value),
            0xA002 => bus.set_chr_register(E, value),
            0xA003 => bus.set_chr_register(F, value),
            0xA004 => bus.set_chr_register(G, value),
            0xA005 => bus.set_chr_register(H, value),
            0xA006 => bus.set_chr_register(I, value),
            0xA007 => bus.set_chr_register(J, value),

            0xC001 => self.irq_counter.set_reload_value(value),
            0xC002 => bus.cpu_pinout.acknowledge_mapper_irq(),
            0xC003 => self.irq_counter.force_reload(),
            0xD000 => bus.set_name_table_mirroring(value & 1),
            0xE000 => {
                bus.cpu_pinout.acknowledge_mapper_irq();
                self.irq_counter.set_enabled(value & 1 == 1);
            }

            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* No additional regs here. */ }
        }
    }

    fn on_ppu_address_change(&mut self, bus: &mut Bus, address: PpuAddress) {
        let transitioned_right = self.transition_detector.detect(address) == Some(PatternTableSide::Right);
        if transitioned_right && self.irq_counter.tick().triggered {
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

impl Mapper117 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            transition_detector: PatternTableTransitionDetector::new(AllowedAddresses::PatternTableOnly),
        }
    }
}