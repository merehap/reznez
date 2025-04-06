use std::num::NonZeroU8;

use crate::memory::bank::bank::Bank;
use crate::memory::bank::bank_index::{BankConfiguration, BankRegisters};
use crate::memory::bank::page::{OuterPage, OuterPageTable};
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ciram::Ciram;
use crate::memory::raw_memory::{RawMemory, RawMemorySlice};
use crate::memory::window::{ChrLocation, ReadWriteStatusInfo, Window};
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

use super::ciram::CiramSide;

pub struct ChrMemory {
    layouts: Vec<ChrLayout>,
    layout_index: u8,
    max_pattern_table_index: u16,
    access_override: Option<AccessOverride>,
    rom_outer_banks: Option<OuterPageTable>,
    ram: Option<OuterPage>,
}

impl ChrMemory {
    pub fn new(
        layouts: Vec<ChrLayout>,
        layout_index: u8,
        align_large_chr_banks: bool,
        access_override: Option<AccessOverride>,
        outer_bank_count: NonZeroU8,
        rom: RawMemory,
        ram: RawMemory,
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

        // The page size for CHR ROM and CHR RAM appear to always match each other.
        let page_size = bank_size.expect("at least one CHR ROM or CHR RAM window");

        let max_pattern_table_index = layouts[0].max_window_index();
        for layout in &layouts {
            assert_eq!(layout.max_window_index(), max_pattern_table_index,
                "The max CHR window index must be the same between all layouts.");
        }

        let rom_outer_banks = OuterPageTable::new(rom, outer_bank_count, page_size, align_large_chr_banks);

        ChrMemory {
            layouts,
            layout_index,
            max_pattern_table_index,
            access_override,
            rom_outer_banks,
            ram: OuterPage::new(ram, page_size, align_large_chr_banks),
        }
    }

    pub fn rom_bank_configuration(&self) -> Option<BankConfiguration> {
        self.rom_outer_banks.as_ref().map(|rob| rob.bank_configuration())
    }

    pub fn ram_bank_configuration(&self) -> Option<BankConfiguration> {
        self.ram.as_ref().map(|ram| ram.bank_configuration())
    }

    #[inline]
    pub fn rom_bank_count(&self) -> Option<u16> {
        self.rom_bank_configuration().map(|c| c.bank_count())
    }

    pub fn bank_size(&self) -> u16 {
        self.rom_bank_configuration()
            .unwrap_or_else(|| self.ram_bank_configuration().unwrap())
            .bank_size()
    }

    pub fn window_count(&self) -> u8 {
        self.current_layout().windows().len().try_into().unwrap()
    }

    pub fn access_override(&self) -> Option<AccessOverride> {
        self.access_override
    }

    pub fn read_write_status_infos(&self) -> Vec<ReadWriteStatusInfo> {
        let mut ids = Vec::new();
        for layout in &self.layouts {
            ids.append(&mut layout.active_read_write_status_register_ids());
        }

        ids
    }

    pub fn peek(&self, registers: &BankRegisters, ciram: &Ciram, address: PpuAddress) -> u8 {
        let (chr_index, _) = self.address_to_chr_index(registers, address.to_u16());
        match chr_index {
            ChrIndex::Rom { page_number, index } => {
                assert_ne!(self.access_override, Some(AccessOverride::ForceRam));
                self.rom_outer_banks.as_ref().expect("ROM access attempted but ROM wasn't present.")
                    .current_outer_page().page(page_number).peek(index)
            }
            ChrIndex::Ram { page_number, index } => {
                assert_ne!(self.access_override, Some(AccessOverride::ForceRom));
                self.ram.as_ref().unwrap().page(page_number).peek(index)
            }
            ChrIndex::Ciram { side, index } => ciram.side(side)[index as usize],
        }
    }

    pub fn write(&mut self, registers: &BankRegisters, ciram: &mut Ciram, address: PpuAddress, value: u8) {
        if self.access_override == Some(AccessOverride::ForceRom) {
            return;
        }

        let (chr_index, writable) = self.address_to_chr_index(registers, address.to_u16());
        // FIXME: Remove access_override check here. It should have already been handled in address_to_chr_index().
        if writable || self.access_override == Some(AccessOverride::ForceRam) {
            match chr_index {
                ChrIndex::Rom {..} => {
                    panic!("CHR ROM cannot be writable.");
                }
                ChrIndex::Ram { page_number, index } => {
                    self.ram.as_mut().unwrap().page_mut(page_number).write(index, value);
                }
                ChrIndex::Ciram { side, index } => {
                    ciram.side_mut(side)[index as usize] = value;
                }
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

    pub fn set_chr_rom_outer_bank_index(&mut self, index: u8) {
        self.rom_outer_banks.as_mut().expect("CHR ROM must be present in order for the CHR outer bank to be set.")
            .set_outer_page_index(index);
    }

    pub fn pattern_table<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram, side: PatternTableSide) -> PatternTable<'a> {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks(registers, ciram)),
            PatternTableSide::Right => PatternTable::new(self.right_chunks(registers, ciram)),
        }
    }

    pub fn save_ram_1kib_page(&self, start: u32) -> &[u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        assert_eq!(self.ram.as_ref().unwrap().page_size().get(), 0x400, "Save RAM page size must be 0x400 in order to take a 1KiB slice.");
        let page_number = (start / 0x400).try_into().expect("Page number too large.");
        self.ram.as_ref().unwrap().page(page_number).as_raw_slice().try_into().unwrap()
    }

    pub fn save_ram_1kib_page_mut(&mut self, start: u32) -> &mut [u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        assert_eq!(self.ram.as_ref().unwrap().page_size().get(), 0x400, "Save RAM page size must be 0x400 in order to take a 1KiB slice.");
        let page_number = (start / 0x400).try_into().expect("Page number too large.");
        self.ram.as_mut().unwrap().page_mut(page_number).as_raw_mut_slice().try_into().unwrap()
    }

    fn address_to_chr_index(&self, registers: &BankRegisters, address: u16) -> (ChrIndex, bool) {
        assert!(address <= self.max_pattern_table_index);

        let mut window_and_bank_offset = None;
        for window in self.current_layout().windows() {
            if let Some(bank_offset) = window.offset(address) {
                window_and_bank_offset = Some((window, bank_offset));
                break;
            }
        }

        let (window, bank_offset) = window_and_bank_offset.unwrap();
        let location = window.resolved_bank_location(
            registers,
            window.location().unwrap(),
            self.rom_bank_configuration(),
            self.ram_bank_configuration(),
            self.access_override,
        );

        match location {
            ChrLocation::RomBankIndex(mut page_number) => {
                let mut index = bank_offset; 
                let page_size = self.rom_outer_banks.as_ref().unwrap().page_size().get();
                while index >= page_size {
                    page_number += 1;
                    index -= page_size;
                }

                (ChrIndex::Rom { page_number, index }, false)
            }
            ChrLocation::RamBankIndex(mut page_number) => {
                let mut index = bank_offset;
                let page_size = self.ram.as_ref().unwrap().page_size().get();
                while index >= page_size {
                    page_number += 1;
                    index -= page_size;
                }

                (ChrIndex::Ram { page_number, index }, window.is_writable(registers))
            }
            ChrLocation::Ciram(side) => {
                let index = bank_offset; 
                (ChrIndex::Ciram { side, index }, true)
            }
        }
    }

    #[inline]
    fn left_chunks<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        self.left_indexes(registers)
            .map(move |chr_index| {
                match chr_index {
                    ChrIndex::Rom { page_number, index } => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.rom_outer_banks.as_ref().unwrap()
                            .current_outer_page().page(page_number).as_raw_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrIndex::Ram { page_number, index } => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.ram.as_ref().unwrap()
                            .page(page_number).as_raw_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrIndex::Ciram { side, .. } => RawMemorySlice::from_raw(ciram.side(side)),
                }
        })
    }

    #[inline]
    fn right_chunks<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        self.right_indexes(registers)
            .map(move |chr_index| {
                match chr_index {
                    ChrIndex::Rom { page_number, index } => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.rom_outer_banks.as_ref().unwrap()
                            .current_outer_page().page(page_number).as_raw_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrIndex::Ram { page_number, index } => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.ram.as_ref().unwrap()
                            .page(page_number).as_raw_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrIndex::Ciram { side, .. } => RawMemorySlice::from_raw(ciram.side(side)),
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
    Rom { page_number: u16, index: u16 },
    Ram { page_number: u16, index: u16 },
    Ciram { side: CiramSide, index: u16 },
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AccessOverride {
    ForceRom,
    ForceRam,
}