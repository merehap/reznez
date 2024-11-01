use ux::u12;

use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(80 * KIBIBYTE)
    .prg_layout(&[
        // The minimum PRG bank size is normally 8KiB. This is weird.
        Window::new(0x5000, 0x5FFF, 4 * KIBIBYTE, Bank::ROM.fixed_index(8)),
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(2)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(1)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(9)),
    ])
    .chr_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .build();

// TONY-I and YS-612 (FDS games in cartridge form).
// TODO: Untested. Need test ROM. In particular, the 0x5000 ROM window might not work.
pub struct Mapper043 {
    irq_enabled: bool,
    irq_pending: bool,
    irq_counter: u12,
}

impl Mapper for Mapper043 {
    fn peek_cartridge_space(&self, params: &MapperParams, address: CpuAddress) -> ReadResult {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => ReadResult::OPEN_BUS,
            // Normally only PRG >= 0x6000 can be peeked.
            0x5000..=0xFFFF => params.peek_prg(address),
        }
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        const INDEXES: [u8; 8] = [4, 3, 4, 4, 4, 7, 5, 6];

        match cpu_address.to_raw() & 0x71FF {
            0x4022 => {
                // The bank index is scrambled for some reason.
                let index = INDEXES[usize::from(value & 0b111)];
                params.set_bank_register(P0, index);
            }
            0x4122 | 0x8122 => {
                self.irq_enabled = value & 1 == 1;
                if !self.irq_enabled {
                    self.irq_pending = false;
                    self.irq_counter = 0.into();
                }
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.irq_enabled {
            self.irq_counter = self.irq_counter.wrapping_add(1.into());
            self.irq_pending = self.irq_counter == 0.into();
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper043 {
    pub fn new() -> Self {
        Self {
            irq_enabled: false,
            irq_pending: false,
            irq_counter: 0.into(),
        }
    }
}
