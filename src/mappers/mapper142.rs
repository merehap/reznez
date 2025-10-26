use ux::u4;

use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
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
    .full_range(0, 0xFFFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::Wrapping)
    // TODO: Verify.
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();

// Kaiser KS202 (UNL-KS7032)
// Similar to VRC3.
// FIXME: Status bar isn't scrolling properly during intro.
pub struct Mapper142 {
    irq_counter: ReloadDrivenCounter,
    selected_prg_bank: Option<PrgBankRegisterId>,
}

impl Mapper for Mapper142 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => self.irq_counter.set_reload_value_lowest_nybble(u4::new(value & 0xF)),
            0x9000..=0x9FFF => self.irq_counter.set_reload_value_second_lowest_nybble(u4::new(value & 0xF)),
            0xA000..=0xAFFF => self.irq_counter.set_reload_value_second_highest_nybble(u4::new(value & 0xF)),
            0xB000..=0xBFFF => self.irq_counter.set_reload_value_highest_nybble(u4::new(value & 0xF)),
            0xC000..=0xCFFF => {
                mem.cpu_pinout.acknowledge_mapper_irq();

                let enabled = value != 0;
                self.irq_counter.set_enabled(enabled);
                if enabled {
                    self.irq_counter.force_reload();
                }
            }
            0xD000..=0xDFFF => {
                mem.cpu_pinout.acknowledge_mapper_irq();
            }
            0xE000..=0xEFFF => {
                match value & 0b111 {
                    0 | 5 | 7 => { /* Unknown behavior. TODO: Log this occurrence. */ }
                    // 0x8000
                    1 => self.selected_prg_bank = Some(P1),
                    // 0xA000
                    2 => self.selected_prg_bank = Some(P2),
                    // 0xC000
                    3 => self.selected_prg_bank = Some(P3),
                    // 0x6000
                    4 => self.selected_prg_bank = Some(P0),
                    6 => self.selected_prg_bank = None,
                    _ => unreachable!(),
                }
            }
            0xF000..=0xFFFF => {
                if let Some(selected_prg_bank) = self.selected_prg_bank {
                    mem.set_prg_register(selected_prg_bank, value & 0b1111);
                }
            }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if self.irq_counter.tick().triggered {
            mem.cpu_pinout.assert_mapper_irq();
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper142 {
    pub fn new() -> Self {
        Self { irq_counter: IRQ_COUNTER, selected_prg_bank: None }
    }
}
