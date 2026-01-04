use crate::mapper::*;
use crate::memory::memory::Bus;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Sunsoft-3 IRQ both auto-reloads (by wrapping around), and has its count set directly,
// rather through modifying a reload value and copying that to the count.
const IRQ_COUNTER: DirectlySetCounter = CounterBuilder::new()
    .step(-1)
    .wraps(true)
    .full_range(0, 0xFFFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::Wrapping)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_directly_set_counter();

// Sunsoft-3
pub struct Mapper067 {
    irq_counter: DirectlySetCounter,
    irq_load_low: bool,
}

impl Mapper for Mapper067 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => bus.cpu_pinout.acknowledge_mapper_irq(),
            0x8800..=0x97FF => bus.set_chr_register(C0, value & 0b0011_1111),
            0x9800..=0xA7FF => bus.set_chr_register(C1, value & 0b0011_1111),
            0xA800..=0xB7FF => bus.set_chr_register(C2, value & 0b0011_1111),
            0xB800..=0xC7FF => bus.set_chr_register(C3, value & 0b0011_1111),
            0xE800..=0xF7FF => bus.set_name_table_mirroring(value & 0b11),
            0xF800..=0xFFFF => bus.set_prg_register(P0, value & 0b1111),

            0xC800..=0xD7FF => {
                if self.irq_load_low {
                    self.irq_counter.set_count_low_byte(value);
                } else {
                    self.irq_counter.set_count_high_byte(value);
                }

                self.irq_load_low = !self.irq_load_low;
            }
            0xD800..=0xE7FF => {
                self.irq_load_low = false;
                if value & 0b0001_0000 == 0 {
                    self.irq_counter.disable();
                } else {
                    self.irq_counter.enable();
                }
            }
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

impl Mapper067 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            irq_load_low: false,
        }
    }
}