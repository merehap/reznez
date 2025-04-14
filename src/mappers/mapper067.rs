use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Sunsoft-3
#[derive(Default)]
pub struct Mapper067 {
    irq_enabled: bool,
    irq_counter: u16,
    irq_load_low: bool,
}

impl Mapper for Mapper067 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => params.set_irq_pending(false),
            0x8800..=0x97FF => params.set_chr_register(C0, value & 0b0011_1111),
            0x9800..=0xA7FF => params.set_chr_register(C1, value & 0b0011_1111),
            0xA800..=0xB7FF => params.set_chr_register(C2, value & 0b0011_1111),
            0xB800..=0xC7FF => params.set_chr_register(C3, value & 0b0011_1111),
            0xC800..=0xD7FF => {
                if self.irq_load_low {
                    self.irq_counter &= 0xFF00;
                    self.irq_counter |= u16::from(value);
                } else {
                    // Load high byte.
                    self.irq_counter &= 0x00FF;
                    self.irq_counter |= u16::from(value) << 8;
                }

                self.irq_load_low = !self.irq_load_low;
            }
            0xD800..=0xE7FF => {
                self.irq_load_low = false;
                self.irq_enabled = value & 0b0001_0000 != 0;
            }
            0xE800..=0xF7FF => params.set_name_table_mirroring(value & 0b11),
            0xF800..=0xFFFF => params.set_prg_register(P0, value & 0b1111),
        }
    }

    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        if !self.irq_enabled {
            return;
        }

        self.irq_counter = self.irq_counter.wrapping_sub(1);
        if self.irq_counter == 0xFFFF {
            params.set_irq_pending(true);
            self.irq_enabled = false;
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
