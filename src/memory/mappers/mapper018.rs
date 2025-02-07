use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::ROM.switchable(C1)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C6)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C7)),
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
    irq_pending: bool,
    irq_counter: u16,
    irq_counter_mask: u16,
    irq_reload_value: u16,
}

impl Mapper for Mapper018 {
    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        // Disch: When enabled, the IRQ counter counts down every CPU cycle.
        //        When it wraps, an IRQ is generated.
        if self.irq_enabled {
            let mut new_counter = self.irq_counter & self.irq_counter_mask;
            if new_counter == 0 {
                self.irq_pending = true;
            }

            new_counter -= 1;
            set_bits(&mut self.irq_counter, new_counter, self.irq_counter_mask);
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        if matches!(cpu_address, 0x6000..=0x7FFF) {
            if self.work_ram_write_enabled {
                params.write_prg(cpu_address, value);
            }

            return;
        }

        let value = u16::from(value);
        match cpu_address & 0b1111_0000_0000_0011 {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => unreachable!(),
            0x8000 => params.set_bank_register_bits(P0, value     , 0b0000_1111),
            0x8001 => params.set_bank_register_bits(P0, value << 4, 0b0011_0000),
            0x8002 => params.set_bank_register_bits(P1, value     , 0b0000_1111),
            0x8003 => params.set_bank_register_bits(P1, value << 4, 0b0011_0000),
            0x9000 => params.set_bank_register_bits(P2, value     , 0b0000_1111),
            0x9001 => params.set_bank_register_bits(P2, value << 4, 0b0011_0000),
            0x9002 => self.work_ram_write_enabled = value & 0b0000_0010 != 0,
            0x9003 => { /* Do nothing */ }
            0xA000 => params.set_bank_register_bits(C0, value     , 0b0000_1111),
            0xA001 => params.set_bank_register_bits(C0, value << 4, 0b1111_0000),
            0xA002 => params.set_bank_register_bits(C1, value     , 0b0000_1111),
            0xA003 => params.set_bank_register_bits(C1, value << 4, 0b1111_0000),
            0xB000 => params.set_bank_register_bits(C2, value     , 0b0000_1111),
            0xB001 => params.set_bank_register_bits(C2, value << 4, 0b1111_0000),
            0xB002 => params.set_bank_register_bits(C3, value     , 0b0000_1111),
            0xB003 => params.set_bank_register_bits(C3, value << 4, 0b1111_0000),
            0xC000 => params.set_bank_register_bits(C4, value     , 0b0000_1111),
            0xC001 => params.set_bank_register_bits(C4, value << 4, 0b1111_0000),
            0xC002 => params.set_bank_register_bits(C5, value     , 0b0000_1111),
            0xC003 => params.set_bank_register_bits(C5, value << 4, 0b1111_0000),
            0xD000 => params.set_bank_register_bits(C6, value     , 0b0000_1111),
            0xD001 => params.set_bank_register_bits(C6, value << 4, 0b1111_0000),
            0xD002 => params.set_bank_register_bits(C7, value     , 0b0000_1111),
            0xD003 => params.set_bank_register_bits(C7, value << 4, 0b1111_0000),
            0xE000 => set_bits(&mut self.irq_reload_value, value      , 0b0000_0000_0000_1111),
            0xE001 => set_bits(&mut self.irq_reload_value, value <<  4, 0b0000_0000_1111_0000),
            0xE002 => set_bits(&mut self.irq_reload_value, value <<  8, 0b0000_1111_0000_0000),
            0xE003 => set_bits(&mut self.irq_reload_value, value << 12, 0b1111_0000_0000_0000),
            0xF000 => {
                self.irq_counter = self.irq_reload_value;
                self.irq_pending = false;
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
                self.irq_pending = false;
            }
            0xF002 => params.set_name_table_mirroring(value as u8 & 0b11),
            0xF003 => todo!("Expansion audio."),
            _ => unreachable!(),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

fn set_bits(value: &mut u16, new_bits: u16, mask: u16) {
    *value = (*value & !mask) | (new_bits & mask);
}
