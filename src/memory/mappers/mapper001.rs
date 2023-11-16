use crate::memory::mapper::*;

const PRG_WINDOWS_FIXED_LAST: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);
const PRG_WINDOWS_FIXED_FIRST: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::FIRST)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
]);
const PRG_WINDOWS_ONE_BIG: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
]);

// TODO: Not all boards support CHR RAM.
const CHR_WINDOWS_ONE_BIG: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::Switchable(Ram, C0)),
]);
const CHR_WINDOWS_TWO_SMALL: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::Switchable(Ram, C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::Switchable(Ram, C1)),
]);

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

// SxROM (MMC1)
pub struct Mapper001 {
    shift: u8,
}

impl Mapper for Mapper001 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS_FIXED_LAST)
            .chr_max_bank_count(32)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS_ONE_BIG)
            .name_table_mirroring_source(NameTableMirroring::OneScreenRightBank.to_source())
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        // Work RAM writes don't trigger any of the shifter logic.
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            params.prg_memory_mut().write(address, value);
            return;
        }

        if value & 0b1000_0000 != 0 {
            self.shift = EMPTY_SHIFT_REGISTER;
            params.prg_memory_mut().set_windows(PRG_WINDOWS_FIXED_LAST);
            return;
        }

        let is_last_shift = self.shift & 1 == 1;

        self.shift >>= 1;
        self.shift |= u8::from(value & 1) << 4;

        if is_last_shift {
            let shift = self.shift;
            match address.to_raw() {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    params.prg_memory_mut().set_windows(Mapper001::next_prg_windows(shift));
                    params.chr_memory_mut().set_windows(Mapper001::next_chr_windows(shift));
                    params.set_name_table_mirroring(Mapper001::next_mirroring(shift));
                }
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => params.chr_memory_mut().set_bank_index_register(C0, shift),
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => params.chr_memory_mut().set_bank_index_register(C1, shift),
                0xE000..=0xFFFF => {
                    params.prg_memory_mut().set_bank_index_register(P0, shift & 0b0_1111);
                    if shift & 0b1_0000 == 0 {
                        params.prg_memory_mut().enable_work_ram(0x6000);
                    } else {
                        params.prg_memory_mut().disable_work_ram(0x6000);
                    }
                }
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }
    }
}

impl Mapper001 {
    pub fn new() -> Self {
        Self { shift: EMPTY_SHIFT_REGISTER }
    }

    fn next_prg_windows(value: u8) -> PrgWindows {
        match (value & 0b0000_1100) >> 2 {
            0b00 | 0b01 => PRG_WINDOWS_ONE_BIG,
            0b10 => PRG_WINDOWS_FIXED_FIRST,
            0b11 => PRG_WINDOWS_FIXED_LAST,
            _ => unreachable!(),
        }
    }

    fn next_chr_windows(value: u8) -> ChrWindows {
        match (value & 0b0001_0000) >> 4 {
            0 => CHR_WINDOWS_ONE_BIG,
            1 => CHR_WINDOWS_TWO_SMALL,
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
