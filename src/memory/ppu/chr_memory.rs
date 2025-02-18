use crate::memory::bank::bank::Bank;
use crate::memory::bank::bank_index::BankRegisters;
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ciram::Ciram;
use crate::memory::raw_memory::{RawMemory, RawMemorySlice};
use crate::memory::window::{ChrLocation, Window};
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

use super::ciram::CiramSide;

pub struct ChrMemory {
    layouts: Vec<ChrLayout>,
    layout_index: u8,
    bank_size: u16,
    align_large_chr_layouts: bool,
    max_pattern_table_index: u16,
    override_write_protection: bool,
    raw_memory: RawMemory,
}

impl ChrMemory {
    pub fn new(
        layouts: Vec<ChrLayout>,
        layout_index: u8,
        align_large_chr_layouts: bool,
        mut raw_memory: RawMemory,
    ) -> ChrMemory {
        let mut bank_size = None;
        for layout in &layouts {
            for window in layout.windows() {
                if matches!(window.bank(), Bank::Rom(..) | Bank::Ram(..)) {
                    if let Some(size) = bank_size {
                        bank_size = Some(std::cmp::min(window.size(), size));
                    } else {
                        bank_size = Some(window.size());
                    }
                }
            }
        }

        let bank_size = bank_size.expect("at least one CHR ROM or CHR RAM window");

        let max_pattern_table_index = layouts[0].max_window_index();
        for layout in &layouts {
            assert_eq!(layout.max_window_index(), max_pattern_table_index,
                "The max CHR window index must be the same between all layouts.");
        }

        // If no CHR data is provided, add 8KiB of CHR RAM and allow writing to read-only layouts.
        let mut override_write_protection = false;
        if raw_memory.is_empty() {
            raw_memory = RawMemory::new(8 * KIBIBYTE);
            override_write_protection = true;
        }

        let chr_memory = ChrMemory {
            layouts,
            layout_index,
            bank_size,
            align_large_chr_layouts,
            max_pattern_table_index,
            override_write_protection,
            raw_memory,
        };

        let bank_count = chr_memory.bank_count();
        assert_eq!(u32::from(bank_count) * u32::from(chr_memory.bank_size), chr_memory.raw_memory.size());
        // Power of 2. FIXME: What's the correct behavior when accessing the high banks? Open bus?
        // assert_eq!(bank_count & (bank_count - 1), 0, "Bank count ({bank_count}) must be a power of 2.");

        chr_memory
    }

    #[inline]
    pub fn bank_count(&self) -> u16 {
        (self.raw_memory.size() / u32::from(self.bank_size))
            .try_into()
            .expect("Way too many CHR banks.")
    }

    pub fn bank_size(&self) -> u16 {
        self.bank_size
    }

    pub fn align_large_layouts(&self) -> bool {
        self.align_large_chr_layouts
    }

    pub fn window_count(&self) -> u8 {
        self.current_layout().windows().len().try_into().unwrap()
    }

    pub fn peek(&self, registers: &BankRegisters, ciram: &Ciram, address: PpuAddress) -> u8 {
        let (chr_index, _) = self.address_to_chr_index(registers, address.to_u16());
        match chr_index {
            ChrIndex::Normal(index) => self.raw_memory[index],
            ChrIndex::Ciram(side, index, ) => ciram.side(side)[index as usize],
        }
    }

    pub fn write(&mut self, registers: &BankRegisters, ciram: &mut Ciram, address: PpuAddress, value: u8) {
        let (chr_index, writable) = self.address_to_chr_index(registers, address.to_u16());
        if writable || self.override_write_protection {
            match chr_index {
                ChrIndex::Normal(index) => self.raw_memory[index] = value,
                ChrIndex::Ciram(side, index, ) => ciram.side_mut(side)[index as usize] = value,
            }
        }
    }

    pub fn window_at(&self, start: u16) -> &Window {
        for window in self.current_layout().windows() {
            if window.start() == start {
                return window;
            }
        }

        panic!("No window exists at {start:X}");
    }

    pub fn layout_index(&self) -> u8 {
        self.layout_index
    }

    pub fn current_layout(&self) -> &ChrLayout {
        &self.layouts[self.layout_index as usize]
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn pattern_table<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram, side: PatternTableSide) -> PatternTable<'a> {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks(registers, ciram)),
            PatternTableSide::Right => PatternTable::new(self.right_chunks(registers, ciram)),
        }
    }

    fn address_to_chr_index(&self, registers: &BankRegisters, address: u16) -> (ChrIndex, bool) {
        assert!(address <= self.max_pattern_table_index);
        for window in self.current_layout().windows() {
            if let Some(bank_offset) = window.offset(address) {
                let location= window.resolved_bank_location(
                    registers,
                    window.location().unwrap(),
                    self.bank_size,
                    self.bank_count(),
                    self.align_large_chr_layouts,
                );

                match location {
                    ChrLocation::BankIndex(raw_bank_index) => {
                        let index = u32::from(raw_bank_index) *
                            u32::from(self.bank_size) +
                            u32::from(bank_offset);
                        return (ChrIndex::Normal(index), window.is_writable(registers));
                    }
                    ChrLocation::Ciram(side ) => {
                        return (ChrIndex::Ciram(side, bank_offset), true);
                    }
                }
            }
        }

        unreachable!();
    }

    #[inline]
    fn left_chunks<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        self.left_indexes(registers)
            .map(move |chr_index| {
                match chr_index {
                    ChrIndex::Normal(index) => self.raw_memory.slice(index..index + 1 * KIBIBYTE),
                    ChrIndex::Ciram(side, ..) => RawMemorySlice::from_raw(ciram.side(side)),
                }
        })
    }

    #[inline]
    fn right_chunks<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        self.right_indexes(registers)
            .map(move |chr_index| {
                match chr_index {
                    ChrIndex::Normal(index) => self.raw_memory.slice(index..index + 1 * KIBIBYTE),
                    ChrIndex::Ciram(side, ..) => RawMemorySlice::from_raw(ciram.side(side)),
                }
        })
    }

    #[inline]
    fn left_indexes(&self, registers: &BankRegisters) -> [ChrIndex; 4] {
        [
            self.address_to_chr_index(registers, 0x0000).0,
            self.address_to_chr_index(registers, 0x0400).0,
            self.address_to_chr_index(registers, 0x0800).0,
            self.address_to_chr_index(registers, 0x0C00).0,
        ]
    }

    #[inline]
    fn right_indexes(&self, registers: &BankRegisters) -> [ChrIndex; 4] {
        [
            self.address_to_chr_index(registers, 0x1000).0,
            self.address_to_chr_index(registers, 0x1400).0,
            self.address_to_chr_index(registers, 0x1800).0,
            self.address_to_chr_index(registers, 0x1C00).0,
        ]
    }
}

pub enum ChrIndex {
    Normal(u32),
    Ciram(CiramSide, u16),
}
