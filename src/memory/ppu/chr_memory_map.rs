use log::warn;

use crate::memory::address_template::address_resolver::AddressResolver;
use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::bank::bank::ChrSourceRegisterId;
use crate::memory::bank::bank_number::{BankNumber, ChrBankRegisterId, ChrBankRegisters, ReadStatus, WriteStatus};
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::window::{ChrBankNumberProvider, ChrSource, ChrSourceProvider, ChrWindow, ChrWindowSize};
use crate::ppu::name_table::name_table_mirroring::{NameTableMirroring, NameTableSource};
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::util::unit::KIBIBYTE;

use super::chr_memory::PeekSource;
use crate::memory::regions::ciram::CiramSide;

const CHR_SLOT_COUNT: usize = 16;

pub struct ChrMemoryMap {
    // 0x0000 through 0x1FFF (2 pattern tables) and 0x2000 through 0x2FFF (4 name tables).
    page_mappings: [ChrMapping; CHR_SLOT_COUNT],
    name_table_mirroring_fixed: bool,
}

impl ChrMemoryMap {
    pub fn new(
        initial_layout: ChrLayout,
        rom_bank_sizes: &BankSizes,
        ram_bank_sizes: &BankSizes,
        cartridge_name_table_mirroring: Option<NameTableMirroring>,
        name_table_mirroring_fixed: bool,
        bank_size: ChrWindowSize,
        _align_large_windows: bool,
        regs: &mut ChrBankRegisters,
    ) -> Result<Self, String> {
        let mut page_mappings = Vec::with_capacity(CHR_SLOT_COUNT);
        for window in initial_layout.windows() {
            let pages_per_window = window.size().page_multiple();
            for page_offset in 0..pages_per_window {
                page_mappings.push(ChrMapping {
                    window: *window,
                    rom_address_resolver: window.get_rom_address_template(rom_bank_sizes),
                    ram_address_resolver: window.ram_address_template(ram_bank_sizes, 0),
                    pages_per_bank: bank_size.page_multiple(),
                    page_offset,
                    ciram_side: CiramSide::Left,
                    mem_type_status: ChrMemTypeStatus::Rom(ReadStatus::Enabled),
                });
            }
        }

        assert!(matches!(page_mappings.len(), 8 | 12 | 16));

        // Most mappers only map 0x0000..=0x1FFF for pattern data, but some map up through 0x2FFF.
        // TODO: Map through 0x3EFF
        if page_mappings.len() == 8 {
            let Some(cartridge_name_table_mirroring) = cartridge_name_table_mirroring else {
                // TODO: Promote this to a panic once VS mode is supported.
                return Err("The mapper must specify mappings from 0x2000 to 0x2FFF when four screen mirroring is specified.".into());
            };

            let address_template = AddressResolver::chr(
                ChrBankNumberProvider::Fixed(BankNumber::ZERO),
                ChrWindowSize::NAME_TABLE_WINDOW_SIZE,
                rom_bank_sizes,
                0,
            );
            if name_table_mirroring_fixed {
                for quadrant in cartridge_name_table_mirroring.quadrants() {
                    assert!(matches!(quadrant, NameTableSource::Ciram(_)), "Configure non-CIRAM mirrorings using chr_layouts instead.");
                    page_mappings.push(ChrMapping::from_name_table_source(quadrant, address_template, address_template));
                }
            } else {
                let quadrants_with_source_reg_ids = [0x2000, 0x2400, 0x2800, 0x2C00].into_iter()
                    .zip(cartridge_name_table_mirroring.quadrants())
                    .zip(ChrSourceRegisterId::ALL_NAME_TABLE_SOURCE_IDS);
                for ((addr, quadrant), reg_id) in quadrants_with_source_reg_ids {
                    assert!(matches!(quadrant, NameTableSource::Ciram(_)), "Configure non-CIRAM mirrorings using chr_layouts instead.");
                    let window = ChrWindow::new(addr, addr + 0x3FF, 0x400, ChrSourceProvider::Switchable(reg_id));
                    page_mappings.push(ChrMapping::from_name_table_source_with_register(
                        window, quadrant, address_template, address_template, reg_id, regs));
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

        Ok(memory_map)
    }

    pub fn index_for_address(&self, address: PpuAddress) -> (ChrMemoryIndex, PeekSource) {
        let address = address.to_u16();
        assert!(address < 0x4000);

        let mapping_index = address / (KIBIBYTE as u16);
        let offset = address % (KIBIBYTE as u16);

        let page_mapping = self.page_mappings[mapping_index as usize];
        let (chr_memory_index, peek_source) = match page_mapping.mem_type_status() {
            ChrMemTypeStatus::Absent => {
                (ChrMemoryIndex::Absent, PeekSource::Void)
            }
            ChrMemTypeStatus::Rom(read_status) => {
                let index = page_mapping.rom_address_resolver.resolve_index(address);
                let bank_number = page_mapping.rom_address_resolver.resolve_inner_bank_number();
                (ChrMemoryIndex::Rom(index, read_status), PeekSource::Rom(BankNumber::from_u16(bank_number)))
            }
            ChrMemTypeStatus::Ram(read_status, write_status) => {
                let index = page_mapping.ram_address_resolver.resolve_index(address);
                let bank_number = page_mapping.rom_address_resolver.resolve_inner_bank_number();
                (ChrMemoryIndex::Ram(index, read_status, write_status), PeekSource::Ram(BankNumber::from_u16(bank_number)))
            }
            ChrMemTypeStatus::Ciram => {
                (ChrMemoryIndex::Ciram(page_mapping.ciram_side, offset.into()), PeekSource::Ciram(page_mapping.ciram_side))
            }
            ChrMemTypeStatus::MapperCustom { page_id } => {
                (ChrMemoryIndex::MapperCustom { page_id, index: offset.into() }, PeekSource::MapperCustom { page_id })
            }
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

    pub fn set_rom_outer_bank_number(&mut self, regs: &ChrBankRegisters, raw_outer_bank_number: u16) {
        for i in 0..CHR_SLOT_COUNT {
            self.page_mappings[i].rom_address_resolver.set_raw_outer_bank_number(raw_outer_bank_number);
        }

        self.update_page_ids(regs);
    }

    pub fn page_start_index(&self, mapping_index: u8) -> ChrMemoryIndex {
        let mapping = self.page_mappings[mapping_index as usize];
        match mapping.mem_type_status {
            ChrMemTypeStatus::Absent => ChrMemoryIndex::Absent,
            ChrMemTypeStatus::Rom(read_status) => {
                let page_number = mapping.rom_address_resolver.resolve_inner_bank_number() * mapping.pages_per_bank + mapping.page_offset;
                ChrMemoryIndex::Rom(u32::from(page_number) * KIBIBYTE, read_status)
            }
            ChrMemTypeStatus::Ram(read_status, write_status) => {
                let page_number = mapping.ram_address_resolver.resolve_inner_bank_number() * mapping.pages_per_bank + mapping.page_offset;
                ChrMemoryIndex::Ram(u32::from(page_number) * KIBIBYTE, read_status, write_status)
            }
            ChrMemTypeStatus::Ciram =>
                ChrMemoryIndex::Ciram(mapping.ciram_side, 0),
            ChrMemTypeStatus::MapperCustom { page_id } =>
                ChrMemoryIndex::MapperCustom { page_id, index: 0 },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrMemoryIndex {
    Absent,
    Rom(u32, ReadStatus),
    Ram(u32, ReadStatus, WriteStatus),
    Ciram(CiramSide, u32),
    // TODO: Should Read/WriteStatus be stored here?
    MapperCustom { page_id: u8, index: u32 },
}

impl ChrMemoryIndex {
    pub fn read_status(self) -> ReadStatus {
        match self {
            // FIXME: This should return Disabled, but that's currently not supported.
            Self::Absent => ReadStatus::Enabled,
            Self::Rom(_, read_status) | Self::Ram(_, read_status, _) => read_status,
            // FIXME: CIRAM can be disabled, and is disabled on hard reset.
            // That is already implemented, but must be hooked up properly.
            Self::Ciram(_, _) => ReadStatus::Enabled, // There's no way to disable reads to CIRAM.
            Self::MapperCustom { .. } => ReadStatus::Enabled, // FIXME: This is inaccurate.
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChrMapping {
    window: ChrWindow,
    rom_address_resolver: AddressResolver<ChrBankRegisterId>,
    ram_address_resolver: AddressResolver<ChrBankRegisterId>,
    // TODO: Figure out how to get rid of this field. It's only used for the PatternTable debug screen.
    pages_per_bank: u16,
    // TODO: Figure out how to get rid of this field. It's only used for the PatternTable debug screen.
    page_offset: u16,
    ciram_side: CiramSide,
    mem_type_status: ChrMemTypeStatus,
}

impl ChrMapping {
    pub fn from_name_table_source(
        name_table_source: NameTableSource,
        rom_address_resolver: AddressResolver<ChrBankRegisterId>,
        ram_address_resolver: AddressResolver<ChrBankRegisterId>,
    ) -> Self {
        let mut mapping = Self {
            window: ChrWindow::new(0, 0x1FFF, 8 * KIBIBYTE, ChrSourceProvider::Fixed(None)),
            rom_address_resolver,
            ram_address_resolver,
            pages_per_bank: 1,
            page_offset: 0,
            ciram_side: CiramSide::Left,
            mem_type_status: ChrMemTypeStatus::Rom(ReadStatus::Enabled),
        };
        mapping.window = match name_table_source {
            NameTableSource::Rom { bank_number } => mapping.window.fixed_index(bank_number.to_raw() as i16),
            NameTableSource::Ram { bank_number } => mapping.window.fixed_index(bank_number.to_raw() as i16),
            NameTableSource::Ciram(ciram_side) => mapping.window.ciram(ciram_side),
            NameTableSource::MapperCustom { page_id } => mapping.window.mapper_sourced(page_id),
        };

        mapping
    }

    pub fn from_name_table_source_with_register(
        window: ChrWindow,
        name_table_source: NameTableSource,
        rom_address_resolver: AddressResolver<ChrBankRegisterId>,
        ram_address_resolver: AddressResolver<ChrBankRegisterId>,
        source_id: ChrSourceRegisterId,
        regs: &mut ChrBankRegisters,
    ) -> Self {
        let mapping = Self {
            window,
            rom_address_resolver,
            ram_address_resolver,
            pages_per_bank: 1,
            page_offset: 0,
            ciram_side: CiramSide::Left,
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

    pub fn rom_page_number(self) -> u16 {
        self.rom_address_resolver.resolve_inner_bank_number() * self.pages_per_bank + self.page_offset
    }

    pub fn ram_page_number(self) -> u16 {
        self.ram_address_resolver.resolve_inner_bank_number() * self.pages_per_bank + self.page_offset
    }

    pub fn ciram_side(&self) -> CiramSide {
        self.ciram_side
    }

    pub fn mem_type_status(self) -> ChrMemTypeStatus {
        self.mem_type_status
    }

    pub fn to_name_table_source(self, regs: &ChrBankRegisters) -> Result<NameTableSource, String> {
        let chr_source = self.window.current_chr_source(regs).expect("NameTableSource can't come from an empty bank.");
        match chr_source {
            ChrSource::RomOrRam => {
                assert!(!regs.has_rom() || !regs.has_ram(),
                    "Don't know what to do with a Chr RomOrRam bank when the cartridge has both ROM and RAM.");
                if regs.has_rom() {
                    Ok(NameTableSource::Rom { bank_number: self.window.bank_number(regs).unwrap() })
                } else if regs.has_ram() {
                    Ok(NameTableSource::Ram { bank_number: self.window.bank_number(regs).unwrap() })
                } else {
                    Err("Absent CHR banks are not yet supported.".to_owned())
                }
            }
            ChrSource::Rom => Ok(NameTableSource::Rom { bank_number: self.window.bank_number(regs).unwrap() }),
            ChrSource::Ciram(ciram_side) => Ok(NameTableSource::Ciram(ciram_side)),
            ChrSource::WorkRam => Ok(NameTableSource::Ram { bank_number: self.window.bank_number(regs).unwrap() }),
            ChrSource::MapperCustom { page_id } => Ok(NameTableSource::MapperCustom { page_id }),
        }
    }

    pub fn update_page_id(&mut self, regs: &ChrBankRegisters) {
        let Self { window, .. } = self;

        let read_status = window.read_status_register_id().map_or(ReadStatus::Enabled, |id| regs.read_status(id));
        let write_status = window.write_status_register_id().map_or(WriteStatus::Enabled, |id| regs.write_status(id));
        match window.current_chr_source(regs) {
            None => todo!("EMPTY bank"),
            Some(ChrSource::RomOrRam) => {
                match (regs.has_rom(), regs.has_ram()) {
                    (false, false) => {
                        self.mem_type_status = ChrMemTypeStatus::Absent;
                    }
                    (true , true ) => {
                        warn!("Not sure what to do for a RomOrRam bank when both are present in the cartridge.");
                    }
                    (true , false) => {
                        self.mem_type_status = ChrMemTypeStatus::Rom(read_status);
                    }
                    (false, true ) => {
                        self.mem_type_status = ChrMemTypeStatus::Ram(read_status, write_status);
                    },
                }
            }
            Some(ChrSource::Rom) => {
                assert!(regs.has_rom());
                self.mem_type_status = ChrMemTypeStatus::Rom(read_status);
            }
            Some(ChrSource::WorkRam) => {
                assert!(regs.has_ram());
                self.mem_type_status = ChrMemTypeStatus::Ram(read_status, write_status);
            }
            Some(ChrSource::Ciram(ciram_side)) => {
                self.ciram_side = ciram_side;
                self.mem_type_status = ChrMemTypeStatus::Ciram;
            },
            Some(ChrSource::MapperCustom { page_id }) => {
                self.mem_type_status = ChrMemTypeStatus::MapperCustom { page_id };
            }
        }

        self.rom_address_resolver.update_chr_inner_bank_number(regs);
        self.ram_address_resolver.update_chr_inner_bank_number(regs);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ChrMemTypeStatus {
    Absent,
    Rom(ReadStatus),
    Ram(ReadStatus, WriteStatus),
    // TODO: Read/Write status here in order to disable CIRAM for mapper 111
    // Or maybe "Disabled" on the wiki just means unmapped?
    Ciram,
    // TODO: Should Read/WriteStatus be stored here?
    MapperCustom { page_id: u8 },
}