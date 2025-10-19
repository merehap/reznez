use crate::mapper::{BankNumber, ChrBank, NameTableMirroring, NameTableQuadrant, NameTableSource, ReadWriteStatus};
use crate::memory::bank::bank::{ChrBankNumberProvider, ChrSource, ChrSourceRegisterId};
use crate::memory::bank::bank_number::ChrBankRegisters;
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::window::ChrWindowSize;
use crate::util::unit::KIBIBYTE;

use super::chr_memory::PeekSource;
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
        name_table_mirroring_fixed: bool,
        bank_size: ChrWindowSize,
        align_large_windows: bool,
        regs: &mut ChrBankRegisters,
    ) -> Self {

        let mut page_mappings = Vec::with_capacity(CHR_SLOT_COUNT);
        for window in initial_layout.windows() {
            let pages_per_window = window.size().page_multiple();
            let mut page_number_mask = 0b1111_1111_1111_1111;
            if align_large_windows {
                page_number_mask &= !(pages_per_window - 1);
            }

            for page_offset in 0..pages_per_window {
                page_mappings.push(ChrMapping {
                    bank: window.bank(),
                    pages_per_bank: bank_size.page_multiple(),
                    page_number_mask,
                    page_offset,
                });
            }
        }

        assert!(matches!(page_mappings.len(), 8 | 12));

        // Most mappers only map 0x0000..=0x1FFF for pattern data, but some map up through 0x2FFF.
        if page_mappings.len() == 8 {
            if name_table_mirroring_fixed {
                for quadrant in name_table_mirroring.quadrants() {
                    page_mappings.push(ChrMapping::from_name_table_source(quadrant));
                }
            } else {
                let quadrants_with_source_reg_ids = name_table_mirroring.quadrants().into_iter()
                    .zip(ChrSourceRegisterId::ALL_NAME_TABLE_SOURCE_IDS);
                for (quadrant, reg_id) in quadrants_with_source_reg_ids {
                    page_mappings.push(ChrMapping::from_name_table_source_with_register(quadrant, reg_id, regs));
                }
            }
        }

        assert_eq!(page_mappings.len(), 12);

        let mut memory_map = Self {
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [(ChrPageId::Rom { page_number: 0, bank_number: BankNumber::from_u8(0) }, ReadWriteStatus::ReadOnly); CHR_SLOT_COUNT],
        };
        memory_map.update_page_ids(regs);

        memory_map
    }

    pub fn index_for_address(&self, address: PpuAddress) -> (ChrMemoryIndex, PeekSource, ReadWriteStatus) {
        let address = address.to_u16();
        match address {
            0x0000..=0x2FFF => {}
            0x3000..=0x3FFF => todo!(),
            0x4000..=0xFFFF => unreachable!(),
        }

        let mapping_index = address / (KIBIBYTE as u16);
        let offset = address % (KIBIBYTE as u16);

        let (page_id, read_write_status) = self.page_ids[mapping_index as usize];
        let (chr_memory_index, peek_source) = match page_id {
            ChrPageId::Rom { page_number, bank_number } => {
                (ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE + u32::from(offset)), PeekSource::Rom(bank_number))
            }
            ChrPageId::Ram { page_number, bank_number } => {
                (ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE + u32::from(offset)), PeekSource::Ram(bank_number))
            }
            ChrPageId::Ciram(side) => {
                (ChrMemoryIndex::Ciram(side, offset), PeekSource::Ciram(side))
            }
            ChrPageId::SaveRam => {
                (ChrMemoryIndex::SaveRam(offset), PeekSource::SaveRam)
            }
            ChrPageId::ExtendedRam => {
                (ChrMemoryIndex::ExtendedRam(offset), PeekSource::ExtendedRam)
            }
            ChrPageId::FillModeTile => {
                (ChrMemoryIndex::FillModeTile, PeekSource::FillModeTile)
            }
        };

        (chr_memory_index, peek_source, read_write_status)
    }

    pub fn page_mappings(&self) -> &[ChrMapping; CHR_SLOT_COUNT] {
        &self.page_mappings
    }

    pub fn pattern_table_page_ids(&self) -> &[(ChrPageId, ReadWriteStatus)] {
        &self.page_ids[0..8]
    }

    pub fn set_name_table_mirroring(&mut self, regs: &ChrBankRegisters, name_table_mirroring: NameTableMirroring) {
        for (i, &quadrant) in name_table_mirroring.quadrants().iter().enumerate() {
            self.page_mappings[8 + i] = ChrMapping::from_name_table_source(quadrant);
        }

        self.update_page_ids(regs);
    }

    pub fn set_name_table_quadrant(
        &mut self, regs: &ChrBankRegisters, quadrant: NameTableQuadrant, source: NameTableSource) {

        self.page_mappings[8 + quadrant as usize] = ChrMapping::from_name_table_source(source);

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
            ChrPageId::Rom { page_number, .. } => ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE),
            ChrPageId::Ram { page_number, .. } => ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE),
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
pub struct ChrMapping {
    bank: ChrBank,
    pages_per_bank: u16,
    page_offset: u16,
    page_number_mask: u16,
}

impl ChrMapping {
    pub fn from_name_table_source(name_table_source: NameTableSource) -> Self {
        let mut mapping = Self {
            bank: ChrBank::RAM,
            pages_per_bank: 1,
            page_offset: 0,
            page_number_mask: 0b1111_1111_1111_1111,
        };
        mapping.bank = match name_table_source {
            NameTableSource::Ram { bank_number } => mapping.bank.fixed_index(bank_number.to_raw() as i16),
            NameTableSource::Ciram(ciram_side) => ChrBank::ciram(ciram_side),
            NameTableSource::ExtendedRam => ChrBank::EXT_RAM,
            NameTableSource::FillModeTile => ChrBank::FILL_MODE_TILE,
        };

        mapping
    }

    pub fn from_name_table_source_with_register(
        name_table_source: NameTableSource,
        source_id: ChrSourceRegisterId,
        regs: &mut ChrBankRegisters,
    ) -> Self {
        let mapping = Self {
            bank: ChrBank::with_switchable_source(source_id),
            pages_per_bank: 1,
            page_offset: 0,
            page_number_mask: 0b1111_1111_1111_1111,
        };

        let chr_source = match name_table_source {
            NameTableSource::Ram {..} => ChrSource::WorkRam,
            NameTableSource::Ciram(ciram_side) => ChrSource::Ciram(ciram_side),
            NameTableSource::ExtendedRam => ChrSource::ExtendedRam,
            NameTableSource::FillModeTile => ChrSource::FillModeTile,
        };
        regs.set_chr_source(source_id, chr_source);

        mapping
    }

    pub fn to_name_table_source(&self, regs: &ChrBankRegisters) -> Result<NameTableSource, String> {
        let chr_source = self.bank.current_chr_source(regs).expect("NameTableSource can't come from an empty bank.");
        match chr_source {
            ChrSource::Rom | ChrSource::SaveRam => Err(format!("{chr_source:?} is not yet a supported CHR source")),
            ChrSource::Ciram(ciram_side) => Ok(NameTableSource::Ciram(ciram_side)),
            ChrSource::WorkRam => Ok(NameTableSource::Ram { bank_number: self.bank.bank_number(regs).unwrap() }),
            ChrSource::ExtendedRam => Ok(NameTableSource::ExtendedRam),
            ChrSource::FillModeTile => Ok(NameTableSource::FillModeTile),
        }
    }

    pub fn page_id(&self, regs: &ChrBankRegisters) -> (ChrPageId, ReadWriteStatus) {
        let Self { bank, pages_per_bank: bank_multiple, page_offset, page_number_mask, .. } = self;
        let location = bank.location().expect("Location to be present in bank.");
        let bank_number = match location {
            ChrBankNumberProvider::Fixed(bank_number) => bank_number,
            ChrBankNumberProvider::Switchable(register_id) => regs.get(register_id),
            ChrBankNumberProvider::MetaSwitchable(meta_id) => regs.get_from_meta(meta_id),
        };

        let page_number = ((bank_multiple * bank_number.to_raw()) & page_number_mask) + page_offset;

        let read_write_status = bank.read_write_status(regs);
        match bank.current_chr_source(regs) {
            None => todo!("EMPTY bank"),
            Some(ChrSource::Rom) => (ChrPageId::Rom { page_number, bank_number }, read_write_status),
            Some(ChrSource::WorkRam) => (ChrPageId::Ram { page_number, bank_number }, read_write_status),
            Some(ChrSource::SaveRam) => (ChrPageId::SaveRam, read_write_status),
            Some(ChrSource::Ciram(ciram_side)) => (ChrPageId::Ciram(ciram_side), ReadWriteStatus::ReadWrite),
            Some(ChrSource::ExtendedRam) => (ChrPageId::ExtendedRam, ReadWriteStatus::ReadWrite),
            Some(ChrSource::FillModeTile) => (ChrPageId::FillModeTile, ReadWriteStatus::ReadOnly),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrPageId {
    Rom { page_number: PageNumber, bank_number: BankNumber },
    Ram { page_number: PageNumber, bank_number: BankNumber },
    Ciram(CiramSide),
    SaveRam,
    ExtendedRam,
    FillModeTile,
}

type PageNumber = u16;