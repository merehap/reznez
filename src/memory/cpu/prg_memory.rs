use log::{info, warn};
use crate::mapper::{BankNumber, KIBIBYTE, PrgBankRegisterId};
use crate::memory::address_template::BankSizes;
use crate::memory::bank::bank::{PrgSource, ReadStatusRegisterId, PrgSourceRegisterId, WriteStatusRegisterId};
use crate::memory::bank::bank_number::{MemType, PrgBankRegisters, ReadStatus, WriteStatus};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::cpu::prg_memory_map::{PageInfo, PrgMemoryMap, PrgPageIdSlot};
use crate::memory::layout::OuterBankLayout;
use crate::memory::raw_memory::{RawMemory, SaveRam};
use crate::memory::read_result::ReadResult;
use crate::memory::window::PrgWindow;

pub struct PrgMemory {
    layouts: Vec<PrgLayout>,
    memory_maps: Vec<PrgMemoryMap>,
    layout_index: u8,
    rom: RawMemory,
    rom_outer_bank_number: u8,
    work_ram: RawMemory,
    save_ram: SaveRam,
    regs: PrgBankRegisters,
}

impl PrgMemory {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        layouts: Vec<PrgLayout>,
        layout_index: u8,
        rom: RawMemory,
        rom_outer_bank_layout: OuterBankLayout,
        mut work_ram: RawMemory,
        mut save_ram: SaveRam,
        regs: PrgBankRegisters,
    ) -> PrgMemory {

        let rom_inner_bank_size = layouts.iter()
            .flat_map(PrgLayout::windows)
            .filter(|window| window.bank().is_rom())
            .map(|window| window.size())
            .reduce(std::cmp::min)
            .expect("at least one ROM window");
        let ram_supported_by_layout = layouts.iter()
            .flat_map(PrgLayout::windows)
            .any(|window| window.bank().supports_ram());

        if (!work_ram.is_empty() || !save_ram.is_empty()) && !ram_supported_by_layout {
            warn!("The PRG RAM that was specified in the rom file will be ignored since it is not \
                    configured in the Layout for this mapper.");
            work_ram = RawMemory::new(0);
            save_ram = SaveRam::empty();
        }

        let rom_outer_bank_count = rom_outer_bank_layout.outer_bank_count(rom.size());
        let rom_outer_bank_size = rom.size() / rom_outer_bank_count.get() as u32;

        assert_eq!(rom_outer_bank_size & (rom_outer_bank_size - 1), 0);
        assert_eq!(rom_outer_bank_size % (8 * KIBIBYTE), 0);
        let rom_bank_sizes = BankSizes::new(
            rom.size(),
            rom_outer_bank_size,
            rom_inner_bank_size.to_raw().into());

        // When a mapper has both Work RAM and Save RAM, the bank/page numbers are shared (save ram gets the lower numbers).
        let ram_size = work_ram.size() + save_ram.size();
        let ram_bank_sizes = BankSizes::new(
            ram_size,
            ram_size,
            8 * KIBIBYTE, // FIXME: Hack
        );

        let memory_maps = layouts.iter()
            .map(|initial_layout| PrgMemoryMap::new(*initial_layout, &rom_bank_sizes, &ram_bank_sizes, &regs))
            .collect();

        PrgMemory {
            layouts,
            memory_maps,
            layout_index,
            rom,
            rom_outer_bank_number: 0,
            work_ram,
            save_ram,
            regs,
        }
    }

    pub fn layout_index(&self) -> u8 {
        self.layout_index
    }

    pub fn peek(&self, address: CpuAddress) -> ReadResult {
        if let Some((mem_type, index)) = self.memory_maps[self.layout_index as usize].index_for_address(self.rom_outer_bank_number, address) {
            match (mem_type, mem_type.read_status()) {
                (_                   , ReadStatus::Disabled     ) => ReadResult::OPEN_BUS,
                (_                   , ReadStatus::ReadOnlyZeros) => ReadResult::full(0),
                (MemType::WorkRam(..), ReadStatus::Enabled      ) => ReadResult::full(self.work_ram[index]),
                (MemType::SaveRam(..), ReadStatus::Enabled      ) => ReadResult::full(self.save_ram[index % self.save_ram.size()]),
                (MemType::Rom(..)    , ReadStatus::Enabled      ) => ReadResult::full(self.rom[index]),
            }
        } else {
            ReadResult::OPEN_BUS
        }
    }

    pub fn peek_raw_rom(&self, index: u32) -> u8 {
        self.rom[index % self.rom.size()]
    }

    pub fn write(&mut self, address: CpuAddress, value: u8) {
        let prg_source_and_index = self.memory_maps[self.layout_index as usize].index_for_address(self.rom_outer_bank_number, address);
        use MemType::*;
        match prg_source_and_index {
            Some((WorkRam(_, WriteStatus::Enabled), index)) => {
                self.work_ram[index] = value;
                info!(target: "mapperramwrites", "Setting PRG [${address}]=${value:02} (Work RAM @ ${index:X})");
            }
            Some((SaveRam(_, WriteStatus::Enabled), index)) => {
                let index = index % self.save_ram.size();
                self.save_ram[index] = value;
                info!(target: "mapperramwrites", "Setting PRG [${address}]=${value:02} (Save RAM @ ${index:X})");
            }
            Some((Rom {..} | WorkRam(_, WriteStatus::Disabled) | SaveRam(_, WriteStatus::Disabled), _)) | None => {
                /* Writes to ROM, absent banks, and disabled banks do nothing. */
            }
        }
    }

    pub fn set_bank_register<INDEX: Into<u16>>(&mut self, id: PrgBankRegisterId, value: INDEX) {
        self.regs.set(id, BankNumber::from_u16(value.into()));
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

    pub fn set_read_status(&mut self, id: ReadStatusRegisterId, read_status: ReadStatus) {
        self.regs.set_read_status(id, read_status);
        self.update_page_ids();
    }

    pub fn set_write_status(&mut self, id: WriteStatusRegisterId, write_status: WriteStatus) {
        self.regs.set_write_status(id, write_status);
        self.update_page_ids();
    }

    pub fn set_rom_ram_mode(&mut self, id: PrgSourceRegisterId, rom_ram_mode: PrgSource) {
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

    pub fn memory_maps(&self) -> &[PrgMemoryMap] {
        &self.memory_maps
    }

    pub fn bank_registers(&self) -> &PrgBankRegisters {
        &self.regs
    }

    pub fn ram_present(&self) -> bool {
        !self.work_ram.is_empty() || !self.save_ram.is_empty()
    }

    pub fn set_layout(&mut self, index: u8) {
        assert!(usize::from(index) < self.layouts.len());
        self.layout_index = index;
    }

    pub fn set_prg_rom_outer_bank_number(&mut self, number: u8) {
        self.rom_outer_bank_number = number;
    }

    pub fn set_prg_rom_outer_bank_size(&mut self, _new_size: u32) {
        todo!()
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

    pub fn prg_rom_bank_string(&self) -> String {
        let mut result = String::new();
        for prg_page_id_slot in self.current_memory_map().page_id_slots() {
            let bank_string = match prg_page_id_slot {
                PrgPageIdSlot::Normal(prg_source_and_page_number) => {
                    match prg_source_and_page_number {
                        None => "E".to_string(),
                        // FIXME: This should be bank number, not page number.
                        // TODO: Add Read/Write status to the output
                        Some(PageInfo { mem_type: MemType::Rom(..), page_number, .. }) => page_number.to_string(),
                        Some(PageInfo { mem_type: MemType::WorkRam(..), page_number, .. }) => format!("W{page_number}"),
                        Some(PageInfo { mem_type: MemType::SaveRam(..), page_number, .. }) => format!("S{page_number}"),
                    }
                }
                PrgPageIdSlot::Multi(_) => "M".to_string(),
            };

            let window_size = 8;

            let left_padding_len;
            let right_padding_len;
            if window_size < 8 {
                left_padding_len = 0;
                right_padding_len = 0;
            } else {
                let padding_size = window_size - 2u16.saturating_sub(u16::try_from(bank_string.len()).unwrap());
                left_padding_len = padding_size / 2;
                right_padding_len = padding_size - left_padding_len;
            }

            let left_padding = " ".repeat(left_padding_len as usize);
            let right_padding = " ".repeat(right_padding_len as usize);

            let segment = format!("|{left_padding}{bank_string}{right_padding}|");
            result.push_str(&segment);
        }

        result
    }
}