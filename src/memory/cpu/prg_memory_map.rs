use crate::memory::window::PrgWindow;
use crate::memory::bank::bank_number::ReadStatus;
use crate::memory::address_template::address_resolver::AddressResolver;
use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::bank::bank::PrgBank;
use crate::memory::bank::bank_number::{MemType, PageNumberSpace, PrgBankRegisters};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::util::unit::KIBIBYTE;

const PRG_SLOT_COUNT: usize = 5;
const PRG_SUB_SLOT_COUNT: usize = 64;
const PAGE_SIZE: u16 = 8 * KIBIBYTE as u16;

// 0x6000 through 0xFFFF
pub struct PrgMemoryMap {
    page_mappings: [PrgMappingSlot; PRG_SLOT_COUNT],
}

impl PrgMemoryMap {
    pub fn new(
        initial_layout: PrgLayout,
        rom_bank_sizes: &BankSizes,
        ram_bank_sizes: &BankSizes,
        regs: &PrgBankRegisters,
    ) -> Self {
        let mut ram_pages_per_inner_bank = u8::try_from(ram_bank_sizes.inner_bank_size() / u32::from(AddressResolver::PRG_PAGE_SIZE)).unwrap();
        ram_pages_per_inner_bank = std::cmp::max(ram_pages_per_inner_bank, 1);
        let work_ram_start_inner_bank_number = regs.work_ram_start_page_number() / u16::from(ram_pages_per_inner_bank);
        let sub_page_mappings: Vec<_> = initial_layout
            .windows()
            .iter()
            .flat_map(|window| {
                Self::window_to_sub_mappings(window, rom_bank_sizes, ram_bank_sizes, work_ram_start_inner_bank_number)
            })
            .collect();
        let page_mappings: Vec<PrgMappingSlot> = sub_page_mappings
            .chunks_exact(PRG_SUB_SLOT_COUNT)
            .map(|sub_mappings| {
                PrgMappingSlot::new(sub_mappings.to_vec().try_into().unwrap())
            })
            .collect();

        let mut memory_map = Self { page_mappings: page_mappings.try_into().unwrap() };
        memory_map.update_page_ids(regs);
        memory_map
    }

    pub fn index_for_address(&self, addr: CpuAddress) -> Option<(MemType, u32)> {
        assert!(matches!(*addr, 0x6000..=0xFFFF));

        let raw_addr = *addr - 0x6000;
        let mapping_index = raw_addr / PAGE_SIZE;
        let offset_in_page = raw_addr % PAGE_SIZE;

        match &self.page_mappings[mapping_index as usize] {
            PrgMappingSlot::Normal(page_mapping) => page_mapping.index_for_address(addr),
            PrgMappingSlot::Multi(page_mappings) => {
                let sub_mapping_index = offset_in_page / (KIBIBYTE as u16 / 8);
                page_mappings[sub_mapping_index as usize].index_for_address(addr)
            }
        }
    }

    pub fn page_mappings(&self) -> &[PrgMappingSlot; PRG_SLOT_COUNT] {
        &self.page_mappings
    }

    pub fn update_page_ids(&mut self, regs: &PrgBankRegisters) {
        for i in 0..PRG_SLOT_COUNT {
            match &mut self.page_mappings[i] {
                PrgMappingSlot::Normal(mapping) => mapping.update(regs),
                PrgMappingSlot::Multi(mappings) => mappings.iter_mut().for_each(|m| m.update(regs)),
            }
        }
    }

    pub fn set_rom_outer_bank_number(&mut self, regs: &PrgBankRegisters, raw_outer_bank_number: u16) {
        for i in 0..PRG_SLOT_COUNT {
            match &mut self.page_mappings[i] {
                PrgMappingSlot::Normal(mapping) => {
                    mapping.rom_address_resolver.set_raw_outer_bank_number(raw_outer_bank_number);
                }
                PrgMappingSlot::Multi(mappings) => {
                    mappings.iter_mut().for_each(|m| m.rom_address_resolver.set_raw_outer_bank_number(raw_outer_bank_number));
                }
            }
        }

        self.update_page_ids(regs);
    }

    fn window_to_sub_mappings(
        window: &PrgWindow,
        rom_bank_sizes: &BankSizes,
        ram_bank_sizes: &BankSizes,
        work_ram_start_inner_bank_number: u16,
    ) -> Vec<PrgMapping> {
        let mapping = PrgMapping {
            bank: window.bank(),
            rom_address_resolver: window.rom_address_template(rom_bank_sizes),
            ram_address_resolver: window.ram_address_template(ram_bank_sizes, work_ram_start_inner_bank_number),
            // This will be immediately updated to the correct value.
            selected_mem_type: MemType::Rom(ReadStatus::Enabled),
        };

        let sub_mapping_count = window.size().to_raw() / 128;
        vec![mapping; sub_mapping_count as usize]
    }
}

#[derive(Clone, Debug)]
pub enum PrgMappingSlot {
    Normal(PrgMapping),
    Multi(Box<[PrgMapping; PRG_SUB_SLOT_COUNT]>),
}

impl PrgMappingSlot {
    // If the ROM templates are the same for each mapping, and the RAM templates are the same for each mapping
    // then condense all 64 128-byte mappings into a single 8 KiB mapping.
    fn new(mappings: [PrgMapping; PRG_SUB_SLOT_COUNT]) -> Self {
        let first_mapping = mappings[0].clone();
        let template_mismatch = mappings
            .iter()
            .any(|mapping| !mapping.same_templates_as(&first_mapping));
        if template_mismatch {
            Self::Multi(Box::new(mappings))
        } else {
            Self::Normal(first_mapping)
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PrgMapping {
    bank: PrgBank,
    rom_address_resolver: AddressResolver,
    ram_address_resolver: AddressResolver,
    selected_mem_type: MemType,
}

impl PrgMapping {
    pub fn index_for_address(&self, addr: CpuAddress) -> Option<(MemType, u32)> {
        if self.bank.is_absent() {
            return None;
        }

        Some((self.selected_mem_type, self.address_resolver().resolve_index(addr)))
    }

    pub fn inner_bank_number(&self) -> Option<(MemType, u16)> {
        if self.bank.is_absent() {
            return None;
        }

        Some((self.selected_mem_type, self.address_resolver().resolve_inner_bank_number()))
    }

    pub fn maybe_address_resolver(&self) -> Option<&AddressResolver> {
        if self.bank.is_absent() {
            return None;
        }

        Some(self.address_resolver())
    }

    fn address_resolver(&self) -> &AddressResolver {
        match self.selected_mem_type {
            MemType::Rom(..) => &self.rom_address_resolver,
            MemType::WorkRam(..) | MemType::SaveRam(..) => &self.ram_address_resolver,
        }
    }

    pub fn update(&mut self, regs: &PrgBankRegisters) {
        let (Ok(_), Some(page_number_space)) = (self.bank.bank_number(regs), self.bank.page_number_space(regs)) else {
            return;
        };

        self.selected_mem_type = match page_number_space {
            PageNumberSpace::Rom(read_status) => MemType::Rom(read_status),
            PageNumberSpace::Ram(read_status, write_status) => {
                if self.ram_address_resolver.is_currently_resolving_to_save_ram() {
                    MemType::SaveRam(read_status, write_status)
                } else {
                    MemType::WorkRam(read_status, write_status)
                }
            }
        };

        self.rom_address_resolver.update_inner_bank_number(regs);
        self.ram_address_resolver.update_inner_bank_number(regs);
    }

    pub fn same_templates_as(&self, other: &Self) -> bool {
        let rom_template_matches = self.rom_address_resolver == other.rom_address_resolver;
        let ram_template_matches = self.ram_address_resolver == other.ram_address_resolver;
        rom_template_matches || ram_template_matches
    }
}