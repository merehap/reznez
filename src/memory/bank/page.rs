use std::num::{NonZeroU16, NonZeroU32, NonZeroU8};

use crate::memory::bank::bank_index::BankConfiguration;
use crate::memory::raw_memory::RawMemory;

pub struct OuterPageTable {
    outer_pages: Vec<OuterPage>,
    // Redundant upon page_tables.len(), but has the correct type.
    outer_page_count: NonZeroU8,
    outer_page_index: u8,
}

impl OuterPageTable {
    pub fn new(memory: RawMemory, outer_page_count: NonZeroU8, inner_page_size: NonZeroU16, align_large_banks: bool) -> Option<Self> {
        let outer_pages: Vec<OuterPage> = memory.split_n(outer_page_count).into_iter()
            .map(|raw_outer_bank| OuterPage::new(raw_outer_bank, inner_page_size, align_large_banks).unwrap())
            .collect();

        if outer_pages.is_empty() {
            return None;
        }

        Some(Self {
            outer_pages,
            outer_page_count,
            outer_page_index: 0,
        })
    }

    pub fn outer_page_count(&self) -> NonZeroU8 {
        self.outer_page_count
    }

    pub fn outer_page_size(&self) -> NonZeroU32 {
        self.outer_pages[0].size()
    }

    pub fn current_outer_page(&self) -> &OuterPage {
        &self.outer_pages[self.outer_page_index as usize]
    }

    pub fn page_count(&self) -> NonZeroU16 {
        self.outer_pages[0].page_count()
    }

    pub fn page_size(&self) -> NonZeroU16 {
        self.outer_pages[0].page(0).size()
    }

    pub fn set_outer_page_index(&mut self, index: u8) {
        assert!(index < self.outer_page_count().get());
        self.outer_page_index = index;
    }

    pub fn bank_configuration(&self) -> BankConfiguration {
        BankConfiguration::new(self.page_size().get(), self.page_count().get(), self.outer_pages[0].align_large_banks)
    }
}

pub struct OuterPage {
    pages: Vec<Page>,
    // Redundant upon pages.len(), but this has the correct type.
    page_count: NonZeroU16,
    // Can be calculated from pages, but it would have to be calculated each time.
    size: NonZeroU32,
    align_large_banks: bool,
}

impl OuterPage {
    pub fn new(raw_outer_page: RawMemory, page_size: NonZeroU16, align_large_banks: bool) -> Option<Self> {
        if raw_outer_page.is_empty() {
            return None;
        }

        let expected_page_count;
        if raw_outer_page.size() % u32::from(page_size.get()) == 0 {
            expected_page_count = (raw_outer_page.size() / u32::from(page_size.get()))
                .try_into()
                .expect("Way too many banks.");
        } else if !raw_outer_page.is_empty() && u32::from(page_size.get()) % raw_outer_page.size() == 0 {
            expected_page_count = 1;
        } else {
            panic!("Bad PRG length: {} . Bank size: {} .", raw_outer_page.size(), page_size);
        }

        let size = NonZeroU32::new(raw_outer_page.size()).unwrap();
        let pages: Vec<Page> = raw_outer_page.chunks(page_size).into_iter()
            .map(Page::new)
            .collect();

        if pages.is_empty() {
            return None;
        }

        let page_count = NonZeroU16::new(pages.len().try_into().unwrap()).unwrap();
        assert_eq!(page_count.get(), expected_page_count);

        Some(Self { pages, page_count, size, align_large_banks })
    }

    pub fn size(&self) -> NonZeroU32 {
        self.size
    }

    pub fn page_size(&self) -> NonZeroU16 {
        self.page(0).size()
    }

    pub fn page_count(&self) -> NonZeroU16 {
        self.page_count
    }

    pub fn page(&self, page_number: u16) -> &Page {
        let page_number = page_number % self.page_count;
        &self.pages[page_number as usize]
    }

    pub fn page_mut(&mut self, page_number: u16) -> &mut Page {
        &mut self.pages[page_number as usize]
    }

    pub fn bank_configuration(&self) -> BankConfiguration {
        BankConfiguration::new(self.page_size().get(), self.page_count().get(), self.align_large_banks)
    }
}

pub struct Page {
    raw_page: RawMemory,
    size: NonZeroU16,
}

impl Page {
    pub fn new(raw_page: RawMemory) -> Self {
        assert!(raw_page.size() <= u32::from(u16::MAX));
        let size = NonZeroU16::new(raw_page.size().try_into().unwrap()).unwrap();
        Self { raw_page, size }
    }
    
    pub fn size(&self) -> NonZeroU16 {
        self.size
    }

    pub fn peek(&self, index: u16) -> u8 {
        self.raw_page[u32::from(index)]
    }

    pub fn write(&mut self, index: u16, value: u8) {
        self.raw_page[u32::from(index)] = value;
    }

    pub fn as_raw_slice(&self) -> &[u8] {
        self.raw_page.as_slice()
    }

    pub fn as_raw_mut_slice(&mut self) -> &mut [u8] {
        self.raw_page.as_mut_slice()
    }
}