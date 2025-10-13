use crate::mapper::*;
use crate::counter::counter::WhenDisabledPrevent;
use crate::util::edge_detector::EdgeDetector;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_rom_outer_bank_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(1024 * KIBIBYTE)
    .chr_rom_outer_bank_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .build();

const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(-1)
    .wraps(false)
    .full_range(0, 64)
    .initial_count(64)
    .auto_trigger_when(AutoTriggerWhen::EndingOn(0))
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::Triggering)
    .build_reload_driven_counter();

// J.Y. Company JY830623C and YY840238C
pub struct Mapper091_0 {
    irq_counter: ReloadDrivenCounter,
    pattern_table_transition_detector: EdgeDetector<PatternTableSide>,
}

impl Mapper for Mapper091_0 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xF003 {
            0x6000 => mem.set_chr_register(C0, value),
            0x6001 => mem.set_chr_register(C1, value),
            0x6002 => mem.set_chr_register(C2, value),
            0x6003 => mem.set_chr_register(C3, value),
            0x7000 => mem.set_prg_register(P0, value & 0b00001111),
            0x7001 => mem.set_prg_register(P1, value & 0b00001111),
            0x7002 => {
                self.irq_counter.disable();
                mem.cpu_pinout.acknowledge_mapper_irq();
            }
            0x7003 => {
                self.irq_counter.enable();
                self.irq_counter.force_reload();
            }
            0x8000..=0x9FFF => {
                let outer_banks = splitbits!(min=u8, *addr, ".... .... .... .pcc");
                mem.set_prg_rom_outer_bank_number(outer_banks.p);
                mem.set_prg_rom_outer_bank_number(outer_banks.c);
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_ppu_address_change(&mut self, mem: &mut Memory, address: PpuAddress) {
        let should_tick = self.pattern_table_transition_detector.set_value_then_detect(address.pattern_table_side());
        if should_tick {
            if self.irq_counter.tick().triggered {
                mem.cpu_pinout.assert_mapper_irq();
            }
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper091_0 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            pattern_table_transition_detector: EdgeDetector::target_value(PatternTableSide::Right),
        }
    }
}