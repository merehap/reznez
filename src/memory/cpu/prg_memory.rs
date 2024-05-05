use crate::memory::bank::bank::{Bank, Location};
use crate::memory::bank::bank_index::{BankIndex, BankRegisters, BankRegisterId};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::read_result::ReadResult;
use crate::memory::writability::Writability;

const PRG_MEMORY_START: CpuAddress = CpuAddress::new(0x6000);

pub struct PrgMemory {
    layout: PrgLayout,
    max_bank_count: u16,
    bank_size: usize,
    bank_count: u16,
    raw_memory: Vec<u8>,
    work_ram_sections: Vec<WorkRam>,
    ram_enabled: bool,
    rom_ram_mode: RomRamMode,
}

impl PrgMemory {
    pub fn new(
        layout: PrgLayout,
        max_bank_count: u16,
        bank_size: usize,
        raw_memory: Vec<u8>,
    ) -> PrgMemory {

        layout.validate_bank_size_multiples(bank_size);
        let bank_count;
        if raw_memory.len() % bank_size == 0 {
            bank_count = (raw_memory.len() / bank_size)
                .try_into()
                .expect("Way too many banks.");
        } else if !raw_memory.is_empty() && bank_size % raw_memory.len() == 0 {
            bank_count = 1;
        } else {
            panic!("Bad PRG length: {} . Bank size: {} .", raw_memory.len(), bank_size);
        }

        let mut prg_memory = PrgMemory {
            layout,
            max_bank_count,
            bank_size,
            bank_count,
            raw_memory,
            work_ram_sections: Vec::new(),
            ram_enabled: true,
            rom_ram_mode: RomRamMode::Rom,
        };

        for window in prg_memory.layout.0 {
            if window.prg_bank.is_work_ram() {
                prg_memory.work_ram_sections.push(WorkRam::new(window.size()));
            }
        }

        let bank_count = prg_memory.bank_count();
        assert!(bank_count <= prg_memory.max_bank_count,
            "Bank count: {bank_count}, max: {max_bank_count}");
        if prg_memory.raw_memory.len() >= usize::from(bank_count) * bank_size {
            assert_eq!(
                prg_memory.raw_memory.len(),
                usize::from(bank_count) * bank_size,
                "Bad PRG data size.",
            );
        }
        //assert_eq!(bank_count & (bank_count - 1), 0);

        prg_memory
    }

    pub fn bank_count(&self) -> u16 {
        self.bank_count
    }

    pub fn last_bank_index(&self) -> u16 {
        self.bank_count() - 1
    }

    pub fn peek(&self, registers: &BankRegisters, address: CpuAddress) -> ReadResult {
        match self.address_to_prg_index(registers, address) {
            PrgMemoryIndex::None => ReadResult::OPEN_BUS,
            PrgMemoryIndex::MappedMemory {writability, index} => {
                if writability.is_writable(self.rom_ram_mode) && !self.ram_enabled {
                    ReadResult::OPEN_BUS
                } else {
                    ReadResult::full(self.raw_memory[index % self.raw_memory.len()])
                }
            }
            PrgMemoryIndex::WorkRam { section_id, index } => {
                let work_ram = &self.work_ram_sections[section_id];
                use RamStatus::*;
                match work_ram.status {
                    Disabled => ReadResult::OPEN_BUS,
                    ReadOnlyZeros => ReadResult::full(0),
                    ReadOnly | ReadWrite => ReadResult::full(work_ram.data[index]),
                }
            }
        }
    }

    pub fn write(&mut self, registers: &BankRegisters, address: CpuAddress, value: u8) {
        match self.address_to_prg_index(registers, address) {
            PrgMemoryIndex::None => {}
            PrgMemoryIndex::MappedMemory { writability, index } => {
                if writability.is_writable(self.rom_ram_mode) {
                    self.raw_memory[index] = value;
                }
            }
            PrgMemoryIndex::WorkRam { section_id, index } => {
                let work_ram = &mut self.work_ram_sections[section_id];
                if work_ram.status == RamStatus::ReadWrite {
                    work_ram.data[index] = value;
                }
            }
        }
    }

    pub fn resolve_selected_bank_indexes(&self, registers: &BankRegisters) -> Vec<u16> {
        let mut indexes = Vec::new();
        for window in self.layout.0 {
            if let Some(bank_index) = window.bank_index(registers) {
                let raw_index = bank_index.to_u16(self.bank_count());
                indexes.push(raw_index);
            }
        }

        indexes
    }

    pub fn window_at(&self, start: u16) -> &PrgWindow {
        self.window_with_index_at(start).0
    }

    pub fn set_work_ram_status_at(&mut self, address: u16, status: RamStatus) {
        self.work_ram_at(address).status = status;
    }

    pub fn disable_work_ram(&mut self, address: u16) {
        self.work_ram_at(address).status = RamStatus::Disabled;
    }

    pub fn enable_work_ram(&mut self, address: u16) {
        self.work_ram_at(address).status = RamStatus::ReadWrite;
    }

    pub fn set_layout(&mut self, windows: PrgLayout) {
        windows.validate_bank_size_multiples(self.bank_size);
        self.layout = windows;
    }

    pub fn set_ram_enabled(&mut self, ram_enabled: bool) {
        self.ram_enabled = ram_enabled;
    }

    pub fn set_rom_ram_mode(&mut self, rom_ram_mode: RomRamMode) {
        self.rom_ram_mode = rom_ram_mode;
    }

    // TODO: Indicate if read-only.
    fn address_to_prg_index(&self, registers: &BankRegisters, address: CpuAddress) -> PrgMemoryIndex {
        assert!(address >= PRG_MEMORY_START);

        let windows = &self.layout.0;
        assert!(!windows.is_empty());

        for i in 0..windows.len() {
            if i == windows.len() - 1 || address < windows[i + 1].start {
                let bank_offset = address.to_raw() - windows[i].start.to_raw();

                let window;
                if let Bank::MirrorOf(mirrored_window_start) = windows[i].prg_bank {
                    window = self.window_at(mirrored_window_start);
                } else {
                    window = &windows[i];
                }

                let prg_memory_index = match window.prg_bank {
                    Bank::Empty => PrgMemoryIndex::None,
                    Bank::MirrorOf(_) => panic!("A mirrored bank must mirror a non-mirrored bank."),
                    Bank::Rom(Location::Fixed(bank_index)) => {
                        // TODO: Consolidate Fixed and Switchable logic.
                        let mut raw_bank_index = bank_index.to_usize(self.bank_count());
                        let window_multiple = window.size() / self.bank_size;
                        // Clear low bits for large windows.
                        raw_bank_index &= !(window_multiple >> 1);
                        let index = raw_bank_index * self.bank_size + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory { writability: Writability::Rom, index }
                    }
                    Bank::Ram(Location::Fixed(bank_index), _) => {
                        // TODO: Consolidate Fixed and Switchable logic.
                        let mut raw_bank_index = bank_index.to_usize(self.bank_count());
                        let window_multiple = window.size() / self.bank_size;
                        // Clear low bits for large windows.
                        raw_bank_index &= !(window_multiple >> 1);
                        let index = raw_bank_index * self.bank_size + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory { writability: Writability::Ram, index }
                    }
                    Bank::Rom(Location::Switchable(register_id)) => {
                        let mut raw_bank_index = registers.get(register_id)
                            .to_usize(self.bank_count());
                        let window_multiple = window.size() / self.bank_size;
                        // Clear low bits for large windows.
                        raw_bank_index &= !(window_multiple >> 1);
                        let index = raw_bank_index * self.bank_size + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory { writability: Writability::Rom, index }
                    }
                    Bank::Ram(Location::Switchable(register_id), _) => {
                        let mut raw_bank_index = registers.get(register_id)
                            .to_usize(self.bank_count());
                        let window_multiple = window.size() / self.bank_size;
                        // Clear low bits for large windows.
                        raw_bank_index &= !(window_multiple >> 1);
                        let index = raw_bank_index * self.bank_size + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory { writability: Writability::Ram, index }
                    }
                    Bank::Rom(_) => todo!("Meta registers"),
                    Bank::Ram(_, _) => todo!("Meta registers"),
                    Bank::WorkRam(s) => {
                        let mut index = usize::from(bank_offset);
                        let mut result = None;
                        for (section_id, work_ram_section) in self.work_ram_sections.iter().enumerate() {
                            if index < work_ram_section.data.len() {
                                result = Some(PrgMemoryIndex::WorkRam { section_id, index });
                                break;
                            }

                            index -= work_ram_section.data.len();
                        }

                        if result.is_none() {
                            println!("WorkRam Bank: {s:?} Index: {index:04X} Address: {address}");
                        }

                        result.unwrap()
                    }
                };
                return prg_memory_index;
            }
        }

        unreachable!();
    }

    // This method assume that all WorkRam is at the start of the PrgLayout.
    fn work_ram_at(&mut self, start: u16) -> &mut WorkRam {
        let (window, index) = self.window_with_index_at(start);
        assert!(window.prg_bank.is_work_ram());
        &mut self.work_ram_sections[index]
    }

    fn window_with_index_at(&self, start: u16) -> (&PrgWindow, usize) {
        for (index, window) in self.layout.0.iter().enumerate() {
            if window.start.to_raw() == start {
                return (window, index);
            }
        }

        panic!("No window exists at {start:?}");
    }
}

#[derive(Clone, Copy)]
pub struct PrgLayout(&'static [PrgWindow]);

impl PrgLayout {
    pub const fn new(windows: &'static [PrgWindow]) -> PrgLayout {
        assert!(!windows.is_empty(), "No PRG windows specified.");

        assert!(windows[0].start.to_raw() == 0x6000,
            "The first PRG window must start at 0x6000.");

        assert!(windows[windows.len() - 1].end.to_raw() == 0xFFFF,
                "The last PRG window must end at 0xFFFF.");

        let mut i = 1;
        while i < windows.len() {
            assert!(windows[i].start.to_raw() == windows[i - 1].end.to_raw() + 1,
                "There must be no gaps nor overlap between PRG windows.");

            i += 1;
        }

        PrgLayout(windows)
    }

    pub fn windows(&self) -> &[PrgWindow] {
        self.0
    }

    const fn validate_bank_size_multiples(&self, bank_size: usize) {
        let mut i = 0;
        while i < self.0.len() {
            let window = self.0[i];
            if !matches!(window.prg_bank, Bank::WorkRam(_) | Bank::Empty | Bank::MirrorOf(_))
                && window.size() % bank_size != 0 {
                panic!("Window size must be a multiple of bank size.");
            }

            i += 1;
        }
    }

    pub fn active_register_ids(&self) -> Vec<BankRegisterId> {
        self.0.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}

enum PrgMemoryIndex {
    None,
    WorkRam { section_id: usize, index: usize },
    MappedMemory { writability: Writability, index: usize },
}

// A PrgWindow is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy)]
pub struct PrgWindow {
    start: CpuAddress,
    end: CpuAddress,
    prg_bank: Bank,
}

impl PrgWindow {
    fn bank_index(self, registers: &BankRegisters) -> Option<BankIndex> {
        self.prg_bank.bank_index(registers)
    }

    const fn size(self) -> usize {
        (self.end.to_raw() - self.start.to_raw() + 1) as usize
    }

    fn register_id(self) -> Option<BankRegisterId> {
        if let Bank::Rom(Location::Switchable(id)) | Bank::Ram(Location::Switchable(id), _) = self.prg_bank {
            Some(id)
        } else {
            None
        }
    }

    pub const fn new(start: u16, end: u16, size: usize, prg_bank: Bank) -> PrgWindow {
        assert!(end > start);
        assert!(end as usize - start as usize + 1 == size);

        PrgWindow {
            start: CpuAddress::new(start),
            end: CpuAddress::new(end),
            prg_bank,
        }
    }
}

#[derive(Clone)]
struct WorkRam {
    data: Vec<u8>,
    status: RamStatus,
}

impl WorkRam {
    fn new(size: usize) -> WorkRam {
        WorkRam {
            data: vec![0; size],
            status: RamStatus::ReadWrite,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RamStatus {
    Disabled,
    ReadOnlyZeros,
    ReadOnly,
    ReadWrite,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RomRamMode {
    Rom,
    Ram,
}
