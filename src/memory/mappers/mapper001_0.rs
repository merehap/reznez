use crate::memory::mapper::*;

const PRG_LAYOUT_FIXED_LAST: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);
const PRG_LAYOUT_FIXED_FIRST: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::FIRST)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
]);
const PRG_LAYOUT_ONE_BIG: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
]);

// TODO: Not all boards support CHR RAM.
const CHR_LAYOUT_ONE_BIG: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::Switchable(Ram, C0)),
]);
const CHR_LAYOUT_TWO_SMALL: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::Switchable(Ram, C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::Switchable(Ram, C1)),
]);

// SxROM (MMC1)
pub struct Mapper001_0 {
    shift_register: ShiftRegister,
}

impl Mapper for Mapper001_0 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT_FIXED_LAST)
            .chr_max_bank_count(32)
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
            ShiftStatus::Clear => params.set_prg_layout(PRG_LAYOUT_FIXED_LAST),
            ShiftStatus::Continue => { /* Do nothing additional. */ }
            ShiftStatus::Done { finished_value } => match address.to_raw() {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    params.set_prg_layout(Mapper001_0::next_prg_windows(finished_value));
                    params.set_chr_layout(Mapper001_0::next_chr_windows(finished_value));
                    params.set_name_table_mirroring(Mapper001_0::next_mirroring(finished_value));
                }
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => params.set_bank_index_register(C0, finished_value),
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => params.set_bank_index_register(C1, finished_value),
                0xE000..=0xFFFF => {
                    params.set_bank_index_register(P0, finished_value & 0b0_1111);
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

impl Mapper001_0 {
    pub fn new() -> Self {
        Self { shift_register: ShiftRegister::new() }
    }

    fn next_prg_windows(value: u8) -> PrgLayout {
        match (value & 0b0000_1100) >> 2 {
            0b00 | 0b01 => PRG_LAYOUT_ONE_BIG,
            0b10 => PRG_LAYOUT_FIXED_FIRST,
            0b11 => PRG_LAYOUT_FIXED_LAST,
            _ => unreachable!(),
        }
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

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

pub struct ShiftRegister {
    value: u8,
}

impl ShiftRegister {
    pub fn new() -> Self {
        Self { value: EMPTY_SHIFT_REGISTER }
    }

    pub fn shift(&mut self, write_value: u8) -> ShiftStatus {
        if write_value & 0b1000_0000 != 0 {
            self.value = EMPTY_SHIFT_REGISTER;
            return ShiftStatus::Clear;
        }

        let is_last_shift = self.value & 1 == 1;
        self.value >>= 1;
        // Copy the last bit from write_value to the front of self.value.
        self.value |= (write_value & 1) << 4;

        if !is_last_shift {
            return ShiftStatus::Continue;
        }

        let finished_value = self.value;
        self.value = EMPTY_SHIFT_REGISTER;
        ShiftStatus::Done { finished_value }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ShiftStatus {
    Clear,
    Continue,
    Done { finished_value: u8 },
}
