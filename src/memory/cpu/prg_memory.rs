use std::num::{NonZeroU8, NonZeroU16};

use log::warn;

use crate::mapper::{BankIndex, PrgBankRegisterId, ReadWriteStatusRegisterId};
use crate::memory::bank::bank::{PrgBank, PrgBankLocation, RomRamModeRegisterId};
use crate::memory::bank::bank_index::{BankConfiguration, PrgBankRegisters, ReadWriteStatus, RomRamMode};
use crate::memory::bank::page::{OuterPageTable, OuterPage};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::ppu::chr_memory::AccessOverride;
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::memory::read_result::ReadResult;
use crate::memory::window::{ReadWriteStatusInfo, PrgWindow};
use crate::util::unit::KIBIBYTE;

const PRG_SLOT_COUNT: usize = 5;
const PRG_SUB_SLOT_COUNT: usize = 64;
const PAGE_SIZE: u16 = 8 * KIBIBYTE as u16;

pub struct PrgMemoryMap {
    // 0x6000 through 0xFFFF
    page_mappings: [PrgMapping; PRG_SLOT_COUNT],
    page_ids: [(PrgPageId, ReadWriteStatus); PRG_SLOT_COUNT],
}

impl PrgMemoryMap {
    pub fn new(
        initial_layout: PrgLayout,
        rom_size: u32,
        rom_bank_size: NonZeroU16,
        ram_size: u32,
        access_override: Option<AccessOverride>,
        regs: &PrgBankRegisters,
    ) -> Self {

        let rom_bank_size = rom_bank_size.get();
        assert_eq!(rom_bank_size % PAGE_SIZE, 0);
        let pages_per_bank = rom_bank_size / PAGE_SIZE;
        assert_eq!(rom_size % u32::from(PAGE_SIZE), 0);
        let rom_page_count: u16 = (rom_size / u32::from(PAGE_SIZE)).try_into().unwrap();
        let ram_page_count: u16 = (ram_size / u32::from(PAGE_SIZE)).try_into().unwrap();

        let mut page_mappings = Vec::with_capacity(PRG_SLOT_COUNT);

        let mut address = 0x6000;
        for window in initial_layout.windows() {
            assert!(window.start() >= 0x6000);
            assert!(window.size().get() >= 0x2000);

            let mut bank = window.bank();
            match access_override {
                None => {}
                Some(AccessOverride::ForceRom) => bank = bank.as_rom(),
                Some(AccessOverride::ForceRam) => panic!("PRG must have some ROM."),
            }

            if bank.is_rom(regs) {
                assert_eq!(window.size().get() % rom_bank_size, 0);
                assert_eq!(window.size().get() % PAGE_SIZE, 0);
            } else if bank.is_prg_ram() {
                assert_eq!(window.size().get() % PAGE_SIZE, 0);
            } else {
                assert_eq!(window.size().get(), PAGE_SIZE);
            }


            let rom_pages_per_window = window.size().get() / PAGE_SIZE;
            let mut rom_page_number_mask = 0b1111_1111_1111_1111;
            rom_page_number_mask &= !(rom_pages_per_window - 1);
            rom_page_number_mask &= rom_page_count - 1;

            let ram_pages_per_window = window.size().get() / PAGE_SIZE;
            let mut ram_page_number_mask = 0b1111_1111_1111_1111;
            ram_page_number_mask &= !(ram_pages_per_window - 1);
            ram_page_number_mask &= ram_page_count - 1;

            let mut page_offset = 0;
            while window.is_in_bounds(address) {
                let mapping = PrgMapping::Banked { bank, pages_per_bank, rom_page_number_mask, ram_page_number_mask, page_offset };
                page_mappings.push(mapping);
                address += PAGE_SIZE;
                page_offset += 1;
                // Mirror high pages to low ones if there isn't enough ROM.
                page_offset %= rom_page_count;
            }
        }

        assert_eq!(page_mappings.len(), 5);

        let mut memory_map = Self {
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [(PrgPageId::Rom(0), ReadWriteStatus::ReadOnly); PRG_SLOT_COUNT],
        };
        memory_map.update_page_ids(regs);
        memory_map
    }

    pub fn index_for_address(&self, address: CpuAddress) -> (PrgIndex, ReadWriteStatus) {
        let address = address.to_raw();
        assert!(matches!(address, 0x4020..=0xFFFF));
        if !matches!(address, 0x6000..=0xFFFF) {
            println!("Low PRG address treated as empty memory for now.");
            return (PrgIndex::None, ReadWriteStatus::Disabled);
        }

        let address = address - 0x6000;
        let mapping_index = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;

        let (page_id, read_write_status) = self.page_ids[mapping_index as usize];
        let prg_memory_index = match page_id {
            PrgPageId::Empty => PrgIndex::None,
            PrgPageId::Rom(page_number) => {
                PrgIndex::Rom(u32::from(page_number) * PAGE_SIZE as u32 + u32::from(offset))
            }
            PrgPageId::Ram(page_number) => {
                PrgIndex::Ram(u32::from(page_number) * PAGE_SIZE as u32 + u32::from(offset))
            }
        };

        (prg_memory_index, read_write_status)
    }

    pub fn page_mappings(&self) -> &[PrgMapping; PRG_SLOT_COUNT] {
        &self.page_mappings
    }

    pub fn update_page_ids(&mut self, regs: &PrgBankRegisters) {
        for i in 0..PRG_SLOT_COUNT {
            self.page_ids[i] = self.page_mappings[i].page_id(regs);
        }
    }

    pub fn page_start_index(&self, mapping_index: u8) -> PrgIndex {
        let page_id = self.page_ids[mapping_index as usize].0;
        match page_id {
            PrgPageId::Empty => PrgIndex::None,
            PrgPageId::Rom(page_number) => PrgIndex::Rom(u32::from(page_number) * KIBIBYTE),
            PrgPageId::Ram(page_number) => PrgIndex::Ram(u32::from(page_number) * KIBIBYTE),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrgIndex {
    None,
    Rom(u32),
    Ram(u32),
}

#[derive(Clone, Copy, Debug)]
pub enum PrgMapping {
    Banked {
        bank: PrgBank,
        pages_per_bank: u16,
        page_offset: u16,
        rom_page_number_mask: u16,
        ram_page_number_mask: u16,
    },
}

impl PrgMapping {
    pub fn page_id(&self, registers: &PrgBankRegisters) -> (PrgPageId, ReadWriteStatus) {
        match self {
            Self::Banked { bank, pages_per_bank, page_offset, rom_page_number_mask, ram_page_number_mask, .. } => {
                let page_number = || {
                    let location = bank.location().expect("Location to be present in bank.");
                    let bank_index = match location {
                        PrgBankLocation::Fixed(bank_index) => bank_index,
                        PrgBankLocation::Switchable(register_id) => registers.get(register_id).index().unwrap(),
                    };

                    if bank.is_rom(registers) {
                        ((pages_per_bank * bank_index.to_raw()) & rom_page_number_mask) + page_offset
                    } else {
                        ((pages_per_bank * bank_index.to_raw()) & ram_page_number_mask) + page_offset
                    }
                };

                match bank {
                    PrgBank::Empty =>
                        (PrgPageId::Empty, ReadWriteStatus::Disabled),
                    PrgBank::Rom(_, None) =>
                        (PrgPageId::Rom(page_number()), ReadWriteStatus::ReadOnly),
                    PrgBank::Rom(_, Some(status_register)) =>
                        (PrgPageId::Rom(page_number()), registers.read_write_status(*status_register)),
                    PrgBank::Ram(_, None) | PrgBank::WorkRam(_, None) =>
                        (PrgPageId::Ram(page_number()), ReadWriteStatus::ReadWrite),
                    PrgBank::Ram(_, Some(status_register)) | PrgBank::WorkRam(_, Some(status_register)) =>
                        (PrgPageId::Ram(page_number()), registers.read_write_status(*status_register)),
                    PrgBank::RomRam(_, status_register, rom_ram_register) => {
                        match registers.rom_ram_mode(*rom_ram_register) {
                            RomRamMode::Rom => (PrgPageId::Rom(page_number()), ReadWriteStatus::ReadOnly),
                            RomRamMode::Ram => (PrgPageId::Ram(page_number()), registers.read_write_status(*status_register)),
                        }
                    }
                    _ => todo!(),
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrgPageId {
    Empty,
    Rom(PageNumber),
    Ram(PageNumber),
}

type PageNumber = u16;

pub struct PrgMemory {
    layouts: Vec<PrgLayout>,
    memory_maps: Vec<PrgMemoryMap>,
    layout_index: u8,
    rom_outer_banks: OuterPageTable,
    work_ram: Option<OuterPage>,
    rom: RawMemory,
    ram: RawMemory,
    extended_ram: RawMemoryArray<KIBIBYTE>,
    access_override: Option<AccessOverride>,
    regs: PrgBankRegisters,
}

impl PrgMemory {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layouts: Vec<PrgLayout>,
        layout_index: u8,
        rom_page_size_override: Option<NonZeroU16>,
        rom: RawMemory,
        rom_outer_bank_count: NonZeroU8,
        ram_size: u32,
        access_override: Option<AccessOverride>,
        regs: PrgBankRegisters,
    ) -> PrgMemory {

        let mut rom_page_size = rom_page_size_override;
        if rom_page_size.is_none() {
            for layout in &layouts {
                for window in layout.windows() {
                    if matches!(window.bank(), PrgBank::Rom(..) | PrgBank::RomRam(..)) {
                        if let Some(size) = rom_page_size {
                            rom_page_size = Some(std::cmp::min(window.size(), size));
                        } else {
                            rom_page_size = Some(window.size());
                        }
                    }
                }
            }
        }

        let rom_bank_size = rom_page_size.expect("at least one ROM or RAM window");

        let rom_outer_banks = OuterPageTable::new(rom.clone(), rom_outer_bank_count, rom_bank_size, true)
            .expect("PRG ROM must not be empty.");

        let work_ram_windows: Vec<_> = layouts[layout_index as usize].windows().iter()
            .filter(|window| window.bank().is_prg_ram())
            .collect();
        let work_ram = if !work_ram_windows.is_empty() && ram_size > 0 {
            let mut work_ram_page_size = NonZeroU16::MAX;
            for window in work_ram_windows {
                if window.size() < work_ram_page_size {
                    work_ram_page_size = window.size();
                }
            }

            OuterPage::new(RawMemory::new(ram_size), work_ram_page_size, true)
        } else if ram_size > 0 {
            warn!("Work RAM specified in ROM file, but not in layout.");
            None
        } else {
            None
        };

        let memory_maps = layouts.iter().map(|initial_layout| PrgMemoryMap::new(
            *initial_layout, rom.size(), rom_bank_size, ram_size,  access_override, &regs,
        )).collect();

        PrgMemory {
            layouts,
            memory_maps,
            layout_index,
            rom_outer_banks,
            work_ram,
            rom,
            ram: RawMemory::new(ram_size),
            extended_ram: RawMemoryArray::new(),
            access_override,
            regs,
        }
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

    pub fn peek(&self, address: CpuAddress) -> ReadResult {
        use ReadWriteStatus::*;
        let old_result = match self.address_to_prg_index(address) {
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
        };

        let (prg_index, read_write_status) =
            self.memory_maps[self.layout_index as usize].index_for_address(address);
        let new_result = if read_write_status == ReadWriteStatus::ReadOnlyZeros {
            ReadResult::full(0)
        } else {
            match prg_index {
                PrgIndex::Rom(index) if read_write_status.is_readable() =>
                    ReadResult::full(self.rom[index]),
                PrgIndex::Ram(index) if read_write_status.is_readable() =>
                    ReadResult::full(self.ram[index]),
                PrgIndex::None | PrgIndex::Rom(_) | PrgIndex::Ram(_) =>
                    ReadResult::OPEN_BUS,
            }
        };

        assert_eq!(old_result, new_result, "Address: {address}");

        old_result
    }

    pub fn write(&mut self, address: CpuAddress, value: u8) {
        let windows = &self.current_layout().windows();
        assert!(!windows.is_empty());
        if address.to_raw() < windows[0].start() {
            return;
        }

        match self.address_to_prg_index(address) {
            PrgMemoryIndex::None => {}
            PrgMemoryIndex::Rom { .. } => { /* ROM is readonly. */}
            PrgMemoryIndex::Ram { page_number, index, read_write_status } => {
                if read_write_status.is_writable() {
                    let work_ram = self.work_ram.as_mut().expect("PRG RAM to be present since it is being written to.");
                    work_ram.page_mut(page_number).write(index, value)
                }
            }
        }

        let (prg_index, read_write_status) =
            self.memory_maps[self.layout_index as usize].index_for_address(address);
        if read_write_status.is_writable() {
            match prg_index {
                PrgIndex::None | PrgIndex::Rom(_) => unreachable!(),
                PrgIndex::Ram(index) => self.ram[index] = value,
            }
        }
    }

    pub fn set_bank_register<INDEX: Into<u16>>(&mut self, id: PrgBankRegisterId, value: INDEX) {
        self.regs.set(id, BankIndex::from_u16(value.into()));
        self.update_page_ids();
    }

    pub fn set_bank_register_bits(&mut self, id: PrgBankRegisterId, new_value: u16, mask: u16) {
        self.regs.set_bits(id, new_value, mask);
        self.update_page_ids();
    }

    pub fn update_bank_register(
        &mut self,
        id: PrgBankRegisterId,
        updater: &dyn Fn(u16) -> u16,
    ) {
        self.regs.update(id, updater);
        self.update_page_ids();
    }

    pub fn set_read_write_status(&mut self, id: ReadWriteStatusRegisterId, read_write_status: ReadWriteStatus) {
        self.regs.set_read_write_status(id, read_write_status);
    }

    pub fn set_rom_ram_mode(&mut self, id: RomRamModeRegisterId, rom_ram_mode: RomRamMode) {
        self.regs.set_rom_ram_mode(id, rom_ram_mode);
        self.update_page_ids();
    }

    pub fn window_at(&self, start: u16) -> &PrgWindow {
        self.window_with_index_at(start).0
    }

    pub fn current_layout(&self) -> &PrgLayout {
        &self.layouts[usize::from(self.layout_index)]
    }

    pub fn bank_registers(&self) -> &PrgBankRegisters {
        &self.regs
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn set_prg_rom_outer_bank_index(&mut self, index: u8) {
        self.rom_outer_banks.set_outer_page_index(index);
    }

    fn update_page_ids(&mut self) {
        for memory_map in &mut self.memory_maps {
            memory_map.update_page_ids(&self.regs);
        }
    }

    fn address_to_prg_index(&self, address: CpuAddress) -> PrgMemoryIndex {
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
        if let PrgBank::MirrorOf(mirrored_window_start) = window.bank() {
            window = self.window_at(mirrored_window_start);
        }

        match window.bank() {
            PrgBank::Empty => PrgMemoryIndex::None,
            PrgBank::MirrorOf(_) => panic!("A mirrored bank must mirror a non-mirrored bank."),
            PrgBank::Rom(location, status_register_id) => {
                let read_status: ReadWriteStatus = status_register_id
                    .map_or(ReadWriteStatus::ReadOnly, |id| self.regs.read_write_status(id));
                assert!(!read_status.is_writable());

                let mut page_number = window.resolved_bank_index(&self.regs, location, self.rom_outer_banks.bank_configuration());
                let mut index = bank_offset;
                // For windows that are larger than the page size,
                // indexes need to be reduced and page numbers need to be increased.
                while index >= self.rom_outer_banks.page_size().get() {
                    page_number += 1;
                    index -= self.rom_outer_banks.page_size().get();
                }

                PrgMemoryIndex::Rom { page_number, index, read_status }
            }
            PrgBank::Ram(location, status_register_id) => {
                let work_ram_bank_configuration = self.work_ram_bank_configuration()
                    .expect("PRG RAM window specified in layout, but not in cartridge.");
                let page_number =
                    window.resolved_bank_index(&self.regs, location, work_ram_bank_configuration);
                let index = bank_offset;
                let read_write_status: ReadWriteStatus = status_register_id
                    .map_or(ReadWriteStatus::ReadWrite, |id| self.regs.read_write_status(id));
                PrgMemoryIndex::Ram { page_number, index, read_write_status }
            }
            PrgBank::RomRam(location, status_register_id, mode_register_id) => {
                match self.regs.rom_ram_mode(mode_register_id) {
                    RomRamMode::Ram => {
                        let work_ram_bank_configuration = self.work_ram_bank_configuration()
                            .expect("PRG RAM window specified in layout, but not in cartridge.");
                        let page_number =
                            window.resolved_bank_index(&self.regs, location, work_ram_bank_configuration);
                        let index = bank_offset;
                        let read_write_status: ReadWriteStatus = self.regs.read_write_status(status_register_id);
                        PrgMemoryIndex::Ram { page_number, index, read_write_status }
                    }
                    RomRamMode::Rom => {
                        let mut page_number = window.resolved_bank_index(&self.regs, location, self.rom_outer_banks.bank_configuration());
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
            PrgBank::WorkRam(location, status_register_id) => {
                let Some(work_ram_bank_configuration) = self.work_ram_bank_configuration() else {
                    return PrgMemoryIndex::None;
                };

                let page_number =
                    window.resolved_bank_index(&self.regs, location, work_ram_bank_configuration);
                let index = bank_offset;
                let read_write_status: ReadWriteStatus = status_register_id
                    .map_or(ReadWriteStatus::ReadWrite, |id| self.regs.read_write_status(id));
                PrgMemoryIndex::Ram { page_number, index, read_write_status }
            }
            // TODO: Save RAM should be separate from WorkRam.
            PrgBank::SaveRam(_) => {
                todo!("Save RAM not yet supported for PRG.");
            }
        }
    }

    fn window_with_index_at(&self, start: u16) -> (&PrgWindow, u32) {
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
}