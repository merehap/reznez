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

// J.Y. Company JY830623C and YY840238C
#[derive(Default)]
pub struct Mapper091_1 {
    irq_enabled: bool,
    irq_counter: u16,
    irq_counter_reload_value: u16,
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
            0x6006 => {
                self.irq_counter_reload_value = (self.irq_counter_reload_value & 0xFF00) | u16::from(value);
                self.irq_counter = self.irq_counter_reload_value;
            }
            0x6007 => {
                self.irq_counter_reload_value = (self.irq_counter_reload_value & 0x00FF) | (u16::from(value) << 8);
            }
            0x7000 => mem.set_prg_register(P0, value & 0b00001111),
            0x7001 => mem.set_prg_register(P1, value & 0b00001111),
            0x7006 => {
                self.irq_enabled = false;
                mem.cpu_pinout.clear_mapper_irq_pending();
            }
            0x7007 => {
                self.irq_enabled = true;
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if !self.irq_enabled {
            return;
        }

        // Only tick the actual IRQ counter every 4 cycles.
        self.irq_sub_counter += 1;
        if self.irq_sub_counter < 4 {
            return;
        }

        self.irq_sub_counter = 0;

        self.irq_counter = self.irq_counter.saturating_sub(5);

        if self.irq_counter == 0 {
            // TODO: Is this reload necessary? Super Fighters 3 works the same without it.
            // SF3 is constantly force-reloading the IRQ counter, presumably because this isn't automatically done.
            // self.irq_counter = self.irq_counter_reload_value;
            mem.cpu_pinout.set_mapper_irq_pending();
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
