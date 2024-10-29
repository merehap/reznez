use crate::memory::bank::bank::{Bank, Location};
use crate::memory::bank::bank_index::{BankIndex, BankRegisters, BankRegisterId};

use crate::memory::raw_memory::{RawMemory, RawMemorySlice};
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

pub struct ChrMemory {
    layouts: Vec<ChrLayout>,
    layout_index: u8,
    bank_size: u32,
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
            for window in layout.0 {
                if matches!(window.bank, Bank::Rom(..) | Bank::Ram(..)) {
                    if let Some(size) = bank_size {
                        bank_size = Some(std::cmp::min(window.size(), size));
                    } else {
                        bank_size = Some(window.size());
                    }
                }
            }
        }

        let bank_size = bank_size.expect("at least one ROM or RAM window");
        for layout in &layouts {
            layout.validate_bank_size_multiples(bank_size);
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
        ((self.raw_memory.size() as u32) / self.bank_size)
            .try_into()
            .expect("Way too many CHR banks.")
    }

    pub fn window_count(&self) -> u8 {
        self.current_layout().0.len().try_into().unwrap()
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

    pub fn resolve_selected_bank_indexes(&self, registers: &BankRegisters) -> Vec<u16> {
        self.current_layout().0.iter()
            .map(|window| window.bank_index(registers).to_u16(self.bank_count()))
            .collect()
    }

    pub fn window_at(&self, start: u16) -> &ChrWindow {
        for window in self.current_layout().0 {
            if window.start.to_u16() == start {
                return window;
            }
        }

        panic!("No window exists at {start:X?}");
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

        for window in self.current_layout().0 {
            if let Some(bank_offset) = window.offset(address) {
                let mut raw_bank_index = window.bank_index(registers).to_u32(self.bank_count());
                if self.align_large_chr_layouts {
                    let window_multiple = window.size() / self.bank_size;
                    raw_bank_index &= !(window_multiple >> 1);
                }

                let index: u32 = raw_bank_index *
                    self.bank_size +
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

#[derive(Clone, Copy)]
pub struct ChrLayout(&'static [ChrWindow]);

impl ChrLayout {
    pub const fn new(windows: &'static [ChrWindow]) -> ChrLayout {
        assert!(!windows.is_empty(), "No PRG layouts specified.");

        assert!(windows[0].start.to_u16() == 0x0000, "The first CHR window must start at 0x0000.");

        assert!(windows[windows.len() - 1].end.to_u16() == 0x1FFF, "The last CHR window must end at 0x1FFF.");

        let mut i = 1;
        while i < windows.len() {
            assert!(windows[i].start.to_u16() == windows[i - 1].end.to_u16() + 1,
                    "There must be no gaps nor overlap between CHR layouts.");

            i += 1;
        }

        ChrLayout(windows)
    }

    pub fn active_register_ids(&self) -> Vec<BankRegisterId> {
        self.0.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }

    const fn validate_bank_size_multiples(&self, bank_size: u32) {
        let mut i = 0;
        while i < self.0.len() {
            let window = self.0[i];
            assert!(window.size() % bank_size == 0, "Window size must be a multiple of bank size.");
            i += 1;
        }
    }
}

// TODO: Switch over to PpuAddress?
#[derive(Clone, Copy, Debug)]
pub struct ChrWindow {
    start: PpuAddress,
    end: PpuAddress,
    bank: Bank,
}

impl ChrWindow {
    #[allow(clippy::identity_op)]
    pub const fn new(start: u16, end: u16, size: u32, bank: Bank) -> ChrWindow {
        //assert!([1 * KIBIBYTE, 2 * KIBIBYTE, 4 * KIBIBYTE, 8 * KIBIBYTE].contains(&size));
        assert!(end > start);
        assert!(end as u32 - start as u32 + 1 == size);

        ChrWindow {
            start: PpuAddress::from_u16(start),
            end: PpuAddress::from_u16(end),
            bank,
        }
    }

    const fn size(self) -> u32 {
        (self.end.to_u16() - self.start.to_u16() + 1) as u32
    }

    fn offset(self, address: u16) -> Option<u16> {
        if self.start.to_u16() <= address && address <= self.end.to_u16() {
            Some(address - self.start.to_u16())
        } else {
            None
        }
    }

    fn bank_index(self, registers: &BankRegisters) -> BankIndex {
        match self.bank {
            Bank::Rom(Location::Fixed(bank_index)) | Bank::Ram(Location::Fixed(bank_index), _) =>
                bank_index,
            Bank::Rom(Location::Switchable(id)) | Bank::Ram(Location::Switchable(id), _) =>
                registers.get(id),
            Bank::Rom(Location::MetaSwitchable(meta_id)) | Bank::Ram(Location::MetaSwitchable(meta_id), _) =>
                registers.get_from_meta(meta_id),
            Bank::Empty | Bank::WorkRam(_) | Bank::MirrorOf(_) => unreachable!(),
        }
    }

    fn is_writable(self, registers: &BankRegisters) -> bool {
        self.bank.is_writable(registers)
    }

    pub fn register_id(self) -> Option<BankRegisterId> {
        if let Bank::Rom(Location::Switchable(id)) | Bank::Ram(Location::Switchable(id), _) = self.bank {
            Some(id)
        } else {
            None
        }
    }
}
