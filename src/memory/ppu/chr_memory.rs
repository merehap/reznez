use crate::memory::bank_index::{BankIndex, BankIndexRegisters, BankIndexRegisterId};
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::writability::Writability;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

pub struct ChrMemory {
    layout: ChrLayout,
    // Mainly (only?) used by MMC3 and variants.
    bank_index_registers: BankIndexRegisters,
    raw_memory: Vec<u8>,
}

impl ChrMemory {
    pub fn new(mut layout: ChrLayout, mut raw_memory: Vec<u8>) -> ChrMemory {
        // If no CHR data is provided, add 8KiB of CHR RAM.
        // This is the only instance where changing the ROM/RAM type after configuration time is
        // allowed.
        if raw_memory.is_empty() {
            raw_memory = vec![0; 8 * KIBIBYTE];
            for window in &mut layout.windows {
                window.chr_type.0 = Writability::Ram;
            }
        }

        let windows = &layout.windows;
        assert!(!windows.is_empty());

        assert_eq!(windows[0].start, 0x0000);
        assert_eq!(windows[windows.len() - 1].end, 0x1FFF);

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start,
                windows[i].end + 1,
            );
        }

        let bank_index_registers =
            BankIndexRegisters::new(&layout.active_register_ids());
        let chr_memory = ChrMemory { layout, bank_index_registers, raw_memory };

        let bank_count = chr_memory.bank_count();
        assert_eq!(usize::from(bank_count) * chr_memory.layout.bank_size, chr_memory.raw_memory.len());
        // Power of 2.
        assert_eq!(bank_count & (bank_count - 1), 0);
        assert!(bank_count <= chr_memory.layout.max_bank_count);

        chr_memory
    }

    #[inline]
    pub fn bank_count(&self) -> u16 {
        (self.raw_memory.len() / self.layout.bank_size)
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
        self.layout.windows.iter()
            .map(|window| window.bank_index().to_u16(&self.bank_index_registers, self.bank_count()))
            .collect()
    }

    pub fn window_at(&mut self, start: u16) -> &mut Window {
        for window in &mut self.layout.windows {
            if window.start == start {
                return window;
            }
        }

        panic!("No window exists at {:X?}", start);
    }

    pub fn set_layout(&mut self, layout: ChrLayout) {
        self.layout = layout;
    }

    pub fn set_bank_index_register(
        &mut self,
        id: BankIndexRegisterId,
        raw_bank_index: u16,
    ) {
        self.bank_index_registers.set(id, raw_bank_index);
    }

    pub fn pattern_table(&self, side: PatternTableSide) -> PatternTable {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks()),
            PatternTableSide::Right => PatternTable::new(self.right_chunks()),
        }
    }

    fn address_to_chr_index(&self, address: u16) -> (usize, bool) {
        assert!(address < 0x2000);

        for window in &self.layout.windows {
            if let Some(bank_offset) = window.offset(address) {
                let raw_bank_index = window.bank_index()
                    .to_u16(&self.bank_index_registers, self.bank_count());
                let index = usize::from(raw_bank_index) *
                    usize::from(self.layout.bank_size) +
                    usize::from(bank_offset);
                return (index, window.is_writable());
            }
        }

        unreachable!();
    }

    #[inline]
    fn left_chunks(&self) -> [&[u8]; 4] {
        self.left_indexes()
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    #[inline]
    fn right_chunks(&self) -> [&[u8]; 4] {
        self.right_indexes()
            .map(|index| &self.raw_memory[index..index + 0x400])
    }

    #[inline]
    fn left_indexes(&self) -> [usize; 4] {
        [
            self.address_to_chr_index(0x0000).0,
            self.address_to_chr_index(0x0400).0,
            self.address_to_chr_index(0x0800).0,
            self.address_to_chr_index(0x0C00).0,
        ]
    }

    #[inline]
    fn right_indexes(&self) -> [usize; 4] {
        [
            self.address_to_chr_index(0x1000).0,
            self.address_to_chr_index(0x1400).0,
            self.address_to_chr_index(0x1800).0,
            self.address_to_chr_index(0x1C00).0,
        ]
    }
}

#[derive(Clone)]
pub struct ChrLayout {
    max_bank_count: u16,
    bank_size: usize,
    windows: Vec<Window>,
}

impl ChrLayout {
    pub fn builder() -> ChrLayoutBuilder {
        ChrLayoutBuilder::new()
    }

    fn new(
        max_bank_count: u16,
        bank_size: usize,
        windows: Vec<Window>,
    ) -> ChrLayout {
        assert!(!windows.is_empty());

        assert_eq!(windows[0].start, 0x0000);
        assert_eq!(windows[windows.len() - 1].end, 0x1FFF);

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start,
                windows[i].end + 1,
            );
        }

        assert_eq!(max_bank_count & (max_bank_count - 1), 0);
        ChrLayout { max_bank_count, bank_size, windows }
    }

    fn active_register_ids(&self) -> Vec<BankIndexRegisterId> {
        self.windows.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}

pub struct ChrLayoutBuilder {
    max_bank_count: Option<u16>,
    bank_size: Option<usize>,
    windows: Vec<Window>,
}

impl ChrLayoutBuilder {
    pub fn max_bank_count(&mut self, max_bank_count: u16) -> &mut ChrLayoutBuilder {
        self.max_bank_count = Some(max_bank_count);
        self
    }

    pub fn bank_size(&mut self, bank_size: usize) -> &mut ChrLayoutBuilder {
        self.bank_size = Some(bank_size);
        self
    }

    pub fn add_window(
        &mut self,
        start: u16,
        end: u16,
        size: usize,
        chr_type: ChrType,
    ) -> &mut ChrLayoutBuilder {
        let bank_size = self.bank_size.unwrap();
        assert!(size % bank_size == 0 || bank_size % size == 0);

        self.windows.push(Window::new(start, end, size, chr_type));
        self
    }

    pub fn build(&mut self) -> ChrLayout {
        ChrLayout::new(
            self.max_bank_count.unwrap(),
            self.bank_size.unwrap(),
            self.windows.clone(),
        )
    }

    fn new() -> ChrLayoutBuilder {
        ChrLayoutBuilder {
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
    write_status: Option<WriteStatus>,
}

impl Window {
    pub fn switch_bank_to<Index>(&mut self, new_bank_index: Index)
    where Index: Into<BankIndex>
    {
        self.chr_type.switch_bank_to(new_bank_index.into());
    }

    fn new(start: u16, end: u16, size: usize, chr_type: ChrType) -> Window {
        assert!([1 * KIBIBYTE, 2 * KIBIBYTE, 4 * KIBIBYTE, 8 * KIBIBYTE].contains(&size));
        assert!(end > start);
        assert_eq!(end as usize - start as usize + 1, size);

        Window { start, end, chr_type, write_status: None }
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
        match (self.chr_type.0, self.write_status) {
            (Writability::Rom   , None) => false,
            (Writability::Ram   , None) => true,
            (Writability::RomRam, Some(WriteStatus::ReadOnly)) => false,
            (Writability::RomRam, Some(WriteStatus::Writable)) => true,
            _ => unreachable!(),
        }
    }

    fn register_id(self) -> Option<BankIndexRegisterId> {
        if let BankIndex::Register(id) = self.chr_type.1 {
            Some(id)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    fn make_writable(&mut self) {
        assert!(self.write_status.is_some(), "Only RamRom can have its WriteStatus changed.");
        self.write_status = Some(WriteStatus::Writable);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChrType(pub Writability, pub BankIndex);

impl ChrType {
    fn bank_index(self) -> BankIndex {
        self.1
    }

    fn switch_bank_to(&mut self, new_bank_index: BankIndex) {
        self.1 = new_bank_index;
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum WriteStatus {
    ReadOnly,
    Writable,
}
