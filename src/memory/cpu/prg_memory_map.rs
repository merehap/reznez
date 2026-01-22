use crate::memory::address_template::AddressTemplate;
use crate::memory::bank::bank::PrgBank;
use crate::memory::bank::bank_number::{MemType, PageNumberSpace, PrgBankRegisters};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
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
    // TODO: First break everything into sub page mappings, then consolidate into pages where appropriate.
    pub fn new(
        initial_layout: PrgLayout,
        rom_address_template: &AddressTemplate,
        ram_address_template: &AddressTemplate,
        regs: &PrgBankRegisters,
    ) -> Self {
        let rom_page_count = rom_address_template.prg_pages_per_outer_bank();

        let mut page_mappings = Vec::with_capacity(PRG_SLOT_COUNT);

        let mut page_offset;
        let mut windows = initial_layout.windows().iter();
        while let Some(mut window) = windows.next() {
            page_offset = 0;
            let page_multiple = window.size().page_multiple();
            if page_multiple >= 1 {
                let rom_address_template = rom_address_template.with_bigger_bank(window.size().bit_count());
                for offset in 0..page_multiple {
                    // Mirror high pages to low ones if there isn't enough ROM.
                    page_offset = offset % rom_page_count;
                    let mapping = PrgMapping {
                        bank: window.bank(),
                        rom_address_template: rom_address_template.clone(),
                        ram_address_template: ram_address_template.clone(),
                        page_offset,
                    };
                    page_mappings.push(PrgMappingSlot::Normal(mapping));
                }

                page_offset = (page_offset + 1) % rom_page_count;
            }

            let mut sub_page_mappings = Vec::with_capacity(PRG_SUB_SLOT_COUNT);
            loop {
                for sub_page_offset in 0..window.size().sub_page_multiple() {
                    let mapping = PrgMapping {
                        bank: window.bank(),
                        rom_address_template: Some(rom_address_template.clone()),
                        ram_address_template: ram_address_template.clone(),
                        page_offset,
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
        };
        memory_map.update_page_ids(regs);
        memory_map
    }

    pub fn index_for_address(&self, rom_outer_bank_number: u8, addr: CpuAddress) -> Option<(MemType, PrgIndex)> {
        assert!(matches!(*addr, 0x6000..=0xFFFF));

        let addr = *addr - 0x6000;
        let mapping_index = addr / PAGE_SIZE;
        let offset_in_page = addr % PAGE_SIZE;

        match &self.page_ids[mapping_index as usize] {
            PrgPageIdSlot::Normal(page_info) => {
                page_info.as_ref().map(|PageInfo { mem_type, page_number, address_template }| {
                    let outer_bank_number = if mem_type.is_rom() { rom_outer_bank_number } else { 0 };
                    (*mem_type, address_template.resolve_index(outer_bank_number, *page_number, offset_in_page))
                })
            }
            PrgPageIdSlot::Multi(page_infos) => {
                let sub_mapping_index = offset_in_page / (KIBIBYTE as u16 / 8);
                let (page_info, sub_page_offset) = page_infos[sub_mapping_index as usize].clone();
                page_info.map(|PageInfo { mem_type, page_number, address_template }| {
                    let outer_bank_number = if mem_type.is_rom() { rom_outer_bank_number } else { 0 };
                    let index = address_template.resolve_subpage_index(outer_bank_number, page_number, sub_page_offset, offset_in_page);
                    (mem_type, index)
                })
            }
        }
    }

    pub fn page_id_slots(&self) -> &[PrgPageIdSlot; PRG_SLOT_COUNT] {
        &self.page_ids
    }

    pub fn update_page_ids(&mut self, regs: &PrgBankRegisters) {
        for i in 0..PRG_SLOT_COUNT {
            match &self.page_mappings[i] {
                PrgMappingSlot::Normal(mapping) => {
                    let page_id = mapping.page_info(regs);
                    self.page_ids[i] = PrgPageIdSlot::Normal(page_id);
                }
                PrgMappingSlot::Multi(mappings) => {
                    let mut page_ids = Vec::new();
                    for (mapping, offset) in mappings.iter() {
                        let page_id = mapping.page_info(regs);
                        page_ids.push((page_id, *offset));
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
    rom_address_template: Option<AddressTemplate>,
    ram_address_template: AddressTemplate,
    page_offset: u16,
}

type SubPageOffset = u8;

impl PrgMapping {
    pub fn page_info(&self, regs: &PrgBankRegisters) -> Option<PageInfo> {
        let (Ok(bank_number), Some(page_number_space)) = (self.bank.bank_number(regs), self.bank.page_number_space(regs)) else {
            return None;
        };

        match page_number_space {
            PageNumberSpace::Rom(read_status) => {
                let rom_address_template = self.rom_address_template.as_ref().unwrap();
                let page_number = rom_address_template.resolve_page_number(bank_number.to_raw(), self.page_offset);
                let mem_type = MemType::Rom(read_status);
                Some(PageInfo { mem_type, page_number, address_template: rom_address_template.clone() })
            }
            PageNumberSpace::Ram(read_status, write_status) => {
                let page_number = self.ram_address_template.resolve_page_number(bank_number.to_raw(), self.page_offset);
                let (mem_type, page_number) = if page_number < regs.work_ram_start_page_number() {
                    (MemType::SaveRam(read_status, write_status), page_number)
                } else {
                    (MemType::WorkRam(read_status, write_status), page_number - regs.work_ram_start_page_number())
                };

                Some(PageInfo { mem_type, page_number, address_template: self.ram_address_template.clone() })
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum PrgPageIdSlot {
    Normal(Option<PageInfo>),
    Multi(Box<[(Option<PageInfo>, SubPageOffset); PRG_SUB_SLOT_COUNT]>),
}

type PageNumber = u16;
type PrgIndex = u32;

#[derive(Clone, Debug)]
pub struct PageInfo {
    pub mem_type: MemType,
    pub page_number: PageNumber,
    pub address_template: AddressTemplate,
}