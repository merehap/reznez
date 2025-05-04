use ux::u12;

use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(80 * KIBIBYTE)
    .prg_layout(&[
        /* FIXME: Represent these Windows manually.
        PrgWindow::new(0x5000, 0x57FF, 2 * KIBIBYTE, PrgBank::ROM.fixed_index(8)),
        PrgWindow::new(0x5800, 0x5FFF, 2 * KIBIBYTE, PrgBank::MirrorOf(0x5000)),
        */
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(2)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(9)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .build();

// TONY-I and YS-612 (FDS games in cartridge form).
// TODO: Untested. Need test ROM. In particular, the 0x5000 ROM window might not work.
// FIXME: PrgMemory under 0x6000 is no longer supported.
// This mapper will need to find a different way to support it.
#[derive(Default)]
pub struct Mapper043 {
    irq_enabled: bool,
    irq_counter: u12,
}

impl Mapper for Mapper043 {
    fn peek_cartridge_space(&self, params: &MapperParams, cpu_address: u16) -> ReadResult {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => ReadResult::OPEN_BUS,
            // Normally only PRG >= 0x6000 can be peeked.
            0x5000..=0xFFFF => params.peek_prg(cpu_address),
        }
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        const INDEXES: [u8; 8] = [4, 3, 4, 4, 4, 7, 5, 6];

        match cpu_address & 0x71FF {
            0x4022 => {
                // The bank index is scrambled for some reason.
                let index = INDEXES[usize::from(value & 0b111)];
                params.set_prg_register(P0, index);
            }
            0x4122 | 0x8122 => {
                self.irq_enabled = value & 1 == 1;
                if !self.irq_enabled {
                    params.set_irq_pending(false);
                    self.irq_counter = 0.into();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        if self.irq_enabled {
            self.irq_counter = self.irq_counter.wrapping_add(1.into());
            params.set_irq_pending(self.irq_counter == 0.into());
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
