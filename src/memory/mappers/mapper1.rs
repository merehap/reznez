use arr_macro::arr;
use log::error;

use crate::cartridge::Cartridge;
use crate::memory::cpu::cartridge_space::{CartridgeSpace, PrgMemory, WindowType};
use crate::memory::cpu::cartridge_space::WindowStart::*;
use crate::memory::cpu::cartridge_space::WindowEnd::*;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::{Chunk, MappedArray};
use crate::util::unit::KIBIBYTE;

const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;
const PRG_RAM_START: CpuAddress = CpuAddress::new(0x6000);

// SxROM (MMC1)
pub struct Mapper1 {
    shift: u8,
    control: Control,
    selected_chr_bank0: u8,
    selected_chr_bank1: u8,
    selected_prg_bank: u8,

    // 32 4KiB banks or 16 8KiB banks.
    raw_pattern_tables: [RawPatternTable; 32],
    cartridge_space: CartridgeSpace,
    prg_ram: [u8; 0x2000],
    last_prg_bank_index: u8,
}

impl Mapper1 {
    pub fn new(cartridge: &Cartridge) -> Mapper1 {
        let mut chr_chunk_iter = cartridge.chr_rom_half_chunks().into_iter();
        let raw_pattern_tables = arr![
                chr_chunk_iter
                    .next()
                    .map(|chunk| MappedArray::new(chunk.clone()))
                    .unwrap_or(MappedArray::empty())
            ; 32];

        let prg_bank_count = cartridge.prg_rom_chunks().len().try_into()
            .expect("Way too many PRG ROM chunks.");
        let last_prg_bank_index = prg_bank_count - 1;

        // TODO: Allow Work RAM to be turned on.
        let prg_memory = PrgMemory::builder()
            .raw_memory(cartridge.prg_rom())
            .bank_count(prg_bank_count)
            .bank_size(16 * KIBIBYTE)
            .add_window(Ox6000, Ox7FFF,  8 * KIBIBYTE, WindowType::Empty)
            .add_window(Ox8000, OxBFFF, 16 * KIBIBYTE, WindowType::Rom { bank_index: 0 })
            .add_window(OxC000, OxFFFF, 16 * KIBIBYTE, WindowType::Rom { bank_index: last_prg_bank_index })
            .build();
        let cartridge_space = CartridgeSpace::new(prg_memory);

        Mapper1 {
            shift: EMPTY_SHIFT_REGISTER,
            control: Control::new(),
            selected_chr_bank0: 0,
            selected_chr_bank1: 0,
            selected_prg_bank: 0,

            raw_pattern_tables,
            cartridge_space,
            prg_ram: [0; 0x2000],
            last_prg_bank_index,
        }
    }

    fn chr_bank_indexes(&self) -> (u8, u8) {
        match self.control.chr_bank_mode {
            ChrBankMode::Large => {
                let index = self.selected_chr_bank0 & 0b0001_1110;
                (index, index + 1)
            }
            ChrBankMode::TwoSmall => (self.selected_chr_bank0, self.selected_chr_bank1),
        }
    }
}

impl Mapper for Mapper1 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.control.mirroring
    }

    fn cartridge_space(&self) -> &CartridgeSpace {
        &self.cartridge_space
    }

    // TODO: Verify if this is always true.
    #[inline]
    fn is_chr_writable(&self) -> bool {
        true
    }

    #[inline]
    fn raw_pattern_table(&self, side: PatternTableSide) -> &RawPatternTable {
        let (selected_bank0, selected_bank1) = self.chr_bank_indexes();
        match side {
            PatternTableSide::Left => &self.raw_pattern_tables[selected_bank0 as usize],
            PatternTableSide::Right => &self.raw_pattern_tables[selected_bank1 as usize],
        }
    }

    fn chr_bank_chunks(&self) -> Vec<Vec<Chunk>> {
        let mut chunks = Vec::with_capacity(32);
        for raw_pattern_table in &self.raw_pattern_tables {
            chunks.push(raw_pattern_table.to_chunks().to_vec());
        }

        chunks
    }

    fn read_prg_ram(&self, address: CpuAddress) -> u8 {
        if address >= PRG_RAM_START {
            self.prg_ram[address.to_usize() - PRG_RAM_START.to_usize()]
        } else {
            // Ignore lower PRG RAM space which is not supported by mapper 1.
            // FIXME: Open bus behavior here instead?
            0
        }
    }

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
                0x6000..=0x7FFF => {
                    self.prg_ram[address.to_usize() - PRG_RAM_START.to_usize()] = value;
                }
                0x8000..=0x9FFF => self.control = Control::from_u8(self.shift),
                // FIXME: Handle cases for special boards.
                0xA000..=0xBFFF => self.selected_chr_bank0 = self.shift,
                // FIXME: Handle cases for special boards.
                0xC000..=0xDFFF => self.selected_chr_bank1 = self.shift,
                0xE000..=0xFFFF => self.selected_prg_bank = self.shift,
            }

            self.shift = EMPTY_SHIFT_REGISTER;
        }

        if get_bit(self.selected_prg_bank, 3) {
            error!("Bypassing PRG fixed bank logic not supported.");
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

        self.cartridge_space.switch_prg_bank_at(Ox8000, left_index);
        self.cartridge_space.switch_prg_bank_at(OxC000, right_index);
    }

    // TODO: Generalize this across mappers.
    fn prg_rom_bank_string(&self) -> String {
        if let &[left_index, right_index] = &self.cartridge_space.selected_prg_bank_indexes()[..] {
            format!(
                "{} and {} [16, 16 KiB banks, mode: {:?}]",
                left_index, right_index, self.control.prg_bank_mode,
            )
        } else {
            unreachable!()
        }
    }

    fn chr_rom_bank_string(&self) -> String {
        let (selected_bank0, selected_bank1) = self.chr_bank_indexes();
        format!(
            "{} and {} [32, 4 KiB banks, mode: {:?}]",
            selected_bank0, selected_bank1, self.control.chr_bank_mode,
        )
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
