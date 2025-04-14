use std::num::{NonZeroU16, NonZeroU8};

use crate::mapper::{BankIndex, ChrBank, ChrBankRegisterId, ChrWindow, MetaRegisterId, NameTableMirroring, NameTableSource, ReadWriteStatus, ReadWriteStatusRegisterId};
use crate::memory::bank::bank::ChrBankLocation;
use crate::memory::bank::bank_index::ChrBankRegisters;
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ciram::Ciram;
use crate::memory::raw_memory::{RawMemory, RawMemorySlice};
use crate::memory::window::ReadWriteStatusInfo;
use crate::ppu::pattern_table::{PatternTable, PatternTableSide};
use crate::util::unit::KIBIBYTE;

use super::ciram::CiramSide;

const CHR_SLOT_COUNT: usize = 12;

pub struct ChrMemoryMap {
    // 0x0000 through 0x1FFF (2 pattern tables) and 0x2000 through 0x2FFF (4 name tables).
    page_mappings: [ChrMapping; CHR_SLOT_COUNT],
    page_ids: [(ChrPageId, ReadWriteStatus); CHR_SLOT_COUNT],
}

impl ChrMemoryMap {
    pub fn new(
        initial_layout: ChrLayout,
        name_table_mirroring: NameTableMirroring,
        bank_size: NonZeroU16,
        access_override: Option<AccessOverride>,
        align_large_windows: bool,
        regs: &ChrBankRegisters,
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
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [(ChrPageId::Rom(0), ReadWriteStatus::ReadOnly); CHR_SLOT_COUNT],
        };
        memory_map.update_page_ids(regs);

        memory_map
    }

    pub fn index_for_address(&self, address: PpuAddress) -> (ChrMemoryIndex, ReadWriteStatus) {
        let address = address.to_u16();
        match address {
            0x0000..=0x2FFF => {}
            0x3000..=0x3FFF => todo!(),
            0x4000..=0xFFFF => unreachable!(),
        }

        let mapping_index = address / (KIBIBYTE as u16);
        let offset = address % (KIBIBYTE as u16);

        let (page_id, read_write_status) = self.page_ids[mapping_index as usize];
        let chr_memory_index = match page_id {
            ChrPageId::Rom(page_number) => ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE + u32::from(offset)),
            ChrPageId::Ram(page_number) => ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE + u32::from(offset)),
            ChrPageId::Ciram(side) => ChrMemoryIndex::Ciram(side, offset),
            ChrPageId::SaveRam => ChrMemoryIndex::SaveRam(offset),
            ChrPageId::ExtendedRam => ChrMemoryIndex::ExtendedRam(offset),
            ChrPageId::FillModeTile => ChrMemoryIndex::FillModeTile,
        };

        (chr_memory_index, read_write_status)
    }

    pub fn pattern_table_page_ids(&self) -> &[(ChrPageId, ReadWriteStatus)] {
        &self.page_ids[0..8]
    }

    pub fn set_name_table_mirroring(&mut self, regs: &ChrBankRegisters, name_table_mirroring: NameTableMirroring) {
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

    pub fn update_page_ids(&mut self, regs: &ChrBankRegisters) {
        for i in 0..CHR_SLOT_COUNT {
            self.page_ids[i] = self.page_mappings[i].page_id(regs);
        }
    }

    fn page_start_index(&self, mapping_index: u8) -> ChrMemoryIndex {
        let page_id = self.page_ids[mapping_index as usize].0;
        match page_id {
            ChrPageId::Rom(page_number) => ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE),
            ChrPageId::Ram(page_number) => ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE),
            ChrPageId::Ciram(side) => ChrMemoryIndex::Ciram(side, 0),
            ChrPageId::SaveRam => ChrMemoryIndex::SaveRam(0),
            ChrPageId::ExtendedRam => ChrMemoryIndex::ExtendedRam(0),
            ChrPageId::FillModeTile => ChrMemoryIndex::FillModeTile,
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
        bank: ChrBank,
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
    pub fn page_id(&self, registers: &ChrBankRegisters) -> (ChrPageId, ReadWriteStatus) {
        match self {
            Self::Banked { bank, pages_per_bank: bank_multiple, page_offset, page_number_mask, .. } => {
                let location = bank.location().expect("Location to be present in bank.");
                let bank_index = match location {
                    ChrBankLocation::Fixed(bank_index) => bank_index,
                    ChrBankLocation::Switchable(register_id) => registers.get(register_id).index().unwrap(),
                    ChrBankLocation::MetaSwitchable(meta_id) => registers.get_from_meta(meta_id).index().unwrap(),
                };

                let page_number = ((bank_multiple * bank_index.to_raw()) & page_number_mask) + page_offset;
                match bank {
                    ChrBank::Rom(_, None) => (ChrPageId::Rom(page_number), ReadWriteStatus::ReadOnly),
                    ChrBank::Rom(_, Some(status_register)) => (ChrPageId::Rom(page_number), registers.read_write_status(*status_register)),
                    ChrBank::Ram(_, None) => (ChrPageId::Ram(page_number), ReadWriteStatus::ReadWrite),
                    ChrBank::Ram(_, Some(status_register)) => (ChrPageId::Ram(page_number), registers.read_write_status(*status_register)),
                    _ => todo!(),
                }
            }
            Self::Ciram(ciram_side) => (ChrPageId::Ciram(*ciram_side), ReadWriteStatus::ReadWrite),
            Self::SaveRam => (ChrPageId::SaveRam, ReadWriteStatus::ReadWrite),
            Self::ExtendedRam => (ChrPageId::ExtendedRam, ReadWriteStatus::ReadWrite),
            Self::FillModeTile => (ChrPageId::FillModeTile, ReadWriteStatus::ReadOnly),
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
    regs: ChrBankRegisters,

    layout_index: u8,
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
        regs: ChrBankRegisters,
    ) -> ChrMemory {
        let mut bank_size = None;
        for layout in &layouts {
            for window in layout.windows() {
                if matches!(window.bank(), ChrBank::Rom(..) | ChrBank::Ram(..)) {
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

        let memory_maps = layouts.iter().map(|layout|
            ChrMemoryMap::new(
                *layout,
                name_table_mirroring,
                bank_size,
                access_override,
                align_large_chr_banks,
                &regs,
        )).collect();

        ChrMemory {
            layouts,
            memory_maps,
            layout_index,
            rom_outer_banks: rom.split_n(rom_outer_bank_count),
            rom_outer_bank_index: 0,
            ram: ram.clone(),
            regs,
        }
    }

    pub fn window_count(&self) -> u8 {
        self.current_layout().windows().len().try_into().unwrap()
    }

    pub fn read_write_status_infos(&self) -> Vec<ReadWriteStatusInfo> {
        let mut ids = Vec::new();
        for layout in &self.layouts {
            ids.append(&mut layout.active_read_write_status_register_ids());
        }

        ids
    }

    pub fn peek(&self, ciram: &Ciram, address: PpuAddress) -> u8 {
        match self.current_memory_map().index_for_address(address).0 {
            ChrMemoryIndex::Rom(index) => {
                self.rom_outer_banks[self.rom_outer_bank_index as usize][index % self.rom_outer_banks[0].size()]
            },
            ChrMemoryIndex::Ram(index) => self.ram[index % self.ram.size()],
            ChrMemoryIndex::Ciram(side, index) => ciram.side(side)[index as usize],
            ChrMemoryIndex::SaveRam(_index) => todo!(),
            ChrMemoryIndex::ExtendedRam(_index) => todo!(),
            ChrMemoryIndex::FillModeTile => todo!(),
        }
    }

    pub fn write(&mut self, ciram: &mut Ciram, address: PpuAddress, value: u8) {
        let (chr_memory_index, read_write_status) = self.current_memory_map().index_for_address(address);
        if !read_write_status.is_writable() {
            return;
        }

        match chr_memory_index {
            ChrMemoryIndex::Rom(_) => {}
            ChrMemoryIndex::Ram(index) => {
                let size = self.ram.size();
                self.ram[index % size] = value;
            }
            ChrMemoryIndex::Ciram(side, index) => {
                ciram.side_mut(side)[index as usize] = value;
            }
            ChrMemoryIndex::SaveRam(_index) => todo!(),
            ChrMemoryIndex::ExtendedRam(_index) => todo!(),
            ChrMemoryIndex::FillModeTile => todo!(),
        }
    }

    pub fn window_at(&self, start: u16) -> &ChrWindow {
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

    pub fn bank_registers(&self) -> &ChrBankRegisters {
        &self.regs
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn set_chr_rom_outer_bank_index(&mut self, index: u8) {
        self.rom_outer_bank_index = index;
    }

    pub fn set_bank_register<INDEX: Into<u16>>(&mut self, id: ChrBankRegisterId, value: INDEX) {
        self.regs.set(id, BankIndex::from_u16(value.into()));
        self.update_page_ids();
    }

    pub fn set_chr_bank_register_bits(&mut self, id: ChrBankRegisterId, new_value: u16, mask: u16) {
        self.regs.set_bits(id, new_value, mask);
        self.update_page_ids();
    }

    pub fn set_chr_meta_register(&mut self, id: MetaRegisterId, value: ChrBankRegisterId) {
        self.regs.set_meta_chr(id, value);
        self.update_page_ids();
    }

    pub fn update_chr_register(
        &mut self,
        id: ChrBankRegisterId,
        updater: &dyn Fn(u16) -> u16,
    ) {
        self.regs.update(id, updater);
        self.update_page_ids();
    }

    pub fn set_chr_bank_register_to_ciram_side(
        &mut self,
        id: ChrBankRegisterId,
        ciram_side: CiramSide,
    ) {
        self.regs.set_to_ciram_side(id, ciram_side);
        self.update_page_ids();
    }

    pub fn set_name_table_mirroring(&mut self, name_table_mirroring: NameTableMirroring) {
        for page_mapping in &mut self.memory_maps {
            page_mapping.set_name_table_mirroring(&self.regs, name_table_mirroring);
        }
    }

    pub fn set_read_write_status(&mut self, id: ReadWriteStatusRegisterId, read_write_status: ReadWriteStatus) {
        self.regs.set_read_write_status(id, read_write_status);
        self.update_page_ids();
    }

    fn update_page_ids(&mut self) {
        for page_mapping in &mut self.memory_maps {
            page_mapping.update_page_ids(&self.regs);
        }
    }

    pub fn pattern_table<'a>(&'a self, ciram: &'a Ciram, side: PatternTableSide) -> PatternTable<'a> {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks(ciram)),
            PatternTableSide::Right => PatternTable::new(self.right_chunks(ciram)),
        }
    }

    pub fn save_ram_1kib_page(&self, start: u32) -> &[u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        let start = start as usize;
        (&self.ram.as_slice()[start..start + 0x400]).try_into().unwrap()
    }

    pub fn save_ram_1kib_page_mut(&mut self, start: u32) -> &mut [u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        let start = start as usize;
        (&mut self.ram.as_mut_slice()[start..start + 0x400]).try_into().unwrap()
    }

    #[inline]
    fn left_chunks<'a>(&'a self, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        let mem = self.current_memory_map();
        [mem.page_start_index(0), mem.page_start_index(1), mem.page_start_index(2), mem.page_start_index(3)]
            .map(move |chr_index| {
                match chr_index {
                    ChrMemoryIndex::Rom(index) => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(
                            &self.rom_outer_banks[self.rom_outer_bank_index as usize].as_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrMemoryIndex::Ram(index) => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.ram.as_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrMemoryIndex::Ciram(side, _) => RawMemorySlice::from_raw(ciram.side(side)),
                    _ => todo!(),
                }
        })
    }

    #[inline]
    fn right_chunks<'a>(&'a self, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        let mem = self.current_memory_map();
        [mem.page_start_index(4), mem.page_start_index(5), mem.page_start_index(6), mem.page_start_index(7)]
            .map(move |chr_index| {
                match chr_index {
                    ChrMemoryIndex::Rom(index) => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(
                            &self.rom_outer_banks[self.rom_outer_bank_index as usize].as_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrMemoryIndex::Ram(index) => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(&self.ram.as_slice()[index..index + 1 * KIBIBYTE as usize])
                    }
                    ChrMemoryIndex::Ciram(side, _) => RawMemorySlice::from_raw(ciram.side(side)),
                    _ => todo!(),
                }
        })
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AccessOverride {
    ForceRom,
    ForceRam,
}