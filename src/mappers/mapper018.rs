use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Jaleco SS 88006
// TODO: Expansion Audio
// TODO: PRG RAM chip enable/disable (remove work_ram_write_enabled)
// TODO: Verify work_ram_write_enabled = false at power-on.
#[derive(Default)]
pub struct Mapper018 {
    work_ram_write_enabled: bool,

    irq_enabled: bool,
    irq_counter: u16,
    irq_counter_mask: u16,
    irq_reload_value: u16,
}

impl Mapper for Mapper018 {
    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        // Disch: When enabled, the IRQ counter counts down every CPU cycle.
        //        When it wraps, an IRQ is generated.
        if self.irq_enabled {
            let mut new_counter = self.irq_counter & self.irq_counter_mask;
            if new_counter == 0 {
                mem.cpu_pinout.set_mapper_irq_pending();
            }

            new_counter -= 1;
            set_bits(&mut self.irq_counter, new_counter, self.irq_counter_mask);
        }
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if matches!(*addr, 0x6000..=0x7FFF) {
            if self.work_ram_write_enabled {
                mem.write_prg(addr, value);
            }

            return;
        }

        let value = u16::from(value);
        match *addr & 0b1111_0000_0000_0011 {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => unreachable!(),
            0x8000 => mem.set_prg_bank_register_bits(P0, value     , 0b0000_1111),
            0x8001 => mem.set_prg_bank_register_bits(P0, value << 4, 0b0011_0000),
            0x8002 => mem.set_prg_bank_register_bits(P1, value     , 0b0000_1111),
            0x8003 => mem.set_prg_bank_register_bits(P1, value << 4, 0b0011_0000),
            0x9000 => mem.set_prg_bank_register_bits(P2, value     , 0b0000_1111),
            0x9001 => mem.set_prg_bank_register_bits(P2, value << 4, 0b0011_0000),
            0x9002 => self.work_ram_write_enabled = value & 0b0000_0010 != 0,
            0x9003 => { /* Do nothing */ }
            0xA000 => mem.set_chr_bank_register_bits(C0, value     , 0b0000_1111),
            0xA001 => mem.set_chr_bank_register_bits(C0, value << 4, 0b1111_0000),
            0xA002 => mem.set_chr_bank_register_bits(C1, value     , 0b0000_1111),
            0xA003 => mem.set_chr_bank_register_bits(C1, value << 4, 0b1111_0000),
            0xB000 => mem.set_chr_bank_register_bits(C2, value     , 0b0000_1111),
            0xB001 => mem.set_chr_bank_register_bits(C2, value << 4, 0b1111_0000),
            0xB002 => mem.set_chr_bank_register_bits(C3, value     , 0b0000_1111),
            0xB003 => mem.set_chr_bank_register_bits(C3, value << 4, 0b1111_0000),
            0xC000 => mem.set_chr_bank_register_bits(C4, value     , 0b0000_1111),
            0xC001 => mem.set_chr_bank_register_bits(C4, value << 4, 0b1111_0000),
            0xC002 => mem.set_chr_bank_register_bits(C5, value     , 0b0000_1111),
            0xC003 => mem.set_chr_bank_register_bits(C5, value << 4, 0b1111_0000),
            0xD000 => mem.set_chr_bank_register_bits(C6, value     , 0b0000_1111),
            0xD001 => mem.set_chr_bank_register_bits(C6, value << 4, 0b1111_0000),
            0xD002 => mem.set_chr_bank_register_bits(C7, value     , 0b0000_1111),
            0xD003 => mem.set_chr_bank_register_bits(C7, value << 4, 0b1111_0000),
            0xE000 => set_bits(&mut self.irq_reload_value, value      , 0b0000_0000_0000_1111),
            0xE001 => set_bits(&mut self.irq_reload_value, value <<  4, 0b0000_0000_1111_0000),
            0xE002 => set_bits(&mut self.irq_reload_value, value <<  8, 0b0000_1111_0000_0000),
            0xE003 => set_bits(&mut self.irq_reload_value, value << 12, 0b1111_0000_0000_0000),
            0xF000 => {
                self.irq_counter = self.irq_reload_value;
                mem.cpu_pinout.clear_mapper_irq_pending();
            }
            0xF001 => {
                if value & 0b0000_1000 != 0 {
                    self.irq_counter_mask = 0b0000_0000_0000_1111;
                } else if value & 0b0000_0100 != 0 {
                    self.irq_counter_mask = 0b0000_0000_1111_1111;
                } else if value & 0b0000_0010 != 0 {
                    self.irq_counter_mask = 0b0000_1111_1111_1111;
                } else { // Full IRQ counter will be used.
                    self.irq_counter_mask = 0b1111_1111_1111_1111;
                }

                self.irq_enabled = value & 0b0000_0001 != 0;
                mem.cpu_pinout.clear_mapper_irq_pending();
            }
            0xF002 => mem.set_name_table_mirroring(value as u8 & 0b11),
            0xF003 => todo!("Expansion audio."),
            _ => unreachable!(),
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(IrqCounterInfo { ticking_enabled: self.irq_enabled, triggering_enabled: self.irq_enabled, count: self.irq_counter })
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

fn set_bits(value: &mut u16, new_bits: u16, mask: u16) {
    *value = (*value & !mask) | (new_bits & mask);
}
