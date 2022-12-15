use crate::memory::bank_index::BankIndex;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::writability::Writability;
use crate::util::unit::KIBIBYTE;

const PRG_MEMORY_START: CpuAddress = CpuAddress::new(0x6000);

pub struct PrgMemory {
    raw_memory: Vec<u8>,
    work_ram: Vec<u8>,
    bank_size: usize,
    windows: Vec<Window>,
}

impl PrgMemory {
    pub fn builder() -> PrgMemoryBuilder {
        PrgMemoryBuilder::new()
    }

    pub fn bank_count(&self) -> u16 {
        (self.raw_memory.len() / usize::from(self.bank_size))
            .try_into()
            .expect("Way too many banks.")
    }

    pub fn last_bank_index(&self) -> u16 {
        self.bank_count() - 1
    }

    pub fn read(&self, address: CpuAddress) -> u8 {
        match self.address_to_prg_index(address) {
            PrgMemoryIndex::None => /* TODO: Open bus behavior instead. */ 0,
            PrgMemoryIndex::MappedMemory(index) => self.raw_memory[index],
            PrgMemoryIndex::WorkRam(index) => self.work_ram[index],
        }
    }

    // TODO: Handle read-only.
    pub fn write(&mut self, address: CpuAddress, value: u8) {
        match self.address_to_prg_index(address) {
            PrgMemoryIndex::None => {},
            PrgMemoryIndex::MappedMemory(index) => self.raw_memory[index] = value,
            PrgMemoryIndex::WorkRam(index) => self.work_ram[index] = value,
        }
    }

    pub fn resolve_selected_bank_indexes(&self) -> Vec<u16> {
        let mut indexes = Vec::new();
        for window in &self.windows {
            if let Some(bank_index) = window.bank_index() {
                indexes.push(bank_index.to_u16(self.bank_count()));
            }
        }

        indexes
    }

    pub fn window_at(&mut self, start: u16) -> &mut Window {
        for window in &mut self.windows {
            if window.start.to_raw() == start {
                return window;
            }
        }

        panic!("No window exists at {:?}", start);
    }

    // TODO: Indicate if read-only.
    fn address_to_prg_index(&self, address: CpuAddress) -> PrgMemoryIndex {
        assert!(address >= PRG_MEMORY_START);
        assert!(!self.windows.is_empty());

        for mut i in 0..self.windows.len() {
            if i == self.windows.len() - 1 || address < self.windows[i + 1].start {
                let bank_offset = address.to_raw() - self.windows[i].start.to_raw();
                // Step backwards until we find which window is being mirrored.
                while self.windows[i].is_mirror() {
                    assert!(i > 0);
                    i -= 1;
                }

                let prg_memory_index = match self.windows[i].window_type {
                    PrgType::Empty => PrgMemoryIndex::None,
                    PrgType::Banked(_, bank_index) => {
                        let mapped_memory_index =
                            bank_index.to_usize(self.bank_count()) * self.bank_size as usize + bank_offset as usize;
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

    fn new(
        raw_memory: Vec<u8>,
        work_ram: Vec<u8>,
        max_bank_count: u16,
        bank_size: usize,
        windows: Vec<Window>,
    ) -> PrgMemory {

        assert!(!windows.is_empty());

        assert!(
            [8 * KIBIBYTE, 16 * KIBIBYTE, 32 * KIBIBYTE].contains(&bank_size),
            "Bad PRG bank size",
        );

        assert_eq!(windows[0].start.to_raw(), 0x6000,
            "Every mapper needs one PRG window starting at 0x6000 (usually WorkRam or Empty).");
        assert_eq!(
            windows[windows.len() - 1].end.to_raw(),
            0xFFFF,"Every mapper needs one PRG window that extends to 0xFFFF",
        );

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start.to_raw(),
                windows[i].end.to_raw() + 1,
                "There must be no gaps nor overlap between PRG windows.",
            );
        }

        let prg_memory = PrgMemory { raw_memory, work_ram, bank_size, windows };
        let bank_count = prg_memory.bank_count();
        assert!(bank_count <= max_bank_count);
        assert_eq!(
            prg_memory.raw_memory.len(),
            usize::from(bank_count) * bank_size,
            "Bad PRG data size.",
        );
        // Power of 2.
        assert_eq!(max_bank_count & (max_bank_count - 1), 0);
        assert_eq!(bank_count & (bank_count - 1), 0);


        prg_memory
    }
}

pub struct PrgMemoryBuilder {
    raw_memory: Option<Vec<u8>>,
    work_ram: Vec<u8>,
    max_bank_count: Option<u16>,
    bank_size: Option<usize>,
    windows: Vec<Window>,
}

impl PrgMemoryBuilder {
    pub fn raw_memory(&mut self, raw_memory: Vec<u8>) -> &mut PrgMemoryBuilder {
        self.raw_memory = Some(raw_memory);
        self
    }

    pub fn max_bank_count(&mut self, max_bank_count: u16) -> &mut PrgMemoryBuilder {
        self.max_bank_count = Some(max_bank_count);
        self
    }

    pub fn bank_size(&mut self, bank_size: usize) -> &mut PrgMemoryBuilder {
        self.bank_size = Some(bank_size);
        self
    }

    pub fn add_window(
        &mut self,
        start: u16,
        end: u16,
        size: usize,
        window_type: PrgType,
    ) -> &mut PrgMemoryBuilder {
        let window = Window {
            start: CpuAddress::new(start),
            end: CpuAddress::new(end),
            window_type
        };

        assert!([8 * KIBIBYTE, 16 * KIBIBYTE, 32 * KIBIBYTE].contains(&size));
        assert!(end > start);
        let size: u16 = size.try_into().unwrap();
        assert_eq!(end - start + 1, size);

        let bank_size = self.bank_size.unwrap() as u16;
        assert!(size % bank_size == 0 || bank_size % size == 0);

        if window_type == PrgType::WorkRam {
            assert!(self.work_ram.is_empty(), "Only one Work RAM section may be specified.");
            self.work_ram = vec![0; usize::from(size)];
        }

        self.windows.push(window);
        self
    }

    pub fn build(&self) -> PrgMemory {
        assert!(!self.windows.is_empty());

        PrgMemory::new(
            self.raw_memory.as_ref().unwrap().clone(),
            self.work_ram.clone(),
            self.max_bank_count.unwrap().clone(),
            self.bank_size.unwrap().clone(),
            self.windows.clone(),
        )
    }

    fn new() -> PrgMemoryBuilder {
        PrgMemoryBuilder {
            raw_memory: None,
            work_ram: Vec::new(),
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
    window_type: PrgType,
}

impl Window {
    pub fn switch_bank_to(&mut self, new_bank_index: BankIndex) {
        self.window_type.switch_bank_to(new_bank_index);
    }

    fn bank_index(self) -> Option<BankIndex> {
        self.window_type.bank_index()
    }

    fn is_mirror(self) -> bool {
        self.window_type == PrgType::MirrorPrevious
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
