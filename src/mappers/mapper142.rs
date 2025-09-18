use crate::mapper::*;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0)),
    ])
    .build();

// Kaiser KS202 (UNL-KS7032)
// Similar to VRC3.
// FIXME: Status bar isn't scrolling properly during intro.
#[derive(Default)]
pub struct Mapper142 {
    irq_enabled: bool,
    irq_counter: u16,
    irq_counter_reload_value: u16,

    selected_prg_bank: Option<PrgBankRegisterId>,
}

impl Mapper for Mapper142 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF => {
                self.irq_counter_reload_value &= 0x000F;
                self.irq_counter_reload_value |= u16::from(value) & 0xF;
            }
            0x9000..=0x9FFF => {
                self.irq_counter_reload_value &= 0x00F0;
                self.irq_counter_reload_value |= (u16::from(value) & 0xF) << 4;
            }
            0xA000..=0xAFFF => {
                self.irq_counter_reload_value &= 0x0F00;
                self.irq_counter_reload_value |= (u16::from(value) & 0xF) << 8;
            }
            0xB000..=0xBFFF => {
                self.irq_counter_reload_value &= 0xF000;
                self.irq_counter_reload_value |= (u16::from(value) & 0xF) << 12;
            }
            0xC000..=0xCFFF => {
                mem.cpu_pinout.clear_mapper_irq_pending();
                self.irq_enabled = value & 0b11 != 0;
                if self.irq_enabled {
                    self.irq_counter = self.irq_counter_reload_value;
                }
            }
            0xD000..=0xDFFF => {
                mem.cpu_pinout.clear_mapper_irq_pending();
            }
            0xE000..=0xEFFF => {
                match value & 0b111 {
                    0 | 5 | 7 => { /* Unknown behavior. TODO: Log this occurrence. */ }
                    // 0x8000
                    1 => self.selected_prg_bank = Some(P1),
                    // 0xA000
                    2 => self.selected_prg_bank = Some(P2),
                    // 0xC000
                    3 => self.selected_prg_bank = Some(P3),
                    // 0x6000
                    4 => self.selected_prg_bank = Some(P0),
                    6 => self.selected_prg_bank = None,
                    _ => unreachable!(),
                }
            }
            0xF000..=0xFFFF => {
                if let Some(selected_prg_bank) = self.selected_prg_bank {
                    mem.set_prg_register(selected_prg_bank, value & 0b1111);
                }
            }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if !self.irq_enabled {
            return;
        }

        // It's not clear if this is supposed to match VRC3's behavior or not. This is off-by-1.
        self.irq_counter += 1;
        if self.irq_counter == 0xFFFF {
            mem.cpu_pinout.set_mapper_irq_pending();
            self.irq_counter = self.irq_counter_reload_value;
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
