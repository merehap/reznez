use crate::memory::mapper::*;
use crate::memory::mappers::common::mmc1::{ShiftRegister, ShiftStatus};

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::FIRST)),
]);

const CHR_LAYOUT_ONE_BIG: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
]);
const CHR_LAYOUT_TWO_SMALL: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
]);

const CHR_LAYOUTS: [ChrLayout; 2] =
    [CHR_LAYOUT_ONE_BIG, CHR_LAYOUT_TWO_SMALL];

const MIRRORINGS: [NameTableMirroring; 4] =
    [
        NameTableMirroring::OneScreenRightBank,
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ];

// SEROM. MMC1 that doesn't support PRG bank switching.
pub struct Mapper001_5 {
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_5 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(1)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(16)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT_ONE_BIG)
            .name_table_mirroring_source(NameTableMirroring::OneScreenRightBank.to_source())
            .build()
    }

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
                    let finished_value = usize::from(finished_value);
                    let mirroring_index =  finished_value & 0b0000_0011;
                    let chr_index       = (finished_value & 0b0001_0000) >> 4;
                    params.set_name_table_mirroring(MIRRORINGS[mirroring_index]);
                    params.set_chr_layout(CHR_LAYOUTS[chr_index]);
                }
                0xA000..=0xBFFF => params.set_bank_index_register(C0, finished_value),
                0xC000..=0xDFFF => params.set_bank_index_register(C1, finished_value),
                0xE000..=0xFFFF => {
                    if finished_value & 0b1_0000 == 0 {
                        params.enable_work_ram(0x6000);
                    } else {
                        params.disable_work_ram(0x6000);
                    }
                }
            }
        }
    }
}

impl Mapper001_5 {
    pub fn new() -> Self {
        Self { shift_register: ShiftRegister::new() }
    }
}