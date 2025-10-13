use std::num::NonZeroU8;

use log::{error, warn};

use crate::mapper::{BankNumber, ChrBank, ChrBankRegisterId, ChrWindow, MetaRegisterId, NameTableMirroring, NameTableQuadrant, NameTableSource, ReadWriteStatus, ReadWriteStatusRegisterId};
use crate::memory::bank::bank::RomRamModeRegisterId;
use crate::memory::bank::bank_number::{ChrBankRegisters, MemType};
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::chr_memory_map::{ChrMemoryMap, ChrMapping, ChrMemoryIndex};
use crate::memory::ppu::ciram::Ciram;
use crate::memory::raw_memory::{RawMemory, RawMemorySlice};
use crate::memory::window::{ChrWindowSize, ReadWriteStatusInfo};
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::util::unit::KIBIBYTE;

use super::ciram::CiramSide;

pub struct ChrMemory {
    layouts: Vec<ChrLayout>,
    memory_maps: Vec<ChrMemoryMap>,
    rom_outer_banks: Vec<RawMemory>,
    rom_outer_bank_number: u8,
    ram: RawMemory,
    bank_size: ChrWindowSize,
    regs: ChrBankRegisters,

    layout_index: u8,
}

impl ChrMemory {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layouts: Vec<ChrLayout>,
        layout_index: u8,
        align_large_chr_banks: bool,
        rom_outer_bank_count: NonZeroU8,
        mut rom: RawMemory,
        mut ram: RawMemory,
        name_table_mirroring: NameTableMirroring,
        regs: ChrBankRegisters,
    ) -> ChrMemory {

        let mut bank_size = None;
        let mut rom_present_in_layout = false;
        let mut ram_present_in_layout = false;
        for layout in &layouts {
            for window in layout.windows() {
                if matches!(window.bank(), ChrBank::Rom(..) | ChrBank::Ram(..) | ChrBank::RomRam(..)) {
                    if let Some(size) = bank_size {
                        bank_size = Some(std::cmp::min(window.size(), size));
                    } else {
                        bank_size = Some(window.size());
                    }
                }

                if window.bank().is_rom() {
                    rom_present_in_layout = true;
                }

                if window.bank().is_ram() {
                    ram_present_in_layout = true;
                }
            }
        }

        // The page size for CHR ROM and CHR RAM appear to always match each other.
        let bank_size = bank_size.expect("at least one CHR ROM or CHR RAM window");
        if !rom.is_empty() && !ram.is_empty() {
            if !rom_present_in_layout {
                warn!("The CHR ROM that was specified in the rom file will be ignored since it is not \
                        configured in the Layout for this mapper.");
                rom = RawMemory::new(0);
            }

            if !ram_present_in_layout {
                warn!("The CHR RAM that was specified in the rom file will be ignored since it is not \
                        configured in the Layout for this mapper.");
                ram = RawMemory::new(0);
            }
        }

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
                align_large_chr_banks,
                &regs,
        )).collect();

        ChrMemory {
            layouts,
            memory_maps,
            layout_index,
            rom_outer_banks: rom.split_n(rom_outer_bank_count),
            rom_outer_bank_number: 0,
            ram: ram.clone(),
            bank_size,
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

    pub fn peek(&self, ciram: &Ciram, address: PpuAddress) -> PpuPeek {
        let (index, source, _read_write_status) = self.current_memory_map().index_for_address(address);

        let value = match index {
            ChrMemoryIndex::Rom(index) => {
                self.rom_outer_banks[self.rom_outer_bank_number as usize][index % self.rom_outer_banks[0].size()]
            },
            ChrMemoryIndex::Ram(index) => {
                self.ram[index % self.ram.size()]
            }
            ChrMemoryIndex::Ciram(side, index) => {
                ciram.side(side)[index as usize]
            }
            ChrMemoryIndex::SaveRam(_index) => todo!(),
            ChrMemoryIndex::ExtendedRam(_index) => todo!(),
            ChrMemoryIndex::FillModeTile => todo!(),
        };

        PpuPeek { value, source }
    }

    pub fn peek_raw(&self, index: u32) -> PpuPeek {
        match (self.rom_present(), self.ram_present()) {
            (false, false) => panic!("CHR ROM or RAM must be present for peek_raw."),
            (true , true ) => panic!("CHR ROM and RAM must not both be present for peek_raw."),
            (true , false) => {
                PpuPeek::new(
                    self.rom_outer_banks[self.rom_outer_bank_number as usize][index % self.rom_outer_banks[0].size()],
                    PeekSource::Rom(0.into()),
                )
            }
            (false, true ) => {
                PpuPeek::new(
                    self.ram[index % self.ram.size()],
                    PeekSource::Ram(0.into()),
                )
            }
        }
    }

    pub fn write(&mut self, regs: &PpuRegisters, ciram: &mut Ciram, address: PpuAddress, value: u8) {
        let (chr_memory_index, _, read_write_status) = self.current_memory_map().index_for_address(address);
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
                ciram.write(regs, side, index, value);
            }
            ChrMemoryIndex::SaveRam(_index) => todo!(),
            ChrMemoryIndex::ExtendedRam(_index) => todo!(),
            ChrMemoryIndex::FillModeTile => todo!(),
        }
    }

    pub fn set_rom_ram_mode(&mut self, id: RomRamModeRegisterId, rom_ram_mode: MemType) {
        self.regs.set_rom_ram_mode(id, rom_ram_mode);
        self.update_page_ids();
    }

    pub fn window_at(&self, start: u16) -> &ChrWindow {
        for window in self.current_layout().windows() {
            if window.start() == start {
                return window;
            }
        }

        panic!("No window exists at {start:X}");
    }

    pub fn rom_bank_count(&self) -> u16 {
        if self.rom_outer_banks.is_empty() {
            return 0;
        }

        let bank_size = u32::from(self.bank_size.to_raw());
        assert_eq!(self.rom_outer_banks[0].size() % bank_size, 0);
        (self.rom_outer_banks[0].size() / bank_size).try_into().unwrap()
    }

    pub fn ram_bank_count(&self) -> u16 {
        let bank_size = u32::from(self.bank_size.to_raw());
        assert_eq!(self.ram.size() % bank_size, 0);
        (self.ram.size() / bank_size).try_into().unwrap()
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

    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        let quadrants = &self.memory_maps[0].page_mappings()[8..12];
        use ChrMapping::*;
        match (quadrants[0], quadrants[1], quadrants[2], quadrants[3]) {
            (NameTableSource(top_left), NameTableSource(top_right), NameTableSource(bottom_left), NameTableSource(bottom_right)) =>
                NameTableMirroring::new(top_left, top_right, bottom_left, bottom_right),
            _ => {
                error!("Unexpected NameTable source! This mapper is not properly supported. Using an incorrect NameTableMirroring in order to proceed.");
                NameTableMirroring::HORIZONTAL
            }
        }
    }

    pub fn bank_registers(&self) -> &ChrBankRegisters {
        &self.regs
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn set_chr_rom_outer_bank_number(&mut self, index: u8) {
        self.rom_outer_bank_number = index;
    }

    pub fn set_bank_register<INDEX: Into<u16>>(&mut self, id: ChrBankRegisterId, value: INDEX) {
        self.regs.set(id, BankNumber::from_u16(value.into()));
        self.update_page_ids();
    }

    pub fn set_bank_register_bits(&mut self, id: ChrBankRegisterId, new_value: u16, mask: u16) {
        self.regs.set_bits(id, new_value, mask);
        self.update_page_ids();
    }

    pub fn set_meta_register(&mut self, id: MetaRegisterId, value: ChrBankRegisterId) {
        self.regs.set_meta_chr(id, value);
        self.update_page_ids();
    }

    pub fn update_bank_register(
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
        for memory_map in &mut self.memory_maps {
            memory_map.set_name_table_mirroring(&self.regs, name_table_mirroring);
        }
    }

    pub fn set_name_table_quadrant(&mut self, quadrant: NameTableQuadrant, source: NameTableSource) {
        for memory_map in &mut self.memory_maps {
            memory_map.set_name_table_quadrant(&self.regs, quadrant, source);
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

    pub fn work_ram_1kib_page(&self, start: u32) -> &[u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        let start = start as usize;
        (&self.ram.as_slice()[start..start + 0x400]).try_into().unwrap()
    }

    pub fn work_ram_1kib_page_mut(&mut self, start: u32) -> &mut [u8; KIBIBYTE as usize] {
        assert_eq!(start % 0x400, 0, "Save RAM 1KiB slices must start on a 1KiB page boundary (e.g. 0x000, 0x400, 0x800).");
        let start = start as usize;
        (&mut self.ram.as_mut_slice()[start..start + 0x400]).try_into().unwrap()
    }

    fn rom_present(&self) -> bool {
        !self.rom_outer_banks.is_empty()
    }

    fn ram_present(&self) -> bool {
        !self.ram.is_empty()
    }

    #[inline]
    pub fn left_chunks<'a>(&'a self, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        let mem = self.current_memory_map();
        [mem.page_start_index(0), mem.page_start_index(1), mem.page_start_index(2), mem.page_start_index(3)]
            .map(move |chr_index| {
                match chr_index {
                    ChrMemoryIndex::Rom(index) => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(
                            &self.rom_outer_banks[self.rom_outer_bank_number as usize].as_slice()[index..index + 1 * KIBIBYTE as usize])
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
    pub fn right_chunks<'a>(&'a self, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        let mem = self.current_memory_map();
        [mem.page_start_index(4), mem.page_start_index(5), mem.page_start_index(6), mem.page_start_index(7)]
            .map(move |chr_index| {
                match chr_index {
                    ChrMemoryIndex::Rom(index) => {
                        let index = index as usize;
                        RawMemorySlice::from_raw(
                            &self.rom_outer_banks[self.rom_outer_bank_number as usize].as_slice()[index..index + 1 * KIBIBYTE as usize])
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

#[derive(Clone, Copy)]
pub struct PpuPeek {
    value: u8,
    source: PeekSource,
}

impl PpuPeek {
    pub const ZERO: PpuPeek = PpuPeek { value: 0, source: PeekSource::Rom(BankNumber::from_u8(0)) };

    pub fn new(value: u8, source: PeekSource) -> Self {
        Self { value, source }
    }

    pub fn value(self) -> u8 {
        self.value
    }

    pub fn source(self) -> PeekSource {
        self.source
    }
}

#[derive(Clone, Copy)]
pub enum PeekSource {
    Rom(BankNumber),
    Ram(BankNumber),
    SaveRam,
    Ciram(CiramSide),
    PaletteTable,
    ExtendedRam,
    FillModeTile,
}

impl PeekSource {
    pub fn from_name_table_source(name_table_source: NameTableSource) -> Self {
        match name_table_source {
            NameTableSource::Ciram(side) => Self::Ciram(side),
            NameTableSource::Ram { bank_number } => Self::Ram(bank_number),
            NameTableSource::ExtendedRam => Self::ExtendedRam,
            NameTableSource::FillModeTile => Self::FillModeTile,
        }
    }
}