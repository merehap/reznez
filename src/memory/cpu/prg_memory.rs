use std::num::{NonZeroU8, NonZeroU16};

use log::warn;

use crate::memory::bank::bank::Bank;
use crate::memory::bank::bank_index::{BankConfiguration, BankRegisters, ReadWriteStatus, RomRamMode};
use crate::memory::bank::page::{OuterPageTable, OuterPage};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::ppu::chr_memory::AccessOverride;
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::memory::read_result::ReadResult;
use crate::memory::window::{ReadWriteStatusInfo, Window};
use crate::util::unit::KIBIBYTE;

pub struct PrgMemory {
    layouts: Vec<PrgLayout>,
    layout_index: u8,
    rom_outer_banks: OuterPageTable,
    work_ram: Option<OuterPage>,
    extended_ram: RawMemoryArray<KIBIBYTE>,
    access_override: Option<AccessOverride>,
}

impl PrgMemory {
    pub fn new(
        layouts: Vec<PrgLayout>,
        layout_index: u8,
        rom_page_size_override: Option<NonZeroU16>,
        prg_rom: RawMemory,
        rom_outer_bank_count: NonZeroU8,
        prg_ram_size: u32,
        access_override: Option<AccessOverride>,
    ) -> PrgMemory {

        let mut rom_page_size = rom_page_size_override;
        if rom_page_size.is_none() {
            for layout in &layouts {
                for window in layout.windows() {
                    if matches!(window.bank(), Bank::Rom(..) | Bank::Ram(..)) {
                        if let Some(size) = rom_page_size {
                            rom_page_size = Some(std::cmp::min(window.size(), size));
                        } else {
                            rom_page_size = Some(window.size());
                        }
                    }
                }
            }
        }

        let rom_page_size = rom_page_size.expect("at least one ROM or RAM window");

        let rom_outer_banks = OuterPageTable::new(prg_rom, rom_outer_bank_count, rom_page_size, true)
            .expect("PRG ROM must not be empty.");

        let work_ram_windows: Vec<_> = layouts[layout_index as usize].windows().iter()
            .filter(|window| window.bank().is_prg_ram())
            .collect();
        let work_ram = if !work_ram_windows.is_empty() && prg_ram_size > 0 {
            let mut work_ram_page_size = NonZeroU16::MAX;
            for window in work_ram_windows {
                if window.size() < work_ram_page_size {
                    work_ram_page_size = window.size();
                }
            }

            OuterPage::new(RawMemory::new(prg_ram_size), work_ram_page_size, true)
        } else if prg_ram_size > 0 {
            warn!("Work RAM specified in ROM file, but not in layout.");
            None
        } else {
            None
        };

        let prg_memory = PrgMemory {
            layouts,
            layout_index,
            rom_outer_banks,
            work_ram,
            extended_ram: RawMemoryArray::new(),
            access_override,
        };

        let bank_count = prg_memory.rom_bank_count();
        let rom_outer_page_size = prg_memory.rom_outer_banks.outer_page_size().get();
        let rom_page_size = prg_memory.rom_outer_banks.page_size().get();
        if rom_outer_page_size >= bank_count as u32 * rom_page_size as u32 {
            assert_eq!(rom_outer_page_size, bank_count as u32 * rom_page_size as u32, "Bad PRG data size.");
        }
        //assert_eq!(bank_count & (bank_count - 1), 0);

        prg_memory
    }

    pub fn bank_configuration(&self) -> BankConfiguration {
        self.rom_outer_banks.bank_configuration()
    }

    pub fn work_ram_bank_configuration(&self) -> Option<BankConfiguration> {
        self.work_ram.as_ref().map(|wr| wr.bank_configuration())
    }

    pub fn rom_bank_size(&self) -> u16 {
        self.rom_outer_banks.page_size().get()
    }

    pub fn rom_bank_count(&self) -> u16 {
        self.rom_outer_banks.page_count().get()
    }

    pub fn last_rom_bank_index(&self) -> u16 {
        self.rom_bank_count() - 1
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

    pub fn read_write_status_infos(&self) -> Vec<ReadWriteStatusInfo> {
        let mut ids = Vec::new();
        for layout in &self.layouts {
            ids.append(&mut layout.read_write_status_infos());
        }

        ids
    }

    pub fn peek(&self, registers: &BankRegisters, address: CpuAddress) -> ReadResult {
        use ReadWriteStatus::*;
        match self.address_to_prg_index(registers, address) {
            PrgMemoryIndex::None => ReadResult::OPEN_BUS,
            PrgMemoryIndex::Rom { page_number, index, read_status: read_write_status } => {
                match read_write_status {
                    Disabled | WriteOnly =>
                        ReadResult::OPEN_BUS,
                    ReadOnlyZeros =>
                        ReadResult::full(0),
                    ReadOnly | ReadWrite =>
                        ReadResult::full(self.rom_outer_banks.current_outer_page().page(page_number).peek(index)),
                }
            }
            PrgMemoryIndex::Ram { page_number, index, read_write_status} => {
                match read_write_status {
                    Disabled | WriteOnly =>
                        ReadResult::OPEN_BUS,
                    ReadOnlyZeros =>
                        ReadResult::full(0),
                    ReadOnly | ReadWrite => {
                        let work_ram = self.work_ram.as_ref().expect("PRG RAM to be present since it is being peeked at.");
                        ReadResult::full(work_ram.page(page_number).peek(index))
                    }
                }
            }
            PrgMemoryIndex::ExtendedRam { index, read_write_status} => {
                match read_write_status {
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
            PrgMemoryIndex::Rom { .. } => { /* ROM is readonly. */}
            PrgMemoryIndex::Ram { page_number, index, read_write_status } => {
                if read_write_status.is_writable() {
                    let work_ram = self.work_ram.as_mut().expect("PRG RAM to be present since it is being written to.");
                    work_ram.page_mut(page_number).write(index, value)
                }
            }
            PrgMemoryIndex::ExtendedRam { index, read_write_status} => {
                if read_write_status.is_writable() {
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
        self.rom_outer_banks.set_outer_page_index(index);
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

        let window_index = (1..windows.len())
            .find(|&i| address < windows[i].start())
            .unwrap_or(windows.len())
            - 1;

        let mut window = &windows[window_index];
        let bank_offset = address - window.start();
        if let Bank::MirrorOf(mirrored_window_start) = window.bank() {
            window = self.window_at(mirrored_window_start);
        }

        match window.bank() {
            Bank::Empty => PrgMemoryIndex::None,
            Bank::MirrorOf(_) => panic!("A mirrored bank must mirror a non-mirrored bank."),
            Bank::Rom(location, status_register_id) => {
                let read_status: ReadWriteStatus = status_register_id
                    .map_or(ReadWriteStatus::ReadOnly, |id| registers.read_write_status(id));
                assert!(!read_status.is_writable());

                let mut page_number = window.resolved_bank_index(registers, location, self.rom_outer_banks.bank_configuration());
                let mut index = bank_offset;
                // For windows that are larger than the page size,
                // indexes need to be reduced and page numbers need to be increased.
                while index >= self.rom_outer_banks.page_size().get() {
                    page_number += 1;
                    index -= self.rom_outer_banks.page_size().get();
                }

                PrgMemoryIndex::Rom { page_number, index, read_status }
            }
            Bank::Ram(location, status_register_id) => {
                let work_ram_bank_configuration = self.work_ram_bank_configuration()
                    .expect("PRG RAM window specified in layout, but not in cartridge.");
                let page_number =
                    window.resolved_bank_index(registers, location, work_ram_bank_configuration);
                let index = bank_offset;
                let read_write_status: ReadWriteStatus = status_register_id
                    .map_or(ReadWriteStatus::ReadWrite, |id| registers.read_write_status(id));
                PrgMemoryIndex::Ram { page_number, index, read_write_status }
            }
            Bank::RomRam(location, status_register_id, mode_register_id) => {
                match registers.rom_ram_mode(mode_register_id) {
                    RomRamMode::Ram => {
                        let work_ram_bank_configuration = self.work_ram_bank_configuration()
                            .expect("PRG RAM window specified in layout, but not in cartridge.");
                        let page_number =
                            window.resolved_bank_index(registers, location, work_ram_bank_configuration);
                        let index = bank_offset;
                        let read_write_status: ReadWriteStatus = registers.read_write_status(status_register_id);
                        PrgMemoryIndex::Ram { page_number, index, read_write_status }
                    }
                    RomRamMode::Rom => {
                        let mut page_number = window.resolved_bank_index(registers, location, self.rom_outer_banks.bank_configuration());
                        let mut index = bank_offset;
                        // For windows that are larger than the page size,
                        // indexes need to be reduced and page numbers need to be increased.
                        while index >= self.rom_outer_banks.page_size().get() {
                            page_number += 1;
                            index -= self.rom_outer_banks.page_size().get();
                        }
                        PrgMemoryIndex::Rom { page_number, index, read_status: ReadWriteStatus::ReadOnly }
                    }
                }
            }
            Bank::WorkRam(location, status_register_id) => {
                let Some(work_ram_bank_configuration) = self.work_ram_bank_configuration() else {
                    return PrgMemoryIndex::None;
                };

                let page_number =
                    window.resolved_bank_index(registers, location, work_ram_bank_configuration);
                let index = bank_offset;
                let read_write_status: ReadWriteStatus = status_register_id
                    .map_or(ReadWriteStatus::ReadWrite, |id| registers.read_write_status(id));
                PrgMemoryIndex::Ram { page_number, index, read_write_status }
            }
            // TODO: Save RAM should be separate from WorkRam.
            Bank::SaveRam(_) => {
                todo!("Save RAM not yet supported for PRG.");
            }
            Bank::ExtendedRam(status_register_id) => {
                let index = u32::from(bank_offset);
                let read_write_status: ReadWriteStatus = status_register_id
                    .map_or(ReadWriteStatus::ReadWrite, |id| registers.read_write_status(id));
                PrgMemoryIndex::ExtendedRam { index, read_write_status }
            }
        }
    }

    fn window_with_index_at(&self, start: u16) -> (&Window, u32) {
        for (index, window) in self.current_layout().windows().iter().enumerate() {
            if window.start() == start {
                return (window, index as u32);
            }
        }

        panic!("No window exists at {start:?}");
    }
}

enum PrgMemoryIndex {
    None,
    // Rom's ReadWriteStatus doesn't actually allow writing.
    Rom { page_number: u16, index: u16, read_status: ReadWriteStatus },
    Ram { page_number: u16, index: u16, read_write_status: ReadWriteStatus },
    ExtendedRam { index: u32, read_write_status: ReadWriteStatus },
}