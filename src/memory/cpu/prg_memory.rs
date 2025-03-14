use std::u16;

use log::warn;

use crate::memory::bank::bank::Bank;
use crate::memory::bank::bank_index::{BankConfiguration, BankRegisters, RamStatus, RomRamMode};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::ppu::chr_memory::AccessOverride;
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::memory::read_result::ReadResult;
use crate::memory::window::{RamStatusInfo, Window};
use crate::util::unit::KIBIBYTE;

pub struct PrgMemory {
    layouts: Vec<PrgLayout>,
    layout_index: u8,
    prg_rom_bank_configuration: BankConfiguration,
    work_ram_bank_configuration: Option<BankConfiguration>,
    prg_rom_outer_banks: Vec<RawMemory>,
    prg_rom_outer_bank_index: u8,
    work_ram: RawMemory,
    extended_ram: RawMemoryArray<KIBIBYTE>,
    access_override: Option<AccessOverride>,
}

impl PrgMemory {
    pub fn new(
        layouts: Vec<PrgLayout>,
        layout_index: u8,
        bank_size_override: Option<u16>,
        prg_rom: RawMemory,
        outer_bank_count: u8,
        work_ram_size: u32,
        access_override: Option<AccessOverride>,
    ) -> PrgMemory {

        let mut bank_size = bank_size_override;
        if bank_size.is_none() {
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
        }

        let prg_rom_bank_size = bank_size.expect("at least one ROM or RAM window");

        let prg_rom_outer_banks = prg_rom.split_n(outer_bank_count);
        let outer_bank_0 = &prg_rom_outer_banks[0];

        let prg_rom_bank_count;
        if outer_bank_0.size() % u32::from(prg_rom_bank_size) == 0 {
            prg_rom_bank_count = (outer_bank_0.size() / u32::from(prg_rom_bank_size))
                .try_into()
                .expect("Way too many banks.");
        } else if !outer_bank_0.is_empty() && u32::from(prg_rom_bank_size) % outer_bank_0.size() == 0 {
            prg_rom_bank_count = 1;
        } else {
            panic!("Bad PRG length: {} . Bank size: {} .", outer_bank_0.size(), prg_rom_bank_size);
        }

        let mut work_ram_bank_configuration = None;
        let work_ram_windows: Vec<_> = layouts[layout_index as usize].windows().iter()
            .filter(|window| window.bank().is_prg_ram())
            .collect();
        if !work_ram_windows.is_empty() && work_ram_size > 0 {
            let mut work_ram_page_size = u16::MAX;
            for window in work_ram_windows {
                if window.size() < work_ram_page_size {
                    work_ram_page_size = window.size();
                }
            }

            let work_ram_bank_count: u16 = (work_ram_size / u32::from(work_ram_page_size)).try_into().unwrap();
            work_ram_bank_configuration = Some(BankConfiguration::new(work_ram_page_size, work_ram_bank_count, true));
        } else if work_ram_size > 0 {
            warn!("Work RAM specified in ROM file, but not in layout.");
        }

        let prg_rom_bank_configuration = BankConfiguration::new(prg_rom_bank_size, prg_rom_bank_count, true);
        let prg_memory = PrgMemory {
            layouts,
            layout_index,
            prg_rom_bank_configuration,
            work_ram_bank_configuration,
            prg_rom_outer_banks,
            prg_rom_outer_bank_index: 0,
            work_ram: RawMemory::new(work_ram_size),
            extended_ram: RawMemoryArray::new(),
            access_override,
        };

        let bank_count = prg_memory.bank_count();
        if prg_memory.prg_rom_outer_banks[0].size() >= bank_count as u32 * prg_rom_bank_size as u32 {
            assert_eq!(
                prg_memory.prg_rom_outer_banks[0].size(),
                bank_count as u32 * prg_rom_bank_size as u32,
                "Bad PRG data size.",
            );
        }
        //assert_eq!(bank_count & (bank_count - 1), 0);

        prg_memory
    }

    pub fn bank_configuration(&self) -> BankConfiguration {
        self.prg_rom_bank_configuration
    }

    pub fn work_ram_bank_configuration(&self) -> Option<BankConfiguration> {
        self.work_ram_bank_configuration
    }

    pub fn bank_size(&self) -> u16 {
        self.prg_rom_bank_configuration.bank_size()
    }

    pub fn bank_count(&self) -> u16 {
        self.prg_rom_bank_configuration.bank_count()
    }

    pub fn last_bank_index(&self) -> u16 {
        self.bank_count() - 1
    }

    pub fn layout_index(&self) -> u8 {
        self.layout_index
    }

    pub fn extended_ram(&self) -> &RawMemoryArray<KIBIBYTE> {
        &self.extended_ram
    }

    pub fn extended_ram_mut(&mut self) -> &mut RawMemoryArray<KIBIBYTE> {
        &mut self.extended_ram
    }

    pub fn access_override(&self) -> Option<AccessOverride> {
        self.access_override
    }

    pub fn ram_status_infos(&self) -> Vec<RamStatusInfo> {
        let mut ids = Vec::new();
        for layout in &self.layouts {
            ids.append(&mut layout.ram_status_infos());
        }

        ids
    }

    pub fn peek(&self, registers: &BankRegisters, address: CpuAddress) -> ReadResult {
        match self.address_to_prg_index(registers, address) {
            PrgMemoryIndex::None => ReadResult::OPEN_BUS,
            PrgMemoryIndex::MappedMemory {index, ram_status } => {
                use RamStatus::*;
                match ram_status {
                    Disabled | WriteOnly =>
                        ReadResult::OPEN_BUS,
                    ReadOnlyZeros =>
                        ReadResult::full(0),
                    ReadOnly | ReadWrite =>
                        ReadResult::full(self.current_outer_prg_rom_bank()[index % self.current_outer_prg_rom_bank().size()]),
                }
            }
            PrgMemoryIndex::WorkRam { index, ram_status} => {
                use RamStatus::*;
                match ram_status {
                    Disabled | WriteOnly =>
                        ReadResult::OPEN_BUS,
                    ReadOnlyZeros =>
                        ReadResult::full(0),
                    ReadOnly | ReadWrite =>
                        ReadResult::full(self.work_ram[index]),
                }
            }
            PrgMemoryIndex::ExtendedRam { index, ram_status} => {
                use RamStatus::*;
                match ram_status {
                    Disabled | WriteOnly =>
                        ReadResult::OPEN_BUS,
                    ReadOnlyZeros =>
                        ReadResult::full(0),
                    ReadOnly | ReadWrite =>
                        ReadResult::full(self.extended_ram[index]),
                }
            }
        }
    }

    pub fn write(&mut self, registers: &BankRegisters, address: CpuAddress, value: u8) {
        let windows = &self.current_layout().windows();
        assert!(!windows.is_empty());
        if address.to_raw() < windows[0].start() {
            return;
        }

        match self.address_to_prg_index(registers, address) {
            PrgMemoryIndex::None => {}
            PrgMemoryIndex::MappedMemory { index, ram_status } => {
                if ram_status.is_writable() {
                    self.current_outer_prg_rom_bank_mut()[index] = value;
                }
            }
            PrgMemoryIndex::WorkRam { index, ram_status} => {
                if ram_status.is_writable() {
                    self.work_ram[index] = value;
                }
            }
            PrgMemoryIndex::ExtendedRam { index, ram_status} => {
                if ram_status.is_writable() {
                    self.extended_ram[index] = value;
                }
            }
        }
    }

    pub fn window_at(&self, start: u16) -> &Window {
        self.window_with_index_at(start).0
    }

    pub fn current_layout(&self) -> &PrgLayout {
        &self.layouts[usize::from(self.layout_index)]
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn set_prg_rom_outer_bank_index(&mut self, index: u8) {
        assert!(index < self.prg_rom_outer_banks.len().try_into().unwrap());
        self.prg_rom_outer_bank_index = index;
    }

    fn address_to_prg_index(&self, registers: &BankRegisters, address: CpuAddress) -> PrgMemoryIndex {
        let address = address.to_raw();
        assert!(address >= 0x4020);

        let windows = &self.current_layout().windows();
        assert!(!windows.is_empty());
        if address < windows[0].start() {
            // Translates to open bus for reads, and an ignored write for writes.
            // Necessary to support mappers that configure memory between 0x4020 and 0x5FFF.
            return PrgMemoryIndex::None;
        }

        for i in 0..windows.len() {
            if i == windows.len() - 1 || address < windows[i + 1].start() {
                let bank_offset = address - windows[i].start();

                let window;
                if let Bank::MirrorOf(mirrored_window_start) = windows[i].bank() {
                    window = self.window_at(mirrored_window_start);
                } else {
                    window = &windows[i];
                }

                let prg_memory_index = match window.bank() {
                    Bank::Empty => PrgMemoryIndex::None,
                    Bank::MirrorOf(_) => panic!("A mirrored bank must mirror a non-mirrored bank."),
                    Bank::Rom(location) => {
                        let resolved_bank_index =
                            window.resolved_bank_index(registers, location, self.prg_rom_bank_configuration);
                        let index = resolved_bank_index as u32 * self.prg_rom_bank_configuration.bank_size() as u32 + bank_offset as u32;
                        PrgMemoryIndex::MappedMemory { index, ram_status: RamStatus::ReadOnly }
                    }
                    Bank::Ram(location, status_register_id) => {
                        let work_ram_bank_configuration = self.work_ram_bank_configuration
                            .expect("PRG RAM window specified in layout, but not in cartridge.");
                        let resolved_bank_index =
                            window.resolved_bank_index(registers, location, work_ram_bank_configuration);
                        let index = resolved_bank_index as u32 * work_ram_bank_configuration.bank_size() as u32 + bank_offset as u32;
                        let ram_status: RamStatus = status_register_id
                            .map_or(RamStatus::ReadWrite, |id| registers.ram_status(id));
                        PrgMemoryIndex::WorkRam { index, ram_status }
                    }
                    Bank::RomRam(location, status_register_id, mode_register_id) => {
                        match registers.rom_ram_mode(mode_register_id) {
                            RomRamMode::Ram => {
                                let work_ram_bank_configuration = self.work_ram_bank_configuration
                                    .expect("PRG RAM window specified in layout, but not in cartridge.");
                                let resolved_bank_index =
                                    window.resolved_bank_index(registers, location, work_ram_bank_configuration);
                                let index = resolved_bank_index as u32 * work_ram_bank_configuration.bank_size() as u32 + bank_offset as u32;
                                let ram_status: RamStatus = registers.ram_status(status_register_id);
                                PrgMemoryIndex::WorkRam { index, ram_status }
                            }
                            RomRamMode::Rom => {
                                let resolved_bank_index =
                                    window.resolved_bank_index(registers, location, self.prg_rom_bank_configuration);
                                let index = resolved_bank_index as u32 * self.prg_rom_bank_configuration.bank_size() as u32 + bank_offset as u32;
                                PrgMemoryIndex::MappedMemory { index, ram_status: RamStatus::ReadOnly }
                            }
                        }
                    }
                    Bank::WorkRam(location, status_register_id) => {
                        let Some(work_ram_bank_configuration) = self.work_ram_bank_configuration else {
                            return PrgMemoryIndex::None;
                        };

                        let resolved_bank_index =
                            window.resolved_bank_index(registers, location, work_ram_bank_configuration);
                        let index = resolved_bank_index as u32 * work_ram_bank_configuration.bank_size() as u32 + bank_offset as u32;
                        let ram_status: RamStatus = status_register_id
                            .map_or(RamStatus::ReadWrite, |id| registers.ram_status(id));
                        PrgMemoryIndex::WorkRam { index, ram_status }
                    }
                    Bank::ExtendedRam(status_register_id) => {
                        let index = u32::from(bank_offset);
                        let ram_status: RamStatus = status_register_id
                            .map_or(RamStatus::ReadWrite, |id| registers.ram_status(id));
                        PrgMemoryIndex::ExtendedRam { index, ram_status }
                    }
                };
                return prg_memory_index;
            }
        }

        unreachable!();
    }

    fn window_with_index_at(&self, start: u16) -> (&Window, u32) {
        for (index, window) in self.current_layout().windows().iter().enumerate() {
            if window.start() == start {
                return (window, index as u32);
            }
        }

        panic!("No window exists at {start:?}");
    }

    fn current_outer_prg_rom_bank(&self) -> &RawMemory {
        &self.prg_rom_outer_banks[self.prg_rom_outer_bank_index as usize]
    }

    fn current_outer_prg_rom_bank_mut(&mut self) -> &mut RawMemory {
        &mut self.prg_rom_outer_banks[self.prg_rom_outer_bank_index as usize]
    }
}

enum PrgMemoryIndex {
    None,
    WorkRam { index: u32, ram_status: RamStatus },
    ExtendedRam { index: u32, ram_status: RamStatus },
    MappedMemory { index: u32, ram_status: RamStatus },
}