use crate::memory::bank::bank::Bank;
use crate::memory::bank::bank_index::{BankRegisters, RamStatus};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::memory::read_result::ReadResult;
use crate::memory::window::Window;
use crate::util::unit::KIBIBYTE;

pub struct PrgMemory {
    layouts: Vec<PrgLayout>,
    layout_index: u8,
    bank_size: u16,
    bank_count: u16,
    raw_memory: RawMemory,
    work_ram_sections: Vec<WorkRam>,
    extended_ram: RawMemoryArray<KIBIBYTE>,
}

impl PrgMemory {
    pub fn new(
        layouts: Vec<PrgLayout>,
        layout_index: u8,
        bank_size_override: Option<u16>,
        raw_memory: RawMemory,
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

        let bank_size = bank_size.expect("at least one ROM or RAM window");

        let bank_count;
        if raw_memory.size() % u32::from(bank_size) == 0 {
            bank_count = (raw_memory.size() / u32::from(bank_size))
                .try_into()
                .expect("Way too many banks.");
        } else if !raw_memory.is_empty() && u32::from(bank_size) % raw_memory.size() == 0 {
            bank_count = 1;
        } else {
            panic!("Bad PRG length: {} . Bank size: {} .", raw_memory.size(), bank_size);
        }

        let mut prg_memory = PrgMemory {
            layouts,
            layout_index,
            bank_size,
            bank_count,
            raw_memory,
            work_ram_sections: Vec::new(),
            extended_ram: RawMemoryArray::new(),
        };

        prg_memory.work_ram_sections = prg_memory.current_layout().windows().iter()
            .filter(|window| window.bank().is_work_ram())
            .map(|window| WorkRam::new(window.size()))
            .collect();

        let bank_count = prg_memory.bank_count();
        if prg_memory.raw_memory.size() >= bank_count as u32 * bank_size as u32 {
            assert_eq!(
                prg_memory.raw_memory.size(),
                bank_count as u32 * bank_size as u32,
                "Bad PRG data size.",
            );
        }
        //assert_eq!(bank_count & (bank_count - 1), 0);

        prg_memory
    }

    pub fn bank_size(&self) -> u16 {
        self.bank_size
    }

    pub fn bank_count(&self) -> u16 {
        self.bank_count
    }

    pub fn last_bank_index(&self) -> u16 {
        self.bank_count() - 1
    }

    pub fn extended_ram(&self) -> &RawMemoryArray<KIBIBYTE> {
        &self.extended_ram
    }

    pub fn extended_ram_mut(&mut self) -> &mut RawMemoryArray<KIBIBYTE> {
        &mut self.extended_ram
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
                        ReadResult::full(self.raw_memory[index % self.raw_memory.size()]),
                }
            }
            PrgMemoryIndex::WorkRam { section_id, index, ram_status} => {
                let work_ram = &self.work_ram_sections[section_id];
                use RamStatus::*;
                match ram_status {
                    Disabled | WriteOnly =>
                        ReadResult::OPEN_BUS,
                    ReadOnlyZeros =>
                        ReadResult::full(0),
                    ReadOnly | ReadWrite =>
                        ReadResult::full(work_ram.data[index as usize]),
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
                    self.raw_memory[index] = value;
                }
            }
            PrgMemoryIndex::WorkRam { section_id, index, ram_status} => {
                let work_ram = &mut self.work_ram_sections[section_id];
                if ram_status.is_writable() {
                    work_ram.data[index as usize] = value;
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

    fn address_to_prg_index(&self, registers: &BankRegisters, address: CpuAddress) -> PrgMemoryIndex {
        let address = address.to_raw();

        let windows = &self.current_layout().windows();
        assert!(!windows.is_empty());
        assert!(address >= windows[0].start());

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
                            window.resolved_bank_index(registers, location, self.bank_size, self.bank_count(), true);
                        let index = resolved_bank_index as u32 * self.bank_size as u32 + bank_offset as u32;
                        PrgMemoryIndex::MappedMemory { index, ram_status: RamStatus::ReadOnly }
                    }
                    Bank::Ram(location, status_register_id) => {
                        let resolved_bank_index =
                            window.resolved_bank_index(registers, location, self.bank_size, self.bank_count(), true);
                        let index = resolved_bank_index as u32 * self.bank_size as u32 + bank_offset as u32;

                        let ram_status: RamStatus = status_register_id
                            .map_or(RamStatus::ReadWrite, |id| registers.ram_status(id));
                        PrgMemoryIndex::MappedMemory { index, ram_status }
                    }
                    Bank::WorkRam(status_register_id) => {
                        let ram_status: RamStatus = status_register_id
                            .map_or(RamStatus::ReadWrite, |id| registers.ram_status(id));
                        let mut index = u32::from(bank_offset);
                        let mut result = None;
                        for (section_id, work_ram_section) in self.work_ram_sections.iter().enumerate() {
                            if index < work_ram_section.data.len() as u32 {
                                result = Some(PrgMemoryIndex::WorkRam { section_id, index, ram_status });
                                break;
                            }

                            index -= work_ram_section.data.len() as u32;
                        }

                        result.unwrap()
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
}

enum PrgMemoryIndex {
    None,
    WorkRam { section_id: usize, index: u32, ram_status: RamStatus },
    ExtendedRam { index: u32, ram_status: RamStatus },
    MappedMemory { index: u32, ram_status: RamStatus },
}

#[derive(Clone)]
struct WorkRam {
    data: Vec<u8>,
}

impl WorkRam {
    fn new(size: u16) -> WorkRam {
        WorkRam {
            data: vec![0; size as usize],
        }
    }
}
