use crate::memory::bank_index::{BankIndex, BankIndexRegisters, BankIndexRegisterId};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::writability::Writability;
use crate::util::unit::KIBIBYTE;

const PRG_MEMORY_START: CpuAddress = CpuAddress::new(0x6000);

pub struct PrgMemory {
    layout: PrgLayout,
    bank_index_registers: BankIndexRegisters,
    raw_memory: Vec<u8>,
    work_ram_sections: Vec<WorkRam>,
}

impl PrgMemory {
    pub fn new(layout: PrgLayout, raw_memory: Vec<u8>) -> PrgMemory {
        let bank_index_registers =
            BankIndexRegisters::new(&layout.active_register_ids());

        let mut prg_memory = PrgMemory {
            layout,
            bank_index_registers,
            raw_memory,
            work_ram_sections: Vec::new(),
        };

        for window in &prg_memory.layout.windows {
            if window.prg_type == PrgType::WorkRam {
                prg_memory.work_ram_sections.push(WorkRam::new(window.size()));
            }
        }

        let bank_count = prg_memory.bank_count();
        assert!(bank_count <= prg_memory.layout.max_bank_count);
        assert_eq!(
            prg_memory.raw_memory.len(),
            usize::from(bank_count) * prg_memory.layout.bank_size,
            "Bad PRG data size.",
        );
        //assert_eq!(bank_count & (bank_count - 1), 0);

        prg_memory
    }

    pub fn bank_count(&self) -> u16 {
        (self.raw_memory.len() / self.layout.bank_size)
            .try_into()
            .expect("Way too many banks.")
    }

    pub fn last_bank_index(&self) -> u16 {
        self.bank_count() - 1
    }

    pub fn peek(&self, address: CpuAddress) -> Option<u8> {
        match self.address_to_prg_index(address) {
            PrgMemoryIndex::None => None,
            PrgMemoryIndex::MappedMemory(index) => Some(self.raw_memory[index]),
            PrgMemoryIndex::WorkRam { section_id, index } => {
                let work_ram = &self.work_ram_sections[section_id];
                use WorkRamStatus::*;
                match work_ram.status {
                    Disabled => None,
                    ReadOnlyZeros => Some(0),
                    ReadOnly | ReadWrite => Some(work_ram.data[index]),
                }
            }
        }
    }

    // TODO: Handle read-only.
    pub fn write(&mut self, address: CpuAddress, value: u8) {
        match self.address_to_prg_index(address) {
            PrgMemoryIndex::None => {},
            PrgMemoryIndex::MappedMemory(index) => self.raw_memory[index] = value,
            PrgMemoryIndex::WorkRam { section_id, index } => {
                let work_ram = &mut self.work_ram_sections[section_id];
                if work_ram.status == WorkRamStatus::ReadWrite {
                    work_ram.data[index] = value;
                }
            }
        }
    }

    pub fn resolve_selected_bank_indexes(&self) -> Vec<u16> {
        let mut indexes = Vec::new();
        for window in &self.layout.windows {
            if let Some(bank_index) = window.bank_index() {
                let raw_index = bank_index.to_u16(&self.bank_index_registers, self.bank_count());
                indexes.push(raw_index);
            }
        }

        indexes
    }

    pub fn window_at(&mut self, start: u16) -> &mut PrgWindow {
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

    pub fn set_layout(&mut self, layout: PrgLayout) {
        let new_bank_index_registers =
            BankIndexRegisters::new(&layout.active_register_ids());
        self.bank_index_registers.merge(&new_bank_index_registers);
        self.layout = layout;
    }

    pub fn set_bank_index_register<INDEX: Into<u16>>(
        &mut self,
        id: BankIndexRegisterId,
        raw_bank_index: INDEX,
    ) {
        self.bank_index_registers.set(id, raw_bank_index.into());
    }

    // TODO: Indicate if read-only.
    fn address_to_prg_index(&self, address: CpuAddress) -> PrgMemoryIndex {
        assert!(address >= PRG_MEMORY_START);

        let windows = &self.layout.windows;
        assert!(!windows.is_empty());

        for i in 0..windows.len() {
            if i == windows.len() - 1 || address < windows[i + 1].start {
                let bank_offset = address.to_raw() - windows[i].start.to_raw();

                let window;
                if let PrgType::Mirror(mirrored_window_start) = windows[i].prg_type {
                    window = self.window(mirrored_window_start);
                } else {
                    window = &windows[i];
                }

                let prg_memory_index = match window.prg_type {
                    PrgType::Empty => PrgMemoryIndex::None,
                    PrgType::Banked(_, bank_index) => {
                        let mut raw_bank_index =
                            bank_index.to_usize(&self.bank_index_registers, self.bank_count());
                        // Clear low bits for large windows.
                        let window_multiple = window.size() / self.layout.bank_size;
                        raw_bank_index &= !(window_multiple >> 1);
                        let mapped_memory_index =
                             raw_bank_index * self.layout.bank_size + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory(mapped_memory_index)
                    }
                    PrgType::WorkRam => {
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
                    PrgType::Mirror(_) => unreachable!(),
                };
                return prg_memory_index;
            }
        }

        unreachable!();
    }

    // This method assume that all WorkRam is at the start of the PrgLayout.
    fn work_ram_at(&mut self, start: u16) -> &mut WorkRam {
        let (window, index) = self.window_with_index_at(start);
        assert_eq!(window.prg_type, PrgType::WorkRam);
        &mut self.work_ram_sections[index]
    }

    fn window_with_index_at(&mut self, start: u16) -> (&mut PrgWindow, usize) {
        for (index, window) in self.layout.windows.iter_mut().enumerate() {
            if window.start.to_raw() == start {
                return (window, index);
            }
        }

        panic!("No window exists at {start:?}");
    }

    fn window(&self, start: u16) -> &PrgWindow {
        for window in &self.layout.windows {
            if window.start.to_raw() == start {
                return window;
            }
        }

        panic!("No window exists at {start:?}");
    }
}

#[derive(Clone)]
pub struct PrgLayout {
    max_bank_count: u16,
    bank_size: usize,
    windows: Vec<PrgWindow>,
}

impl PrgLayout {
    pub fn builder() -> PrgLayoutBuilder {
        PrgLayoutBuilder::new()
    }

    pub fn new(
        max_bank_count: u16,
        bank_size: usize,
        windows: Vec<PrgWindow>,
    ) -> PrgLayout {

        assert!(!windows.is_empty());

        assert!(
            [8 * KIBIBYTE, 16 * KIBIBYTE, 32 * KIBIBYTE].contains(&bank_size),
            "Bad PRG bank size",
        );

        assert_eq!(
            windows[0].start.to_raw(),
            0x6000,
            "Every mapper needs one PRG window starting at 0x6000 (usually WorkRam or Empty).");
        assert_eq!(
            windows[windows.len() - 1].end.to_raw(),
            0xFFFF,
            "Every mapper needs one PRG window that extends to 0xFFFF",
        );

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start.to_raw(),
                windows[i].end.to_raw() + 1,
                "There must be no gaps nor overlap between PRG windows.",
            );
        }


        // Power of 2.
        assert_eq!(max_bank_count & (max_bank_count - 1), 0);

        PrgLayout { max_bank_count, bank_size, windows }
    }

    fn active_register_ids(&self) -> Vec<BankIndexRegisterId> {
        self.windows.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}

pub struct PrgLayoutBuilder {
    max_bank_count: Option<u16>,
    bank_size: Option<usize>,
    windows: Vec<PrgWindow>,
}

impl PrgLayoutBuilder {
    pub fn max_bank_count(&mut self, max_bank_count: u16) -> &mut PrgLayoutBuilder {
        self.max_bank_count = Some(max_bank_count);
        self
    }

    pub fn bank_size(&mut self, bank_size: usize) -> &mut PrgLayoutBuilder {
        self.bank_size = Some(bank_size);
        self
    }

    pub fn window(
        &mut self,
        start: u16,
        end: u16,
        size: usize,
        window_type: PrgType,
    ) -> &mut PrgLayoutBuilder {
        let bank_size = self.bank_size.unwrap();
        if window_type != PrgType::WorkRam {
            assert!(size % bank_size == 0 || bank_size % size == 0);
        }

        self.windows.push(PrgWindow::new(start, end, size, window_type));
        self
    }

    pub fn build(&self) -> PrgLayout {
        assert!(!self.windows.is_empty());

        PrgLayout::new(
            self.max_bank_count.unwrap(),
            self.bank_size.unwrap(),
            self.windows.clone(),
        )
    }

    fn new() -> PrgLayoutBuilder {
        PrgLayoutBuilder {
            max_bank_count: None,
            bank_size: None,
            windows: Vec::new(),
        }
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
    prg_type: PrgType,
}

impl PrgWindow {
    fn bank_index(self) -> Option<BankIndex> {
        self.prg_type.bank_index()
    }

    fn size(self) -> usize {
        usize::from(self.end.to_raw() - self.start.to_raw() + 1)
    }

    fn register_id(self) -> Option<BankIndexRegisterId> {
        if let PrgType::Banked(_, BankIndex::Register(id)) = self.prg_type {
            Some(id)
        } else {
            None
        }
    }

    pub const fn new(start: u16, end: u16, _size: usize, prg_type: PrgType) -> PrgWindow {
        /*
        assert!(end > start);
        assert_eq!(end as usize - start as usize + 1, size,
            "Interval from 0x{:04X} to 0x{:04X} is {}KiB, but it is specified as {}Kib",
            start,
            end,
            (end - start + 1) as usize / KIBIBYTE,
            size / KIBIBYTE,
        );
        */

        PrgWindow {
            start: CpuAddress::new(start),
            end: CpuAddress::new(end),
            prg_type,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrgType {
    Empty,
    Banked(Writability, BankIndex),
    // WRAM, Save RAM, SRAM, ambiguously "PRG RAM".
    WorkRam,
    Mirror(u16),
}

impl PrgType {
    fn bank_index(self) -> Option<BankIndex> {
        use PrgType::*;
        match self {
            Banked(_, bank_index) => Some(bank_index),
            Empty | Mirror(_) | WorkRam => None,
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
