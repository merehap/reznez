use ux::u15;

use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// FDS games hacked into cartridge form.
// Unknown if subject to bus conflicts.
// FIXME: Bottom status bar scrolls when it should be stationary in Bio Miracle Bokutte Upa.
pub struct Mapper042 {
    chr_board: ChrBoard,
    irq_enabled: bool,
    irq_counter: u15,
}

impl Mapper for Mapper042 {
    fn init_mapper_params(&self, params: &mut MapperParams) {
        params.set_chr_layout(self.chr_board as u8);
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address & 0xE003 {
            0x8000 => params.set_chr_register(C0, value & 0b1111),
            0xE000 => params.set_bank_register(P0, value & 0b1111),
            0xE001 => {
                let mirroring = splitbits_named!(min=u8, value, "....m...");
                params.set_name_table_mirroring(mirroring);
            }
            0xE002 => {
                self.irq_enabled = splitbits_named!(value, "......e.");
                if !self.irq_enabled {
                    self.irq_counter = 0.into();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        if self.irq_enabled {
            self.irq_counter = self.irq_counter.wrapping_add(1.into());
            params.set_irq_pending(u16::from(self.irq_counter) >= 0x6000u16);
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper042 {
    pub fn new(chr_ram_size: u32) -> Self {
        const CHR_RAM_SIZE: u32 = 8 * KIBIBYTE;

        let chr_board = match chr_ram_size {
            0 => ChrBoard::SwitchableRom,
            CHR_RAM_SIZE => ChrBoard::FixedRam,
            _ => panic!("Bad CHR RAM size for mapper 42: {chr_ram_size}"),
        };

        Self {
            chr_board,
            irq_enabled: false,
            irq_counter: 0.into(),
        }
    }
}

#[derive(Clone, Copy)]
enum ChrBoard {
    // Ai Senshi Nicol, for example
    SwitchableRom = 0,
    // Bio Miracle Bokutte Upa, for example.
    FixedRam = 1,
}
