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

/**
 * ```text
 * *** EXAMPLE RESOLVED PRG ROM ADDRESS TEMPLATE ***
 *
 *                +------------------------- Outer bank number (width is outer_bank_count())
 *                |
 *                |        +---------------- Inner bank number (width is inner_bank_count())
 *                |        |
 *                |        |                 Base address (width is inner_bank_size())
 *                |        |                        |
 *                v        v                        v
 * Components   O₀₁O₀₀ I₀₂I₀₁I₀₀ A₁₃A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 * Full Address A₁₈A₁₇ A₁₆A₁₅A₁₄ A₁₃A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 *              |      |         |  |         Page size  (always 8 KiB)   |
 *              |      |         |  +-------------------------------------|
 *              |      |         |            Inner bank size  (16 KiB)   |
 *              |      |         +----------------------------------------|
 *              |      |                      Outer bank size (128 KiB)   |
 *              |      +--------------------------------------------------|
 *              |                             ROM size        (512 KiB)   |
 *              +---------------------------------------------------------+
 * 
 * 
 * *** EXAMPLE RESOLVED PRG ROM ADDRESS TEMPLATE WITH SUB-PAGES ***
 *
 *                +--------------------------------- Outer bank number (width is outer_bank_count())
 *                |
 *                |        +------------------------ Inner bank number (width is inner_bank_count())
 *                |        |
 *                |        |                 +------ Sub-page number
 *                |        |                 |
 *                |        |                 |       Base address (width is 128 B)
 *                |        |                 |                    |
 *                v        v                 v                    v
 * Components   O₀₁O₀₀ I₀₂I₀₁I₀₀ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇ A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 * Full Address A₁₈A₁₇ A₁₆A₁₅A₁₄ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇ A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 *              |      |         |   |                  | Sub-page (128 B)  |
 *              |      |         |   |                  +-------------------|
 *              |      |         |   |        Page size  (always 8 KiB)     |
 *              |      |         |   +--------------------------------------|
 *              |      |         |            Inner bank size  (16 KiB)     |
 *              |      |         +------------------------------------------|
 *              |      |                      Outer bank size (128 KiB)     |
 *              |      +----------------------------------------------------|
 *              |                             ROM size        (512 KiB)     |
 *              +-----------------------------------------------------------+
 * ```
**/ 

#[derive(Clone, Debug, Default)]
pub struct AddressTemplate {
    // Bit widths
    outer_bank_number_width: u8,
    inner_bank_number_width: u8,
    base_address_width: u8,

    outer_bank_mask: u8,
    inner_bank_mask: u16,
    base_address_mask: u16,

    inner_bank_low_bit_index: u8,
}

impl AddressTemplate {
    pub const PRG_PAGE_NUMBER_WIDTH: u8 = 13;
    pub const PRG_PAGE_SIZE: u16 = 2u16.pow(Self::PRG_PAGE_NUMBER_WIDTH as u32);

    pub fn new(
        (outer_bank_total_width, outer_bank_low_bit_index): (u8, u8),
        // 16 KiB
        (inner_bank_total_width, inner_bank_low_bit_index): (u8, u8),
        // 32 KiB
        (mut base_address_width, base_address_low_bit_index): (u8, u8),
    ) -> Self {
        assert_eq!(base_address_low_bit_index, 0);
        assert_eq!(outer_bank_total_width, inner_bank_total_width);
        assert_eq!(outer_bank_low_bit_index, 0);

        // If the ROM is undersized, reduce the base address bit count, effectively mirroring the ROM until it's the right size.
        base_address_width = std::cmp::min(base_address_width, inner_bank_total_width);
        let inner_bank_number_width = inner_bank_total_width - base_address_width;

        let outer_bank_bit_count = 0;
        let address_template = Self {
            outer_bank_number_width: 0,
            inner_bank_number_width,
            base_address_width,

            outer_bank_mask: create_mask(outer_bank_bit_count, outer_bank_low_bit_index).try_into().unwrap(),
            inner_bank_mask: create_mask(inner_bank_number_width, inner_bank_low_bit_index),
            base_address_mask: create_mask(base_address_width, base_address_low_bit_index),

            inner_bank_low_bit_index,
        };
        assert!(address_template.total_width() <= 32);

        address_template
    }

    pub fn total_width(&self) -> u8 {
        self.outer_bank_number_width + self.inner_bank_number_width + self.base_address_width
    }

    pub fn inner_bank_size(&self) -> u16 {
        1 << (self.base_address_width - self.inner_bank_low_bit_index)
    }

    pub fn inner_bank_count(&self) -> u16 {
        1 << self.inner_bank_number_width
    }

    pub fn outer_bank_count(&self) -> u8 {
        1 << self.outer_bank_number_width
    }

    pub fn outer_bank_size(&self) -> u32 {
        u32::from(self.inner_bank_count()) * u32::from(self.inner_bank_size())
    }

    pub fn rom_size(&self) -> u32 {
        u32::from(self.outer_bank_count()) * self.outer_bank_size()
    }

    pub fn prg_pages_per_inner_bank(&self) -> u8 {
        u8::try_from(self.inner_bank_size() / Self::PRG_PAGE_SIZE).unwrap()
    }

    pub fn prg_pages_per_outer_bank(&self) -> u16 {
        u16::try_from(self.outer_bank_size() / u32::from(Self::PRG_PAGE_SIZE)).unwrap()
    }

    pub fn total_prg_pages(&self) -> u16 {
        u16::try_from(self.rom_size() / u32::from(Self::PRG_PAGE_SIZE)).unwrap()
    }

    pub fn page_number_mask(&self) -> u16 {
        self.prg_pages_per_outer_bank() - 1
    }

    /**
     * PRG Address                            A₁₇A₁₆A₁₅A₁₄ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     * Components Before ( 8 KiB inner banks) O₀₁O₀₀I₀₂I₀₁ I₀₀ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     * Components After  (16 KiB inner banks) O₀₁O₀₀I₀₂I₀₁ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     */
    pub fn with_bigger_bank(&self, new_base_address_bit_count: u8) -> Option<Self> {
        let bank_size_shift = new_base_address_bit_count.checked_sub(self.base_address_width)?;

        let mut big_banked = self.clone();
        big_banked.inner_bank_low_bit_index += bank_size_shift;
        big_banked.base_address_width += bank_size_shift;
        big_banked.base_address_mask = create_mask(big_banked.base_address_width, 0);
        big_banked.inner_bank_mask = create_mask(big_banked.inner_bank_number_width, bank_size_shift);
        Some(big_banked)
    }

    pub fn resolve_page_number(&self, raw_inner_bank_number: u16, page_offset: u16) -> u16 {
        let inner_bank_number = raw_inner_bank_number & self.inner_bank_mask;
        let raw_page_number = inner_bank_number * u16::from(self.prg_pages_per_inner_bank()) + page_offset;
        raw_page_number & self.page_number_mask()
    }

    pub fn resolve(&self, raw_outer_number: u8, raw_inner_number: u16, address_bus_value: u16) -> AddressInfo {
        let outer_bank_number = raw_outer_number & self.outer_bank_mask;
        let shifted_outer_bank_number = u32::from(outer_bank_number) << (self.inner_bank_number_width + self.base_address_width);

        let inner_bank_number = raw_inner_number & self.inner_bank_mask;
        let shifted_inner_bank_number = u32::from(inner_bank_number) << self.base_address_width;

        let base_address = address_bus_value & self.base_address_mask;

        AddressInfo {
            full_address: shifted_outer_bank_number | shifted_inner_bank_number | u32::from(base_address),
            outer_bank_number,
            inner_bank_number,
            base_address,
        }
    }
}

pub struct AddressInfo {
    pub full_address: u32,

    // Components of the full address.
    pub outer_bank_number: u8,
    pub inner_bank_number: u16,
    pub base_address: u16,
}

fn create_mask(bit_count: u8, low_bit_index: u8) -> u16 {
    //assert!(bit_count >= low_bit_index, "Bit count: {bit_count}, low bit index: {low_bit_index}");
    ((1 << bit_count) - 1) & !((1 << low_bit_index) - 1)
}

use std::fmt;

impl fmt::Display for AddressTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", "o".repeat(self.outer_bank_number_width.into()))?;
        write!(f, "{}", "i".repeat(self.inner_bank_number_width.into()))?;
        write!(f, "{}", "a".repeat(self.base_address_width.into()))
    }
}

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
        work_ram_size: u32,
        save_ram_size: u32,
        regs: &PrgBankRegisters,
    ) -> Self {

        assert_eq!(rom_size & (rom_size - 1), 0);
        let rom_address_template = AddressTemplate::new(
            ((rom_size - 1).count_ones() as u8, 0),
            ((rom_size - 1).count_ones() as u8, 0),
            (rom_bank_size.bit_count(), 0),
        );

        let ram_size = work_ram_size + save_ram_size;
        let ram_size_width = if ram_size == 0 { 0 } else { (ram_size - 1).count_ones() as u8 };
        let ram_address_template = AddressTemplate::new(
            (ram_size_width, 0),
            (ram_size_width, 0),
            // FIXME: Hack
            (((8 * KIBIBYTE) - 1).count_ones() as u8, 0),
        );

        assert_eq!(rom_size % u32::from(PAGE_SIZE), 0);

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
    rom_address_template: Option<AddressTemplate>,
    ram_address_template: AddressTemplate,
    page_offset: u16,
}

type SubPageOffset = u8;

impl PrgMapping {
    pub fn page_id(&self, regs: &PrgBankRegisters, save_ram_bank_count: u16) -> Option<(MemType, PageNumber)> {
        let (Ok(bank_number), Some(mem_type)) = (self.bank.bank_number(regs), self.bank.memory_type(regs)) else {
            return None;
        };

        match mem_type {
            MemType::Rom(_) => {
                let rom_address_template = self.rom_address_template.as_ref().unwrap();
                let page_number = rom_address_template.resolve_page_number(bank_number.to_raw(), self.page_offset);
                //println!("Page number within mapping: {page_number}. Bank Index: {}. Page offset: {}", bank_number.to_raw(), self.page_offset);
                Some((mem_type, page_number))
            }
            // FIXME: Pull these out into separate cases, and handle the splitting earlier?
            MemType::WorkRam(read_status_register_id, write_status_register_id)
                    | MemType::SaveRam(read_status_register_id, write_status_register_id) => {
                let mut page_number = self.ram_address_template.resolve_page_number(bank_number.to_raw(), self.page_offset);
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