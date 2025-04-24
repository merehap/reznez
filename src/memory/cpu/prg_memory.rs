use std::num::{NonZeroU8, NonZeroU16};

use crate::mapper::{BankIndex, PrgBankRegisterId, ReadWriteStatusRegisterId};
use crate::memory::bank::bank::{PrgBank, PrgBankLocation, RomRamModeRegisterId};
use crate::memory::bank::bank_index::{PrgBankRegisters, ReadWriteStatus, RomRamMode};
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
    page_mappings: [PrgMappingSlot; PRG_SLOT_COUNT],
    page_ids: [PrgPageIdSlot; PRG_SLOT_COUNT],
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
        let rom_pages_per_bank = rom_bank_size / PAGE_SIZE;
        assert_eq!(rom_size % u32::from(PAGE_SIZE), 0);

        let rom_page_count: u16 = (rom_size / u32::from(PAGE_SIZE)).try_into().unwrap();
        let mut rom_page_number_mask = 0b1111_1111_1111_1111;
        rom_page_number_mask &= rom_page_count - 1;

        let ram_page_count: u16 = (ram_size / u32::from(PAGE_SIZE)).try_into().unwrap();
        let mut ram_page_number_mask = 0b1111_1111_1111_1111;
        ram_page_number_mask &= ram_page_count - 1;

        let mut page_mappings = Vec::with_capacity(PRG_SLOT_COUNT);
        let mut sub_page_mappings = Vec::with_capacity(PRG_SUB_SLOT_COUNT);

        let mut address = 0x6000;
        for window in initial_layout.windows() {
            assert!(window.start() >= 0x6000);
            let mut bank = window.bank();
            if let PrgBank::MirrorOf(mirroree_address) = bank {
                for mirroree in initial_layout.windows() {
                    if mirroree.start() == mirroree_address {
                        bank = mirroree.bank();
                    }
                }

                assert!(!matches!(bank, PrgBank::MirrorOf(_)), "Invalid mirror window.")
            }

            if window.size().get() >= 0x2000 {
                assert_eq!(window.start() % 0x2000, 0, "Windows must start on a page boundary.");

                match access_override {
                    None => {}
                    // TODO: Work RAM should become disabled, not ROM.
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
                let rom_page_number_mask = rom_page_number_mask & !(rom_pages_per_window - 1);

                let mut page_offset = 0;
                while window.is_in_bounds(address) {
                    let mapping = PrgMappingSlot::Normal(PrgMapping {
                        bank,
                        rom_pages_per_bank,
                        rom_page_number_mask,
                        ram_page_number_mask,
                        page_offset,
                    });
                    page_mappings.push(mapping);
                    address += PAGE_SIZE;
                    page_offset += 1;
                    // Mirror high pages to low ones if there isn't enough ROM.
                    page_offset %= rom_page_count;
                }
            } else {
                match access_override {
                    None => {}
                    // TODO: Work RAM should become disabled, not ROM.
                    Some(AccessOverride::ForceRom) => bank = bank.as_rom(),
                    Some(AccessOverride::ForceRam) => panic!("PRG must have some ROM."),
                }

                let mut sub_page_offset = 0;
                while window.is_in_bounds(address) {
                    let mapping = PrgMapping {
                        bank,
                        rom_pages_per_bank: 1,
                        rom_page_number_mask,
                        ram_page_number_mask,
                        page_offset: 0,
                    };
                    sub_page_mappings.push((mapping, sub_page_offset));
                    address += PAGE_SIZE / 64;
                    sub_page_offset += 1;
                }

                if sub_page_mappings.len() == 64 {
                    page_mappings.push(PrgMappingSlot::Multi(Box::new(sub_page_mappings.try_into().unwrap())));
                    sub_page_mappings = Vec::new();
                }
            }
        }

        assert_eq!(page_mappings.len(), 5);

        let mut memory_map = Self {
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [const { PrgPageIdSlot::Normal(PrgPageId::Rom(0), ReadWriteStatus::ReadOnly) }; PRG_SLOT_COUNT],
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

        match &self.page_ids[mapping_index as usize] {
            PrgPageIdSlot::Normal(page_id, read_write_status) => {
                let prg_memory_index = match page_id {
                    PrgPageId::Empty => PrgIndex::None,
                    PrgPageId::Rom(page_number) => {
                        PrgIndex::Rom(u32::from(*page_number) * PAGE_SIZE as u32 + u32::from(offset))
                    }
                    PrgPageId::Ram(page_number) => {
                        PrgIndex::Ram(u32::from(*page_number) * PAGE_SIZE as u32 + u32::from(offset))
                    }
                };
                (prg_memory_index, *read_write_status)
            }
            PrgPageIdSlot::Multi(page_ids) => {
                let sub_mapping_index = offset / (KIBIBYTE as u16 / 8);
                let (page_id, read_write_status, sub_page_offset) = page_ids[sub_mapping_index as usize];
                let prg_memory_index = match page_id {
                    PrgPageId::Empty => PrgIndex::None,
                    PrgPageId::Rom(page_number) => {
                        PrgIndex::Rom(u32::from(page_number) * PAGE_SIZE as u32 + (PAGE_SIZE as u32 / 64) * sub_page_offset as u32 + u32::from(offset))
                    }
                    PrgPageId::Ram(page_number) => {
                        PrgIndex::Ram(u32::from(page_number) * PAGE_SIZE as u32 + (PAGE_SIZE as u32 / 64) * sub_page_offset as u32 + u32::from(offset))
                    }
                };
                (prg_memory_index, read_write_status)
            }
        }
    }

    /*
    pub fn page_mappings(&self) -> &[PrgMapping; PRG_SLOT_COUNT] {
        &self.page_mappings
    }
    */
    pub fn page_id_slots(&self) -> &[PrgPageIdSlot; PRG_SLOT_COUNT] {
        &self.page_ids
    }

    pub fn update_page_ids(&mut self, regs: &PrgBankRegisters) {
        for i in 0..PRG_SLOT_COUNT {
            match &self.page_mappings[i] {
                PrgMappingSlot::Normal(mapping) => {
                    let (page_id, read_write_status) = mapping.page_id(regs);
                    self.page_ids[i] = PrgPageIdSlot::Normal(page_id, read_write_status);
                }
                PrgMappingSlot::Multi(mappings) => {
                    let mut page_ids = Vec::new();
                    for (mapping, offset) in mappings.iter() {
                        let (page_id, read_write_status) = mapping.page_id(regs);
                        page_ids.push((page_id, read_write_status, *offset));
                    }

                    self.page_ids[i] = PrgPageIdSlot::Multi(Box::new(page_ids.try_into().unwrap()));
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrgIndex {
    None,
    Rom(u32),
    Ram(u32),
}

#[derive(Clone, Debug)]
pub enum PrgMappingSlot {
    Normal(PrgMapping),
    Multi(Box<[(PrgMapping, SubPageOffset); PRG_SUB_SLOT_COUNT]>),
}

#[derive(Clone, Debug)]
pub struct PrgMapping {
    bank: PrgBank,
    rom_pages_per_bank: u16,
    page_offset: u16,
    rom_page_number_mask: PageNumberMask,
    ram_page_number_mask: PageNumberMask,
}

type PageNumberMask = u16;
type SubPageOffset = u8;

impl PrgMapping {
    pub fn page_id(&self, registers: &PrgBankRegisters) -> (PrgPageId, ReadWriteStatus) {
        let page_number = || {
            let location = self.bank.location().expect("Location to be present in bank.");
            let bank_index = match location {
                PrgBankLocation::Fixed(bank_index) => bank_index,
                PrgBankLocation::Switchable(register_id) => registers.get(register_id).index().unwrap(),
            };

            if self.bank.is_rom(registers) {
                ((self.rom_pages_per_bank * bank_index.to_raw()) & self.rom_page_number_mask) + self.page_offset
            } else {
                (bank_index.to_raw() & self.ram_page_number_mask) + self.page_offset
            }
        };

        match self.bank {
            PrgBank::Empty =>
                (PrgPageId::Empty, ReadWriteStatus::Disabled),
            PrgBank::Rom(_, None) =>
                (PrgPageId::Rom(page_number()), ReadWriteStatus::ReadOnly),
            PrgBank::Rom(_, Some(status_register)) =>
                (PrgPageId::Rom(page_number()), registers.read_write_status(status_register)),
            PrgBank::Ram(_, None) | PrgBank::WorkRam(_, None) =>
                (PrgPageId::Ram(page_number()), ReadWriteStatus::ReadWrite),
            PrgBank::Ram(_, Some(status_register)) | PrgBank::WorkRam(_, Some(status_register)) =>
                (PrgPageId::Ram(page_number()), registers.read_write_status(status_register)),
            PrgBank::RomRam(_, status_register, rom_ram_register) => {
                match registers.rom_ram_mode(rom_ram_register) {
                    RomRamMode::Rom => (PrgPageId::Rom(page_number()), ReadWriteStatus::ReadOnly),
                    RomRamMode::Ram => (PrgPageId::Ram(page_number()), registers.read_write_status(status_register)),
                }
            }
            PrgBank::MirrorOf(_) => unreachable!("Mirrored banks should have been resolved by now."),
            _ => todo!(),
        }
    }
}


#[derive(Clone, Debug)]
pub enum PrgPageIdSlot {
    Normal(PrgPageId, ReadWriteStatus),
    Multi(Box<[(PrgPageId, ReadWriteStatus, SubPageOffset); PRG_SUB_SLOT_COUNT]>),
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
    rom: Vec<RawMemory>,
    rom_outer_bank_index: u8,
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
        let rom_outer_bank_size = rom.size() / rom_outer_bank_count.get() as u32;
        let memory_maps = layouts.iter().map(|initial_layout| PrgMemoryMap::new(
            *initial_layout, rom_outer_bank_size, rom_bank_size, ram_size, access_override, &regs,
        )).collect();

        PrgMemory {
            layouts,
            memory_maps,
            layout_index,
            rom: rom.split_n(rom_outer_bank_count),
            rom_outer_bank_index: 0,
            ram: RawMemory::new(ram_size),
            extended_ram: RawMemoryArray::new(),
            access_override,
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
        let (prg_index, read_write_status) =
            self.memory_maps[self.layout_index as usize].index_for_address(address);
        if read_write_status == ReadWriteStatus::ReadOnlyZeros {
            ReadResult::full(0)
        } else {
            match prg_index {
                PrgIndex::Rom(index) if read_write_status.is_readable() =>
                    ReadResult::full(self.rom[self.rom_outer_bank_index as usize][index]),
                PrgIndex::Ram(index) if read_write_status.is_readable() =>
                    ReadResult::full(self.ram[index]),
                PrgIndex::None | PrgIndex::Rom(_) | PrgIndex::Ram(_) =>
                    ReadResult::OPEN_BUS,
            }
        }
    }

    pub fn write(&mut self, address: CpuAddress, value: u8) {
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