use std::fmt;

use crate::memory::bank::bank_number::BankNumber;
use crate::memory::bit_template::BitTemplate;
use crate::memory::window::PrgWindow;

const MAX_WIDTH: u8 = 32;
const BASE_ADDRESS_SEGMENT: u8 = 0;
const INNER_BANK_SEGMENT: u8 = 1;
const OUTER_BANK_SEGMENT: u8 = 2;

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
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct AddressTemplate {
    bit_template: BitTemplate,
    fixed_inner_bank_number: Option<u16>,
}

impl AddressTemplate {
    pub const PRG_PAGE_NUMBER_WIDTH: u8 = 13;
    pub const PRG_PAGE_SIZE: u16 = 2u16.pow(Self::PRG_PAGE_NUMBER_WIDTH as u32);
    const PRG_SUB_PAGE_SIZE: u16 = Self::PRG_PAGE_SIZE / 64;

    pub fn new(
        (outer_bank_total_width, outer_bank_low_bit_index): (u8, u8),
        (inner_bank_total_width, inner_bank_low_bit_index): (u8, u8),
        (mut base_address_width, base_address_low_bit_index): (u8, u8),
        fixed_inner_bank_number: Option<u16>,
    ) -> Self {
        assert_eq!(base_address_low_bit_index, 0);
        assert_eq!(outer_bank_low_bit_index, 0);

        let outer_bank_number_width = outer_bank_total_width.strict_sub(inner_bank_total_width);
        // If the ROM is undersized, reduce the base address bit count, effectively mirroring the ROM until it's the right size.
        base_address_width = std::cmp::min(base_address_width, outer_bank_total_width);
        let mut inner_bank_total_width = inner_bank_total_width;
        inner_bank_total_width = std::cmp::min(inner_bank_total_width, outer_bank_total_width);
        let inner_bank_number_width = inner_bank_total_width.strict_sub(base_address_width);

        let address_template = Self {
            bit_template: BitTemplate::right_to_left(&[
                ("a", base_address_width),
                ("i", inner_bank_number_width),
                ("o", outer_bank_number_width),
            ]),

            fixed_inner_bank_number,
        };
        assert!(address_template.total_width() <= MAX_WIDTH);

        address_template
    }

    pub fn total_width(&self) -> u8 {
        self.bit_template.width()
    }

    pub fn inner_bank_size(&self) -> u16 {
        1 << self.bit_template.original_magnitude_of(BASE_ADDRESS_SEGMENT)
    }

    pub fn inner_bank_count(&self) -> u16 {
        1 << self.bit_template.magnitude_of(INNER_BANK_SEGMENT)
    }

    pub fn outer_bank_count(&self) -> u8 {
        1 << self.bit_template.magnitude_of(OUTER_BANK_SEGMENT)
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
    pub fn apply_prg_window(&mut self, window: &PrgWindow) {
        if window.size().page_multiple() == 0 {
            return;
        }

        let mut new_base_address_bit_count = window.size().bit_count();
        let fixed_inner_bank_number = window.bank().fixed_bank_number().map(BankNumber::to_raw);
        self.fixed_inner_bank_number = fixed_inner_bank_number;
        if let Some(fixed_inner_bank_number) = self.fixed_inner_bank_number {
            self.bit_template.constify_segment(INNER_BANK_SEGMENT, fixed_inner_bank_number);
        }

        // Don't expand the bank size larger than the total memory size.
        new_base_address_bit_count = std::cmp::min(new_base_address_bit_count, self.total_width());
        if new_base_address_bit_count > self.bit_template.magnitude_of(BASE_ADDRESS_SEGMENT) {
            self.bit_template.increase_segment_magnitude(BASE_ADDRESS_SEGMENT, new_base_address_bit_count);
        }
    }

    pub fn resolve_page_number(&self, raw_inner_bank_number: u16, page_offset: u16) -> u16 {
        let inner_bank_number = self.bit_template.resolve_segment(INNER_BANK_SEGMENT, raw_inner_bank_number);
        let raw_page_number = inner_bank_number * u16::from(self.prg_pages_per_inner_bank()) + page_offset;
        raw_page_number & self.page_number_mask()
    }

    pub fn resolve_index(&self, raw_outer_bank_number: u8, page_number: u16, offset_in_page: u16) -> u32 {
        let outer_bank_number = self.bit_template.resolve_segment(OUTER_BANK_SEGMENT, raw_outer_bank_number.into());
        let outer_bank_start = u32::from(outer_bank_number) * self.outer_bank_size();
        let page_start = u32::from(page_number) * u32::from(Self::PRG_PAGE_SIZE);
        outer_bank_start | page_start | u32::from(offset_in_page)
    }

    pub fn resolve_subpage_index(&self, raw_outer_bank_number: u8, page_number: u16, sub_page_offset: u16, offset_in_page: u16) -> u32 {
        let outer_bank_number = self.bit_template.resolve_segment(OUTER_BANK_SEGMENT, raw_outer_bank_number.into());
        let outer_bank_start = u32::from(outer_bank_number) * self.outer_bank_size();
        let page_start = u32::from(page_number) * Self::PRG_PAGE_SIZE as u32;
        let subpage_start = Self::PRG_SUB_PAGE_SIZE as u32 * sub_page_offset as u32;
        let offset_in_subpage = u32::from(offset_in_page % Self::PRG_SUB_PAGE_SIZE);
        outer_bank_start | page_start | subpage_start | offset_in_subpage
    }

    pub fn formatted(&self) -> String {
        self.bit_template.formatted()
    }
}

impl fmt::Display for AddressTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.formatted())
    }
}