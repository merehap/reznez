use crate::memory::bank_index::BankIndex;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::writability::Writability;
use crate::util::unit::KIBIBYTE;

const PRG_MEMORY_START: CpuAddress = CpuAddress::new(0x6000);

pub struct PrgMemory {
    layout: PrgLayout,
    raw_memory: Vec<u8>,
    work_ram: Option<WorkRam>,
}

impl PrgMemory {
    pub fn new(layout: PrgLayout, raw_memory: Vec<u8>) -> PrgMemory {
        let mut prg_memory = PrgMemory { layout, raw_memory, work_ram: None};
        for window in &prg_memory.layout.windows {
            if window.prg_type == PrgType::WorkRam {
                assert!(prg_memory.work_ram.is_none(), "Only one Work RAM section may be specified.");
                prg_memory.work_ram = Some(WorkRam::new(window.size()));
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
        (self.raw_memory.len() / usize::from(self.layout.bank_size))
            .try_into()
            .expect("Way too many banks.")
    }

    pub fn last_bank_index(&self) -> u16 {
        self.bank_count() - 1
    }

    pub fn read(&self, address: CpuAddress) -> Option<u8> {
        match self.address_to_prg_index(address) {
            PrgMemoryIndex::None => None,
            PrgMemoryIndex::MappedMemory(index) => Some(self.raw_memory[index]),
            PrgMemoryIndex::WorkRam(index) => {
                let work_ram = self.work_ram.as_ref()
                    .expect("Attempted to read from WorkRam but it is not present.");
                if work_ram.enabled {
                    Some(work_ram.data[index])
                } else {
                    None
                }
            }
        }
    }

    // TODO: Handle read-only.
    pub fn write(&mut self, address: CpuAddress, value: u8) {
        match self.address_to_prg_index(address) {
            PrgMemoryIndex::None => {},
            PrgMemoryIndex::MappedMemory(index) => self.raw_memory[index] = value,
            PrgMemoryIndex::WorkRam(index) => {
                let work_ram = self.work_ram.as_mut()
                    .expect("Attempted to write to WorkRam but but it is not present.");
                if work_ram.enabled {
                    work_ram.data[index] = value;
                }
            }
        }
    }

    pub fn resolve_selected_bank_indexes(&self) -> Vec<u16> {
        let mut indexes = Vec::new();
        for window in &self.layout.windows {
            if let Some(bank_index) = window.bank_index() {
                indexes.push(bank_index.to_u16(self.bank_count()));
            }
        }

        indexes
    }

    pub fn window_at(&mut self, start: u16) -> &mut Window {
        for window in &mut self.layout.windows {
            if window.start.to_raw() == start {
                return window;
            }
        }

        panic!("No window exists at {:?}", start);
    }

    pub fn disable_work_ram(&mut self) {
        self.work_ram.as_mut().unwrap().enabled = false;
    }

    pub fn enable_work_ram(&mut self) {
        self.work_ram.as_mut().unwrap().enabled = true;
    }

    pub fn set_layout(&mut self, layout: PrgLayout) {
        self.layout = layout;
    }

    // TODO: Indicate if read-only.
    fn address_to_prg_index(&self, address: CpuAddress) -> PrgMemoryIndex {
        assert!(address >= PRG_MEMORY_START);

        let windows = &self.layout.windows;
        assert!(!windows.is_empty());

        for mut i in 0..windows.len() {
            if i == windows.len() - 1 || address < windows[i + 1].start {
                let bank_offset = address.to_raw() - windows[i].start.to_raw();
                // Step backwards until we find which window is being mirrored.
                while windows[i].is_mirror() {
                    assert!(i > 0);
                    i -= 1;
                }

                let prg_memory_index = match windows[i].prg_type {
                    PrgType::Empty => PrgMemoryIndex::None,
                    PrgType::Banked(_, bank_index) => {
                        let mapped_memory_index =
                            bank_index.to_usize(self.bank_count()) * self.layout.bank_size as usize + bank_offset as usize;
                        PrgMemoryIndex::MappedMemory(mapped_memory_index)
                    }
                    PrgType::WorkRam => PrgMemoryIndex::WorkRam(usize::from(bank_offset)),
                    PrgType::MirrorPrevious => unreachable!(),
                };
                return prg_memory_index;
            }
        }

        unreachable!();
    }
}

#[derive(Clone)]
pub struct PrgLayout {
    max_bank_count: u16,
    bank_size: usize,
    windows: Vec<Window>,
}

impl PrgLayout {
    pub fn builder() -> PrgLayoutBuilder {
        PrgLayoutBuilder::new()
    }

    fn new(
        max_bank_count: u16,
        bank_size: usize,
        windows: Vec<Window>,
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
}

pub struct PrgLayoutBuilder {
    max_bank_count: Option<u16>,
    bank_size: Option<usize>,
    windows: Vec<Window>,
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

    pub fn add_window(
        &mut self,
        start: u16,
        end: u16,
        size: usize,
        window_type: PrgType,
    ) -> &mut PrgLayoutBuilder {
        let bank_size = self.bank_size.unwrap();
        assert!(size % bank_size == 0 || bank_size % size == 0);

        self.windows.push(Window::new(start, end, size, window_type));
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
    WorkRam(usize),
    MappedMemory(usize),
}

// A Window is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy)]
pub struct Window {
    start: CpuAddress,
    end: CpuAddress,
    prg_type: PrgType,
}

impl Window {
    pub fn switch_bank_to<Index>(&mut self, new_bank_index: Index)
    where Index: Into<BankIndex>
    {
        self.prg_type.switch_bank_to(new_bank_index.into());
    }

    fn bank_index(self) -> Option<BankIndex> {
        self.prg_type.bank_index()
    }

    fn is_mirror(self) -> bool {
        self.prg_type == PrgType::MirrorPrevious
    }

    fn size(self) -> usize {
        usize::from(self.end.to_raw() - self.start.to_raw() + 1)
    }

    fn new(start: u16, end: u16, size: usize, prg_type: PrgType) -> Window {
        assert!([8 * KIBIBYTE, 16 * KIBIBYTE, 32 * KIBIBYTE].contains(&size));
        assert!(end > start);
        assert_eq!(end as usize - start as usize + 1, size);

        Window {
            start: CpuAddress::new(start),
            end: CpuAddress::new(end),
            prg_type,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum PrgType {
    Empty,
    Banked(Writability, BankIndex),
    // WRAM, Save RAM, SRAM, ambiguously "PRG RAM".
    WorkRam,
    MirrorPrevious,
}

impl PrgType {
    fn bank_index(self) -> Option<BankIndex> {
        use PrgType::*;
        match self {
            Banked(_, bank_index) => Some(bank_index),
            Empty | MirrorPrevious | WorkRam => None,
        }
    }

    fn switch_bank_to(&mut self, new_bank_index: BankIndex) {
        use PrgType::*;
        match self {
            Banked(writability, _) =>
                *self = PrgType::Banked(*writability, new_bank_index),
            Empty | MirrorPrevious | WorkRam => unreachable!(),
        }
    }
}

#[derive(Clone)]
struct WorkRam {
    data: Vec<u8>,
    enabled: bool,
}

impl WorkRam {
    fn new(size: usize) -> WorkRam {
        WorkRam {
            data: vec![0; usize::from(size)],
            enabled: true,
        }
    }
}
