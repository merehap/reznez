use crate::memory::cpu::cpu_address::CpuAddress;
use crate::util::unit::KIBIBYTE;

const PRG_MEMORY_START: CpuAddress = CpuAddress::new(0x6000);
const PRG_ROM_SIZE: usize = 32 * KIBIBYTE;

pub struct CartridgeSpace {
    prg_memory: PrgMemory,
}

impl CartridgeSpace {
    pub fn new(prg_memory: PrgMemory) -> CartridgeSpace {
        CartridgeSpace { prg_memory }
    }

    pub fn read_prg(&self, address: CpuAddress) -> u8 {
        self.prg_memory.read(address)
    }

    pub fn write_prg(&mut self, address: CpuAddress, value: u8) {
        self.prg_memory.write(address, value);
    }

    pub fn selected_prg_bank_indexes(&self) -> Vec<BankIndex> {
        self.prg_memory.selected_bank_indexes()
    }

    pub fn switch_prg_bank_at(&mut self, address: WindowStart, new_bank_index: BankIndex) {
        self.prg_memory.switch_bank_at(address, new_bank_index);
    }

    pub fn single_bank(bank: Box<[u8; PRG_ROM_SIZE]>) -> CartridgeSpace {
        use WindowStart::*;
        use WindowEnd::*;

        // Only a single bank and only a single index, which points at it.
        let prg_memory = PrgMemory::builder()
            .raw_memory(bank.to_vec())
            .bank_count(1)
            .bank_size(32 * KIBIBYTE)
            .add_window(Ox6000, Ox7FFF,  8 * KIBIBYTE, WindowType::Unmapped)
            .add_window(Ox8000, OxFFFF, 32 * KIBIBYTE, WindowType::Rom { bank_index: 0 })
            .build();

        CartridgeSpace { prg_memory }
    }

    pub fn single_bank_mirrored(bank: Box<[u8; PRG_ROM_SIZE / 2]>) -> CartridgeSpace {
        use WindowStart::*;
        use WindowEnd::*;

        // Only a single bank that is half the size of indexable PRG ROM,
        // so two indexes are necessary, both which point to the only bank.
        let prg_memory = PrgMemory::builder()
            .raw_memory(bank.to_vec())
            .bank_count(1)
            .bank_size(16 * KIBIBYTE)
            .add_window(Ox6000, Ox7FFF,  8 * KIBIBYTE, WindowType::Unmapped)
            .add_window(Ox8000, OxBFFF, 16 * KIBIBYTE, WindowType::Rom { bank_index: 0 })
            .add_window(OxC000, OxFFFF, 16 * KIBIBYTE, WindowType::MirrorPrevious)
            .build();
        CartridgeSpace { prg_memory }
    }
}

pub struct PrgMemory {
    raw_memory: Vec<u8>,
    bank_count: u8,
    bank_size: usize,
    windows: Vec<Window>,
}

impl PrgMemory {
    pub fn builder() -> PrgMemoryBuilder {
        PrgMemoryBuilder::new()
    }

    fn selected_bank_indexes(&self) -> Vec<BankIndex> {
        let mut indexes = Vec::new();
        for window in &self.windows {
            if let Some(bank_index) = window.bank_index() {
                indexes.push(bank_index);
            }
        }

        indexes
    }

    fn switch_bank_at(&mut self, start: WindowStart, new_bank_index: BankIndex) {
        assert!(new_bank_index < self.bank_count);

        for window in &mut self.windows {
            if window.start == start {
                window.switch_bank(new_bank_index);
                return;
            }
        }

        panic!("No window exists at {:?}", start);
    }

    fn read(&self, address: CpuAddress) -> u8 {
        if let Some(index) = self.address_to_prg_index(address) {
            self.raw_memory[index]
        } else {
            // TODO: Open bus behavior instead.
            0
        }
    }

    fn write(&mut self, address: CpuAddress, value: u8) {
        if let Some(index) = self.address_to_prg_index(address) {
            self.raw_memory[index] = value;
        }
    }

    fn address_to_prg_index(&self, address: CpuAddress) -> Option<usize> {
        assert!(address >= PRG_MEMORY_START);
        assert!(!self.windows.is_empty());

        for mut i in 0..self.windows.len() {
            if i == self.windows.len() - 1 || address < self.windows[i + 1].start.to_cpu_address() {
                let bank_offset = address.to_raw() - (self.windows[i].start as u16);
                // Step backwards until we find which window is being mirrored.
                while self.windows[i].is_mirror() {
                    assert!(i > 0);
                    i -= 1;
                }

                let bank_index = self.windows[i].bank_index()?;
                return Some(bank_index as usize * self.bank_size as usize + bank_offset as usize);
            }
        }

        unreachable!();
    }

    fn new(
        raw_memory: Vec<u8>,
        bank_count: u8,
        bank_size: usize,
        windows: Vec<Window>,
    ) -> PrgMemory {
        assert!(!windows.is_empty());

        assert!([8 * KIBIBYTE, 16 * KIBIBYTE, 32 * KIBIBYTE].contains(&bank_size));
        assert_eq!(usize::from(bank_count) * bank_size, raw_memory.len());

        assert_eq!(windows[0].start, WindowStart::Ox6000);
        assert_eq!(windows[windows.len() - 1].end, WindowEnd::OxFFFF);

        for i in 0..windows.len() - 1 {
            assert_eq!(
                windows[i + 1].start as usize,
                windows[i].end as usize + 1,
            );
        }

        PrgMemory {
            raw_memory,
            bank_count,
            bank_size,
            windows,
        }
    }
}

pub struct PrgMemoryBuilder {
    raw_memory: Option<Vec<u8>>,
    bank_count: Option<u8>,
    bank_size: Option<usize>,
    windows: Vec<Window>,
}

impl PrgMemoryBuilder {
    pub fn raw_memory(mut self, raw_memory: Vec<u8>) -> PrgMemoryBuilder {
        self.raw_memory = Some(raw_memory);
        self
    }

    pub fn bank_count(mut self, bank_count: u8) -> PrgMemoryBuilder {
        self.bank_count = Some(bank_count);
        self
    }

    pub fn bank_size(mut self, bank_size: usize) -> PrgMemoryBuilder {
        self.bank_size = Some(bank_size);
        self
    }

    pub fn add_window(
        mut self,
        start: WindowStart,
        end: WindowEnd,
        size: usize,
        window_type: WindowType,
    ) -> PrgMemoryBuilder {
        let window = Window { start, end, window_type };

        let start = start as u16;
        let end = end as u16;
        assert!([8 * KIBIBYTE, 16 * KIBIBYTE, 32 * KIBIBYTE].contains(&size));
        assert!(end > start);
        let size: u16 = size.try_into().unwrap();
        assert_eq!(end - start + 1, size);

        let bank_size = self.bank_size.unwrap() as u16;
        println!("Size: {}, bank size: {}", size, bank_size);
        assert!(size % bank_size == 0 || bank_size % size == 0);
        self.windows.push(window);
        self
    }

    pub fn build(self) -> PrgMemory {
        PrgMemory::new(
            self.raw_memory.unwrap(),
            self.bank_count.unwrap(),
            self.bank_size.unwrap(),
            self.windows,
        )
    }

    fn new() -> PrgMemoryBuilder {
        PrgMemoryBuilder {
            raw_memory: None,
            bank_count: None,
            bank_size: None,
            windows: Vec::new(),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum WindowStart {
    Ox6000 = 0x6000,
    Ox8000 = 0x8000,
    OxA000 = 0xA000,
    OxC000 = 0xC000,
    OxE000 = 0xE000,
}

impl WindowStart {
    fn to_cpu_address(self) -> CpuAddress {
        CpuAddress::new(self as u16)
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum WindowEnd {
    Ox7FFF = 0x7FFF,
    Ox9FFF = 0x9FFF,
    OxBFFF = 0xBFFF,
    OxDFFF = 0xDFFF,
    OxFFFF = 0xFFFF,
}

// A Window is a range within addressable memory.
// If the specified bank cannot fill the window, adjacent banks will be included too.
#[derive(Clone, Copy)]
struct Window {
    start: WindowStart,
    end: WindowEnd,
    window_type: WindowType,
}

impl Window {
    fn bank_index(self) -> Option<BankIndex> {
        self.window_type.bank_index()
    }

    fn switch_bank(&mut self, new_bank_index: BankIndex) {
        self.window_type.switch_bank(new_bank_index);
    }

    fn is_mirror(self) -> bool {
        self.window_type == WindowType::MirrorPrevious
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WindowType {
    Unmapped,
    Rom { bank_index: BankIndex },
    Ram { bank_index: BankIndex },
    MirrorPrevious,
}

impl WindowType {
    fn bank_index(self) -> Option<BankIndex> {
        use WindowType::*;
        match self {
            Rom { bank_index } => Some(bank_index),
            Ram { bank_index } => Some(bank_index),
            Unmapped | MirrorPrevious => None,
        }
    }

    fn switch_bank(&mut self, new_bank_index: BankIndex) {
        use WindowType::*;
        match self {
            Rom {..} => *self = Rom { bank_index: new_bank_index },
            Ram {..} => *self = Ram { bank_index: new_bank_index },
            Unmapped | MirrorPrevious => unreachable!(),
        }
    }
}

type BankIndex = u8;
