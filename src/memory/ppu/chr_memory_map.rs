use crate::mapper::{BankNumber, ChrBank, NameTableMirroring, NameTableQuadrant, NameTableSource};
use crate::memory::bank::bank::{ChrBankNumberProvider, ChrSource, ChrSourceRegisterId};
use crate::memory::bank::bank_number::{ChrBankRegisters, ReadStatus, WriteStatus};
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::window::ChrWindowSize;
use crate::util::unit::KIBIBYTE;

use super::chr_memory::PeekSource;
use super::ciram::CiramSide;

const CHR_SLOT_COUNT: usize = 16;

pub struct ChrMemoryMap {
    // 0x0000 through 0x1FFF (2 pattern tables) and 0x2000 through 0x2FFF (4 name tables).
    page_mappings: [ChrMapping; CHR_SLOT_COUNT],
    name_table_mirroring_fixed: bool,
}

impl ChrMemoryMap {
    pub fn new(
        initial_layout: ChrLayout,
        cartridge_name_table_mirroring: Option<NameTableMirroring>,
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
                    page_number: 0,
                    bank_number: BankNumber::from_u8(0),
                    mem_type_status: ChrMemTypeStatus::Rom(ReadStatus::Enabled),
                });
            }
        }

        assert!(matches!(page_mappings.len(), 8 | 12 | 16));

        // Most mappers only map 0x0000..=0x1FFF for pattern data, but some map up through 0x2FFF.
        // TODO: Map through 0x3EFF
        if page_mappings.len() == 8 {
            let cartridge_name_table_mirroring = cartridge_name_table_mirroring
                .expect("The mapper must specify mappings from 0x2000 to 0x2FFF when four screen mirroring is specified.");
            if name_table_mirroring_fixed {
                for quadrant in cartridge_name_table_mirroring.quadrants() {
                    page_mappings.push(ChrMapping::from_name_table_source(quadrant));
                }
            } else {
                let quadrants_with_source_reg_ids = cartridge_name_table_mirroring.quadrants().into_iter()
                    .zip(ChrSourceRegisterId::ALL_NAME_TABLE_SOURCE_IDS);
                for (quadrant, reg_id) in quadrants_with_source_reg_ids {
                    page_mappings.push(ChrMapping::from_name_table_source_with_register(quadrant, reg_id, regs));
                }
            }
        }

        if page_mappings.len() == 12 {
            // Normally, 0x3000 through 0x3EFF is a mirror of 0x2000 through 0x2EFF.
            page_mappings.push(page_mappings[8]);
            page_mappings.push(page_mappings[9]);
            page_mappings.push(page_mappings[10]);
            page_mappings.push(page_mappings[11]);
        }

        assert_eq!(page_mappings.len(), 16);
        let mut memory_map = Self {
            page_mappings: page_mappings.try_into().unwrap(),
            name_table_mirroring_fixed,
        };
        memory_map.update_page_ids(regs);

        memory_map
    }

    pub fn index_for_address(&self, address: PpuAddress) -> (ChrMemoryIndex, PeekSource) {
        let address = address.to_u16();
        assert!(address < 0x4000);

        let mapping_index = address / (KIBIBYTE as u16);
        let offset = address % (KIBIBYTE as u16);

        let page_mapping = self.page_mappings[mapping_index as usize];
        let page_number = page_mapping.page_number;
        let bank_number = page_mapping.bank_number;
        let (chr_memory_index, peek_source) = match page_mapping.mem_type_status() {
            ChrMemTypeStatus::Rom(read_status) => {
                (ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE + u32::from(offset), read_status), PeekSource::Rom(bank_number))
            }
            ChrMemTypeStatus::Ram(read_status, write_status) => {
                (ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE + u32::from(offset), read_status, write_status), PeekSource::Ram(bank_number))
            }
            ChrMemTypeStatus::Ciram => {
                let side = CiramSide::from_page_number(page_number);
                (ChrMemoryIndex::Ciram(side, offset.into()), PeekSource::Ciram(side))
            }
            ChrMemTypeStatus::MapperCustom { page_id } =>
                (ChrMemoryIndex::MapperCustom { page_id, index: offset.into() }, PeekSource::MapperCustom { page_id }),
        };

        (chr_memory_index, peek_source)
    }

    pub fn page_mappings(&self) -> &[ChrMapping; CHR_SLOT_COUNT] {
        &self.page_mappings
    }

    pub fn pattern_table_page_mappings(&self) -> &[ChrMapping] {
        &self.page_mappings[0..8]
    }

    pub fn set_name_table_mirroring(&mut self, regs: &mut ChrBankRegisters, name_table_mirroring: NameTableMirroring) {
        for (i, quadrant) in NameTableQuadrant::ALL.iter().enumerate() {
            self.set_name_table_quadrant(regs, *quadrant, name_table_mirroring.quadrants()[i]);
        }

        self.update_page_ids(regs);
    }

    pub fn set_name_table_quadrant(&mut self, regs: &mut ChrBankRegisters, quadrant: NameTableQuadrant, source: NameTableSource) {
        assert!(!self.name_table_mirroring_fixed);
        let (chr_source, bank_number) = ChrSource::from_name_table_source(source);
        let (chr_source_reg_id, chr_bank_reg_id) = quadrant.register_ids();
        regs.set_chr_source(chr_source_reg_id, chr_source);
        if let Some(bank_number) = bank_number {
            regs.set(chr_bank_reg_id, bank_number);
        }

        self.update_page_ids(regs);
    }

    pub fn update_page_ids(&mut self, regs: &ChrBankRegisters) {
        for i in 0..CHR_SLOT_COUNT {
            self.page_mappings[i].update_page_id(regs);
        }
    }

    pub fn page_start_index(&self, mapping_index: u8) -> ChrMemoryIndex {
        let mapping = self.page_mappings[mapping_index as usize];
        let page_number = mapping.page_number;
        match mapping.mem_type_status {
            ChrMemTypeStatus::Rom(read_status) =>
                ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE, read_status),
            ChrMemTypeStatus::Ram(read_status, write_status) =>
                ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE, read_status, write_status),
            ChrMemTypeStatus::Ciram =>
                ChrMemoryIndex::Ciram(CiramSide::from_page_number(page_number), 0),
            ChrMemTypeStatus::MapperCustom { page_id } =>
                ChrMemoryIndex::MapperCustom { page_id, index: 0 },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrMemoryIndex {
    Rom(u32, ReadStatus),
    Ram(u32, ReadStatus, WriteStatus),
    Ciram(CiramSide, u32),
    // TODO: Should Read/WriteStatus be stored here?
    MapperCustom { page_id: u8, index: u32 },
}

impl ChrMemoryIndex {
    pub fn read_status(self) -> ReadStatus {
        match self {
            Self::Rom(_, read_status) | Self::Ram(_, read_status, _) => read_status,
            Self::Ciram(_, _) => ReadStatus::Enabled, // There's no way to disable reads to CIRAM.
            Self::MapperCustom { .. } => ReadStatus::Enabled, // FIXME: This is inaccurate.
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChrMapping {
    bank: ChrBank,
    pages_per_bank: u16,
    page_offset: u16,
    page_number_mask: u16,
    page_number: u16,
    bank_number: BankNumber,
    mem_type_status: ChrMemTypeStatus,
}

impl ChrMapping {
    pub fn from_name_table_source(name_table_source: NameTableSource) -> Self {
        let mut mapping = Self {
            bank: ChrBank::ROM,
            pages_per_bank: 1,
            page_offset: 0,
            page_number_mask: 0b1111_1111_1111_1111,
            page_number: 0,
            bank_number: BankNumber::from_u8(0),
            mem_type_status: ChrMemTypeStatus::Rom(ReadStatus::Enabled),
        };
        mapping.bank = match name_table_source {
            NameTableSource::Rom { bank_number } => mapping.bank.fixed_index(bank_number.to_raw() as i16),
            NameTableSource::Ram { bank_number } => mapping.bank.fixed_index(bank_number.to_raw() as i16),
            NameTableSource::Ciram(ciram_side) => ChrBank::ciram(ciram_side),
            NameTableSource::MapperCustom { page_id: page_number } => ChrBank::mapper_sourced(page_number),
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
            page_number: 0,
            bank_number: BankNumber::from_u8(0),
            mem_type_status: ChrMemTypeStatus::Rom(ReadStatus::Enabled),
        };

        let chr_source = match name_table_source {
            NameTableSource::Rom {..} => ChrSource::Rom,
            NameTableSource::Ram {..} => ChrSource::WorkRam,
            NameTableSource::Ciram(ciram_side) => ChrSource::Ciram(ciram_side),
            NameTableSource::MapperCustom { page_id } => ChrSource::MapperCustom { page_id },
        };
        regs.set_chr_source(source_id, chr_source);

        mapping
    }

    pub fn page_number(self) -> u16 {
        self.page_number
    }

    pub fn mem_type_status(self) -> ChrMemTypeStatus {
        self.mem_type_status
    }

    pub fn to_name_table_source(self, regs: &ChrBankRegisters) -> Result<NameTableSource, String> {
        let chr_source = self.bank.current_chr_source(regs).expect("NameTableSource can't come from an empty bank.");
        match chr_source {
            ChrSource::RomOrRam => {
                assert!(!regs.cartridge_has_rom() || !regs.cartridge_has_ram(),
                    "Don't know what to do with a Chr RomOrRam bank when the cartridge has both ROM and RAM.");
                if regs.cartridge_has_rom() {
                    Ok(NameTableSource::Rom { bank_number: self.bank.bank_number(regs).unwrap() })
                } else if regs.cartridge_has_ram() {
                    Ok(NameTableSource::Ram { bank_number: self.bank.bank_number(regs).unwrap() })
                } else {
                    Err("Absent CHR banks are not yet supported.".to_owned())
                }
            }
            ChrSource::Rom => Ok(NameTableSource::Rom { bank_number: self.bank.bank_number(regs).unwrap() }),
            ChrSource::Ciram(ciram_side) => Ok(NameTableSource::Ciram(ciram_side)),
            ChrSource::WorkRam => Ok(NameTableSource::Ram { bank_number: self.bank.bank_number(regs).unwrap() }),
            ChrSource::MapperCustom { page_id } => Ok(NameTableSource::MapperCustom { page_id }),
        }
    }

    pub fn update_page_id(&mut self, regs: &ChrBankRegisters) {
        let Self { bank, pages_per_bank: bank_multiple, page_offset, page_number_mask, .. } = self;
        let location = bank.location().expect("Location to be present in bank.");
        let bank_number = match location {
            ChrBankNumberProvider::Fixed(bank_number) => bank_number,
            ChrBankNumberProvider::Switchable(register_id) => regs.get(register_id),
            ChrBankNumberProvider::MetaSwitchable(meta_id) => regs.get_from_meta(meta_id),
        };

        let page_number = ((*bank_multiple * bank_number.to_raw()) & *page_number_mask) + *page_offset;
        self.page_number = page_number;
        self.bank_number = bank_number;

        let read_status = bank.read_status_register_id().map_or(ReadStatus::Enabled, |id| regs.read_status(id));
        let write_status = bank.write_status_register_id().map_or(WriteStatus::Enabled, |id| regs.write_status(id));
        match bank.current_chr_source(regs) {
            None => todo!("EMPTY bank"),
            Some(ChrSource::RomOrRam) => {
                match (regs.cartridge_has_rom(), regs.cartridge_has_ram()) {
                    (false, false) => todo!("Absent CHR pages."),
                    (true , true ) => panic!("Not sure what to do for a RomOrRam bank when both are present in the cartridge."),
                    (true , false) => {
                        self.mem_type_status = ChrMemTypeStatus::Rom(read_status);
                    }
                    (false, true ) => {
                        self.mem_type_status = ChrMemTypeStatus::Ram(read_status, write_status);
                    },
                }
            }
            Some(ChrSource::Rom) => {
                assert!(regs.cartridge_has_rom());
                self.mem_type_status = ChrMemTypeStatus::Rom(read_status);
            }
            Some(ChrSource::WorkRam) => {
                assert!(regs.cartridge_has_ram());
                self.mem_type_status = ChrMemTypeStatus::Ram(read_status, write_status);
            }
            Some(ChrSource::Ciram(ciram_side)) => {
                self.page_number = ciram_side.to_page_number();
                self.bank_number = BankNumber::from_u16(ciram_side.to_page_number());
                assert!(self.page_number < 2);
                self.mem_type_status = ChrMemTypeStatus::Ciram;
            },
            Some(ChrSource::MapperCustom { page_id }) => {
                assert_eq!(self.page_number, 0);
                self.mem_type_status = ChrMemTypeStatus::MapperCustom { page_id };
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrMemTypeStatus {
    Rom(ReadStatus),
    Ram(ReadStatus, WriteStatus),
    // TODO: Read/Write status here in order to disable CIRAM for mapper 111
    // Or maybe "Disabled" on the wiki just means unmapped?
    Ciram,
    // TODO: Should Read/WriteStatus be stored here?
    MapperCustom { page_id: u8 },
}