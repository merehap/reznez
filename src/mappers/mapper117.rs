use crate::mapper::*;
use crate::util::edge_detector::EdgeDetector;

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
    .initial_range(0, 255)
    .initial_count(0)
    .step(-1)
    .wraps(false)
    .auto_trigger_when(AutoTriggerWhen::StepSizedTransitionTo(0))
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();

// Future Media
pub struct Mapper117 {
    irq_counter: ReloadDrivenCounter,
    pattern_table_side_detector: EdgeDetector<PatternTableSide>,
}

impl Mapper for Mapper117 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x8000..=0x8002 => {
                let reg_id = [P, Q, R][*addr as usize - 0x8000];
                bus.set_prg_register(reg_id, value);
            }
            0xA000..=0xA007 => {
                let reg_id = [C, D, E, F, G, H, I, J][*addr as usize - 0xA000];
                bus.set_chr_register(reg_id, value);
            }
            0xC001 => {
                //self.irq_counter.set_reload_value(value);
                self.irq_counter.force_reload();
            }
            0xC002 => {
                bus.cpu_pinout.acknowledge_mapper_irq();
            }
            0xC003 => {
                self.irq_counter.force_reload();
            }
            0xD000 => {
                bus.set_name_table_mirroring(value & 1);
            }
            0xE000 => {
                bus.cpu_pinout.acknowledge_mapper_irq();
                self.irq_counter.set_enabled(value & 1 == 1);
            }

            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* No additional regs here. */ }
        }
    }

    fn on_ppu_address_change(&mut self, bus: &mut Bus, address: PpuAddress) {
        if self.pattern_table_side_detector.set_value_then_detect(address.pattern_table_side())
                && self.irq_counter.tick().triggered {
            bus.cpu_pinout.assert_mapper_irq();
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper117 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Right),
        }
    }
}