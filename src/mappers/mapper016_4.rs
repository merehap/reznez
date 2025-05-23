use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
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
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// FCG-1 ASIC
#[derive(Default)]
pub struct Mapper016_4 {
    irq_pending: bool,
    irq_counter_enabled: bool,
    irq_counter: u16,
}

impl Mapper for Mapper016_4 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address & 0xE00F {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000 => params.set_chr_register(C0, value),
            0x6001 => params.set_chr_register(C1, value),
            0x6002 => params.set_chr_register(C2, value),
            0x6003 => params.set_chr_register(C3, value),
            0x6004 => params.set_chr_register(C4, value),
            0x6005 => params.set_chr_register(C5, value),
            0x6006 => params.set_chr_register(C6, value),
            0x6007 => params.set_chr_register(C7, value),
            0x6008 => params.set_prg_register(P0, value & 0b1111),
            0x6009 => params.set_name_table_mirroring(value & 0b11),
            0x600A => {
                self.irq_pending = false;
                self.irq_counter_enabled = value & 1 == 1;
                if self.irq_counter_enabled && self.irq_counter == 0 {
                    self.irq_pending = true;
                }
            }
            0x600B => {
                // Set the low byte.
                self.irq_counter &= 0b1111_1111_0000_0000;
                self.irq_counter |= u16::from(value);
            }
            0x600C => {
                // Set the high byte.
                self.irq_counter &= 0b0000_0000_1111_1111;
                self.irq_counter |= u16::from(value) << 8;
            }
            0x600D..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _params: &mut MapperParams, _cycle: i64) {
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
