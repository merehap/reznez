use std::num::NonZeroU16;

use crate::mapper::{ChrBank, NameTableMirroring, NameTableQuadrant, NameTableSource, ReadWriteStatus};
use crate::memory::bank::bank::ChrBankLocation;
use crate::memory::bank::bank_index::ChrBankRegisters;
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
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

            let bank = window.bank();
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

            page_mappings.push(ChrMapping::NameTableSource(NameTableSource::Ciram(ciram_side)));
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

    pub fn page_mappings(&self) -> &[ChrMapping; CHR_SLOT_COUNT] {
        &self.page_mappings
    }

    pub fn pattern_table_page_ids(&self) -> &[(ChrPageId, ReadWriteStatus)] {
        &self.page_ids[0..8]
    }

    pub fn set_name_table_mirroring(&mut self, regs: &ChrBankRegisters, name_table_mirroring: NameTableMirroring) {
        for (i, &quadrant) in name_table_mirroring.quadrants().iter().enumerate() {
            self.page_mappings[8 + i] = ChrMapping::NameTableSource(quadrant);
        }

        self.update_page_ids(regs);
    }

    pub fn set_name_table_quadrant(
        &mut self, regs: &ChrBankRegisters, quadrant: NameTableQuadrant, source: NameTableSource) {

        self.page_mappings[8 + quadrant as usize] = ChrMapping::NameTableSource(source);

        self.update_page_ids(regs);
    }

    pub fn update_page_ids(&mut self, regs: &ChrBankRegisters) {
        for i in 0..CHR_SLOT_COUNT {
            self.page_ids[i] = self.page_mappings[i].page_id(regs);
        }
    }

    pub fn page_start_index(&self, mapping_index: u8) -> ChrMemoryIndex {
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
    NameTableSource(NameTableSource),
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
            Self::NameTableSource(source) => match source {
                NameTableSource::Ciram(ciram_side) => (ChrPageId::Ciram(*ciram_side), ReadWriteStatus::ReadWrite),
                NameTableSource::SaveRam(_) => (ChrPageId::SaveRam, ReadWriteStatus::ReadWrite),
                NameTableSource::ExtendedRam => (ChrPageId::ExtendedRam, ReadWriteStatus::ReadWrite),
                NameTableSource::FillModeTile => (ChrPageId::FillModeTile, ReadWriteStatus::ReadOnly),
            }
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