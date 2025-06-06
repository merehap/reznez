use std::num::NonZeroU8;

use log::warn;

use crate::mapper::{BankIndex, PrgBankRegisterId, ReadWriteStatusRegisterId};
use crate::memory::bank::bank::{PrgBank, RomRamModeRegisterId};
use crate::memory::bank::bank_index::{PrgBankRegisters, ReadWriteStatus, MemoryType};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::cpu::prg_memory_map::{PrgMemoryMap, PrgIndex};
use crate::memory::raw_memory::{RawMemory, RawMemoryArray, SaveRam};
use crate::memory::read_result::ReadResult;
use crate::memory::window::{ReadWriteStatusInfo, PrgWindow};
use crate::util::unit::KIBIBYTE;

pub struct PrgMemory {
    layouts: Vec<PrgLayout>,
    memory_maps: Vec<PrgMemoryMap>,
    layout_index: u8,
    rom: Vec<RawMemory>,
    rom_outer_bank_index: u8,
    work_ram: RawMemory,
    save_ram: SaveRam,
    extended_ram: RawMemoryArray<KIBIBYTE>,
    regs: PrgBankRegisters,
}

impl PrgMemory {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layouts: Vec<PrgLayout>,
        layout_index: u8,
        rom: RawMemory,
        rom_outer_bank_count: NonZeroU8,
        work_ram: RawMemory,
        save_ram: SaveRam,
        regs: PrgBankRegisters,
    ) -> PrgMemory {

        let mut rom_bank_size = None;
        let mut ram_present_in_layout = false;
        for layout in &layouts {
            for window in layout.windows() {
                if matches!(window.bank(), PrgBank::Rom(..) | PrgBank::RomRam(..)) {
                    if let Some(size) = rom_bank_size {
                        rom_bank_size = Some(std::cmp::min(window.size(), size));
                    } else {
                        rom_bank_size = Some(window.size());
                    }
                }

                if window.bank().is_ram() {
                    ram_present_in_layout = true;
                }
            }
        }

        let rom_bank_size = rom_bank_size.expect("at least one ROM window");
        if (!work_ram.is_empty() || !save_ram.is_empty()) && !ram_present_in_layout {
            warn!("The PRG RAM that was specified in the rom file will be ignored since it is not \
                    configured in the Layout for this mapper.");
        }

        let rom_outer_bank_size = rom.size() / rom_outer_bank_count.get() as u32;
        let memory_maps = layouts.iter().map(|initial_layout| PrgMemoryMap::new(
            *initial_layout, rom_outer_bank_size, rom_bank_size, work_ram.size(), save_ram.size(), &regs,
        )).collect();

        PrgMemory {
            layouts,
            memory_maps,
            layout_index,
            rom: rom.split_n(rom_outer_bank_count),
            rom_outer_bank_index: 0,
            work_ram,
            save_ram,
            extended_ram: RawMemoryArray::new(),
            regs,
        }
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

    pub fn read_write_status_infos(&self) -> Vec<ReadWriteStatusInfo> {
        let mut ids = Vec::new();
        for layout in &self.layouts {
            ids.append(&mut layout.read_write_status_infos());
        }

        ids
    }

    pub fn peek(&self, address: CpuAddress) -> ReadResult {
        let (prg_index, read_write_status) =
            self.memory_maps[self.layout_index as usize].index_for_address(address);
        if read_write_status == ReadWriteStatus::ReadOnlyZeros {
            ReadResult::full(0)
        } else {
            match prg_index {
                Some(PrgIndex::Rom(index)) if read_write_status.is_readable() =>
                    ReadResult::full(self.rom[self.rom_outer_bank_index as usize][index]),
                Some(PrgIndex::WorkRam(index)) if read_write_status.is_readable() =>
                    ReadResult::full(self.work_ram[index]),
                Some(PrgIndex::SaveRam(index)) if read_write_status.is_readable() =>
                    ReadResult::full(self.save_ram[index]),
                None | Some(PrgIndex::Rom(_) | PrgIndex::WorkRam(_) | PrgIndex::SaveRam(_)) =>
                    ReadResult::OPEN_BUS,
            }
        }
    }

    pub fn write(&mut self, address: CpuAddress, value: u8) {
        let (prg_index, read_write_status) =
            self.memory_maps[self.layout_index as usize].index_for_address(address);
        if read_write_status.is_writable() {
            match prg_index {
                None | Some(PrgIndex::Rom(_)) => unreachable!(),
                Some(PrgIndex::WorkRam(index)) => self.work_ram[index] = value,
                Some(PrgIndex::SaveRam(index)) => self.save_ram[index] = value,
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

    pub fn set_rom_ram_mode(&mut self, id: RomRamModeRegisterId, rom_ram_mode: MemoryType) {
        self.regs.set_rom_ram_mode(id, rom_ram_mode);
        self.update_page_ids();
    }

    pub fn window_at(&self, start: u16) -> &PrgWindow {
        self.window_with_index_at(start).0
    }

    pub fn current_layout(&self) -> &PrgLayout {
        &self.layouts[usize::from(self.layout_index)]
    }

    pub fn current_memory_map(&self) -> &PrgMemoryMap {
        &self.memory_maps[self.layout_index as usize]
    }

    pub fn bank_registers(&self) -> &PrgBankRegisters {
        &self.regs
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn set_prg_rom_outer_bank_index(&mut self, index: u8) {
        self.rom_outer_bank_index = index;
    }

    fn update_page_ids(&mut self) {
        for memory_map in &mut self.memory_maps {
            memory_map.update_page_ids(&self.regs);
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