use crate::memory::mapper::*;
use crate::memory::mappers::common::mmc1::{ShiftRegister, ShiftStatus};

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .chr_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::ROM.switchable(C1)),
    ])
    .override_initial_name_table_mirroring(NameTableMirroring::ONE_SCREEN_RIGHT_BANK)
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .ram_statuses(&[
        RamStatus::ReadWrite,
        RamStatus::Disabled,
    ])
    .build();

// SEROM. MMC1 that doesn't support PRG bank switching.
#[derive(Default)]
pub struct Mapper001_5 {
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_5 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        if matches!(cpu_address, 0x6000..=0x7FFF) {
            // Work RAM writes don't trigger shifter logic.
            return;
        }

        match self.shift_register.shift(value) {
            ShiftStatus::Clear | ShiftStatus::Continue => { /* Do nothing additional. */ }
            ShiftStatus::Done { finished_value } => match cpu_address {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    let fields = splitbits!(min=u8, finished_value, "...c..mm");
                    params.set_chr_layout(fields.c);
                    params.set_name_table_mirroring(fields.m);
                }
                0xA000..=0xBFFF => params.set_bank_register(C0, finished_value),
                0xC000..=0xDFFF => params.set_bank_register(C1, finished_value),
                0xE000..=0xFFFF => {
                    let fields = splitbits!(min=u8, finished_value, "...s....");
                    params.set_ram_status(S0, fields.s);
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
