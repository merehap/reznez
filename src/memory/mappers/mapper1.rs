use crate::memory::mapper::*;
use crate::util::bit_util::get_bit;

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

const INITIAL_LAYOUT: InitialLayout = InitialLayout {
    prg_max_bank_count: 16,
    prg_bank_size: 16 * KIBIBYTE,
    prg_windows_by_board: &[(Board::Any, PRG_WINDOWS_FIXED_LAST)],

    chr_max_bank_count: 32,
    chr_bank_size: 4 * KIBIBYTE,
    chr_windows: CHR_WINDOWS_ONE_BIG,

    name_table_mirroring_source: NameTableMirroringSource::Direct(NameTableMirroring::OneScreenRightBank),
};

const PRG_WINDOWS_FIXED_LAST: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST)),
];
const PRG_WINDOWS_FIXED_FIRST: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::FIRST)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
];
const PRG_WINDOWS_ONE_BIG: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
];

// TODO: Not all boards support CHR RAM.
const CHR_WINDOWS_ONE_BIG: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Ram, BankIndex::Register(C0))),
];
const CHR_WINDOWS_TWO_SMALL: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType(Ram, BankIndex::Register(C0))),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType(Ram, BankIndex::Register(C1))),
];

// SxROM (MMC1)
pub struct Mapper1 {
    shift: u8,
    params: MapperParams,
}

impl Mapper for Mapper1 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        // Work RAM writes don't trigger any of the shifter logic.
        if matches!(address.to_raw(), 0x6000..=0x7FFF) {
            self.prg_memory_mut().write(address, value);
            return;
        }

        if get_bit(value, 0) {
            self.shift = EMPTY_SHIFT_REGISTER;
            self.prg_memory_mut().set_windows(PRG_WINDOWS_FIXED_LAST.clone());
            return;
        }

        let is_last_shift = get_bit(self.shift, 7);

        self.shift >>= 1;
        self.shift |= u8::from(get_bit(value, 7)) << 4;

        if is_last_shift {
            let shift = self.shift;
            match address.to_raw() {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => unreachable!(),
                0x8000..=0x9FFF => {
                    self.prg_memory_mut().set_windows(Mapper1::next_prg_windows(shift));
                    self.chr_memory_mut().set_windows(Mapper1::next_chr_windows(shift));
                    self.set_name_table_mirroring(Mapper1::next_mirroring(shift));
                }
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => self.chr_memory_mut().set_bank_index_register(C0, shift),
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => self.chr_memory_mut().set_bank_index_register(C1, shift),
                0xE000..=0xFFFF => {
                    self.prg_memory_mut().set_bank_index_register(P0, shift & 0b0_1111);
                    if shift & 0b1_0000 == 0 {
                        self.prg_memory_mut().enable_work_ram(0x6000);
                    } else {
                        self.prg_memory_mut().disable_work_ram(0x6000);
                    }
                }
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper1 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper1, String> {
        Ok(Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        })
    }

    fn next_prg_windows(value: u8) -> &'static [PrgWindow] {
        match (value & 0b0000_1100) >> 2 {
            0b00 | 0b01 => PRG_WINDOWS_ONE_BIG.clone(),
            0b10 => PRG_WINDOWS_FIXED_FIRST.clone(),
            0b11 => PRG_WINDOWS_FIXED_LAST.clone(),
            _ => unreachable!(),
        }
    }

    fn next_chr_windows(value: u8) -> &'static [ChrWindow] {
        match (value & 0b0001_0000) >> 4 {
            0 => CHR_WINDOWS_ONE_BIG.clone(),
            1 => CHR_WINDOWS_TWO_SMALL.clone(),
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
