use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0xF)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0x8)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0x9)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0xB)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .build();

// N-32 conversion of Super Mario Bros. 2 (J). PCB code 761214.
#[derive(Default)]
pub struct Mapper050 {
    irq_enabled: bool,
    irq_counter: u16,
}

impl Mapper for Mapper050 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address & 0x4120 {
            0x4020 => {
                //println!("Setting PRG bank. Value: {value:b} . Address: 0x{cpu_address:04X}");
                let prg_bank = splitbits_then_combine!(value, "....hllm",
                                                              "0000hmll");

                let prg_bank2 = (value & 0x08) | ((value & 0x01) << 2) | ((value & 0x06) >> 1);
                assert_eq!(prg_bank, prg_bank2);

                //println!("\tActual value : {prg_bank:b}");
                params.set_bank_register(P0, prg_bank);
            }
            0x4120 => {
                //println!("Setting IRQ. Value: {value}");
                self.irq_enabled = value & 1 == 1;
                if !self.irq_enabled {
                    self.irq_counter = 0;
                    params.set_irq_pending(false);
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        if !self.irq_enabled {
            return;
        }

        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter == 0x1000 {
            params.set_irq_pending(true);
            self.irq_enabled = false;
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
