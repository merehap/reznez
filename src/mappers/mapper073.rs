use ux::u4;

use crate::mapper::*;
use crate::memory::memory::Bus;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.fixed_index(0)),
    ])
    .fixed_name_table_mirroring()
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(1)
    .wraps(true)
    .full_range(0, 0xFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::Wrapping)
    // TODO: Verify.
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();

// VRC3
pub struct Mapper073 {
    low_irq_counter: ReloadDrivenCounter,
    high_irq_counter: ReloadDrivenCounter,
    irq_mode: IrqMode,
    irq_enabled_on_acknowledgement: bool,
}

impl Mapper for Mapper073 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => self.low_irq_counter.set_reload_value_lowest_nybble(u4::new(value & 0xF)),
            0x9000..=0x9FFF => self.low_irq_counter.set_reload_value_second_lowest_nybble(u4::new(value & 0xF)),
            0xA000..=0xAFFF => self.high_irq_counter.set_reload_value_lowest_nybble(u4::new(value & 0xF)),
            0xB000..=0xBFFF => self.high_irq_counter.set_reload_value_second_lowest_nybble(u4::new(value & 0xF)),
            0xC000..=0xCFFF => {
                bus.cpu_pinout.acknowledge_mapper_irq();

                let (byte_mode, enabled, enabled_on_acknowledgement) = splitbits_named!(value, ".....mea");
                self.irq_mode = if byte_mode { IrqMode::EightBit } else { IrqMode::SixteenBit };
                self.low_irq_counter.set_enabled(enabled);
                self.high_irq_counter.set_enabled(enabled);
                if enabled {
                    self.low_irq_counter.force_reload();
                    self.high_irq_counter.force_reload();
                }

                self.irq_enabled_on_acknowledgement = enabled_on_acknowledgement;
            }
            0xD000..=0xDFFF => {
                bus.cpu_pinout.acknowledge_mapper_irq();
                self.low_irq_counter.set_enabled(self.irq_enabled_on_acknowledgement);
                self.high_irq_counter.set_enabled(self.irq_enabled_on_acknowledgement);
            }
            0xE000..=0xEFFF => { /* Do nothing. */ }
            0xF000..=0xFFFF => bus.set_prg_register(P0, value & 0b111),
        }
    }

    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        let low_triggered = self.low_irq_counter.tick().triggered;
        if low_triggered {
            match self.irq_mode {
                IrqMode::EightBit => {
                    bus.cpu_pinout.assert_mapper_irq();
                }
                IrqMode::SixteenBit => {
                    let high_triggered = self.high_irq_counter.tick().triggered;
                    if high_triggered {
                        bus.cpu_pinout.assert_mapper_irq();
                    }
                }
            }
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        let low_count = self.low_irq_counter.to_irq_counter_info().count;
        let high_count = self.high_irq_counter.to_irq_counter_info().count;
        let count = match self.irq_mode {
            IrqMode::EightBit => low_count,
            IrqMode::SixteenBit => (high_count << 8) | low_count,
        };

        Some(IrqCounterInfo {
            count,
            .. self.low_irq_counter.to_irq_counter_info()
        })
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper073 {
    pub fn new() -> Self {
        Self {
            low_irq_counter: IRQ_COUNTER,
            high_irq_counter: IRQ_COUNTER,
            irq_mode: IrqMode::SixteenBit,
            irq_enabled_on_acknowledgement: false,
        }
    }
}

#[derive(Debug)]
enum IrqMode {
    SixteenBit,
    EightBit,
}
