use crate::memory::bank_index::{BankIndex, BankIndexRegisters, BankIndexRegisterId};
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
        };

        for window in prg_memory.layout.0 {
            if window.prg_bank == PrgBank::WorkRam {
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

    pub fn peek(&self, registers: &BankIndexRegisters, address: CpuAddress) -> ReadResult {
        match self.address_to_prg_index(registers, address) {
            PrgMemoryIndex::None => ReadResult::OPEN_BUS,
            PrgMemoryIndex::MappedMemory(index) =>
                ReadResult::full(self.raw_memory[index % self.raw_memory.len()]),
            PrgMemoryIndex::WorkRam { section_id, index } => {
                let work_ram = &self.work_ram_sections[section_id];
                use WorkRamStatus::*;
                match work_ram.status {
                    Disabled => ReadResult::OPEN_BUS,
                    ReadOnlyZeros => ReadResult::full(0),
                    ReadOnly | ReadWrite => ReadResult::full(work_ram.data[index]),
                }
            }
        }
    }

    // TODO: Handle read-only.
    pub fn write(&mut self, registers: &BankIndexRegisters, address: CpuAddress, value: u8) {
        match self.address_to_prg_index(registers, address) {
            PrgMemoryIndex::None => {}
            PrgMemoryIndex::MappedMemory(index) => self.raw_memory[index] = value,
            PrgMemoryIndex::WorkRam { section_id, index } => {
                let work_ram = &mut self.work_ram_sections[section_id];
                if work_ram.status == WorkRamStatus::ReadWrite {
                    work_ram.data[index] = value;
                }
            }
        }
    }

    pub fn resolve_selected_bank_indexes(&self, registers: &BankIndexRegisters) -> Vec<u16> {
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

    pub fn set_work_ram_status_at(&mut self, address: u16, status: WorkRamStatus) {
        self.work_ram_at(address).status = status;
    }

    pub fn disable_work_ram(&mut self, address: u16) {
        self.work_ram_at(address).status = WorkRamStatus::Disabled;
    }

    pub fn enable_work_ram(&mut self, address: u16) {
        self.work_ram_at(address).status = WorkRamStatus::ReadWrite;
    }

    pub fn set_layout(&mut self, windows: PrgLayout) {
        windows.validate_bank_size_multiples(self.bank_size);
        self.layout = windows;
    }

    // TODO: Indicate if read-only.
    fn address_to_prg_index(&self, registers: &BankIndexRegisters, address: CpuAddress) -> PrgMemoryIndex {
        assert!(address >= PRG_MEMORY_START);

        let windows = &self.layout.0;
        assert!(!windows.is_empty());

        for i in 0..windows.len() {
            if i == windows.len() - 1 || address < windows[i + 1].start {
                let bank_offset = address.to_raw() - windows[i].start.to_raw();

                let window;
                if let PrgBank::MirrorOf(mirrored_window_start) = windows[i].prg_bank {
                    window = self.window_at(mirrored_window_start);
                } else {
                    window = &windows[i];
                }

                let prg_memory_index = match window.prg_bank {
                    PrgBank::Empty => PrgMemoryIndex::None,
                    PrgBank::MirrorOf(_) => panic!("A mirrored bank must mirror a non-mirrored bank."),
                    PrgBank::Fixed(_, bank_index) => {
                        // TODO: Consolidate Fixed and Switchable logic.
                        let mut raw_bank_index = bank_index.to_usize(self.bank_count());
                        let window_multiple = window.size() / self.bank_size;
                        // Clear low bits for large windows.
                        raw_bank_index &= !(window_multiple >> 1);
                        let mapped_memory_index =
                             raw_bank_index * self.bank_size + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory(mapped_memory_index)
                    }
                    PrgBank::Switchable(_, register_id) => {
                        let mut raw_bank_index = registers.get(register_id)
                            .to_usize(self.bank_count());
                        let window_multiple = window.size() / self.bank_size;
                        // Clear low bits for large windows.
                        raw_bank_index &= !(window_multiple >> 1);
                        let mapped_memory_index =
                             raw_bank_index * self.bank_size + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory(mapped_memory_index)
                    }
                    PrgBank::WorkRam => {
                        let mut index = usize::from(bank_offset);
                        let mut result = None;
                        for (section_id, work_ram_section) in self.work_ram_sections.iter().enumerate() {
                            if index < work_ram_section.data.len() {
                                result = Some(PrgMemoryIndex::WorkRam { section_id, index });
                                break;
                            }

                            index -= work_ram_section.data.len();
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
        assert_eq!(window.prg_bank, PrgBank::WorkRam);
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
            if !matches!(window.prg_bank, PrgBank::WorkRam | PrgBank::Empty | PrgBank::MirrorOf(_))
                && window.size() % bank_size != 0 {
                panic!("Window size must be a multiple of bank size.");
            }

            i += 1;
        }
    }

    pub fn active_register_ids(&self) -> Vec<BankIndexRegisterId> {
        self.0.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}

enum PrgMemoryIndex {
    None,
    WorkRam { section_id: usize, index: usize },
    MappedMemory(usize),
}

// A PrgWindow is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy)]
pub struct PrgWindow {
    start: CpuAddress,
    end: CpuAddress,
    prg_bank: PrgBank,
}

impl PrgWindow {
    fn bank_index(self, registers: &BankIndexRegisters) -> Option<BankIndex> {
        self.prg_bank.bank_index(registers)
    }

    const fn size(self) -> usize {
        (self.end.to_raw() - self.start.to_raw() + 1) as usize
    }

    fn register_id(self) -> Option<BankIndexRegisterId> {
        if let PrgBank::Switchable(_, id) = self.prg_bank {
            Some(id)
        } else {
            None
        }
    }

    pub const fn new(start: u16, end: u16, size: usize, prg_bank: PrgBank) -> PrgWindow {
        assert!(end > start);
        assert!(end as usize - start as usize + 1 == size);

        PrgWindow {
            start: CpuAddress::new(start),
            end: CpuAddress::new(end),
            prg_bank,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgBank {
    Empty,
    Fixed(Writability, BankIndex),
    Switchable(Writability, BankIndexRegisterId),
    // WRAM, Save RAM, SRAM, ambiguously "PRG RAM".
    WorkRam,
    MirrorOf(u16),
}

impl PrgBank {
    fn bank_index(self, registers: &BankIndexRegisters) -> Option<BankIndex> {
        use PrgBank::*;
        match self {
            Fixed(_, bank_index) => Some(bank_index),
            Switchable(_, register_id) => Some(registers.get(register_id)),
            Empty | WorkRam | MirrorOf(_) => None,
        }
    }
}

#[derive(Clone)]
struct WorkRam {
    data: Vec<u8>,
    status: WorkRamStatus,
}

impl WorkRam {
    fn new(size: usize) -> WorkRam {
        WorkRam {
            data: vec![0; size],
            status: WorkRamStatus::ReadWrite,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum WorkRamStatus {
    Disabled,
    ReadOnlyZeros,
    ReadOnly,
    ReadWrite,
}
