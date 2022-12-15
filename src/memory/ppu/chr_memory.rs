use crate::memory::bank_index::BankIndex;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

pub struct ChrMemory {
    raw_memory: Vec<u8>,
    bank_size: usize,
    windows: Vec<Window>,
}

impl ChrMemory {
    pub fn builder() -> ChrMemoryBuilder {
        ChrMemoryBuilder::new()
    }

    pub fn bank_count(&self) -> u16 {
        (self.raw_memory.len() / self.bank_size)
            .try_into()
            .expect("Way too many CHR banks.")
    }

    pub fn read(&self, address: PpuAddress) -> u8 {
        let (index, _) = self.address_to_chr_index(address.to_u16());
        self.raw_memory[index]
    }

    pub fn write(&mut self, address: PpuAddress, value: u8) {
        let (index, writable) = self.address_to_chr_index(address.to_u16());
        if writable {
            self.raw_memory[index] = value;
        }
    }

    pub fn resolve_selected_bank_indexes(&self) -> Vec<u16> {
        self.windows.iter()
            .map(|window| window.bank_index().to_u16(self.bank_count()))
            .collect()
    }

    pub fn window_at(&mut self, start: u16) -> &mut Window {
        for window in &mut self.windows {
            if window.start == start {
                return window;
            }
        }

        panic!("No window exists at {:X?}", start);
    }

    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks()),
            PatternTableSide::Right => PatternTable::new(self.right_chunks()),
        }
    }

    fn address_to_chr_index(&self, address: u16) -> (usize, bool) {
        assert!(address < 0x2000);

        for window in &self.windows {
            if let Some(bank_offset) = window.offset(address) {
                let index = usize::from(window.bank_index().to_u16(self.bank_count())) *
                    usize::from(self.bank_size) +
                    usize::from(bank_offset);
                return (index, window.is_writable());
            }
        }

        unreachable!();
    }

    fn left_chunks(&self) -> [&[u8]; 4] {
        self.left_indexes()
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    fn right_chunks(&self) -> [&[u8]; 4] {
        self.right_indexes()
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    fn left_indexes(&self) -> [usize; 4] {
        [
            self.address_to_chr_index(0x0000).0,
            self.address_to_chr_index(0x0400).0,
            self.address_to_chr_index(0x0800).0,
            self.address_to_chr_index(0x0C00).0,
        ]
    }

    fn right_indexes(&self) -> [usize; 4] {
        [
            self.address_to_chr_index(0x1000).0,
            self.address_to_chr_index(0x1400).0,
            self.address_to_chr_index(0x1800).0,
            self.address_to_chr_index(0x1C00).0,
        ]
    }

    fn new(
        raw_memory: Vec<u8>,
        max_bank_count: u16,
        bank_size: usize,
        windows: Vec<Window>,
    ) -> ChrMemory {
        assert!(
            !raw_memory.is_empty(),
            "No CHR memory provided. Is this mapper missing 8 KiB CHR RAM defaulting?",
        );

        assert!(!windows.is_empty());

        assert_eq!(windows[0].start, 0x0000);
        assert_eq!(windows[windows.len() - 1].end, 0x1FFF);

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start,
                windows[i].end + 1,
            );
        }

        let chr_memory = ChrMemory { raw_memory, bank_size, windows };

        let bank_count = chr_memory.bank_count();
        assert_eq!(usize::from(bank_count) * bank_size, chr_memory.raw_memory.len());
        // Power of 2.
        assert_eq!(max_bank_count & (max_bank_count - 1), 0);
        assert_eq!(bank_count & (bank_count - 1), 0);
        assert!(bank_count <= max_bank_count);

        chr_memory
    }
}

pub struct ChrMemoryBuilder {
    raw_memory: Option<Vec<u8>>,
    max_bank_count: Option<u16>,
    bank_size: Option<usize>,
    windows: Vec<Window>,
}

impl ChrMemoryBuilder {
    pub fn raw_memory(&mut self, raw_memory: Vec<u8>) -> &mut ChrMemoryBuilder {
        self.raw_memory = Some(raw_memory);
        self
    }

    pub fn max_bank_count(&mut self, max_bank_count: u16) -> &mut ChrMemoryBuilder {
        self.max_bank_count = Some(max_bank_count);
        self
    }

    pub fn bank_size(&mut self, bank_size: usize) -> &mut ChrMemoryBuilder {
        self.bank_size = Some(bank_size);
        self
    }

    pub fn add_window(
        &mut self,
        start: u16,
        end: u16,
        size: usize,
        chr_type: ChrType,
    ) -> &mut ChrMemoryBuilder {
        assert!([1 * KIBIBYTE, 2 * KIBIBYTE, 4 * KIBIBYTE, 8 * KIBIBYTE].contains(&size));
        assert!(end > start);
        let size: u16 = size.try_into().unwrap();
        assert_eq!(end - start + 1, size);

        let bank_size = self.bank_size.unwrap() as u16;
        assert!(size % bank_size == 0 || bank_size % size == 0);

        self.windows.push(Window { start, end, chr_type });
        self
    }

    pub fn add_default_ram_if_chr_data_missing(&mut self) -> ChrMemory {
        // If no CHR data is provided, add 8KiB of CHR RAM.
        if self.raw_memory.as_ref().unwrap().is_empty() {
            self.raw_memory = Some(vec![0; 8 * KIBIBYTE]);
            for window in &mut self.windows {
                window.make_writable();
            }
        }

        ChrMemory::new(
            self.raw_memory.clone().unwrap(),
            self.max_bank_count.unwrap(),
            self.bank_size.unwrap(),
            self.windows.clone(),
        )
    }

    fn new() -> ChrMemoryBuilder {
        ChrMemoryBuilder {
            raw_memory: None,
            max_bank_count: None,
            bank_size: None,
            windows: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Window {
    start: u16,
    end: u16,
    chr_type: ChrType,
}

impl Window {
    pub fn switch_bank_to(&mut self, new_bank_index: BankIndex) {
        self.chr_type.switch_bank_to(new_bank_index);
    }

    fn offset(self, address: u16) -> Option<u16> {
        if self.start <= address && address <= self.end {
            Some(address - self.start)
        } else {
            None
        }
    }

    fn bank_index(self) -> BankIndex {
        self.chr_type.bank_index()
    }

    fn is_writable(self) -> bool {
        self.chr_type.is_writable()
    }

    fn make_writable(&mut self) {
        self.chr_type.make_writable();
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrType {
    Rom(BankIndex),
    Ram(BankIndex),
}

impl ChrType {
    fn bank_index(self) -> BankIndex {
        use ChrType::*;
        match self {
            Rom(bank_index) | Ram(bank_index) => bank_index,
        }
    }

    fn switch_bank_to(&mut self, new_bank_index: BankIndex) {
        use ChrType::*;
        match self {
            Rom(_) => *self = Rom(new_bank_index),
            Ram(_) => *self = Ram(new_bank_index),
        }
    }

    fn is_writable(self) -> bool {
        matches!(self, ChrType::Ram(_))
    }

    fn make_writable(&mut self) {
        *self = ChrType::Ram(self.bank_index());
    }
}
