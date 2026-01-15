use crate::mapper::*;
use crate::bus::Bus;

const LAYOUT: Layout = Layout::builder()
    .override_prg_bank_register(P1, 1)
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-2)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
    ])
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(-1)
    .wraps(false)
    .full_range(0, 0xFFFF)
    .initial_range(0, 0)
    .auto_trigger_when(AutoTriggerWhen::EndingOn(0))
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::Counting)
    .build_reload_driven_counter();

const CHR_REGISTER_IDS: [ChrBankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, C6, C7];

// Irem's H3001
pub struct Mapper065 {
    irq_counter: ReloadDrivenCounter,
}

impl Mapper for Mapper065 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }

            0x8000 => bus.set_prg_register(P0, value),
            0xA000 => bus.set_prg_register(P1, value),
            0xB000..=0xB007 => {
                let reg_id = CHR_REGISTER_IDS[usize::from(*addr - 0xB000)];
                bus.set_chr_register(reg_id, value);
            }
            0x9000 => bus.set_prg_layout(value >> 7),
            0x9001 => bus.set_name_table_mirroring(value >> 6),

            0x9003 => {
                if value >> 7 == 1 {
                    self.irq_counter.enable();
                } else {
                    self.irq_counter.disable();
                }
            }
            0x9004 => {
                self.irq_counter.force_reload();
                bus.cpu_pinout.acknowledge_mapper_irq();
            }
            0x9005 => {
                self.irq_counter.set_reload_value_high_byte(value);
            }
            0x9006 => {
                self.irq_counter.set_reload_value_low_byte(value);
            }
            _ => { /* Do nothing. */ }
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

impl Mapper065 {
    pub fn new() -> Self {
        Self { irq_counter: IRQ_COUNTER }
    }
}