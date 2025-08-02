use crate::memory::bank::bank::PrgBank;
use crate::memory::bank::bank_index::{PrgBankRegisters, ReadWriteStatus, MemType};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::window::PrgWindowSize;
use crate::util::unit::KIBIBYTE;

const PRG_SLOT_COUNT: usize = 5;
const PRG_SUB_SLOT_COUNT: usize = 64;
const PAGE_SIZE: u16 = 8 * KIBIBYTE as u16;

pub struct PrgMemoryMap {
    // 0x6000 through 0xFFFF
    page_mappings: [PrgMappingSlot; PRG_SLOT_COUNT],
    page_ids: [PrgPageIdSlot; PRG_SLOT_COUNT],
    save_ram_size: u16,
}

impl PrgMemoryMap {
    pub fn new(
        initial_layout: PrgLayout,
        rom_size: u32,
        rom_bank_size: PrgWindowSize,
        ram_size: u32,
        save_ram_size: u32,
        regs: &PrgBankRegisters,
    ) -> Self {

        let rom_pages_per_bank = rom_bank_size.page_multiple();
        assert_eq!(rom_size % u32::from(PAGE_SIZE), 0);

        let rom_page_count: u16 = (rom_size / u32::from(PAGE_SIZE)).try_into().unwrap();
        let mut rom_page_number_mask = 0b1111_1111_1111_1111;
        rom_page_number_mask &= rom_page_count - 1;

        let ram_page_count: u16 = ((ram_size + save_ram_size) / u32::from(PAGE_SIZE)).try_into().unwrap();
        let mut ram_page_number_mask = 0b1111_1111_1111_1111;
        ram_page_number_mask &= ram_page_count - 1;

        let mut page_mappings = Vec::with_capacity(PRG_SLOT_COUNT);
        let mut sub_page_mappings = Vec::with_capacity(PRG_SUB_SLOT_COUNT);

        for window in initial_layout.windows() {
            let bank = window.bank();

            let page_multiple = window.size().page_multiple();
            if page_multiple >= 1 {
                let rom_page_number_mask = rom_page_number_mask & !(page_multiple - 1);
                for page_offset in 0..page_multiple {
                    // Mirror high pages to low ones if there isn't enough ROM.
                    let page_offset = page_offset % rom_page_count;
                    let mapping = PrgMapping {
                        bank, rom_pages_per_bank, rom_page_number_mask, ram_page_number_mask, page_offset,
                    };
                    page_mappings.push(PrgMappingSlot::Normal(mapping));
                }
            } else {
                for sub_page_offset in 0..window.size().sub_page_multiple() {
                    let mapping = PrgMapping {
                        bank, rom_pages_per_bank: 1, rom_page_number_mask, ram_page_number_mask, page_offset: 0,
                    };
                    sub_page_mappings.push((mapping, sub_page_offset));
                }

                assert!(sub_page_mappings.len() <= 64);
                if sub_page_mappings.len() == 64 {
                    page_mappings.push(PrgMappingSlot::Multi(Box::new(sub_page_mappings.try_into().unwrap())));
                    sub_page_mappings = Vec::new();
                }
            }
        }

        assert_eq!(page_mappings.len(), 5);

        let mut memory_map = Self {
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [const { PrgPageIdSlot::Normal(None, ReadWriteStatus::Disabled) }; PRG_SLOT_COUNT],
            save_ram_size: save_ram_size.try_into().unwrap(),
        };
        memory_map.update_page_ids(regs);
        memory_map
    }

    pub fn index_for_address(&self, address: CpuAddress) -> (Option<(MemType, PrgIndex)>, ReadWriteStatus) {
        let address = address.to_raw();
        assert!(matches!(address, 0x6000..=0xFFFF));

        let address = address - 0x6000;
        let mapping_index = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;

        match &self.page_ids[mapping_index as usize] {
            PrgPageIdSlot::Normal(prg_source_and_page_number, read_write_status) => {
                let prg_memory_index = prg_source_and_page_number.map(|(prg_source, page_number)| {
                    let index = u32::from(page_number) * PAGE_SIZE as u32 + u32::from(offset);
                    (prg_source, index)
                });
                (prg_memory_index, *read_write_status)
            }
            PrgPageIdSlot::Multi(page_ids) => {
                let sub_mapping_index = offset / (KIBIBYTE as u16 / 8);
                let (prg_source_and_page_number, read_write_status, sub_page_offset) = page_ids[sub_mapping_index as usize];
                let prg_memory_index = prg_source_and_page_number.map(|(source, page_number)| {
                    let index = u32::from(page_number) * PAGE_SIZE as u32 + (PAGE_SIZE as u32 / 64) * sub_page_offset as u32 + u32::from(offset);
                    (source, index)
                });
                (prg_memory_index, read_write_status)
            }
        }
    }

    pub fn page_id_slots(&self) -> &[PrgPageIdSlot; PRG_SLOT_COUNT] {
        &self.page_ids
    }

    pub fn update_page_ids(&mut self, regs: &PrgBankRegisters) {
        let save_ram_bank_count = self.save_ram_size / (8 * KIBIBYTE as u16);
        for i in 0..PRG_SLOT_COUNT {
            match &self.page_mappings[i] {
                PrgMappingSlot::Normal(mapping) => {
                    let (page_id, read_write_status) = mapping.page_id(regs, save_ram_bank_count);
                    self.page_ids[i] = PrgPageIdSlot::Normal(page_id, read_write_status);
                }
                PrgMappingSlot::Multi(mappings) => {
                    let mut page_ids = Vec::new();
                    for (mapping, offset) in mappings.iter() {
                        let (page_id, read_write_status) = mapping.page_id(regs, save_ram_bank_count);
                        page_ids.push((page_id, read_write_status, *offset));
                    }

                    self.page_ids[i] = PrgPageIdSlot::Multi(Box::new(page_ids.try_into().unwrap()));
                }
            }
        }
    }
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
    pub fn page_id(&self, regs: &PrgBankRegisters, save_ram_bank_count: u16) -> (Option<(MemType, PageNumber)>, ReadWriteStatus) {
        let (Ok(location), Some(memory_type)) = (self.bank.location(), self.bank.memory_type(regs)) else {
            return (None, ReadWriteStatus::Disabled);
        };

        let bank_index = location.bank_index(regs);

        let default_rw_status;
        let prg_source_and_page_number;
        match memory_type {
            MemType::Rom => {
                default_rw_status = ReadWriteStatus::ReadOnly;
                let page_number = ((self.rom_pages_per_bank * bank_index.to_raw()) & self.rom_page_number_mask) + self.page_offset;
                prg_source_and_page_number = (MemType::Rom, page_number);
            }
            // FIXME: Pull these out into separate cases, and handle the splitting earlier?
            MemType::WorkRam | MemType::SaveRam => {
                default_rw_status = ReadWriteStatus::ReadWrite;
                let mut page_number = (bank_index.to_raw() & self.ram_page_number_mask) + self.page_offset;
                if page_number < save_ram_bank_count {
                    prg_source_and_page_number = (MemType::SaveRam, page_number);
                } else {
                    page_number -= save_ram_bank_count;
                    prg_source_and_page_number = (MemType::WorkRam, page_number);
                }
            }
        }

        let read_write_status = self.bank.status_register_id()
            .map(|id| regs.read_write_status(id))
            .unwrap_or(default_rw_status);

        (Some(prg_source_and_page_number), read_write_status)
    }
}

#[derive(Clone, Debug)]
pub enum PrgPageIdSlot {
    Normal(Option<PrgSourceAndPageNumber>, ReadWriteStatus),
    Multi(Box<[(Option<PrgSourceAndPageNumber>, ReadWriteStatus, SubPageOffset); PRG_SUB_SLOT_COUNT]>),
}

type PageNumber = u16;
type PrgIndex = u32;
type PrgSourceAndPageNumber = (MemType, PageNumber);