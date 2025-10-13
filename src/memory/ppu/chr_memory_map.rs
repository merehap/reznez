use crate::mapper::{BankIndex, ChrBank, NameTableMirroring, NameTableQuadrant, NameTableSource, ReadWriteStatus};
use crate::memory::bank::bank::ChrBankLocation;
use crate::memory::bank::bank_index::{ChrBankRegisters, MemType};
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
        bank_size: ChrWindowSize,
        align_large_windows: bool,
        regs: &ChrBankRegisters,
    ) -> Self {

        let mut page_mappings = Vec::with_capacity(CHR_SLOT_COUNT);
        for window in initial_layout.windows() {
            let pages_per_window = window.size().page_multiple();
            let mut page_number_mask = 0b1111_1111_1111_1111;
            if align_large_windows {
                page_number_mask &= !(pages_per_window - 1);
            }

            for page_offset in 0..pages_per_window {
                page_mappings.push(ChrMapping::Banked {
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
            for quadrant in name_table_mirroring.quadrants() {
                let mapping = match quadrant {
                    NameTableSource::Ciram(_) => ChrMapping::NameTableSource(quadrant),
                    NameTableSource::Ram {..} => ChrMapping::NameTableSource(quadrant),
                    _ => panic!("{quadrant:?} is not yet supported for high PPU memory mapping."),
                };
                page_mappings.push(mapping);
            }
        }

        assert_eq!(page_mappings.len(), 12);

        let mut memory_map = Self {
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [(ChrPageId::Rom { page_number: 0, bank_index: BankIndex::from_u8(0) }, ReadWriteStatus::ReadOnly); CHR_SLOT_COUNT],
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
            ChrPageId::Rom { page_number, bank_index } => {
                (ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE + u32::from(offset)), PeekSource::Rom(bank_index))
            }
            ChrPageId::Ram { page_number, bank_index } => {
                (ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE + u32::from(offset)), PeekSource::Ram(bank_index))
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
    pub fn page_id(&self, regs: &ChrBankRegisters) -> (ChrPageId, ReadWriteStatus) {
        match self {
            Self::Banked { bank, pages_per_bank: bank_multiple, page_offset, page_number_mask, .. } => {
                let location = bank.location().expect("Location to be present in bank.");
                let bank_index = match location {
                    ChrBankLocation::Fixed(bank_index) => bank_index,
                    ChrBankLocation::Switchable(register_id) => regs.get(register_id).index().unwrap(),
                    ChrBankLocation::MetaSwitchable(meta_id) => regs.get_from_meta(meta_id).index().unwrap(),
                };

                let page_number = ((bank_multiple * bank_index.to_raw()) & page_number_mask) + page_offset;
                match bank {
                    ChrBank::Rom(_, None) => (ChrPageId::Rom { page_number, bank_index }, ReadWriteStatus::ReadOnly),
                    ChrBank::Rom(_, Some(status_register)) => (ChrPageId::Rom { page_number, bank_index }, regs.read_write_status(*status_register)),
                    ChrBank::Ram(_, None) => (ChrPageId::Ram { page_number, bank_index }, ReadWriteStatus::ReadWrite),
                    ChrBank::Ram(_, Some(status_register)) => (ChrPageId::Ram { page_number, bank_index }, regs.read_write_status(*status_register)),
                    ChrBank::RomRam(_, status_register, rom_ram_register_id) => {
                        let read_write_status = status_register.map(|reg| regs.read_write_status(reg));
                        match regs.rom_ram_mode(*rom_ram_register_id) {
                            MemType::Rom => (ChrPageId::Rom { page_number, bank_index }, read_write_status.unwrap_or(ReadWriteStatus::ReadOnly)),
                            MemType::WorkRam => (ChrPageId::Ram { page_number, bank_index }, read_write_status.unwrap_or(ReadWriteStatus::ReadWrite)),
                            MemType::SaveRam => unimplemented!("SaveRam is not currently supported in RomRam banks."),
                        }
                    }
                    ChrBank::SaveRam(index) => {
                        // FIXME: Implement this properly? Hack so that the ROM Query page doesn't crash on Napoleon Senki.
                        (ChrPageId::Ram { page_number: 0, bank_index: BankIndex::from_u16((*index).try_into().unwrap()) }, ReadWriteStatus::ReadWrite)
                    }
                }
            }
            Self::NameTableSource(source) => match source {
                NameTableSource::Ciram(ciram_side) => (ChrPageId::Ciram(*ciram_side), ReadWriteStatus::ReadWrite),
                NameTableSource::Ram {..} => (ChrPageId::SaveRam, ReadWriteStatus::ReadWrite),
                NameTableSource::ExtendedRam => (ChrPageId::ExtendedRam, ReadWriteStatus::ReadWrite),
                NameTableSource::FillModeTile => (ChrPageId::FillModeTile, ReadWriteStatus::ReadOnly),
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrPageId {
    Rom { page_number: PageNumber, bank_index: BankIndex },
    Ram { page_number: PageNumber, bank_index: BankIndex },
    Ciram(CiramSide),
    SaveRam,
    ExtendedRam,
    FillModeTile,
}

type PageNumber = u16;