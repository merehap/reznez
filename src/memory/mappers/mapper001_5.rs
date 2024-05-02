use crate::memory::mapper::*;

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

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

// SEROM. MMC1 that doesn't support PRG bank switching.
pub struct Mapper001_5 {
    shift: u8,
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

        if value & 0b1000_0000 != 0 {
            self.shift = EMPTY_SHIFT_REGISTER;
            return;
        }

        let is_last_shift = self.shift & 1 == 1;

        self.shift >>= 1;
        self.shift |= (value & 1) << 4;

        if !is_last_shift {
            return;
        }

        let shift = self.shift;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => unreachable!(),
            0x8000..=0x9FFF => {
                params.set_chr_layout(Mapper001_5::next_chr_windows(shift));
                params.set_name_table_mirroring(Mapper001_5::next_mirroring(shift));
            }
            // FIXME: Handle cases for special boards.
            0xA000..=0xBFFF => params.set_bank_index_register(C0, shift),
            // FIXME: Handle cases for special boards.
            0xC000..=0xDFFF => params.set_bank_index_register(C1, shift),
            0xE000..=0xFFFF => {
                if shift & 0b1_0000 == 0 {
                    params.enable_work_ram(0x6000);
                } else {
                    params.disable_work_ram(0x6000);
                }
            }
        }

        self.shift = EMPTY_SHIFT_REGISTER;
    }
}

impl Mapper001_5 {
    pub fn new() -> Self {
        Self { shift: EMPTY_SHIFT_REGISTER }
    }

    fn next_chr_windows(value: u8) -> ChrLayout {
        match (value & 0b0001_0000) >> 4 {
            0 => CHR_LAYOUT_ONE_BIG,
            1 => CHR_LAYOUT_TWO_SMALL,
            _ => unreachable!(),
        }
    }

    fn next_mirroring(value: u8) -> NameTableMirroring {
        match value & 0b0000_0011 {
            0b00 => NameTableMirroring::OneScreenRightBank,
            0b01 => NameTableMirroring::OneScreenLeftBank,
            0b10 => NameTableMirroring::Vertical,
            0b11 => NameTableMirroring::Horizontal,
            _ => unreachable!(),
        }
    }
}
