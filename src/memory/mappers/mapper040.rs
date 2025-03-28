use ux::u12;

use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // TODO: Verify if this is the correct max size.
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(6)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(4)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(5)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(7)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .build();

// NTDEC 2722 and NTDEC 2752 PCB and imitations.
// Used for conversions of the Japanese version of Super Mario Bros. 2
#[derive(Default)]
pub struct Mapper040 {
    irq_enabled: bool,
    irq_counter: u12,
}

impl Mapper for Mapper040 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                params.set_irq_pending(false);
                self.irq_enabled = false;
            }
            0xA000..=0xBFFF => {
                params.set_irq_pending(true);
            }
            0xC000..=0xDFFF => { /* TODO: NTDEC 2752 outer bank register. Test ROM needed. */ }
            0xE000..=0xFFFF => {
                params.set_bank_register(P0, value)
            }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        if !self.irq_enabled {
            return;
        }

        self.irq_counter = self.irq_counter.wrapping_add(1.into());
        if self.irq_counter == 0.into() {
            params.set_irq_pending(true);
            self.irq_enabled = false;
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
