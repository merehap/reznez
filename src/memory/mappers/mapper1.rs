use arr_macro::arr;

use crate::cartridge::Cartridge;
use crate::memory::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::{MappedArray, MappedArrayMut};

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;
const EMPTY_CHR_BANK: [u8; 0x1000] = [0; 0x1000];
const EMPTY_PRG_BANK: [u8; 0x4000] = [0; 0x4000];

// SxROM (MMC1)
pub struct Mapper1 {
    shift: u8,
    control: Control,
    selected_chr_bank0: u8,
    selected_chr_bank1: u8,
    selected_prg_bank: u8,

    // 32 4KiB banks or 16 8KiB banks.
    chr_banks: [Box<[u8; 0x1000]>; 32],
    // 16 16KiB banks or 8 32KiB banks.
    prg_banks: [Box<[u8; 0x4000]>; 16],
    last_prg_bank_index: u8,
}

impl Mapper1 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper1, String> {
        let mut chr_chunk_iter = cartridge.chr_rom_half_chunks().into_iter();
        let chr_banks = arr![chr_chunk_iter.next().unwrap_or(Box::new(EMPTY_CHR_BANK)).clone(); 32];

        let mut prg_chunk_iter = cartridge.prg_rom_chunks().into_iter();
        let prg_banks = arr![prg_chunk_iter.next().unwrap_or(&Box::new(EMPTY_PRG_BANK)).clone(); 16];

        Ok(Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,
            control: Control::from_u8(0),
            selected_chr_bank0: 0,
            selected_chr_bank1: 0,
            selected_prg_bank: 0,

            chr_banks,
            prg_banks,
            last_prg_bank_index: (cartridge.prg_rom_chunks().len() - 1) as u8,
        })
    }

    pub fn state_string(&self) -> String {
        format!("Shift: 0b{:05b}, Control: {:?}, CB0: {}, CB1: {}, PB: {}",
            self.shift, self.control, self.selected_chr_bank0,
            self.selected_chr_bank1, self.selected_prg_bank)
    }

    fn chr_bank_indexes(&self) -> (u8, u8) {
        match self.control.chr_bank_mode {
            ChrBankMode::Large => {
                let index = self.selected_chr_bank0 & 0b0001_1110;
                (index, index + 1)
            },
            ChrBankMode::TwoSmall =>
                (self.selected_chr_bank0, self.selected_chr_bank1),
        }
    }
}

impl Mapper for Mapper1 {
    fn prg_rom(&self) -> MappedArray<'_, 32> {
        let selected_bank = self.selected_prg_bank;
        let (first_index, second_index) =
            match self.control.prg_bank_mode {
                PrgBankMode::Large => {
                    let first_index = selected_bank & 0b0000_1110;
                    (first_index, first_index + 1)
                },
                PrgBankMode::FixedFirst => (0, selected_bank),
                PrgBankMode::FixedLast => (selected_bank, self.last_prg_bank_index),
            };

        MappedArray::from_halves(
            self.prg_banks[first_index as usize].as_ref(),
            self.prg_banks[second_index as usize].as_ref(),
        )
    }

    #[inline]
    fn raw_pattern_table(
        &self,
        side: PatternTableSide,
    ) -> MappedArray<'_, 4> {
        let (selected_bank0, selected_bank1) = self.chr_bank_indexes(); 

        match side {
            PatternTableSide::Left =>
                MappedArray::new(self.chr_banks[selected_bank0 as usize].as_ref()),
            PatternTableSide::Right =>
                MappedArray::new(self.chr_banks[selected_bank1 as usize].as_ref()),
        }
    }

    #[inline]
    fn raw_pattern_table_mut(
        &mut self,
        side: PatternTableSide,
    ) -> MappedArrayMut<'_, 4> {
        let (selected_bank0, selected_bank1) = self.chr_bank_indexes(); 
        match side {
            PatternTableSide::Left =>
                MappedArrayMut::new(self.chr_banks[selected_bank0 as usize].as_mut()),
            PatternTableSide::Right =>
                MappedArrayMut::new(self.chr_banks[selected_bank1 as usize].as_mut()),
        }
    }

    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        if get_bit(value, 0) {
            self.shift = EMPTY_SHIFT_REGISTER;
            return;
        }

        let is_last_shift = get_bit(self.shift, 7);

        self.shift >>= 1;
        self.shift |= (get_bit(value, 7) as u8) << 4;

        if is_last_shift {
            match address.to_raw() {
                0x0000..=0x401F => unreachable!("{}", address),
                0x4020..=0x7FFF => {/* Do nothing. */},
                0x8000..=0x9FFF => self.control = Control::from_u8(self.shift),
                0xA000..=0xBFFF => self.selected_chr_bank0 = self.shift,
                0xC000..=0xDFFF => self.selected_chr_bank1 = self.shift,
                0xE000..=0xFFFF => self.selected_prg_bank = self.shift,
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }

        println!("{}", self.state_string());

        if get_bit(self.selected_prg_bank, 3) {
            todo!("Bypassing PRG fixed bank logic not supported.");
        }
    }
}

#[derive(Debug)]
struct Control {
    chr_bank_mode: ChrBankMode,
    prg_bank_mode: PrgBankMode,
    mirroring: Mirroring,
}

impl Control {
    fn from_u8(value: u8) -> Control {
        Control {
            chr_bank_mode:
                if get_bit(value, 3) {
                    ChrBankMode::Large
                } else {
                    ChrBankMode::TwoSmall
                },
            prg_bank_mode:
                match (get_bit(value, 4), get_bit(value, 5)) {
                    (false, _    ) => PrgBankMode::Large,
                    (true , false) => PrgBankMode::FixedFirst,
                    (true , true ) => PrgBankMode::FixedLast,
                },
            mirroring:
                match (get_bit(value, 6), get_bit(value, 7)) {
                    (false, false) => Mirroring::LowerBankOneScreen,
                    (false, true ) => Mirroring::UpperBankOneScreen,
                    (true , false) => Mirroring::Vertical,
                    (true , true ) => Mirroring::Horizontal,
                },
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum ChrBankMode {
    Large,
    TwoSmall,
}

#[derive(PartialEq, Eq, Debug)]
enum PrgBankMode {
    Large,
    FixedFirst,
    FixedLast,
}

#[derive(Debug)]
enum Mirroring {
    LowerBankOneScreen,
    UpperBankOneScreen,
    Vertical,
    Horizontal,
}
