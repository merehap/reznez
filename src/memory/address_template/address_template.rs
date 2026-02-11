use std::fmt;

use crate::mapper::{CpuAddress, PrgBankRegisterId};
use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::address_template::bit_template::BitTemplate;
use crate::memory::address_template::segment::Segment;
use crate::memory::bank::bank_number::{BankNumber, PrgBankRegisters};
use crate::memory::window::PrgWindow;
use crate::util::const_vec::ConstVec;

const MAX_WIDTH: u8 = 32;
const BASE_ADDRESS_SEGMENT: u8 = 0;
const INNER_BANK_SEGMENT: u8 = 1;

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
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AddressTemplate {
    // Never changed after initialization.
    bit_template: BitTemplate,
    // Never changed after initialization.
    inner_bank_width: u8,

    raw_outer_bank_number: u16,
    raw_inner_bank_number: u16,

    reg_id: Option<PrgBankRegisterId>,
}

impl AddressTemplate {
    pub const PRG_PAGE_NUMBER_WIDTH: u8 = 13;
    pub const PRG_PAGE_SIZE: u16 = 2u16.pow(Self::PRG_PAGE_NUMBER_WIDTH as u32);

    /**
     * PRG Address                           A₁₇A₁₆A₁₅A₁₄ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     * Components Before (8 KiB inner banks) O₀₁O₀₀I₀₂I₀₁ I₀₀ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     * Components After (16 KiB inner banks) O₀₁O₀₀I₀₂I₀₁ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     */
    pub const fn prg(window: &PrgWindow, bank_sizes: &BankSizes) -> Self {
        let fixed_inner_bank_number =
            window.bank().fixed_bank_number().map(BankNumber::to_raw);

        let inner_bank_width = bank_sizes.inner_bank_width();
        let address_bus_segment = Segment::named('a', inner_bank_width);
        let inner_bank_segment =
            if let Some(fixed_inner_bank_number) = fixed_inner_bank_number {
                // o₀₁o₀₀1₁₆1₁₅1₁₄1₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
                Segment::constant(
                    fixed_inner_bank_number,
                    inner_bank_width,
                    bank_sizes.inner_bank_number_width(),
                    0,
                )
            } else {
                // o₀₁o₀₀i₀₃i₀₂i₀₁i₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
                Segment::named('i', bank_sizes.inner_bank_number_width())
            };

        let outer_bank_segment = Segment::named('o', bank_sizes.outer_bank_number_width());

        let mut segments = ConstVec::new();
        segments.push(address_bus_segment);
        segments.push(inner_bank_segment);
        segments.push(outer_bank_segment);
        let mut bit_template = BitTemplate::right_to_left(segments);

        // Don't expand the bank size larger than the total memory size.
        let new_base_address_bit_count = std::cmp::min(window.size().bit_count(), bit_template.width());
        if new_base_address_bit_count > bit_template.magnitude_of(BASE_ADDRESS_SEGMENT).unwrap() {
            bit_template.increase_segment_magnitude(BASE_ADDRESS_SEGMENT, new_base_address_bit_count);
        }

        let address_template = Self {
            bit_template,
            inner_bank_width,
            raw_outer_bank_number: 0,
            raw_inner_bank_number: 0,
            reg_id: window.register_id(),
        };
        assert!(address_template.total_width() <= MAX_WIDTH);

        if window.size().page_multiple() == 0 {
            return address_template;
        }

        address_template
    }

    pub const fn from_formatted(text: &'static str) -> Result<Self, &'static str> {
        Self::from_bit_template(BitTemplate::from_formatted(text)?)
    }

    const fn from_bit_template(bit_template: BitTemplate) -> Result<Self, &'static str> {
        if bit_template.width() > 32 {
            return Err("AddressTemplate must not be longer than 32 bits.");
        }

        let Some(base_address_width) = bit_template.width_of(BASE_ADDRESS_SEGMENT) else {
            panic!();
        };
        let inner_bank_ignored_low_count = match bit_template.ignored_low_count_of(INNER_BANK_SEGMENT) {
            None => 0,
            Some(count) => count,
        };
        let inner_bank_width = base_address_width + inner_bank_ignored_low_count;

        Ok(Self {
            bit_template,
            inner_bank_width,
            raw_outer_bank_number: 0,
            raw_inner_bank_number: 0,
            // TODO: Parse this from the template.
            reg_id: None,
        })
    }

    pub fn reduced(&self, bank_sizes: &BankSizes) -> Self {
        let mut result = self.clone();
        result.bit_template.shorten(bank_sizes.full_width());
        result.inner_bank_width = bank_sizes.inner_bank_width();
        result
    }

    pub const fn total_width(&self) -> u8 {
        self.bit_template.width()
    }

    pub fn resolve_page_number(
        &self,
        raw_inner_bank_number: u16,
        page_offset: u16,
    ) -> u16 {
        let inner_bank_number = self
            .bit_template
            .resolve_segment(INNER_BANK_SEGMENT, raw_inner_bank_number);
        let page_offset = page_offset % self.prg_pages_per_outer_bank();
        let raw_page_number = inner_bank_number * u16::from(self.prg_pages_per_inner_bank()) + page_offset;
        raw_page_number & self.page_number_mask()
    }

    pub fn resolve_inner_bank_number(&self) -> u16 {
        self.bit_template.resolve_segment(INNER_BANK_SEGMENT, self.raw_inner_bank_number)
    }

    pub fn resolve_index(&self, addr: CpuAddress) -> u32 {
        self.bit_template.resolve(&[*addr, self.raw_inner_bank_number, self.raw_outer_bank_number])
    }

    pub fn resolve_subpage_index(&self, addr: CpuAddress) -> u32 {
        self.bit_template.resolve(&[*addr, self.raw_inner_bank_number, self.raw_outer_bank_number])
    }

    pub fn formatted(&self) -> String {
        self.bit_template.formatted()
    }

    pub fn set_raw_outer_bank_number(&mut self, number: u16) {
        self.raw_outer_bank_number = number;
    }

    pub fn update_inner_bank_number(&mut self, regs: &PrgBankRegisters) {
        if let Some(reg_id) = self.reg_id {
            self.raw_inner_bank_number = regs.get(reg_id).index().unwrap().to_raw();
        }
    }

    fn inner_bank_count(&self) -> u16 {
        match self.bit_template.magnitude_of(INNER_BANK_SEGMENT) {
            None => 1,
            Some(magnitude) => 1 << magnitude,
        }
    }

    fn inner_bank_size(&self) -> u16 {
        1 << self.inner_bank_width
    }

    fn outer_bank_size(&self) -> u32 {
        u32::from(self.inner_bank_count()) * u32::from(self.inner_bank_size())
    }

    fn prg_pages_per_inner_bank(&self) -> u8 {
        u8::try_from(self.inner_bank_size() / Self::PRG_PAGE_SIZE).unwrap()
    }

    fn prg_pages_per_outer_bank(&self) -> u16 {
        u16::try_from(self.outer_bank_size() / u32::from(Self::PRG_PAGE_SIZE)).unwrap()
    }

    fn page_number_mask(&self) -> u16 {
        self.prg_pages_per_outer_bank() - 1
    }
}

impl fmt::Display for AddressTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.formatted())
    }
}
