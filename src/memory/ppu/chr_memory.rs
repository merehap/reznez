use crate::memory::bank::bank::Bank;
use crate::memory::bank::bank_index::BankRegisters;
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::raw_memory::{RawMemory, RawMemorySlice};
use crate::memory::window::Window;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

pub struct ChrMemory {
    layouts: Vec<ChrLayout>,
    layout_index: u8,
    bank_size: u16,
    align_large_chr_layouts: bool,
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

        let bank_size = bank_size.expect("at least one ROM or RAM window");

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
            override_write_protection,
            raw_memory,
        };

        let bank_count = chr_memory.bank_count();
        assert_eq!(u32::from(bank_count) * u32::from(chr_memory.bank_size), chr_memory.raw_memory.size() as u32);
        // Power of 2.
        assert_eq!(bank_count & (bank_count - 1), 0);

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

    pub fn peek(&self, registers: &BankRegisters, address: PpuAddress) -> u8 {
        let (index, _) = self.address_to_chr_index(registers, address.to_u16());
        self.raw_memory[index]
    }

    pub fn write(&mut self, registers: &BankRegisters, address: PpuAddress, value: u8) {
        let (index, writable) = self.address_to_chr_index(registers, address.to_u16());
        if writable || self.override_write_protection {
            self.raw_memory[index] = value;
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

    pub fn current_layout(&self) -> &ChrLayout {
        &self.layouts[self.layout_index as usize]
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn pattern_table(&self, registers: &BankRegisters, side: PatternTableSide) -> PatternTable {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks(registers)),
            PatternTableSide::Right => PatternTable::new(self.right_chunks(registers)),
        }
    }

    fn address_to_chr_index(&self, registers: &BankRegisters, address: u16) -> (u32, bool) {
        assert!(address < 0x2000);

        for window in self.current_layout().windows() {
            if let Some(bank_offset) = window.offset(address) {
                let raw_bank_index = window.resolved_bank_index(
                    registers,
                    window.location().unwrap(),
                    self.bank_size,
                    self.bank_count(),
                    self.align_large_chr_layouts,
                );
                let index = u32::from(raw_bank_index) *
                    u32::from(self.bank_size) +
                    u32::from(bank_offset);
                return (index, window.is_writable(registers));
            }
        }

        unreachable!();
    }

    #[inline]
    fn left_chunks(&self, registers: &BankRegisters) -> [RawMemorySlice; 4] {
        self.left_indexes(registers)
            .map(|index| self.raw_memory.slice(index..index + 0x400))
    }

    #[inline]
    fn right_chunks(&self, registers: &BankRegisters) -> [RawMemorySlice; 4] {
        self.right_indexes(registers)
            .map(|index| self.raw_memory.slice(index..index + 0x400))
    }

    #[inline]
    fn left_indexes(&self, registers: &BankRegisters) -> [u32; 4] {
        [
            self.address_to_chr_index(registers, 0x0000).0,
            self.address_to_chr_index(registers, 0x0400).0,
            self.address_to_chr_index(registers, 0x0800).0,
            self.address_to_chr_index(registers, 0x0C00).0,
        ]
    }

    #[inline]
    fn right_indexes(&self, registers: &BankRegisters) -> [u32; 4] {
        [
            self.address_to_chr_index(registers, 0x1000).0,
            self.address_to_chr_index(registers, 0x1400).0,
            self.address_to_chr_index(registers, 0x1800).0,
            self.address_to_chr_index(registers, 0x1C00).0,
        ]
    }
}
