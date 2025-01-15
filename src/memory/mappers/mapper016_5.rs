use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
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
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::OneScreenRightBank,
    ])
    .build();

// LZ93D50 ASIC
// FIXME: Dragon Ball Z - Kyoushuu! Saiya Jin (J) freezes after joypad input. Possibly
// EEPROM-related since it's supposed to be mapper 159, not 16.
#[derive(Default)]
pub struct Mapper016_5 {
    irq_pending: bool,
    irq_counter_enabled: bool,
    irq_counter_latch: u16,
    irq_counter: u16,
}

impl Mapper for Mapper016_5 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address & 0x800F {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000 => params.set_bank_register(C0, value),
            0x8001 => params.set_bank_register(C1, value),
            0x8002 => params.set_bank_register(C2, value),
            0x8003 => params.set_bank_register(C3, value),
            0x8004 => params.set_bank_register(C4, value),
            0x8005 => params.set_bank_register(C5, value),
            0x8006 => params.set_bank_register(C6, value),
            0x8007 => params.set_bank_register(C7, value),
            0x8008 => params.set_bank_register(P0, value & 0b1111),
            0x8009 => params.set_name_table_mirroring(value & 0b11),
            0x800A => {
                self.irq_pending = false;
                self.irq_counter_enabled = value & 1 == 1;
                self.irq_counter = self.irq_counter_latch;
                if self.irq_counter_enabled && self.irq_counter == 0 {
                    self.irq_pending = true;
                }
            }
            0x800B => {
                // Set the low byte.
                self.irq_counter_latch &= 0b1111_1111_0000_0000;
                self.irq_counter_latch |= u16::from(value);
            }
            0x800C => {
                // Set the high byte.
                self.irq_counter_latch &= 0b0000_0000_1111_1111;
                self.irq_counter_latch |= u16::from(value) << 8;
            }
            0x800D => { /* TODO: Submapper 5 EEPROM Control. */ },
            0x800E..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.irq_counter_enabled && self.irq_counter > 0 {
            self.irq_counter -= 1;
            if self.irq_counter == 0 {
                self.irq_pending = true;
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
