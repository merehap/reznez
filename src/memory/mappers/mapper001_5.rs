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
    .override_initial_name_table_mirroring(NameTableMirroring::OneScreenRightBank)
    .name_table_mirrorings(&[
        NameTableMirroring::OneScreenRightBank,
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ])
    .build();

const RAM_STATUSES: [RamStatus; 2] =
    [
        RamStatus::ReadWrite,
        RamStatus::Disabled,
    ];

// SEROM. MMC1 that doesn't support PRG bank switching.
pub struct Mapper001_5 {
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_5 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        // Work RAM writes don't trigger any of the shifter logic.
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            params.write_prg(address, value);
            return;
        }

        match self.shift_register.shift(value) {
            ShiftStatus::Clear | ShiftStatus::Continue => { /* Do nothing additional. */ }
            ShiftStatus::Done { finished_value } => match address.to_raw() {
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
                    let fields = splitbits!(finished_value, "...s....");
                    params.set_ram_status(S0, RAM_STATUSES[fields.s as usize]);
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper001_5 {
    pub fn new() -> Self {
        Self { shift_register: ShiftRegister::new() }
    }
}
