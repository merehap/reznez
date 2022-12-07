use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::mapped_array::Chunk;
use crate::util::unit::KIBIBYTE;

pub struct ChrMemory {
    raw_memory: Vec<u8>,
    bank_count: u8,
    bank_size: usize,
    windows: Vec<Window>,
}

impl ChrMemory {
    pub fn builder() -> ChrMemoryBuilder {
        ChrMemoryBuilder::new()
    }

    pub fn read(&self, address: PpuAddress) -> u8 {
        let index = self.address_to_chr_index(address);
        self.raw_memory[index]
    }

    pub fn write(&mut self, address: PpuAddress, value: u8) {
        let index = self.address_to_chr_index(address);
        self.raw_memory[index] = value;
    }

    pub fn selected_bank_indexes(&self) -> Vec<BankIndex> {
        self.windows.iter()
            .map(|window| window.bank_index())
            .collect()
    }

    pub fn switch_bank_at(&mut self, start: u16, mut new_bank_index: BankIndex) {
        // Power of 2.
        if self.bank_count & (self.bank_count - 1) == 0 {
            // Ignore irrelevant high bits. TODO: Make it work for non-powers-of-2.
            new_bank_index %= self.bank_count;
        }

        assert!(new_bank_index < self.bank_count);

        for window in &mut self.windows {
            if window.start.to_u16() == start {
                window.switch_bank(new_bank_index);
                return;
            }
        }

        for window in &mut self.windows {
            println!("Window: {:X} {:X}", window.start.to_u16(), window.end.to_u16());
        }

        panic!("No window exists at {:X?}", start);
    }

    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks()),
            PatternTableSide::Right => PatternTable::new(self.right_chunks()),
        }
    }

    pub fn chr_bank_chunks(&self) -> Vec<Vec<Chunk>> {
        Vec::new()
    }

    fn address_to_chr_index(&self, address: PpuAddress) -> usize {
        assert!(address.to_u16() < 0x2000);

        for window in &self.windows {
            if let Some(bank_offset) = window.offset(address) {
                return usize::from(window.bank_index()) *
                    usize::from(self.bank_size) +
                    usize::from(bank_offset);
            }
        }

        println!("CHR Address? {}", address);
        println!("Window {:X}-{:X}", self.windows[0].start.to_u16(), self.windows[0].end.to_u16());

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
            self.address_to_chr_index(PpuAddress::from_u16(0x0000)),
            self.address_to_chr_index(PpuAddress::from_u16(0x0400)),
            self.address_to_chr_index(PpuAddress::from_u16(0x0800)),
            self.address_to_chr_index(PpuAddress::from_u16(0x0C00)),
        ]
    }

    fn right_indexes(&self) -> [usize; 4] {
        [
            self.address_to_chr_index(PpuAddress::from_u16(0x1000)),
            self.address_to_chr_index(PpuAddress::from_u16(0x1400)),
            self.address_to_chr_index(PpuAddress::from_u16(0x1800)),
            self.address_to_chr_index(PpuAddress::from_u16(0x1C00)),
        ]
    }

    fn new(
        raw_memory: Vec<u8>,
        bank_count: u8,
        bank_size: usize,
        windows: Vec<Window>,
    ) -> ChrMemory {
        assert!(
            !raw_memory.is_empty(),
            "No CHR memory provided. Is this mapper missing 8 KiB CHR RAM defaulting?",
        );

        assert!(!windows.is_empty());

        println!("Count: {}, Size: {}, Len: {}", bank_count, bank_size, raw_memory.len());
        assert_eq!(usize::from(bank_count) * bank_size, raw_memory.len());

        assert_eq!(windows[0].start.to_u16(), 0x0000);
        assert_eq!(windows[windows.len() - 1].end.to_u16(), 0x1FFF);

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start.to_u16(),
                windows[i].end.to_u16() + 1,
            );
        }

        ChrMemory { raw_memory, bank_count, bank_size, windows }
    }
}

pub struct ChrMemoryBuilder {
    raw_memory: Option<Vec<u8>>,
    bank_count: Option<u8>,
    bank_size: Option<usize>,
    windows: Vec<Window>,
}

impl ChrMemoryBuilder {
    pub fn raw_memory(&mut self, raw_memory: Vec<u8>) -> &mut ChrMemoryBuilder {
        self.raw_memory = Some(raw_memory);
        self
    }

    pub fn bank_count(&mut self, bank_count: u8) -> &mut ChrMemoryBuilder {
        self.bank_count = Some(bank_count);
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
        let window = Window {
            start: PpuAddress::from_u16(start),
            end: PpuAddress::from_u16(end),
            chr_type,
        };

        assert!([1 * KIBIBYTE, 2 * KIBIBYTE, 4 * KIBIBYTE, 8 * KIBIBYTE].contains(&size));
        assert!(end > start);
        let size: u16 = size.try_into().unwrap();
        assert_eq!(end - start + 1, size);

        let bank_size = self.bank_size.unwrap() as u16;
        assert!(size % bank_size == 0 || bank_size % size == 0);

        self.windows.push(window);
        self
    }

    pub fn build(&mut self) -> ChrMemory {
        ChrMemory::new(
            self.raw_memory.clone().unwrap(),
            self.bank_count.unwrap(),
            self.bank_size.unwrap(),
            self.windows.clone(),
        )
    }

    fn new() -> ChrMemoryBuilder {
        ChrMemoryBuilder {
            raw_memory: None,
            bank_count: None,
            bank_size: None,
            windows: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Window {
    start: PpuAddress,
    end: PpuAddress,
    chr_type: ChrType,
}

impl Window {
    fn offset(self, address: PpuAddress) -> Option<u16> {
        if self.start.to_u16() <= address.to_u16() && address.to_u16() <= self.end.to_u16() {
            Some(address.to_u16() - self.start.to_u16())
        } else {
            None
        }
    }

    fn bank_index(self) -> u8 {
        self.chr_type.bank_index()
    }

    fn switch_bank(&mut self, new_bank_index: BankIndex) {
        self.chr_type.switch_bank(new_bank_index);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrType {
    Rom { bank_index: u8 },
    Ram { bank_index: u8 },
}

impl ChrType {
    fn bank_index(self) -> u8 {
        use ChrType::*;
        match self {
            Rom { bank_index } | Ram { bank_index } => bank_index,
        }
    }

    fn switch_bank(&mut self, new_bank_index: BankIndex) {
        use ChrType::*;
        match self {
            Rom {..} => *self = Rom { bank_index: new_bank_index },
            Ram {..} => *self = Ram { bank_index: new_bank_index },
        }
    }
}

type BankIndex = u8;
