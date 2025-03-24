use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        // FIXME: Marked as RAM because it needs a status register, but it's actually ROM.
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(0).status_register(S0)),
    ])
    .ram_statuses(&[
        RamStatus::Disabled,
        RamStatus::ReadOnly,
    ])
    .build();

// CNROM with copy protection
pub struct CnromWithChrDisable {
    correct_chip_select_value: u8,
}

impl Mapper for CnromWithChrDisable {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let copy_protection_passed = value & 0b11 == self.correct_chip_select_value;
                params.set_ram_status(S0, copy_protection_passed as u8);
            }
        }
    }

    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl CnromWithChrDisable {
    pub const fn new(correct_chip_select_value: u8) -> Self {
        Self { correct_chip_select_value }
    }
}
