use crate::cartridge::Cartridge;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrType, AddDefaultRamIfRomMissing};
use crate::memory::cpu::prg_memory::{PrgMemory, PrgType};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::bit_util::get_bit;
use crate::util::unit::KIBIBYTE;

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

// SxROM (MMC1)
pub struct Mapper1 {
    shift: u8,
    control: Control,
    selected_chr_bank0: u8,
    selected_chr_bank1: u8,
    selected_prg_bank: u8,
    last_prg_bank_index: u8,

    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
}

impl Mapper1 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper1, String> {
        validate_chr_data_length(cartridge, |len| len <= 128 * KIBIBYTE)?;

        let bank_count = cartridge.prg_rom_chunks().len().try_into()
            .expect("Way too many PRG ROM chunks.");
        let last_prg_bank_index = bank_count - 1;

        // TODO: Allow Work RAM to be turned on/off.
        let prg_memory = PrgMemory::builder()
            .raw_memory(cartridge.prg_rom())
            .bank_count(bank_count)
            .bank_size(16 * KIBIBYTE)
            .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam)
            .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
            .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Rom { bank_index: last_prg_bank_index })
            .build();

        // TODO: Not all boards support CHR RAM.
        let chr_memory = ChrMemory::builder()
            .raw_memory(cartridge.chr_rom())
            .bank_count((2 * cartridge.chr_rom_chunks().len()).try_into().unwrap())
            .bank_size(4 * KIBIBYTE)
            .add_window(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType::Ram { bank_index: 0 })
            .add_window(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType::Ram { bank_index: 0 })
            .build(AddDefaultRamIfRomMissing::No);

        Ok(Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,
            control: Control::new(),
            selected_chr_bank0: 0,
            selected_chr_bank1: 0,
            selected_prg_bank: 0,
            prg_memory,
            last_prg_bank_index,
            chr_memory,
        })
    }
}

impl Mapper for Mapper1 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        if get_bit(value, 0) {
            self.shift = EMPTY_SHIFT_REGISTER;
            return;
        }

        let is_last_shift = get_bit(self.shift, 7);

        self.shift >>= 1;
        self.shift |= u8::from(get_bit(value, 7)) << 4;

        if is_last_shift {
            match address.to_raw() {
                0x0000..=0x401F => unreachable!(),
                0x4020..=0x5FFF => { /* Do nothing. */ }
                0x6000..=0x7FFF => self.prg_memory.write(address, value),
                0x8000..=0x9FFF => self.control = Control::from_u8(self.shift),
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => self.selected_chr_bank0 = self.shift,
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => self.selected_chr_bank1 = self.shift,
                0xE000..=0xFFFF => self.selected_prg_bank = self.shift,
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }

        let (left_bank, right_bank) = match self.control.chr_bank_mode {
            ChrBankMode::Large => {
                let index = self.selected_chr_bank0 & 0b0001_1110;
                (index, index + 1)
            }
            ChrBankMode::TwoSmall => (self.selected_chr_bank0, self.selected_chr_bank1),
        };

        self.chr_memory.switch_bank_at(0x0000, left_bank);
        self.chr_memory.switch_bank_at(0x1000, right_bank);

        if get_bit(self.selected_prg_bank, 3) {
            unimplemented!("Bypassing PRG fixed bank logic not supported.");
        }

        // Clear the high bit which is never used to change the PRG bank.
        self.selected_prg_bank &= 0b0_1111;

        let (left_index, right_index) = match self.control.prg_bank_mode {
            PrgBankMode::Large => {
                let left_index = self.selected_prg_bank & 0b0000_1110;
                (left_index, left_index + 1)
            }
            PrgBankMode::FixedFirst => (0, self.selected_prg_bank),
            PrgBankMode::FixedLast => (self.selected_prg_bank, self.last_prg_bank_index),
        };

        self.prg_memory.switch_bank_at(0x8000, left_index);
        self.prg_memory.switch_bank_at(0xC000, right_index);
    }

    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.control.mirroring
    }

    fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    fn chr_memory_mut(&mut self) -> &mut ChrMemory {
        &mut self.chr_memory
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

    #[rustfmt::skip]
    fn from_u8(value: u8) -> Control {
        Control {
            chr_bank_mode: if get_bit(value, 3) {
                ChrBankMode::TwoSmall
            } else {
                ChrBankMode::Large
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
