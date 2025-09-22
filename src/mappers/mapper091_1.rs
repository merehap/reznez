use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();


const HORIZONTAL: u8 = 0;
const VERTICAL: u8 = 1;

const IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .trigger_on(TriggerOn::AnyTransitionToZero)
    .auto_reload(false)
    .forced_reload_behavior(ForcedReloadBehavior::Immediate)
    .decrement_size(5)
    .build();

// J.Y. Company JY830623C and YY840238C
pub struct Mapper091_1 {
    irq_counter: DecrementingCounter,
    irq_sub_counter: u8,
}

impl Mapper for Mapper091_1 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xF007 {
            0x6000 => mem.set_chr_register(C0, value),
            0x6001 => mem.set_chr_register(C1, value),
            0x6002 => mem.set_chr_register(C2, value),
            0x6003 => mem.set_chr_register(C3, value),
            0x6004 => mem.set_name_table_mirroring(HORIZONTAL),
            0x6005 => mem.set_name_table_mirroring(VERTICAL),
            0x7000 => mem.set_prg_register(P0, value & 0b00001111),
            0x7001 => mem.set_prg_register(P1, value & 0b00001111),

            0x6006 => {
                self.irq_counter.set_reload_value_low_byte(value);
                self.irq_counter.force_reload();
            }
            0x6007 => {
                self.irq_counter.set_reload_value_high_byte(value);
            }
            0x7006 => {
                self.irq_counter.disable_triggering();
                mem.cpu_pinout.clear_mapper_irq_pending();
            }
            0x7007 => {
                self.irq_counter.enable_triggering();
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        // TODO: Check if this should call ticking_enabled() instead.
        if !self.irq_counter.triggering_enabled() {
            return;
        }

        // Only tick the actual IRQ counter every 4 cycles.
        self.irq_sub_counter += 1;
        if self.irq_sub_counter < 4 {
            return;
        }

        self.irq_sub_counter = 0;

        let should_trigger_irq = self.irq_counter.tick();
        if should_trigger_irq {
            // TODO: Is this commented-out reload necessary? Super Fighters 3 works the same without it.
            // SF3 is constantly force-reloading the IRQ counter, presumably because this isn't automatically done.
            // self.irq_counter = self.irq_counter_reload_value;
            mem.cpu_pinout.set_mapper_irq_pending();
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper091_1 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            irq_sub_counter: 0,
        }
    }
}