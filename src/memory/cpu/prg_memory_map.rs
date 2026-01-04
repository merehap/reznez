use crate::memory::bank::bank::PrgBank;
use crate::memory::bank::bank_number::{MemType, PrgBankRegisters};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::window::PrgWindowSize;
use crate::util::unit::KIBIBYTE;

const PRG_SLOT_COUNT: usize = 5;
const PRG_SUB_SLOT_COUNT: usize = 64;
const PAGE_SIZE: u16 = 8 * KIBIBYTE as u16;
const SUB_PAGE_SIZE: u16 = PAGE_SIZE / 64;

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
        let ram_page_number_mask = ram_page_count.saturating_sub(1);

        let mut page_mappings = Vec::with_capacity(PRG_SLOT_COUNT);

        let mut page_offset;
        let mut windows = initial_layout.windows().iter();
        while let Some(mut window) = windows.next() {
            page_offset = 0;
            let page_multiple = window.size().page_multiple();
            if page_multiple >= 1 {
                let rom_page_number_mask = rom_page_number_mask & !(page_multiple - 1);
                for offset in 0..page_multiple {
                    // Mirror high pages to low ones if there isn't enough ROM.
                    page_offset = offset % rom_page_count;
                    let mapping = PrgMapping {
                        bank: window.bank(), rom_pages_per_bank, rom_page_number_mask, ram_page_number_mask, page_offset,
                    };
                    page_mappings.push(PrgMappingSlot::Normal(mapping));
                }

                page_offset = (page_offset + 1) % rom_page_count;
            }

            let mut sub_page_mappings = Vec::with_capacity(PRG_SUB_SLOT_COUNT);
            loop {
                for sub_page_offset in 0..window.size().sub_page_multiple() {
                    let mapping = PrgMapping {
                        bank: window.bank(), rom_pages_per_bank, rom_page_number_mask, ram_page_number_mask, page_offset,
                    };
                    sub_page_mappings.push((mapping, sub_page_offset));
                }

                if sub_page_mappings.is_empty() || sub_page_mappings.len() >= PRG_SUB_SLOT_COUNT {
                    break;
                }

                window = windows.next().unwrap();
            }

            if !sub_page_mappings.is_empty() {
                assert_eq!(sub_page_mappings.len(), 64);
                page_mappings.push(PrgMappingSlot::Multi(Box::new(sub_page_mappings.try_into().unwrap())));
            }
        }

        assert_eq!(page_mappings.len(), 5);

        let mut memory_map = Self {
            page_mappings: page_mappings.try_into().unwrap(),
            page_ids: [const { PrgPageIdSlot::Normal(None) }; PRG_SLOT_COUNT],
            save_ram_size: save_ram_size.try_into().unwrap(),
        };
        memory_map.update_page_ids(regs);
        memory_map
    }

    pub fn index_for_address(&self, addr: CpuAddress) -> Option<(MemType, PrgIndex)> {
        assert!(matches!(*addr, 0x6000..=0xFFFF));

        let addr = *addr - 0x6000;
        let mapping_index = addr / PAGE_SIZE;
        let offset = addr % PAGE_SIZE;

        match &self.page_ids[mapping_index as usize] {
            PrgPageIdSlot::Normal(prg_source_and_page_number) => {
                prg_source_and_page_number.map(|(prg_source, page_number)| {
                    let index = u32::from(page_number) * PAGE_SIZE as u32 + u32::from(offset);
                    //log::info!("Normal slot. Index: {index}, Page number: {page_number}");
                    (prg_source, index)
                })
            }
            PrgPageIdSlot::Multi(page_ids) => {
                let sub_mapping_index = offset / (KIBIBYTE as u16 / 8);
                let (prg_source_and_page_number, sub_page_offset) = page_ids[sub_mapping_index as usize];
                let offset = offset % SUB_PAGE_SIZE;
                prg_source_and_page_number.map(|(source, page_number)| {
                    let index = u32::from(page_number) * PAGE_SIZE as u32 + SUB_PAGE_SIZE as u32 * sub_page_offset as u32 + u32::from(offset);
                    //log::info!("Sub slot. Index: {index:X}, Page number: {page_number:X}, Sub page: {sub_page_offset:X} Offset: {offset:X}");
                    //log::info!("    Page block: {:X}, Sub page block: {:X}", u32::from(page_number) * PAGE_SIZE as u32, SUB_PAGE_SIZE as u32 * sub_page_offset as u32);
                    (source, index)
                })
            }
        }
    }

    pub fn page_id_slots(&self) -> &[PrgPageIdSlot; PRG_SLOT_COUNT] {
        &self.page_ids
    }

    pub fn update_page_ids(&mut self, regs: &PrgBankRegisters) {
        let save_ram_bank_count = if self.save_ram_size > 0 && (self.save_ram_size as u32) < 8 * KIBIBYTE {
            1
        } else {
            self.save_ram_size / (8 * KIBIBYTE as u16)
        };

        for i in 0..PRG_SLOT_COUNT {
            match &self.page_mappings[i] {
                PrgMappingSlot::Normal(mapping) => {
                    let page_id = mapping.page_id(regs, save_ram_bank_count);
                    self.page_ids[i] = PrgPageIdSlot::Normal(page_id);
                    //log::info!("Page ID: {:?}", self.page_ids[i]);
                }
                PrgMappingSlot::Multi(mappings) => {
                    let mut page_ids = Vec::new();
                    for (mapping, offset) in mappings.iter() {
                        let page_id = mapping.page_id(regs, save_ram_bank_count);
                        page_ids.push((page_id, *offset));
                    }

                    self.page_ids[i] = PrgPageIdSlot::Multi(Box::new(page_ids.try_into().unwrap()));
                    //log::info!("Page ID: {:?}", self.page_ids[i]);
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
    pub fn page_id(&self, regs: &PrgBankRegisters, save_ram_bank_count: u16) -> Option<(MemType, PageNumber)> {
        let (Ok(bank_number), Some(mem_type)) = (self.bank.bank_number(regs), self.bank.memory_type(regs)) else {
            return None;
        };

        match mem_type {
            MemType::Rom(_) => {
                let page_number = ((self.rom_pages_per_bank * bank_number.to_raw()) & self.rom_page_number_mask) + self.page_offset;
                //println!("Page number within mapping: {page_number}. Bank Index: {}. Page offset: {}", bank_number.to_raw(), self.page_offset);
                Some((mem_type, page_number))
            }
            // FIXME: Pull these out into separate cases, and handle the splitting earlier?
            MemType::WorkRam(read_status_register_id, write_status_register_id)
                    | MemType::SaveRam(read_status_register_id, write_status_register_id) => {
                let mut page_number = (bank_number.to_raw() & self.ram_page_number_mask) + self.page_offset;
                if page_number < save_ram_bank_count {
                    Some((MemType::SaveRam(read_status_register_id, write_status_register_id), page_number))
                } else {
                    page_number -= save_ram_bank_count;
                    Some((MemType::WorkRam(read_status_register_id, write_status_register_id), page_number))
                }
            }
        }
    }
}

// FIXME
#[allow(clippy::type_complexity)]
#[derive(Clone, Debug)]
pub enum PrgPageIdSlot {
    Normal(Option<MemTypeAndPageNumber>),
    Multi(Box<[(Option<MemTypeAndPageNumber>, SubPageOffset); PRG_SUB_SLOT_COUNT]>),
}

type PageNumber = u16;
type PrgIndex = u32;
type MemTypeAndPageNumber = (MemType, PageNumber);