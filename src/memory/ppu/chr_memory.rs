use std::num::{NonZeroU16, NonZeroU8};

use crate::mapper::{NameTableMirroring, NameTableSource};
use crate::memory::bank::bank::{Bank, Location};
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

const CHR_SLOT_COUNT: usize = 12;

pub struct ChrMemoryMap {
    initial_layout: ChrLayout,
    // 0x0000 through 0x1FFF (2 pattern tables) and 0x2000 through 0x2FFF (4 name tables).
    page_mappings: [ChrMapping; CHR_SLOT_COUNT],
    page_ids: [ChrPageId; CHR_SLOT_COUNT],
}

impl ChrMemoryMap {
    pub fn new(
        initial_layout: ChrLayout,
        name_table_mirroring: NameTableMirroring,
        bank_size: NonZeroU16,
        regs: &BankRegisters,
        access_override: Option<AccessOverride>,
        align_large_windows: bool,
    ) -> Self {

        let bank_size = bank_size.get();
        assert_eq!(bank_size % 0x400, 0);
        let pages_per_bank = bank_size / 0x400;

        let mut page_mappings = Vec::with_capacity(CHR_SLOT_COUNT);

        let mut address = 0x0000;
        for window in initial_layout.windows() {
            assert_eq!(window.size().get() % bank_size, 0);
            let pages_per_window = window.size().get() / 0x400;
            let mut page_number_mask = 0b1111_1111_1111_1111;
            if align_large_windows {
                page_number_mask &= !(pages_per_window - 1);
            }

            let mut bank = window.bank();
            match access_override {
                None => {}
                Some(AccessOverride::ForceRom) => bank = bank.as_rom(),
                Some(AccessOverride::ForceRam) => bank = bank.as_ram(),
            }

            let mut page_offset = 0;
            while window.is_in_bounds(address) {
                let mapping = ChrMapping::Banked { bank, pages_per_bank, page_number_mask, page_offset };
                page_mappings.push(mapping);
                address += 0x400;
                page_offset += 1;
            }
        }

        assert_eq!(page_mappings.len(), 8);

        for quadrant in name_table_mirroring.quadrants() {
            let NameTableSource::Ciram(ciram_side) = quadrant else {
                panic!("Only CIRAM is supported so far.");
            };

            page_mappings.push(ChrMapping::Ciram(ciram_side));
        }

        let mut memory_map = Self {
            initial_layout,
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [ChrPageId::Rom(0); CHR_SLOT_COUNT],
        };
        memory_map.update_page_ids(regs);

        memory_map
    }

    pub fn index_for_address(&self, address: PpuAddress) -> ChrMemoryIndex {
        let address = address.to_u16();
        match address {
            0x0000..=0x2FFF => {}
            0x3000..=0x3FFF => todo!(),
            0x4000..=0xFFFF => unreachable!(),
        }

        let mapping_index = address / (KIBIBYTE as u16);
        let offset = address % (KIBIBYTE as u16);

        let page_id = self.page_ids[mapping_index as usize];
        match page_id {
            ChrPageId::Rom(page_number) => ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE + u32::from(offset)),
            ChrPageId::Ram(page_number) => ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE + u32::from(offset)),
            ChrPageId::Ciram(side) => ChrMemoryIndex::Ciram(side, offset),
            ChrPageId::SaveRam => ChrMemoryIndex::SaveRam(offset),
            ChrPageId::ExtendedRam => ChrMemoryIndex::ExtendedRam(offset),
            ChrPageId::FillModeTile => ChrMemoryIndex::FillModeTile,
        }
    }

    pub fn set_name_table_mirroring(&mut self, name_table_mirroring: NameTableMirroring, regs: &BankRegisters) {
        for (i, &quadrant) in name_table_mirroring.quadrants().iter().enumerate() {
            let mapping = match quadrant {
                NameTableSource::Ciram(ciram_side) => ChrMapping::Ciram(ciram_side),
                NameTableSource::SaveRam(_) => ChrMapping::SaveRam,
                NameTableSource::ExtendedRam => ChrMapping::ExtendedRam,
                NameTableSource::FillModeTile => ChrMapping::FillModeTile,
            };
            self.page_mappings[8 + i] = mapping;
        }

        self.update_page_ids(regs);
    }

    pub fn update_page_ids(&mut self, regs: &BankRegisters) {
        for i in 0..CHR_SLOT_COUNT {
            self.page_ids[i] = self.page_mappings[i].page_id(regs);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrMemoryIndex {
    Rom(u32),
    Ram(u32),
    Ciram(CiramSide, u16),
    SaveRam(u16),
    ExtendedRam(u16),
    FillModeTile,
}

#[derive(Clone, Copy, Debug)]
pub enum ChrMapping {
    Banked {
        bank: Bank,
        pages_per_bank: u16,
        page_offset: u16,
        page_number_mask: u16,
    },
    Ciram(CiramSide),
    SaveRam,
    ExtendedRam,
    FillModeTile,
}

impl ChrMapping {
    pub fn page_id(&self, registers: &BankRegisters) -> ChrPageId {
        match self {
            Self::Banked { bank, pages_per_bank: bank_multiple, page_offset, page_number_mask, .. } => {
                let location = bank.location().expect("Location to be present in bank.");
                let bank_index = match location {
                    Location::Fixed(bank_index) => bank_index,
                    Location::Switchable(register_id) => registers.get(register_id).index().unwrap(),
                    Location::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id).index().unwrap(),
                };

                let page_number = ((bank_multiple * bank_index.to_raw()) & page_number_mask) + page_offset;
                match bank {
                    Bank::Rom(..) => ChrPageId::Rom(page_number),
                    Bank::Ram(..) => ChrPageId::Ram(page_number),
                    _ => todo!(),
                }
            }
            Self::Ciram(ciram_side) => ChrPageId::Ciram(*ciram_side),
            Self::SaveRam => ChrPageId::SaveRam,
            Self::ExtendedRam => ChrPageId::ExtendedRam,
            Self::FillModeTile => ChrPageId::FillModeTile,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrPageId {
    Rom(PageNumber),
    Ram(PageNumber),
    Ciram(CiramSide),
    SaveRam,
    ExtendedRam,
    FillModeTile,
}

type PageNumber = u16;

pub struct ChrMemory {
    layouts: Vec<ChrLayout>,
    memory_maps: Vec<ChrMemoryMap>,
    rom_outer_banks: Vec<RawMemory>,
    rom_outer_bank_index: u8,
    ram: RawMemory,

    layout_index: u8,

    max_pattern_table_index: u16,
    access_override: Option<AccessOverride>,

    old_rom_outer_banks: Option<OuterPageTable>,
    old_ram: Option<OuterPage>,
}

impl ChrMemory {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layouts: Vec<ChrLayout>,
        layout_index: u8,
        align_large_chr_banks: bool,
        access_override: Option<AccessOverride>,
        rom_outer_bank_count: NonZeroU8,
        rom: RawMemory,
        ram: RawMemory,
        name_table_mirroring: NameTableMirroring,
        bank_registers: &BankRegisters,
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
        let bank_size = bank_size.expect("at least one CHR ROM or CHR RAM window");

        let max_pattern_table_index = layouts[0].max_window_index();
        for layout in &layouts {
            assert_eq!(layout.max_window_index(), max_pattern_table_index,
                "The max CHR window index must be the same between all layouts.");
        }

        let rom_outer_banks = OuterPageTable::new(rom.clone(), rom_outer_bank_count, bank_size, align_large_chr_banks);

        let memory_maps = layouts.iter()
            .map(|layout| ChrMemoryMap::new(
                *layout,
                name_table_mirroring,
                bank_size,
                bank_registers,
                access_override,
                align_large_chr_banks,
            ))
            .collect();

        ChrMemory {
            layouts,
            memory_maps,
            layout_index,
            max_pattern_table_index,
            access_override,
            rom_outer_banks: rom.split_n(rom_outer_bank_count),
            rom_outer_bank_index: 0,
            ram: ram.clone(),
            old_rom_outer_banks: rom_outer_banks,
            old_ram: OuterPage::new(ram, bank_size, align_large_chr_banks),
        }
    }

    pub fn rom_bank_configuration(&self) -> Option<BankConfiguration> {
        self.old_rom_outer_banks.as_ref().map(|rob| rob.bank_configuration())
    }

    pub fn ram_bank_configuration(&self) -> Option<BankConfiguration> {
        self.old_ram.as_ref().map(|ram| ram.bank_configuration())
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
        let old_result = match chr_index {
            ChrIndex::Rom { page_number, index } => {
                assert_ne!(self.access_override, Some(AccessOverride::ForceRam));
                self.old_rom_outer_banks.as_ref().expect("ROM access attempted but ROM wasn't present.")
                    .current_outer_page().page(page_number).peek(index)
            }
            ChrIndex::Ram { page_number, index } => {
                assert_ne!(self.access_override, Some(AccessOverride::ForceRom));
                self.old_ram.as_ref().unwrap().page(page_number).peek(index)
            }
            ChrIndex::Ciram { side, index } => ciram.side(side)[index as usize],
        };

        let new_result = match self.current_memory_map().index_for_address(address) {
            ChrMemoryIndex::Rom(index) => {
                self.rom_outer_banks[self.rom_outer_bank_index as usize][index % self.rom_outer_banks[0].size()]
            },
            ChrMemoryIndex::Ram(index) => self.ram[index % self.ram.size()],
            ChrMemoryIndex::Ciram(side, index) => ciram.side(side)[index as usize],
            ChrMemoryIndex::SaveRam(_index) => todo!(),
            ChrMemoryIndex::ExtendedRam(_index) => todo!(),
            ChrMemoryIndex::FillModeTile => todo!(),
        };

        assert_eq!(old_result, new_result, "Peeks didn't match. Address: {address} Index: {:08X?}, CHR Index: {chr_index:?}",
            self.current_memory_map().index_for_address(address));

        new_result
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
                    self.old_ram.as_mut().unwrap().page_mut(page_number).write(index, value);
                }
                ChrIndex::Ciram { side, index } => {
                    ciram.side_mut(side)[index as usize] = value;
                }
            }

            match self.current_memory_map().index_for_address(address) {
                ChrMemoryIndex::Rom(_) => {}
                ChrMemoryIndex::Ram(index) => {
                    let size = self.ram.size();
                    self.ram[index % size] = value;
                }
                ChrMemoryIndex::Ciram(..) => {}
                ChrMemoryIndex::SaveRam(_index) => todo!(),
                ChrMemoryIndex::ExtendedRam(_index) => todo!(),
                ChrMemoryIndex::FillModeTile => todo!(),
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

    pub fn current_memory_map(&self) -> &ChrMemoryMap {
        &self.memory_maps[self.layout_index as usize]
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn set_chr_rom_outer_bank_index(&mut self, index: u8) {
        self.old_rom_outer_banks.as_mut().expect("CHR ROM must be present in order for the CHR outer bank to be set.")
            .set_outer_page_index(index);
        self.rom_outer_bank_index = index;
    }

    pub fn update_page_ids(&mut self, regs: &BankRegisters) {
        for page_mapping in &mut self.memory_maps {
            page_mapping.update_page_ids(regs);
        }
    }

    pub fn set_name_table_mirroring(&mut self, name_table_mirroring: NameTableMirroring, regs: &BankRegisters) {
        for page_mapping in &mut self.memory_maps {
            page_mapping.set_name_table_mirroring(name_table_mirroring, regs);
        }
    }

    pub fn pattern_table<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram, side: PatternTableSide) -> PatternTable<'a> {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks(registers, ciram)),
            PatternTableSide::Right => PatternTable::new(self.right_chunks(registers, ciram)),
        }
    }

    pub fn save_ram_1kib_page(&self, start: u32) -> &[u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        assert_eq!(self.old_ram.as_ref().unwrap().page_size().get(), 0x400, "Save RAM page size must be 0x400 in order to take a 1KiB slice.");
        let page_number = (start / 0x400).try_into().expect("Page number too large.");
        self.old_ram.as_ref().unwrap().page(page_number).as_raw_slice().try_into().unwrap()
    }

    pub fn save_ram_1kib_page_mut(&mut self, start: u32) -> &mut [u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        assert_eq!(self.old_ram.as_ref().unwrap().page_size().get(), 0x400, "Save RAM page size must be 0x400 in order to take a 1KiB slice.");
        let page_number = (start / 0x400).try_into().expect("Page number too large.");
        self.old_ram.as_mut().unwrap().page_mut(page_number).as_raw_mut_slice().try_into().unwrap()
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
                let page_size = self.old_rom_outer_banks.as_ref().unwrap().page_size().get();
                while index >= page_size {
                    page_number += 1;
                    index -= page_size;
                }

                (ChrIndex::Rom { page_number, index }, false)
            }
            ChrLocation::RamBankIndex(mut page_number) => {
                let mut index = bank_offset;
                let page_size = self.old_ram.as_ref().unwrap().page_size().get();
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
                        RawMemorySlice::from_raw(&self.old_rom_outer_banks.as_ref().unwrap()
                            .current_outer_page().page(page_number).as_raw_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrIndex::Ram { page_number, index } => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.old_ram.as_ref().unwrap()
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
                        RawMemorySlice::from_raw(&self.old_rom_outer_banks.as_ref().unwrap()
                            .current_outer_page().page(page_number).as_raw_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrIndex::Ram { page_number, index } => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.old_ram.as_ref().unwrap()
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

#[derive(Debug)]
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