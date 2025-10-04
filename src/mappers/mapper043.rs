use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(80 * KIBIBYTE)
    .prg_layout(&[
        /* FIXME: Represent these Windows manually.
        PrgWindow::new(0x5000, 0x57FF, 2 * KIBIBYTE, PrgBank::ROM.fixed_index(8)),
        PrgWindow::new(0x5800, 0x5FFF, 2 * KIBIBYTE, PrgBank::MirrorOf(0x5000)),
        */
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(2)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(9)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .initial_count_and_reload_value(0)
    .step(1)
    .auto_triggered_by(AutoTriggeredBy::AlreadyOn, 0xFFF)
    .when_target_reached(WhenTargetReached::Reload)
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::TickingAndTriggering)
    .build_reload_driven_counter();

// TONY-I and YS-612 (FDS games in cartridge form).
// TODO: Untested. Need test ROM. In particular, the 0x5000 ROM window might not work.
// FIXME: PrgMemory under 0x6000 is no longer supported.
// This mapper will need to find a different way to support it.
pub struct Mapper043 {
    irq_counter: ReloadDrivenCounter,
}

impl Mapper for Mapper043 {
    fn peek_cartridge_space(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => ReadResult::OPEN_BUS,
            // Normally only PRG >= 0x6000 can be peeked.
            0x5000..=0xFFFF => mem.peek_prg(addr),
        }
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        const INDEXES: [u8; 8] = [4, 3, 4, 4, 4, 7, 5, 6];

        match *addr & 0x71FF {
            0x4022 => {
                // The bank index is scrambled for some reason.
                let index = INDEXES[usize::from(value & 0b111)];
                mem.set_prg_register(P0, index);
            }
            0x4122 | 0x8122 => {
                if value & 1 == 1 {
                    self.irq_counter.enable();
                } else {
                    self.irq_counter.disable();
                    // It's not clear that this is correct since the counter already wraps. A ROM test of the hardware is needed.
                    self.irq_counter.force_reload();
                    mem.cpu_pinout.acknowledge_mapper_irq();
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

        if tick_result.wrapped {
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


impl Mapper043 {
    pub fn new() -> Self {
        Self { irq_counter: IRQ_COUNTER }
    }
}