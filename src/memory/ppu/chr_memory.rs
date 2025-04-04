use std::num::NonZeroU8;

use crate::memory::bank::bank::Bank;
use crate::memory::bank::bank_index::{BankConfiguration, BankRegisters};
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
    rom_bank_configuration: Option<BankConfiguration>,
    ram_bank_configuration: Option<BankConfiguration>,
    max_pattern_table_index: u16,
    access_override: Option<AccessOverride>,
    rom_outer_banks: Vec<RawMemory>,
    rom_outer_bank_index: usize,
    ram: RawMemory,
}

impl ChrMemory {
    pub fn new(
        layouts: Vec<ChrLayout>,
        layout_index: u8,
        align_large_chr_layouts: bool,
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

        let bank_size = bank_size.expect("at least one CHR ROM or CHR RAM window");

        let max_pattern_table_index = layouts[0].max_window_index();
        for layout in &layouts {
            assert_eq!(layout.max_window_index(), max_pattern_table_index,
                "The max CHR window index must be the same between all layouts.");
        }

        let rom_outer_banks = rom.split_n(outer_bank_count);

        let rom_bank_configuration = if let Some(outer_bank_0) = rom_outer_banks.first() {
            let bank_count = (outer_bank_0.size() / u32::from(bank_size.get()))
                .try_into()
                .expect("Way too many CHR ROM banks.");
            Some(BankConfiguration::new(bank_size.get(), bank_count, align_large_chr_layouts))
        } else {
            None
        };

        let ram_bank_count = (ram.size() / u32::from(bank_size.get()))
            .try_into()
            .expect("Way too many CHR RAM banks.");
        let ram_bank_configuration = if ram_bank_count == 0 {
            None
        } else {
            Some(BankConfiguration::new(bank_size.get(), ram_bank_count, align_large_chr_layouts))
        };

        let chr_memory = ChrMemory {
            layouts,
            layout_index,
            rom_bank_configuration,
            ram_bank_configuration,
            max_pattern_table_index,
            access_override,
            rom_outer_banks,
            rom_outer_bank_index: 0,
            ram,
        };

        if let Some(bank_count) = chr_memory.rom_bank_count() {
            assert_eq!(u32::from(bank_count) * u32::from(chr_memory.bank_size()), chr_memory.rom_outer_banks[0].size());
        }
        // Power of 2. FIXME: What's the correct behavior when accessing the high banks? Open bus?
        // assert_eq!(bank_count & (bank_count - 1), 0, "Bank count ({bank_count}) must be a power of 2.");

        chr_memory
    }

    pub fn rom_bank_configuration(&self) -> Option<BankConfiguration> {
        self.rom_bank_configuration
    }

    pub fn ram_bank_configuration(&self) -> Option<BankConfiguration> {
        self.ram_bank_configuration
    }

    #[inline]
    pub fn rom_bank_count(&self) -> Option<u16> {
        self.rom_bank_configuration.map(|c| c.bank_count())
    }

    pub fn bank_size(&self) -> u16 {
        self.rom_bank_configuration
            .unwrap_or_else(|| self.ram_bank_configuration.unwrap())
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
            ChrIndex::Rom(index) => {
                if self.access_override == Some(AccessOverride::ForceRam) {
                    self.ram[index]
                } else {
                    self.rom_outer_banks[self.rom_outer_bank_index][index]
                }
            }
            ChrIndex::Ram(index) => self.ram[index],
            ChrIndex::Ciram(side, index, ) => ciram.side(side)[index as usize],
        }
    }

    pub fn write(&mut self, registers: &BankRegisters, ciram: &mut Ciram, address: PpuAddress, value: u8) {
        if self.access_override == Some(AccessOverride::ForceRom) {
            return;
        }

        let (chr_index, writable) = self.address_to_chr_index(registers, address.to_u16());
        if writable || self.access_override == Some(AccessOverride::ForceRam) {
            match chr_index {
                ChrIndex::Rom(index) => {
                    if self.access_override == Some(AccessOverride::ForceRam) {
                        self.ram[index] = value;
                    }
                }
                ChrIndex::Ram(index) => self.ram[index] = value,
                ChrIndex::Ciram(side, index, ) => ciram.side_mut(side)[index as usize] = value,
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
        assert!(index < self.rom_outer_banks.len().try_into().unwrap());
        self.rom_outer_bank_index = index as usize;
    }

    pub fn pattern_table<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram, side: PatternTableSide) -> PatternTable<'a> {
        match side {
            PatternTableSide::Left => PatternTable::new(self.left_chunks(registers, ciram)),
            PatternTableSide::Right => PatternTable::new(self.right_chunks(registers, ciram)),
        }
    }

    pub fn chr_ram_slice<const SIZE: usize>(&self, start: u32) -> &[u8; SIZE] {
        self.ram.sized_slice(start)
    }

    pub fn chr_ram_slice_mut<const SIZE: usize>(&mut self, start: u32) -> &mut [u8; SIZE] {
        self.ram.sized_slice_mut(start)
    }

    fn address_to_chr_index(&self, registers: &BankRegisters, address: u16) -> (ChrIndex, bool) {
        assert!(address <= self.max_pattern_table_index);
        for window in self.current_layout().windows() {
            if let Some(bank_offset) = window.offset(address) {
                let location = window.resolved_bank_location(
                    registers,
                    window.location().unwrap(),
                    self.rom_bank_configuration,
                    self.ram_bank_configuration,
                    self.access_override,
                );

                match location {
                    ChrLocation::RomBankIndex(raw_bank_index) => {
                        let index = u32::from(raw_bank_index) *
                            u32::from(self.bank_size()) +
                            u32::from(bank_offset);
                        return (ChrIndex::Rom(index), false);
                    }
                    ChrLocation::RamBankIndex(raw_bank_index) => {
                        let index = u32::from(raw_bank_index) *
                            u32::from(self.bank_size()) +
                            u32::from(bank_offset);
                        return (ChrIndex::Ram(index), window.is_writable(registers));
                    }
                    ChrLocation::Ciram(side) => {
                        return (ChrIndex::Ciram(side, bank_offset), true);
                    }
                }
            }
        }

        unreachable!();
    }

    #[inline]
    fn left_chunks<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        self.left_indexes(registers)
            .map(move |chr_index| {
                match chr_index {
                    ChrIndex::Rom(index) => self.rom_outer_banks[self.rom_outer_bank_index]
                        .slice(index..index + 1 * KIBIBYTE),
                    ChrIndex::Ram(index) => self.ram.slice(index..index + 1 * KIBIBYTE),
                    ChrIndex::Ciram(side, ..) => RawMemorySlice::from_raw(ciram.side(side)),
                }
        })
    }

    #[inline]
    fn right_chunks<'a>(&'a self, registers: &BankRegisters, ciram: &'a Ciram) -> [RawMemorySlice<'a>; 4] {
        self.right_indexes(registers)
            .map(move |chr_index| {
                match chr_index {
                    ChrIndex::Rom(index) => self.rom_outer_banks[self.rom_outer_bank_index]
                        .slice(index..index + 1 * KIBIBYTE),
                    ChrIndex::Ram(index) => self.ram.slice(index..index + 1 * KIBIBYTE),
                    ChrIndex::Ciram(side, ..) => RawMemorySlice::from_raw(ciram.side(side)),
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
    Rom(u32),
    Ram(u32),
    Ciram(CiramSide, u16),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AccessOverride {
    ForceRom,
    ForceRam,
}