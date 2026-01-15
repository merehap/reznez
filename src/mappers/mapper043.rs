use crate::mapper::*;
use crate::bus::Bus;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(80 * KIBIBYTE)
    .prg_layout(&[
        // Layout doesn't support PRG banks below 0x6000, so this mapper implements the equivalent of the following manually:
        // PrgWindow::new(0x5000, 0x57FF, 2 * KIBIBYTE, PrgBank::ROM.fixed_index(8)),
        // PrgWindow::new(0x5800, 0x5FFF, 2 * KIBIBYTE, PrgBank::ROM.fixed_index(8)),
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(2)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(0)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(9)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .fixed_name_table_mirroring()
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(1)
    .wraps(true)
    .full_range(0, 0xFFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::Wrapping)
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();

// TONY-I and YS-612 (FDS games in cartridge form).
// TODO: Untested. Need test ROM.
pub struct Mapper043 {
    irq_counter: ReloadDrivenCounter,
}

impl Mapper for Mapper043 {
    fn peek_register(&self, bus: &Bus, addr: CpuAddress) -> ReadResult {
        match *addr {
            // Manually map PRG ROM bank #8 to this address range.
            0x5000..=0x5BFF => ReadResult::full(bus.prg_memory.peek_raw_rom(64 * KIBIBYTE + addr.to_u32() - 0x5000)),
            // A mirror of the same 2KiB of PRG ROM above.
            0x5C00..=0x5FFF => ReadResult::full(bus.prg_memory.peek_raw_rom(64 * KIBIBYTE + addr.to_u32() - 0x5C00)),
            _ => ReadResult::OPEN_BUS,
        }
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        const INDEXES: [u8; 8] = [4, 3, 4, 4, 4, 7, 5, 6];
        match *addr & 0x71FF {
            0x4022 => {
                // The bank index is scrambled for some reason.
                let index = INDEXES[usize::from(value & 0b111)];
                bus.set_prg_register(P0, index);
            }
            0x4122 | 0x8122 => {
                if value & 1 == 1 {
                    self.irq_counter.enable();
                } else {
                    self.irq_counter.disable();
                    // It's not clear that this is correct since the counter already wraps. A ROM test of the hardware is needed.
                    self.irq_counter.force_reload();
                    bus.cpu_pinout.acknowledge_mapper_irq();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        let tick_result = self.irq_counter.tick();
        if tick_result.triggered {
            bus.cpu_pinout.assert_mapper_irq();
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