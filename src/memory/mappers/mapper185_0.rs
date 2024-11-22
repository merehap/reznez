use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        // FIXME: Marked as RAM because it needs a status register, but it's actually ROM.
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(0).status_register(S0)),
    ])
    .ram_statuses(&[
        RamStatus::Disabled,
        RamStatus::ReadOnly,
    ])
    .build();

const DISABLED_CHR_ROM: u8 = 0;
const READABLE_CHR_ROM: u8 = 1;

// CNROM with CHR disable
pub struct Mapper185_0 {
    ppu_data_read_count: u8,
}

impl Mapper for Mapper185_0 {
    fn on_cpu_read(&mut self, params: &mut MapperParams, address: CpuAddress) {
        if address.to_raw() == 0x2007 {
            if self.ppu_data_read_count < 2 {
                params.set_ram_status(S0, DISABLED_CHR_ROM);
                self.ppu_data_read_count += 1;
            } else {
                params.set_ram_status(S0, READABLE_CHR_ROM);
            }
        }
    }

    fn write_to_cartridge_space(&mut self, _params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper185_0 {
    pub fn new() -> Self {
        Self { ppu_data_read_count: 0 }
    }
}
