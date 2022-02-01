use std::cell::RefCell;
use std::rc::Rc;

use arr_macro::arr;

use crate::cartridge::Cartridge;
use crate::memory::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::MappedArray;

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;
const EMPTY_PRG_BANK: [u8; 0x4000] = [0; 0x4000];

// SxROM (MMC1)
pub struct Mapper1 {
    shift: u8,
    control: Control,
    selected_chr_bank0: u8,
    selected_chr_bank1: u8,
    selected_prg_bank: u8,

    // 32 4KiB banks or 16 8KiB banks.
    raw_pattern_tables: [MappedArray<4>; 32],
    // 16 16KiB banks or 8 32KiB banks.
    prg_banks: [Rc<RefCell<[u8; 0x4000]>>; 16],
    prg_rom: MappedArray<32>,
    last_prg_bank_index: u8,
}

impl Mapper1 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper1, String> {
        let mut chr_chunk_iter = cartridge.chr_rom_half_chunks().into_iter();
        let raw_pattern_tables =
            arr![
                chr_chunk_iter
                    .next()
                    .map(|chunk| MappedArray::new(*chunk.clone()))
                    .unwrap_or(MappedArray::empty())
            ; 32];

        let mut prg_chunk_iter = cartridge.prg_rom_chunks().into_iter();
        let prg_banks = arr![Rc::new(RefCell::new(*prg_chunk_iter.next().unwrap_or(&Box::new(EMPTY_PRG_BANK)).clone())); 16];
        let mut prg_rom = MappedArray::empty();
        let last_prg_bank_index =  (cartridge.prg_rom_chunks().len() - 1) as u8;
        prg_rom.update_from_halves(
            prg_banks[0].clone(),
            prg_banks[usize::from(last_prg_bank_index)].clone(),
        );

        Ok(Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,
            control: Control::new(),
            selected_chr_bank0: 0,
            selected_chr_bank1: 0,
            selected_prg_bank: 0,

            raw_pattern_tables,
            prg_banks,
            prg_rom,
            last_prg_bank_index,
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

    fn prg_bank_indexes(&self) -> (u8, u8) {
        let selected_bank = self.selected_prg_bank;
        match self.control.prg_bank_mode {
            PrgBankMode::Large => {
                let first_index = selected_bank & 0b0000_1110;
                (first_index, first_index + 1)
            },
            PrgBankMode::FixedFirst => (0, selected_bank),
            PrgBankMode::FixedLast => (selected_bank, self.last_prg_bank_index),
        }
    }
}

impl Mapper for Mapper1 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.control.mirroring
    }

    fn prg_rom(&self) -> &MappedArray<32> {
        &self.prg_rom
    }

    #[inline]
    fn raw_pattern_table(
        &self,
        side: PatternTableSide,
    ) -> &MappedArray<4> {
        let (selected_bank0, selected_bank1) = self.chr_bank_indexes();

        match side {
            PatternTableSide::Left =>
                &self.raw_pattern_tables[selected_bank0 as usize],
            PatternTableSide::Right =>
                &self.raw_pattern_tables[selected_bank1 as usize],
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

        let (first_index, second_index) = self.prg_bank_indexes();
        self.prg_rom.update_from_halves(
            self.prg_banks[first_index as usize].clone(),
            self.prg_banks[second_index as usize].clone(),
        );
    }
}

#[derive(Debug)]
struct Control {
    chr_bank_mode: ChrBankMode,
    prg_bank_mode: PrgBankMode,
    mirroring: NameTableMirroring,
}

impl Control {
    fn new() -> Control {
        Control {
            chr_bank_mode: ChrBankMode::Large,
            prg_bank_mode: PrgBankMode::FixedLast,
            mirroring: NameTableMirroring::OneScreenRightBank,
        }
    }

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
                    (false, false) => NameTableMirroring::OneScreenRightBank,
                    (false, true ) => NameTableMirroring::OneScreenLeftBank,
                    (true , false) => NameTableMirroring::Vertical,
                    (true , true ) => NameTableMirroring::Horizontal,
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
